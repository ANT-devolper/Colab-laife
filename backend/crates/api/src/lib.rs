use axum::routing::{get, patch, post};
use axum::Router;
use sea_orm::DatabaseConnection;
use service::tenant::TenantRegistry;
use std::sync::Arc;

mod auth;
pub mod extract;
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
        .with_state(state)
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
