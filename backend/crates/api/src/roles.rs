//! `/roles` CRUD, guarded by RBAC. Roles live in the tenant schema and carry the
//! legacy set of description fields, so every handler queries `ctx.tenant_db`
//! after the `ctx.require(...)` guard. Removal is a soft delete (`active = false`).
//!
//! On `PATCH`, an omitted field is left untouched (we cannot distinguish an
//! explicit `null` from an absent field, so clearing a field is out of scope for
//! now); only the fields present in the body are written.

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use entity::role;
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use service::permission::Resource;

use crate::extract::{AuthRejection, TenantContext};

#[derive(Serialize)]
struct RoleView {
    id: String,
    name: String,
    profile_suggestion: Option<String>,
    objective: Option<String>,
    requirement_education: Option<String>,
    requirement_experience: Option<String>,
    requirement_attention: Option<String>,
    requirement_knowledge: Option<String>,
    requirement_skill: Option<String>,
    requirement_attitude: Option<String>,
    requirement_delivery: Option<String>,
    observation: Option<String>,
    active: bool,
}

impl From<role::Model> for RoleView {
    fn from(model: role::Model) -> Self {
        Self {
            id: model.id.to_string(),
            name: model.name,
            profile_suggestion: model.profile_suggestion,
            objective: model.objective,
            requirement_education: model.requirement_education,
            requirement_experience: model.requirement_experience,
            requirement_attention: model.requirement_attention,
            requirement_knowledge: model.requirement_knowledge,
            requirement_skill: model.requirement_skill,
            requirement_attitude: model.requirement_attitude,
            requirement_delivery: model.requirement_delivery,
            observation: model.observation,
            active: model.active,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateRole {
    name: String,
    #[serde(default)]
    profile_suggestion: Option<String>,
    #[serde(default)]
    objective: Option<String>,
    #[serde(default)]
    requirement_education: Option<String>,
    #[serde(default)]
    requirement_experience: Option<String>,
    #[serde(default)]
    requirement_attention: Option<String>,
    #[serde(default)]
    requirement_knowledge: Option<String>,
    #[serde(default)]
    requirement_skill: Option<String>,
    #[serde(default)]
    requirement_attitude: Option<String>,
    #[serde(default)]
    requirement_delivery: Option<String>,
    #[serde(default)]
    observation: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateRole {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    profile_suggestion: Option<String>,
    #[serde(default)]
    objective: Option<String>,
    #[serde(default)]
    requirement_education: Option<String>,
    #[serde(default)]
    requirement_experience: Option<String>,
    #[serde(default)]
    requirement_attention: Option<String>,
    #[serde(default)]
    requirement_knowledge: Option<String>,
    #[serde(default)]
    requirement_skill: Option<String>,
    #[serde(default)]
    requirement_attitude: Option<String>,
    #[serde(default)]
    requirement_delivery: Option<String>,
    #[serde(default)]
    observation: Option<String>,
    #[serde(default)]
    active: Option<bool>,
}

/// `GET /roles` — lists the tenant's active roles. Requires `role.read`.
pub async fn list(ctx: TenantContext) -> Result<Response, AuthRejection> {
    ctx.require(Resource::RoleRead).await?;

    let roles = role::Entity::find()
        .filter(role::Column::Active.eq(true))
        .order_by_asc(role::Column::Name)
        .all(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?;

    let body: Vec<RoleView> = roles.into_iter().map(RoleView::from).collect();
    Ok(Json(body).into_response())
}

/// `POST /roles` — creates a role. Requires `role.create`.
pub async fn create(
    ctx: TenantContext,
    Json(body): Json<CreateRole>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::RoleCreate).await?;

    let created = role::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name),
        profile_suggestion: Set(body.profile_suggestion),
        objective: Set(body.objective),
        requirement_education: Set(body.requirement_education),
        requirement_experience: Set(body.requirement_experience),
        requirement_attention: Set(body.requirement_attention),
        requirement_knowledge: Set(body.requirement_knowledge),
        requirement_skill: Set(body.requirement_skill),
        requirement_attitude: Set(body.requirement_attitude),
        requirement_delivery: Set(body.requirement_delivery),
        observation: Set(body.observation),
        active: Set(true),
        ..Default::default()
    }
    .insert(ctx.tenant_db.as_ref())
    .await
    .map_err(|_| AuthRejection::Internal)?;

    Ok((StatusCode::CREATED, Json(RoleView::from(created))).into_response())
}

/// `PATCH /roles/{id}` — updates the fields present in the body. Requires
/// `role.update`. Unknown id → `404`.
pub async fn update(
    ctx: TenantContext,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateRole>,
) -> Result<Response, AuthRejection> {
    ctx.require(Resource::RoleUpdate).await?;

    let Some(model) = role::Entity::find_by_id(id)
        .one(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?
    else {
        return Ok(not_found());
    };

    let mut active_model: role::ActiveModel = model.into();
    if let Some(name) = body.name {
        active_model.name = Set(name);
    }
    if let Some(value) = body.profile_suggestion {
        active_model.profile_suggestion = Set(Some(value));
    }
    if let Some(value) = body.objective {
        active_model.objective = Set(Some(value));
    }
    if let Some(value) = body.requirement_education {
        active_model.requirement_education = Set(Some(value));
    }
    if let Some(value) = body.requirement_experience {
        active_model.requirement_experience = Set(Some(value));
    }
    if let Some(value) = body.requirement_attention {
        active_model.requirement_attention = Set(Some(value));
    }
    if let Some(value) = body.requirement_knowledge {
        active_model.requirement_knowledge = Set(Some(value));
    }
    if let Some(value) = body.requirement_skill {
        active_model.requirement_skill = Set(Some(value));
    }
    if let Some(value) = body.requirement_attitude {
        active_model.requirement_attitude = Set(Some(value));
    }
    if let Some(value) = body.requirement_delivery {
        active_model.requirement_delivery = Set(Some(value));
    }
    if let Some(value) = body.observation {
        active_model.observation = Set(Some(value));
    }
    if let Some(active) = body.active {
        active_model.active = Set(active);
    }
    let updated = active_model
        .update(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?;

    Ok(Json(RoleView::from(updated)).into_response())
}

/// `DELETE /roles/{id}` — deactivates a role (soft delete). Requires
/// `role.delete`. Unknown id → `404`; success → `204`.
pub async fn delete(ctx: TenantContext, Path(id): Path<Uuid>) -> Result<Response, AuthRejection> {
    ctx.require(Resource::RoleDelete).await?;

    let Some(model) = role::Entity::find_by_id(id)
        .one(ctx.tenant_db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?
    else {
        return Ok(not_found());
    };

    let mut active_model: role::ActiveModel = model.into();
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
        Json(serde_json::json!({ "error": "role not found" })),
    )
        .into_response()
}
