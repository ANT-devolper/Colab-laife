use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use entity::user;
use sea_orm::prelude::Uuid;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Serialize;
use service::permission::Resource;

use crate::extract::{AuthRejection, TenantContext};
use crate::AppState;

#[derive(Serialize)]
struct UserView {
    id: String,
    name: String,
    email: String,
    is_admin: bool,
}

/// `GET /users` — lists the (non-deleted) users of the caller's organization.
/// First route guarded by RBAC: requires the `user.read` resource (`403`
/// otherwise). Identity tables live in `public`, so the listing uses the public
/// connection while authorization is checked against the tenant schema.
pub async fn list(
    State(state): State<AppState>,
    ctx: TenantContext,
) -> Result<impl IntoResponse, AuthRejection> {
    ctx.require(Resource::UserRead).await?;

    let organization_id = Uuid::parse_str(&ctx.claims.org).map_err(|_| AuthRejection::Internal)?;
    let users = user::Entity::find()
        .filter(user::Column::OrganizationId.eq(organization_id))
        .filter(user::Column::Deleted.eq(false))
        .all(state.db.as_ref())
        .await
        .map_err(|_| AuthRejection::Internal)?;

    let body: Vec<UserView> = users
        .into_iter()
        .map(|u| UserView {
            id: u.id.to_string(),
            name: u.name,
            email: u.email,
            is_admin: u.is_admin,
        })
        .collect();
    Ok(Json(body))
}
