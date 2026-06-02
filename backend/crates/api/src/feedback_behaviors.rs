//! `/feedback-behaviors` CRUD, guarded by RBAC. A feedback behavior (a scored
//! DISC-values line) lives in the tenant schema and belongs to a feedback, so
//! every handler queries `ctx.tenant_db` after the `ctx.require(...)` guard.
//! `create` validates the referenced feedback exists (a dangling reference →
//! `422`). Removal is a soft delete; `PATCH` writes only the fields present.

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use entity::{feedback, feedback_behavior as behavior};
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use service::permission::Resource;

use crate::extract::{AuthRejection, TenantContext};

#[derive(Serialize)]
struct BehaviorView {
    id: String,
    feedback_id: String,
    value_description: String,
    behavior_description: String,
    behavior_obs: Option<String>,
    value_instruction: Option<String>,
    score: i32,
    active: bool,
}

impl From<behavior::Model> for BehaviorView {
    fn from(model: behavior::Model) -> Self {
        Self {
            id: model.id.to_string(),
            feedback_id: model.feedback_id.to_string(),
            value_description: model.value_description,
            behavior_description: model.behavior_description,
            behavior_obs: model.behavior_obs,
            value_instruction: model.value_instruction,
            score: model.score,
            active: model.active,
        }
    }
}

#[derive(Deserialize)]
pub struct BehaviorQuery {
    #[serde(default)]
    feedback_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CreateBehavior {
    feedback_id: Uuid,
    value_description: String,
    behavior_description: String,
    #[serde(default)]
    behavior_obs: Option<String>,
    #[serde(default)]
    value_instruction: Option<String>,
    score: i32,
}

#[derive(Deserialize)]
pub struct UpdateBehavior {
    #[serde(default)]
    value_description: Option<String>,
    #[serde(default)]
    behavior_description: Option<String>,
    #[serde(default)]
    behavior_obs: Option<String>,
    #[serde(default)]
    value_instruction: Option<String>,
    #[serde(default)]
    score: Option<i32>,
    #[serde(default)]
    active: Option<bool>,
}

/// `GET /feedback-behaviors` — lists active behaviors; optional `?feedback_id=`
/// filter. Requires `feedback_behavior.read`.
pub async fn list(
    ctx: TenantContext,
    Query(query): Query<BehaviorQuery>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::FeedbackBehaviorRead).await?;

    let mut find = behavior::Entity::find().filter(behavior::Column::Active.eq(true));
    if let Some(feedback_id) = query.feedback_id {
        find = find.filter(behavior::Column::FeedbackId.eq(feedback_id));
    }
    let behaviors = find
        .order_by_asc(behavior::Column::CreatedAt)
        .all(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?;

    let body: Vec<BehaviorView> = behaviors.into_iter().map(BehaviorView::from).collect();
    Ok(Json(body).into_response())
}

/// `POST /feedback-behaviors` — creates a behavior under a feedback. Requires
/// `feedback_behavior.create`. Unknown feedback → `422`.
pub async fn create(
    ctx: TenantContext,
    Json(body): Json<CreateBehavior>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::FeedbackBehaviorCreate).await?;

    let conn = ctx.tenant_db.as_ref();
    if !feedback_exists(conn, body.feedback_id).await? {
        return Ok(unprocessable("unknown feedback"));
    }

    let created = behavior::ActiveModel {
        id: Set(Uuid::new_v4()),
        feedback_id: Set(body.feedback_id),
        value_description: Set(body.value_description),
        behavior_description: Set(body.behavior_description),
        behavior_obs: Set(body.behavior_obs),
        value_instruction: Set(body.value_instruction),
        score: Set(body.score),
        active: Set(true),
        ..Default::default()
    }
    .insert(conn)
    .await
    .map_err(|_| AuthRejection::Internal)?;

    Ok((StatusCode::CREATED, Json(BehaviorView::from(created))).into_response())
}

/// `PATCH /feedback-behaviors/{id}` — updates the fields present in the body.
/// Requires `feedback_behavior.update`. Unknown id → `404`.
pub async fn update(
    ctx: TenantContext,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateBehavior>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::FeedbackBehaviorUpdate).await?;

    let conn = ctx.tenant_db.as_ref();
    let Some(model) = behavior::Entity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?
    else {
        return Ok(not_found());
    };

    let mut active_model: behavior::ActiveModel = model.into();
    if let Some(value) = body.value_description {
        active_model.value_description = Set(value);
    }
    if let Some(value) = body.behavior_description {
        active_model.behavior_description = Set(value);
    }
    if let Some(value) = body.behavior_obs {
        active_model.behavior_obs = Set(Some(value));
    }
    if let Some(value) = body.value_instruction {
        active_model.value_instruction = Set(Some(value));
    }
    if let Some(score) = body.score {
        active_model.score = Set(score);
    }
    if let Some(active) = body.active {
        active_model.active = Set(active);
    }
    let updated = active_model
        .update(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?;

    Ok(Json(BehaviorView::from(updated)).into_response())
}

/// `DELETE /feedback-behaviors/{id}` — deactivates a behavior (soft delete).
/// Requires `feedback_behavior.delete`. Unknown id → `404`; success → `204`.
pub async fn delete(ctx: TenantContext, Path(id): Path<Uuid>) -> Result<Response, AuthRejection> {
    ctx.require(Resource::FeedbackBehaviorDelete).await?;

    let conn = ctx.tenant_db.as_ref();
    let Some(model) = behavior::Entity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?
    else {
        return Ok(not_found());
    };

    let mut active_model: behavior::ActiveModel = model.into();
    active_model.active = Set(false);
    active_model
        .update(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?;

    Ok(StatusCode::NO_CONTENT.into_response())
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
        Json(serde_json::json!({ "error": "feedback behavior not found" })),
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
