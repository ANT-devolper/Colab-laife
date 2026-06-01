use axum::routing::get;
use axum::Router;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

mod health;

/// Shared, cheaply cloneable application state. `DatabaseConnection` is not
/// `Clone`, so we hand handlers an `Arc` to it.
pub type AppState = Arc<DatabaseConnection>;

/// Builds the application router with its shared state. Kept separate from
/// `main` so tests can drive the real routes over HTTP.
pub fn build_router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/health/ready", get(health::ready))
        .with_state(Arc::new(db))
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
        crate::build_router(db)
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
