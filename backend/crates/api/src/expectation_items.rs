//! `/expectation-items` CRUD, guarded by RBAC. An expectation-contract item lives
//! in the tenant schema and belongs to a feedback; `kind` is `goal` or `behavior`
//! (the legacy model used two identical tables — we unify them here). Every handler
//! queries `ctx.tenant_db` after the `ctx.require(...)` guard. `create` validates
//! the referenced feedback exists and that `kind` is valid (otherwise `422`).
//! Removal is a soft delete; `PATCH` writes only the fields present in the body.

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use entity::{expectation_contract_item as item, feedback};
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use service::permission::Resource;

use crate::extract::{AuthRejection, TenantContext};

/// The two kinds of expectation-contract item.
const KINDS: [&str; 2] = ["goal", "behavior"];

#[derive(Serialize)]
struct ItemView {
    id: String,
    feedback_id: String,
    kind: String,
    description: Option<String>,
    done: bool,
    active: bool,
}

impl From<item::Model> for ItemView {
    fn from(model: item::Model) -> Self {
        Self {
            id: model.id.to_string(),
            feedback_id: model.feedback_id.to_string(),
            kind: model.kind,
            description: model.description,
            done: model.done,
            active: model.active,
        }
    }
}

#[derive(Deserialize)]
pub struct ItemQuery {
    #[serde(default)]
    feedback_id: Option<Uuid>,
    #[serde(default)]
    kind: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateItem {
    feedback_id: Uuid,
    kind: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    done: bool,
}

#[derive(Deserialize)]
pub struct UpdateItem {
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    done: Option<bool>,
    #[serde(default)]
    active: Option<bool>,
}

/// `GET /expectation-items` — lists active items; optional `?feedback_id=` and
/// `?kind=` filters. Requires `expectation.read`.
pub async fn list(
    ctx: TenantContext,
    Query(query): Query<ItemQuery>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::ExpectationRead).await?;

    let mut find = item::Entity::find().filter(item::Column::Active.eq(true));
    if let Some(feedback_id) = query.feedback_id {
        find = find.filter(item::Column::FeedbackId.eq(feedback_id));
    }
    if let Some(kind) = query.kind {
        find = find.filter(item::Column::Kind.eq(kind));
    }
    let items = find
        .order_by_asc(item::Column::CreatedAt)
        .all(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?;

    let body: Vec<ItemView> = items.into_iter().map(ItemView::from).collect();
    Ok(Json(body).into_response())
}

/// `POST /expectation-items` — creates an item under a feedback. Requires
/// `expectation.create`. Invalid `kind` or unknown feedback → `422`.
pub async fn create(
    ctx: TenantContext,
    Json(body): Json<CreateItem>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::ExpectationCreate).await?;

    if !is_valid_kind(&body.kind) {
        return Ok(unprocessable("invalid kind"));
    }
    let conn = ctx.tenant_db.as_ref();
    if !feedback_exists(conn, body.feedback_id).await? {
        return Ok(unprocessable("unknown feedback"));
    }

    let created = item::ActiveModel {
        id: Set(Uuid::new_v4()),
        feedback_id: Set(body.feedback_id),
        kind: Set(body.kind),
        description: Set(body.description),
        done: Set(body.done),
        active: Set(true),
        ..Default::default()
    }
    .insert(conn)
    .await
    .map_err(|_| AuthRejection::Internal)?;

    Ok((StatusCode::CREATED, Json(ItemView::from(created))).into_response())
}

/// `PATCH /expectation-items/{id}` — updates the fields present in the body.
/// Requires `expectation.update`. Invalid `kind` → `422`; unknown id → `404`.
pub async fn update(
    ctx: TenantContext,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateItem>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::ExpectationUpdate).await?;

    if let Some(kind) = &body.kind {
        if !is_valid_kind(kind) {
            return Ok(unprocessable("invalid kind"));
        }
    }

    let conn = ctx.tenant_db.as_ref();
    let Some(model) = item::Entity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?
    else {
        return Ok(not_found());
    };

    let mut active_model: item::ActiveModel = model.into();
    if let Some(kind) = body.kind {
        active_model.kind = Set(kind);
    }
    if let Some(description) = body.description {
        active_model.description = Set(Some(description));
    }
    if let Some(done) = body.done {
        active_model.done = Set(done);
    }
    if let Some(active) = body.active {
        active_model.active = Set(active);
    }
    let updated = active_model
        .update(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?;

    Ok(Json(ItemView::from(updated)).into_response())
}

/// `DELETE /expectation-items/{id}` — deactivates an item (soft delete). Requires
/// `expectation.delete`. Unknown id → `404`; success → `204`.
pub async fn delete(ctx: TenantContext, Path(id): Path<Uuid>) -> Result<Response, AuthRejection> {
    ctx.require(Resource::ExpectationDelete).await?;

    let conn = ctx.tenant_db.as_ref();
    let Some(model) = item::Entity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?
    else {
        return Ok(not_found());
    };

    let mut active_model: item::ActiveModel = model.into();
    active_model.active = Set(false);
    active_model
        .update(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

fn is_valid_kind(kind: &str) -> bool {
    KINDS.contains(&kind)
}

/// Whether an active feedback with `id` exists in the tenant.
async fn feedback_exists(
    conn: &impl sea_orm::ConnectionTrait,
    id: Uuid,
) -> Result<bool, AuthRejection> {
    let found = feedback::Entity::find_by_id(id)
        .filter(feedback::Column::Active.eq(true))
        .one(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?;
    Ok(found.is_some())
}

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "expectation item not found" })),
    )
        .into_response()
}

fn unprocessable(message: &str) -> Response {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": message })),
    )
        .into_response()
}
