use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use service::auth::{authenticate, encode_token, AuthError, Authenticated, Claims, DEFAULT_TTL};

use crate::extract::TenantContext;
use crate::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
    token_type: &'static str,
}

/// `POST /auth/login` — exchanges credentials for a session JWT. Invalid
/// credentials map to `401`, a deactivated organization to `403`, and any
/// internal failure to `500`.
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> impl IntoResponse {
    match authenticate(&state.db, &body.email, &body.password).await {
        Ok(Authenticated { user, organization }) => {
            let claims = Claims::new(
                user.id.to_string(),
                organization.id.to_string(),
                organization.name,
                user.is_admin,
                DEFAULT_TTL,
            );
            match encode_token(&claims, &state.jwt_secret) {
                Ok(token) => (
                    StatusCode::OK,
                    Json(LoginResponse {
                        token,
                        token_type: "Bearer",
                    }),
                )
                    .into_response(),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "internal error" })),
                )
                    .into_response(),
            }
        }
        Err(AuthError::InvalidCredentials) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid credentials" })),
        )
            .into_response(),
        Err(AuthError::OrganizationInactive) => (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "organization is inactive" })),
        )
            .into_response(),
        Err(AuthError::Db(_)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "internal error" })),
        )
            .into_response(),
    }
}

/// `GET /auth/me` — returns the authenticated caller's identity, drawn from the
/// verified token. Requires a valid `Authorization: Bearer` token (`401`
/// otherwise, via the `TenantContext` extractor).
pub async fn me(ctx: TenantContext) -> impl IntoResponse {
    Json(json!({
        "user_id": ctx.claims.sub,
        "organization_id": ctx.claims.org,
        "schema": ctx.claims.schema,
        "is_admin": ctx.claims.is_admin,
    }))
}
