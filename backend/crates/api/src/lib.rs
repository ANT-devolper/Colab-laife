use axum::routing::{get, patch, post};
use axum::Router;
use sea_orm::DatabaseConnection;
use service::tenant::TenantRegistry;
use std::path::Path;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

mod auth;
mod collaborators;
mod expectation_items;
pub mod extract;
mod feedback;
mod feedback_behaviors;
mod health;
mod organizations;
mod roles;
mod sectors;
mod users;

/// Shared, cheaply cloneable application state. `DatabaseConnection` is not
/// `Clone`, so it lives behind an `Arc`; `database_url` lets handlers open the
/// search-path connections that tenant provisioning needs; `tenants` resolves a
/// per-request connection to the caller's tenant schema; `jwt_secret` signs and
/// verifies session tokens.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub database_url: Arc<str>,
    pub tenants: Arc<TenantRegistry>,
    pub jwt_secret: Arc<[u8]>,
}

/// Builds the application router with its shared state. Kept separate from
/// `main` so tests can drive the real routes over HTTP.
pub fn build_router(
    db: DatabaseConnection,
    database_url: impl Into<String>,
    jwt_secret: impl Into<Vec<u8>>,
) -> Router {
    let database_url = database_url.into();
    let state = AppState {
        db: Arc::new(db),
        database_url: Arc::from(database_url.clone()),
        tenants: Arc::new(TenantRegistry::new(database_url)),
        jwt_secret: Arc::from(jwt_secret.into()),
    };
    Router::new()
        .route("/health", get(health::health))
        .route("/health/ready", get(health::ready))
        .route("/organizations", post(organizations::create))
        .route("/auth/login", post(auth::login))
        .route("/auth/me", get(auth::me))
        .route("/users", get(users::list))
        .route("/sectors", get(sectors::list).post(sectors::create))
        .route(
            "/sectors/{id}",
            patch(sectors::update).delete(sectors::delete),
        )
        .route("/roles", get(roles::list).post(roles::create))
        .route("/roles/{id}", patch(roles::update).delete(roles::delete))
        .route(
            "/collaborators",
            get(collaborators::list).post(collaborators::create),
        )
        .route(
            "/collaborators/{id}",
            patch(collaborators::update).delete(collaborators::delete),
        )
        .route("/feedbacks", get(feedback::list).post(feedback::create))
        .route(
            "/feedbacks/{id}",
            patch(feedback::update).delete(feedback::delete),
        )
        .route(
            "/expectation-items",
            get(expectation_items::list).post(expectation_items::create),
        )
        .route(
            "/expectation-items/{id}",
            patch(expectation_items::update).delete(expectation_items::delete),
        )
        .route(
            "/feedback-behaviors",
            get(feedback_behaviors::list).post(feedback_behaviors::create),
        )
        .route(
            "/feedback-behaviors/{id}",
            patch(feedback_behaviors::update).delete(feedback_behaviors::delete),
        )
        .with_state(state)
}

/// Wraps an API router so requests that match no API route are served from the
/// built Elm SPA in `dist_dir` (single origin, no CORS — see ADR 0011). Real
/// files (the compiled JS, assets) are returned from disk; any other path falls
/// back to `index.html` so the SPA can boot and route on the client. API routes
/// keep precedence: they are matched before this fallback runs.
pub fn with_static_spa(router: Router, dist_dir: impl AsRef<Path>) -> Router {
    let dist = dist_dir.as_ref();
    let serve = ServeDir::new(dist).fallback(ServeFile::new(dist.join("index.html")));
    router.fallback_service(serve)
}

#[cfg(test)]
mod tests {
    use axum_test::TestServer;
    use pretty_assertions::assert_eq;
    use sea_orm::{DatabaseBackend, MockDatabase};
    use serde_json::json;

    // A mock connection lets us build the full router without a real database,
    // so the liveness probe can be exercised without Docker.
    fn mock_router() -> axum::Router {
        let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
        crate::build_router(db, "postgres://unused", b"test-secret".to_vec())
    }

    #[tokio::test]
    async fn health_reports_alive_without_touching_the_database() {
        let server = TestServer::new(mock_router());

        let response = server.get("/health").await;

        response.assert_status_ok();
        assert_eq!(
            response.json::<serde_json::Value>(),
            json!({ "status": "ok" })
        );
    }
}
