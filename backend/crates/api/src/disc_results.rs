//! `/disc-results` CRUD, guarded by RBAC. A DISC result is a collaborator's four
//! dimension scores (executor = D, communicator = I, planner = S, analyst = C),
//! stored as history in the tenant schema. The primary/secondary profile is
//! derived at read time via `service::disc`. `create` validates the referenced
//! collaborator exists and is active (a dangling reference → `422`). Results are
//! immutable: there is no update, and `delete` is a hard delete.

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use entity::{collaborator, collaborator_disc_result as disc_result};
use sea_orm::prelude::{DateTimeWithTimeZone, Uuid};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use service::disc::{profile, DiscScores};
use service::permission::Resource;

use crate::extract::{AuthRejection, TenantContext};

#[derive(Serialize)]
struct DiscResultView {
    id: String,
    collaborator_id: String,
    executor: i32,
    communicator: i32,
    planner: i32,
    analyst: i32,
    primary_profile: String,
    secondary_profile: String,
    created_at: DateTimeWithTimeZone,
}

impl From<disc_result::Model> for DiscResultView {
    fn from(model: disc_result::Model) -> Self {
        let derived = profile(&DiscScores {
            executor: model.executor,
            communicator: model.communicator,
            planner: model.planner,
            analyst: model.analyst,
        });
        Self {
            id: model.id.to_string(),
            collaborator_id: model.collaborator_id.to_string(),
            executor: model.executor,
            communicator: model.communicator,
            planner: model.planner,
            analyst: model.analyst,
            primary_profile: derived.primary.to_owned(),
            secondary_profile: derived.secondary.to_owned(),
            created_at: model.created_at,
        }
    }
}

#[derive(Deserialize)]
pub struct DiscResultQuery {
    #[serde(default)]
    collaborator_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CreateDiscResult {
    collaborator_id: Uuid,
    executor: i32,
    communicator: i32,
    planner: i32,
    analyst: i32,
}

/// `GET /disc-results` — lists results newest-first; optional `?collaborator_id=`
/// filter. Requires `disc.read`.
pub async fn list(
    ctx: TenantContext,
    Query(query): Query<DiscResultQuery>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::DiscRead).await?;

    let mut find = disc_result::Entity::find();
    if let Some(collaborator_id) = query.collaborator_id {
        find = find.filter(disc_result::Column::CollaboratorId.eq(collaborator_id));
    }
    let results = find
        .order_by_desc(disc_result::Column::CreatedAt)
        .all(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?;

    let body: Vec<DiscResultView> = results.into_iter().map(DiscResultView::from).collect();
    Ok(Json(body).into_response())
}

/// `POST /disc-results` — records a DISC result for a collaborator. Requires
/// `disc.create`. An unknown/inactive collaborator → `422`.
pub async fn create(
    ctx: TenantContext,
    Json(body): Json<CreateDiscResult>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::DiscCreate).await?;

    let conn = ctx.tenant_db.as_ref();
    if !collaborator_exists(conn, body.collaborator_id).await? {
        return Ok(unprocessable("unknown collaborator"));
    }

    let created = disc_result::ActiveModel {
        id: Set(Uuid::new_v4()),
        collaborator_id: Set(body.collaborator_id),
        executor: Set(body.executor),
        communicator: Set(body.communicator),
        planner: Set(body.planner),
        analyst: Set(body.analyst),
        ..Default::default()
    }
    .insert(conn)
    .await
    .map_err(|_| AuthRejection::Internal)?;

    Ok((StatusCode::CREATED, Json(DiscResultView::from(created))).into_response())
}

/// `DELETE /disc-results/{id}` — removes a DISC result (hard delete; results are
/// immutable history). Requires `disc.delete`. Unknown id → `404`; success → `204`.
pub async fn delete(ctx: TenantContext, Path(id): Path<Uuid>) -> Result<Response, AuthRejection> {
    ctx.require(Resource::DiscDelete).await?;

    let conn = ctx.tenant_db.as_ref();
    let deleted = disc_result::Entity::delete_by_id(id)
        .exec(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?;

    if deleted.rows_affected == 0 {
        return Ok(not_found());
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Whether an active collaborator with `id` exists in the tenant.
async fn collaborator_exists(
    conn: &impl sea_orm::ConnectionTrait,
    id: Uuid,
) -> Result<bool, AuthRejection> {
    let found = collaborator::Entity::find_by_id(id)
        .filter(collaborator::Column::Active.eq(true))
        .one(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?;
    Ok(found.is_some())
}

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "disc result not found" })),
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
