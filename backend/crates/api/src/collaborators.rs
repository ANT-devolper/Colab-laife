//! `/collaborators` CRUD, guarded by RBAC. Collaborators live in the tenant
//! schema and reference a sector, a role and an optional manager (self-ref), so
//! every handler queries `ctx.tenant_db` after the `ctx.require(...)` guard.
//!
//! `create`/`update` validate that any referenced `sector_id`/`role_id`/
//! `manager_id` points at an existing active row in the tenant; a dangling
//! reference is rejected with `422` (the database FK would also reject it, but we
//! check first to return a clean domain error). Removal is a soft delete
//! (`active = false`). On `PATCH`, an omitted field is left untouched.

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use entity::{collaborator, role, sector};
use sea_orm::prelude::{Date, Uuid};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use service::permission::Resource;

use crate::extract::{AuthRejection, TenantContext};

#[derive(Serialize)]
struct CollaboratorView {
    id: String,
    name: String,
    sector_id: Option<String>,
    role_id: Option<String>,
    manager_id: Option<String>,
    whatsapp: Option<String>,
    email: Option<String>,
    is_manager: bool,
    user_id: Option<String>,
    date_of_hire: Option<Date>,
    active: bool,
}

impl From<collaborator::Model> for CollaboratorView {
    fn from(model: collaborator::Model) -> Self {
        Self {
            id: model.id.to_string(),
            name: model.name,
            sector_id: model.sector_id.map(|id| id.to_string()),
            role_id: model.role_id.map(|id| id.to_string()),
            manager_id: model.manager_id.map(|id| id.to_string()),
            whatsapp: model.whatsapp,
            email: model.email,
            is_manager: model.is_manager,
            user_id: model.user_id.map(|id| id.to_string()),
            date_of_hire: model.date_of_hire,
            active: model.active,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateCollaborator {
    name: String,
    #[serde(default)]
    sector_id: Option<Uuid>,
    #[serde(default)]
    role_id: Option<Uuid>,
    #[serde(default)]
    manager_id: Option<Uuid>,
    #[serde(default)]
    whatsapp: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    is_manager: bool,
    #[serde(default)]
    user_id: Option<Uuid>,
    #[serde(default)]
    date_of_hire: Option<Date>,
}

#[derive(Deserialize)]
pub struct UpdateCollaborator {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    sector_id: Option<Uuid>,
    #[serde(default)]
    role_id: Option<Uuid>,
    #[serde(default)]
    manager_id: Option<Uuid>,
    #[serde(default)]
    whatsapp: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    is_manager: Option<bool>,
    #[serde(default)]
    user_id: Option<Uuid>,
    #[serde(default)]
    date_of_hire: Option<Date>,
    #[serde(default)]
    active: Option<bool>,
}

/// `GET /collaborators` — lists the tenant's active collaborators. Requires
/// `collaborator.read`.
pub async fn list(ctx: TenantContext) -> Result<Response, AuthRejection> {
    ctx.require(Resource::CollaboratorRead).await?;

    let collaborators = collaborator::Entity::find()
        .filter(collaborator::Column::Active.eq(true))
        .order_by_asc(collaborator::Column::Name)
        .all(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?;

    let body: Vec<CollaboratorView> = collaborators
        .into_iter()
        .map(CollaboratorView::from)
        .collect();
    Ok(Json(body).into_response())
}

/// `POST /collaborators` — creates a collaborator. Requires `collaborator.create`.
/// Dangling sector/role/manager references → `422`.
pub async fn create(
    ctx: TenantContext,
    Json(body): Json<CreateCollaborator>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::CollaboratorCreate).await?;

    let conn = ctx.tenant_db.as_ref();
    if let Some(rejection) =
        validate_references(conn, body.sector_id, body.role_id, body.manager_id).await?
    {
        return Ok(rejection);
    }

    let created = collaborator::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name),
        sector_id: Set(body.sector_id),
        role_id: Set(body.role_id),
        manager_id: Set(body.manager_id),
        whatsapp: Set(body.whatsapp),
        email: Set(body.email),
        is_manager: Set(body.is_manager),
        user_id: Set(body.user_id),
        date_of_hire: Set(body.date_of_hire),
        active: Set(true),
        ..Default::default()
    }
    .insert(conn)
    .await
    .map_err(|_| AuthRejection::Internal)?;

    Ok((StatusCode::CREATED, Json(CollaboratorView::from(created))).into_response())
}

/// `PATCH /collaborators/{id}` — updates the fields present in the body. Requires
/// `collaborator.update`. Unknown id → `404`; dangling references → `422`.
pub async fn update(
    ctx: TenantContext,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateCollaborator>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::CollaboratorUpdate).await?;

    let conn = ctx.tenant_db.as_ref();
    let Some(model) = collaborator::Entity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?
    else {
        return Ok(not_found());
    };

    if let Some(rejection) =
        validate_references(conn, body.sector_id, body.role_id, body.manager_id).await?
    {
        return Ok(rejection);
    }

    let mut active_model: collaborator::ActiveModel = model.into();
    if let Some(name) = body.name {
        active_model.name = Set(name);
    }
    if let Some(sector_id) = body.sector_id {
        active_model.sector_id = Set(Some(sector_id));
    }
    if let Some(role_id) = body.role_id {
        active_model.role_id = Set(Some(role_id));
    }
    if let Some(manager_id) = body.manager_id {
        active_model.manager_id = Set(Some(manager_id));
    }
    if let Some(whatsapp) = body.whatsapp {
        active_model.whatsapp = Set(Some(whatsapp));
    }
    if let Some(email) = body.email {
        active_model.email = Set(Some(email));
    }
    if let Some(is_manager) = body.is_manager {
        active_model.is_manager = Set(is_manager);
    }
    if let Some(user_id) = body.user_id {
        active_model.user_id = Set(Some(user_id));
    }
    if let Some(date_of_hire) = body.date_of_hire {
        active_model.date_of_hire = Set(Some(date_of_hire));
    }
    if let Some(active) = body.active {
        active_model.active = Set(active);
    }
    let updated = active_model
        .update(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?;

    Ok(Json(CollaboratorView::from(updated)).into_response())
}

/// `DELETE /collaborators/{id}` — deactivates a collaborator (soft delete).
/// Requires `collaborator.delete`. Unknown id → `404`; success → `204`.
pub async fn delete(ctx: TenantContext, Path(id): Path<Uuid>) -> Result<Response, AuthRejection> {
    ctx.require(Resource::CollaboratorDelete).await?;

    let conn = ctx.tenant_db.as_ref();
    let Some(model) = collaborator::Entity::find_by_id(id)
        .one(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?
    else {
        return Ok(not_found());
    };

    let mut active_model: collaborator::ActiveModel = model.into();
    active_model.active = Set(false);
    active_model
        .update(conn)
        .await
        .map_err(|_| AuthRejection::Internal)?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Checks that each provided reference points at an existing active row. Returns
/// `Ok(Some(response))` with a `422` when one is dangling, `Ok(None)` when all
/// references are valid.
async fn validate_references(
    conn: &impl ConnectionTrait,
    sector_id: Option<Uuid>,
    role_id: Option<Uuid>,
    manager_id: Option<Uuid>,
) -> Result<Option<Response>, AuthRejection> {
    if let Some(sector_id) = sector_id {
        let exists = sector::Entity::find_by_id(sector_id)
            .filter(sector::Column::Active.eq(true))
            .one(conn)
            .await
            .map_err(|_| AuthRejection::Internal)?
            .is_some();
        if !exists {
            return Ok(Some(unprocessable("unknown sector")));
        }
    }
    if let Some(role_id) = role_id {
        let exists = role::Entity::find_by_id(role_id)
            .filter(role::Column::Active.eq(true))
            .one(conn)
            .await
            .map_err(|_| AuthRejection::Internal)?
            .is_some();
        if !exists {
            return Ok(Some(unprocessable("unknown role")));
        }
    }
    if let Some(manager_id) = manager_id {
        let exists = collaborator::Entity::find_by_id(manager_id)
            .filter(collaborator::Column::Active.eq(true))
            .one(conn)
            .await
            .map_err(|_| AuthRejection::Internal)?
            .is_some();
        if !exists {
            return Ok(Some(unprocessable("unknown manager")));
        }
    }
    Ok(None)
}

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "collaborator not found" })),
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
