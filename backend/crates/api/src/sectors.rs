//! `/sectors` CRUD, guarded by RBAC. Sectors live in the tenant schema, so every
//! handler queries `ctx.tenant_db` (resolved from the caller's token) after the
//! `ctx.require(...)` guard. Removal is a soft delete (`active = false`).

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use entity::sector;
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use service::permission::Resource;

use crate::extract::{AuthRejection, TenantContext};

#[derive(Serialize)]
struct SectorView {
    id: String,
    name: String,
    active: bool,
}

impl From<sector::Model> for SectorView {
    fn from(model: sector::Model) -> Self {
        Self {
            id: model.id.to_string(),
            name: model.name,
            active: model.active,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateSector {
    name: String,
}

#[derive(Deserialize)]
pub struct UpdateSector {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    active: Option<bool>,
}

/// `GET /sectors` — lists the tenant's active sectors. Requires `sector.read`.
pub async fn list(ctx: TenantContext) -> Result<Response, AuthRejection> {
    ctx.require(Resource::SectorRead).await?;

    let sectors = sector::Entity::find()
        .filter(sector::Column::Active.eq(true))
        .order_by_asc(sector::Column::Name)
        .all(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?;

    let body: Vec<SectorView> = sectors.into_iter().map(SectorView::from).collect();
    Ok(Json(body).into_response())
}

/// `POST /sectors` — creates a sector. Requires `sector.create`.
pub async fn create(
    ctx: TenantContext,
    Json(body): Json<CreateSector>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::SectorCreate).await?;

    let created = sector::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name),
        active: Set(true),
        ..Default::default()
    }
    .insert(ctx.tenant_db.as_ref())
    .await
    .map_err(|_| AuthRejection::Internal)?;

    Ok((StatusCode::CREATED, Json(SectorView::from(created))).into_response())
}

/// `PATCH /sectors/{id}` — updates a sector's name and/or active flag. Requires
/// `sector.update`. Unknown id → `404`.
pub async fn update(
    ctx: TenantContext,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateSector>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::SectorUpdate).await?;

    let Some(model) = sector::Entity::find_by_id(id)
        .one(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?
    else {
        return Ok(not_found());
    };

    let mut active_model: sector::ActiveModel = model.into();
    if let Some(name) = body.name {
        active_model.name = Set(name);
    }
    if let Some(active) = body.active {
        active_model.active = Set(active);
    }
    let updated = active_model
        .update(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?;

    Ok(Json(SectorView::from(updated)).into_response())
}

/// `DELETE /sectors/{id}` — deactivates a sector (soft delete). Requires
/// `sector.delete`. Unknown id → `404`; success → `204`.
pub async fn delete(ctx: TenantContext, Path(id): Path<Uuid>) -> Result<Response, AuthRejection> {
    ctx.require(Resource::SectorDelete).await?;

    let Some(model) = sector::Entity::find_by_id(id)
        .one(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?
    else {
        return Ok(not_found());
    };

    let mut active_model: sector::ActiveModel = model.into();
    active_model.active = Set(false);
    active_model
        .update(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "sector not found" })),
    )
        .into_response()
}
