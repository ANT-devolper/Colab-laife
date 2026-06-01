use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::AppState;

/// Liveness probe: confirms the process is up and serving. It deliberately does
/// not touch the database, so it stays green even while dependencies are down.
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

/// Readiness probe: confirms the process can reach PostgreSQL. A failing ping
/// means we should not receive traffic yet, hence `503`.
pub async fn ready(State(db): State<AppState>) -> impl IntoResponse {
    match db.ping().await {
        Ok(()) => (StatusCode::OK, Json(json!({ "status": "ready" }))),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "unavailable" })),
        ),
    }
}
