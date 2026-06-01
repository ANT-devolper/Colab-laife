//! Integration test for the RBAC-guarded `/sectors` CRUD routes. Sectors live in
//! the tenant schema, so this also exercises the `TenantContext::tenant_db`
//! connection end to end. Provisions a tenant and drives the routes over real
//! HTTP, covering the authorized admin and a member that lacks the permission.
//!
//! Requires Docker to be available to the test runner.

use api::build_router;
use axum::http::StatusCode;
use axum_test::TestServer;
use entity::user;
use migration::{MigratorTrait, PublicMigrator};
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, Set};
use serde_json::json;
use service::password::hash_password;
use service::provisioning::{provision_organization, NewOrganization, Provisioned};
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers_modules::postgres::Postgres;

const SECRET: &[u8] = b"test-secret-key";
const ADMIN_PASSWORD: &str = "s3cret-pass";
const MEMBER_PASSWORD: &str = "member-pass";

/// Provisions "acme" and returns the test server, a public connection (to seed
/// extra rows) and the provisioning result.
async fn setup() -> (TestServer, DatabaseConnection, Provisioned) {
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
    let provisioned = provision_organization(
        &db,
        &url,
        NewOrganization {
            name: "acme".to_owned(),
            plan: None,
            admin_name: "Admin".to_owned(),
            admin_email: "admin@acme.test".to_owned(),
            admin_password: ADMIN_PASSWORD.to_owned(),
        },
    )
    .await
    .expect("provisioning should succeed");

    let server = TestServer::new(build_router(db, url.clone(), SECRET.to_vec()));
    let test_db = Database::connect(&url).await.expect("connect");
    (server, test_db, provisioned)
}

/// Logs in with `email`/`password` and returns the session token.
async fn token(server: &TestServer, email: &str, password: &str) -> String {
    let response = server
        .post("/auth/login")
        .json(&json!({ "email": email, "password": password }))
        .await;
    response.assert_status(StatusCode::OK);
    response.json::<serde_json::Value>()["token"]
        .as_str()
        .expect("token string")
        .to_owned()
}

#[tokio::test]
async fn admin_can_create_list_update_and_deactivate_sectors() {
    let (server, _db, _provisioned) = setup().await;
    let token = token(&server, "admin@acme.test", ADMIN_PASSWORD).await;
    let auth = format!("Bearer {token}");

    // Create.
    let created = server
        .post("/sectors")
        .add_header("Authorization", &auth)
        .json(&json!({ "name": "Engineering" }))
        .await;
    created.assert_status(StatusCode::CREATED);
    let body = created.json::<serde_json::Value>();
    let id = body["id"].as_str().expect("id").to_owned();
    assert_eq!(body["name"], "Engineering");
    assert_eq!(body["active"], true);

    // List shows it.
    let listed = server
        .get("/sectors")
        .add_header("Authorization", &auth)
        .await;
    listed.assert_status(StatusCode::OK);
    let sectors = listed.json::<serde_json::Value>();
    assert!(
        sectors
            .as_array()
            .expect("array")
            .iter()
            .any(|s| s["id"] == id && s["name"] == "Engineering"),
        "the created sector should be listed"
    );

    // Update the name.
    let updated = server
        .patch(&format!("/sectors/{id}"))
        .add_header("Authorization", &auth)
        .json(&json!({ "name": "Platform" }))
        .await;
    updated.assert_status(StatusCode::OK);
    assert_eq!(updated.json::<serde_json::Value>()["name"], "Platform");

    // Deactivate (soft delete).
    server
        .delete(&format!("/sectors/{id}"))
        .add_header("Authorization", &auth)
        .await
        .assert_status(StatusCode::NO_CONTENT);

    // The list no longer shows it.
    let after = server
        .get("/sectors")
        .add_header("Authorization", &auth)
        .await;
    after.assert_status(StatusCode::OK);
    assert!(
        !after
            .json::<serde_json::Value>()
            .as_array()
            .expect("array")
            .iter()
            .any(|s| s["id"] == id),
        "a deactivated sector should not be listed"
    );
}

#[tokio::test]
async fn a_user_without_the_permission_is_forbidden() {
    let (server, db, provisioned) = setup().await;

    // A second user in the same org, with no profile → no permissions.
    user::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Member".to_owned()),
        email: Set("member@acme.test".to_owned()),
        password_hash: Set(hash_password(MEMBER_PASSWORD).expect("hash")),
        is_admin: Set(false),
        organization_id: Set(provisioned.organization.id),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("insert member");

    let token = token(&server, "member@acme.test", MEMBER_PASSWORD).await;

    server
        .get("/sectors")
        .add_header("Authorization", format!("Bearer {token}"))
        .await
        .assert_status(StatusCode::FORBIDDEN);
}
