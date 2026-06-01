//! Integration test for the authenticated `GET /auth/me` route, driven over
//! real HTTP against a throwaway PostgreSQL with a provisioned "acme" tenant.
//! Exercises the `TenantContext` extractor end to end.
//!
//! Requires Docker to be available to the test runner.

use api::build_router;
use axum::http::StatusCode;
use axum_test::TestServer;
use migration::{MigratorTrait, PublicMigrator};
use sea_orm::Database;
use serde_json::json;
use service::auth::{encode_token, Claims};
use service::provisioning::{provision_organization, NewOrganization};
use std::time::Duration;
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers_modules::postgres::Postgres;

const SECRET: &[u8] = b"test-secret-key";
const PASSWORD: &str = "s3cret-pass";

async fn setup() -> TestServer {
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
    provision_organization(
        &db,
        &url,
        NewOrganization {
            name: "acme".to_owned(),
            plan: None,
            admin_name: "Admin".to_owned(),
            admin_email: "admin@acme.test".to_owned(),
            admin_password: PASSWORD.to_owned(),
        },
    )
    .await
    .expect("provisioning should succeed");

    TestServer::new(build_router(db, url, SECRET.to_vec()))
}

/// Logs in and returns the issued session token.
async fn login(server: &TestServer) -> String {
    let response = server
        .post("/auth/login")
        .json(&json!({ "email": "admin@acme.test", "password": PASSWORD }))
        .await;
    response.assert_status(StatusCode::OK);
    response.json::<serde_json::Value>()["token"]
        .as_str()
        .expect("token string")
        .to_owned()
}

#[tokio::test]
async fn me_returns_the_identity_for_a_valid_token() {
    let server = setup().await;
    let token = login(&server).await;

    let response = server
        .get("/auth/me")
        .add_header("Authorization", format!("Bearer {token}"))
        .await;

    response.assert_status(StatusCode::OK);
    let body = response.json::<serde_json::Value>();
    pretty_assertions::assert_eq!(body["schema"], "acme");
    pretty_assertions::assert_eq!(body["is_admin"], true);
}

#[tokio::test]
async fn me_rejects_a_request_without_a_token() {
    let server = setup().await;

    server
        .get("/auth/me")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_rejects_a_token_signed_with_another_secret() {
    let server = setup().await;
    // A well-formed token for "acme" but signed with the wrong secret.
    let claims = Claims::new("u", "o", "acme", true, Duration::from_secs(3600));
    let forged = encode_token(&claims, b"not-the-server-secret").expect("encode");

    server
        .get("/auth/me")
        .add_header("Authorization", format!("Bearer {forged}"))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}
