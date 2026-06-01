//! Authentication extractor. `TenantContext` turns a request's
//! `Authorization: Bearer <jwt>` header into verified session claims plus a
//! connection pinned to the caller's tenant schema (see ADR 0009). Handlers opt
//! into authentication by taking it as an argument; routes that omit it stay
//! public.

use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use sea_orm::DatabaseConnection;
use serde_json::json;
use service::auth::{decode_token, Claims};

use crate::AppState;

/// A verified session: the token's claims and a connection to the caller's
/// tenant schema.
pub struct TenantContext {
    pub claims: Claims,
    pub tenant_db: Arc<DatabaseConnection>,
}

/// Why authentication failed. Token problems are `401`; an unreachable tenant
/// connection is an internal `500`.
#[derive(Debug)]
pub enum AuthRejection {
    /// No `Authorization` header, or not a `Bearer` token.
    MissingToken,
    /// The token failed signature or expiry validation.
    InvalidToken,
    /// The tenant connection could not be resolved.
    TenantUnavailable,
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "missing or malformed Authorization header",
            ),
            Self::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid or expired token"),
            Self::TenantUnavailable => (StatusCode::INTERNAL_SERVER_ERROR, "tenant unavailable"),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

/// Extracts and verifies the bearer token from the headers. Pure (no database),
/// so the parsing/validation rules are unit-testable without Docker.
fn bearer_claims(headers: &HeaderMap, secret: &[u8]) -> Result<Claims, AuthRejection> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(AuthRejection::MissingToken)?;

    decode_token(token, secret).map_err(|_| AuthRejection::InvalidToken)
}

impl FromRequestParts<AppState> for TenantContext {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let claims = bearer_claims(&parts.headers, &state.jwt_secret)?;
        let tenant_db = state
            .tenants
            .connection(&claims.schema)
            .await
            .map_err(|_| AuthRejection::TenantUnavailable)?;
        Ok(Self { claims, tenant_db })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use service::auth::{encode_token, Claims};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    const SECRET: &[u8] = b"test-secret";

    fn headers_with(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {token}").parse().expect("header value"),
        );
        headers
    }

    fn token_with(secret: &[u8], ttl: Duration) -> String {
        let claims = Claims::new("user-1", "org-1", "acme", true, ttl);
        encode_token(&claims, secret).expect("encode")
    }

    #[test]
    fn accepts_a_valid_bearer_token() {
        let headers = headers_with(&token_with(SECRET, Duration::from_secs(3600)));

        let claims = bearer_claims(&headers, SECRET).expect("valid token");

        assert_eq!(claims.schema, "acme");
        assert!(claims.is_admin);
    }

    #[test]
    fn rejects_a_missing_header() {
        let headers = HeaderMap::new();

        assert!(matches!(
            bearer_claims(&headers, SECRET),
            Err(AuthRejection::MissingToken)
        ));
    }

    #[test]
    fn rejects_a_header_without_the_bearer_scheme() {
        let mut headers = HeaderMap::new();
        let raw = token_with(SECRET, Duration::from_secs(3600));
        headers.insert(AUTHORIZATION, raw.parse().expect("header value"));

        assert!(matches!(
            bearer_claims(&headers, SECRET),
            Err(AuthRejection::MissingToken)
        ));
    }

    #[test]
    fn rejects_a_token_signed_with_another_secret() {
        let headers = headers_with(&token_with(b"other-secret", Duration::from_secs(3600)));

        assert!(matches!(
            bearer_claims(&headers, SECRET),
            Err(AuthRejection::InvalidToken)
        ));
    }

    #[test]
    fn rejects_an_expired_token() {
        // Build an already-expired token (an hour past, beyond the 60s leeway).
        let mut claims = Claims::new("user-1", "org-1", "acme", true, Duration::from_secs(3600));
        claims.exp = (SystemTime::now() - Duration::from_secs(3600))
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        let token = encode_token(&claims, SECRET).expect("encode");
        let headers = headers_with(&token);

        assert!(matches!(
            bearer_claims(&headers, SECRET),
            Err(AuthRejection::InvalidToken)
        ));
    }
}
