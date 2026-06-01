use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use service::provisioning::{provision_organization, NewOrganization, ProvisionError, Provisioned};

use crate::AppState;

#[derive(Deserialize)]
pub struct CreateOrganization {
    name: String,
    #[serde(default)]
    plan: Option<String>,
    admin: AdminInput,
}

#[derive(Deserialize)]
struct AdminInput {
    name: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
struct OrganizationCreated {
    id: String,
    name: String,
    admin: AdminCreated,
}

#[derive(Serialize)]
struct AdminCreated {
    id: String,
    email: String,
}

/// `POST /organizations` — provisions a tenant (schema + admin). Validation and
/// duplicate detection are mapped to client errors; everything else is a 500.
pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateOrganization>,
) -> impl IntoResponse {
    let input = NewOrganization {
        name: body.name,
        plan: body.plan,
        admin_name: body.admin.name,
        admin_email: body.admin.email,
        admin_password: body.admin.password,
    };

    match provision_organization(&state.db, &state.database_url, input).await {
        Ok(Provisioned {
            organization,
            admin,
        }) => (
            StatusCode::CREATED,
            Json(OrganizationCreated {
                id: organization.id.to_string(),
                name: organization.name,
                admin: AdminCreated {
                    id: admin.id.to_string(),
                    email: admin.email,
                },
            }),
        )
            .into_response(),
        Err(ProvisionError::InvalidName) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid organization name" })),
        )
            .into_response(),
        Err(ProvisionError::NameTaken) => (
            StatusCode::CONFLICT,
            Json(json!({ "error": "organization name already taken" })),
        )
            .into_response(),
        Err(ProvisionError::Hash) | Err(ProvisionError::Db(_)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "internal error" })),
        )
            .into_response(),
    }
}
