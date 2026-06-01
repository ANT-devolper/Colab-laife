//! Integration test for `POST /organizations`: provisions a tenant over real
//! HTTP against a throwaway PostgreSQL.
//!
//! Requires Docker to be available to the test runner.

use api::build_router;
use axum::http::StatusCode;
use axum_test::TestServer;
use migration::{MigratorTrait, PublicMigrator};
use sea_orm::Database;
use serde_json::json;
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers_modules::postgres::Postgres;

async fn server() -> TestServer {
    let container = Postgres::default()
        .with_tag("16-alpine")
        .start()
        .await
        .expect("failed to start postgres container");
    let host = container.get_host().await.expect("host");
    let port = container.get_host_port_ipv4(5432).await.expect("port");
    Box::leak(Box::new(container));

    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
    let db = Database::connect(&url).await.expect("connect");
    PublicMigrator::up(&db, None)
        .await
        .expect("public migrations should apply");
    TestServer::new(build_router(db, url))
}

#[tokio::test]
async fn post_organizations_provisions_a_tenant() {
    let server = server().await;

    let response = server
        .post("/organizations")
        .json(&json!({
            "name": "acme",
            "admin": { "name": "Admin", "email": "admin@acme.test", "password": "s3cret" }
        }))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body = response.json::<serde_json::Value>();
    pretty_assertions::assert_eq!(body["name"], "acme");
    pretty_assertions::assert_eq!(body["admin"]["email"], "admin@acme.test");
}

#[tokio::test]
async fn post_organizations_rejects_a_duplicate_name() {
    let server = server().await;

    server
        .post("/organizations")
        .json(&json!({
            "name": "acme",
            "admin": { "name": "A", "email": "a@acme.test", "password": "x" }
        }))
        .await
        .assert_status(StatusCode::CREATED);

    server
        .post("/organizations")
        .json(&json!({
            "name": "acme",
            "admin": { "name": "B", "email": "b@acme.test", "password": "x" }
        }))
        .await
        .assert_status(StatusCode::CONFLICT);
}

#[tokio::test]
async fn post_organizations_rejects_an_invalid_name() {
    let server = server().await;

    server
        .post("/organizations")
        .json(&json!({
            "name": "Bad Name!",
            "admin": { "name": "A", "email": "a@x.test", "password": "x" }
        }))
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}
