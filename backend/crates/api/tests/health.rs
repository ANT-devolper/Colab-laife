//! Integration test for the readiness probe. Spins up a throwaway PostgreSQL
//! via testcontainers and drives the real router over HTTP, so it exercises the
//! whole stack (Axum + SeaORM + a live database).
//!
//! Requires Docker to be available to the test runner.

use api::build_router;
use axum_test::TestServer;
use sea_orm::Database;
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers_modules::postgres::Postgres;

#[tokio::test]
async fn ready_reports_ready_when_postgres_is_reachable() {
    let container = Postgres::default()
        .with_tag("16-alpine")
        .start()
        .await
        .expect("failed to start postgres container");
    let host = container.get_host().await.expect("failed to read host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("failed to read mapped port");
    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let db = Database::connect(&url)
        .await
        .expect("failed to connect to postgres");
    let server = TestServer::new(build_router(db, url));

    let response = server.get("/health/ready").await;

    response.assert_status_ok();
}
