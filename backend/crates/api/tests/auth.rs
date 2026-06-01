//! Integration test for `POST /auth/login`: logs in over real HTTP against a
//! throwaway PostgreSQL with a provisioned "acme" tenant.
//!
//! Requires Docker to be available to the test runner.

use api::build_router;
use axum::http::StatusCode;
use axum_test::TestServer;
use entity::organization;
use migration::{MigratorTrait, PublicMigrator};
use sea_orm::{ActiveModelTrait, ColumnTrait, Database, EntityTrait, QueryFilter, Set};
use serde_json::json;
use service::auth::decode_token;
use service::provisioning::{provision_organization, NewOrganization};
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers_modules::postgres::Postgres;

const SECRET: &[u8] = b"test-secret-key";
const PASSWORD: &str = "s3cret-pass";

/// Spins a database with a provisioned "acme" tenant and returns the test
/// server plus the database URL (so a test can mutate the tenant directly).
async fn setup() -> (TestServer, String) {
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

    let server = TestServer::new(build_router(db, url.clone(), SECRET.to_vec()));
    (server, url)
}

#[tokio::test]
async fn login_returns_a_valid_token_for_good_credentials() {
    let (server, _url) = setup().await;

    let response = server
        .post("/auth/login")
        .json(&json!({ "email": "admin@acme.test", "password": PASSWORD }))
        .await;

    response.assert_status(StatusCode::OK);
    let body = response.json::<serde_json::Value>();
    pretty_assertions::assert_eq!(body["token_type"], "Bearer");
    let token = body["token"].as_str().expect("token string");

    // The token is a real, verifiable session for this tenant's admin.
    let claims = decode_token(token, SECRET).expect("token should verify");
    pretty_assertions::assert_eq!(claims.schema, "acme");
    assert!(claims.is_admin);
}

#[tokio::test]
async fn login_rejects_a_wrong_password() {
    let (server, _url) = setup().await;

    server
        .post("/auth/login")
        .json(&json!({ "email": "admin@acme.test", "password": "wrong" }))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_rejects_an_inactive_organization() {
    let (server, url) = setup().await;

    // Deactivate "acme" through a separate connection.
    let db = Database::connect(&url).await.expect("connect");
    let org = organization::Entity::find()
        .filter(organization::Column::Name.eq("acme"))
        .one(&db)
        .await
        .expect("query org")
        .expect("org exists");
    let mut active: organization::ActiveModel = org.into();
    active.is_active = Set(false);
    active.update(&db).await.expect("deactivate organization");

    server
        .post("/auth/login")
        .json(&json!({ "email": "admin@acme.test", "password": PASSWORD }))
        .await
        .assert_status(StatusCode::FORBIDDEN);
}
