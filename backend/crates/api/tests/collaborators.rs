//! Integration test for the RBAC-guarded `/collaborators` CRUD routes.
//! Collaborators live in the tenant schema and reference a sector, a role and an
//! optional manager (self-reference). Provisions a tenant and drives the routes
//! over real HTTP, covering the authorized admin (including referenced-entity
//! validation) and a member that lacks the permission.
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

/// Creates a sector via the API and returns its id.
async fn create_sector(server: &TestServer, auth: &str, name: &str) -> String {
    let response = server
        .post("/sectors")
        .add_header("Authorization", auth)
        .json(&json!({ "name": name }))
        .await;
    response.assert_status(StatusCode::CREATED);
    response.json::<serde_json::Value>()["id"]
        .as_str()
        .expect("sector id")
        .to_owned()
}

/// Creates a role via the API and returns its id.
async fn create_role(server: &TestServer, auth: &str, name: &str) -> String {
    let response = server
        .post("/roles")
        .add_header("Authorization", auth)
        .json(&json!({ "name": name }))
        .await;
    response.assert_status(StatusCode::CREATED);
    response.json::<serde_json::Value>()["id"]
        .as_str()
        .expect("role id")
        .to_owned()
}

#[tokio::test]
async fn admin_can_create_list_update_and_deactivate_collaborators() {
    let (server, _db, _provisioned) = setup().await;
    let token = token(&server, "admin@acme.test", ADMIN_PASSWORD).await;
    let auth = format!("Bearer {token}");

    let sector_id = create_sector(&server, &auth, "Engineering").await;
    let role_id = create_role(&server, &auth, "Backend Engineer").await;

    // Create a manager first, then a report that points at it.
    let manager = server
        .post("/collaborators")
        .add_header("Authorization", &auth)
        .json(&json!({
            "name": "Alice Manager",
            "sector_id": sector_id,
            "role_id": role_id,
            "is_manager": true,
            "email": "alice@acme.test"
        }))
        .await;
    manager.assert_status(StatusCode::CREATED);
    let manager_body = manager.json::<serde_json::Value>();
    let manager_id = manager_body["id"].as_str().expect("id").to_owned();
    assert_eq!(manager_body["name"], "Alice Manager");
    assert_eq!(manager_body["sector_id"], sector_id);
    assert_eq!(manager_body["role_id"], role_id);
    assert_eq!(manager_body["is_manager"], true);
    assert_eq!(manager_body["active"], true);

    let created = server
        .post("/collaborators")
        .add_header("Authorization", &auth)
        .json(&json!({
            "name": "Bob Report",
            "sector_id": sector_id,
            "role_id": role_id,
            "manager_id": manager_id
        }))
        .await;
    created.assert_status(StatusCode::CREATED);
    let body = created.json::<serde_json::Value>();
    let id = body["id"].as_str().expect("id").to_owned();
    assert_eq!(body["manager_id"], manager_id);
    assert_eq!(body["is_manager"], false);

    // List shows both.
    let listed = server
        .get("/collaborators")
        .add_header("Authorization", &auth)
        .await;
    listed.assert_status(StatusCode::OK);
    let collaborators = listed.json::<serde_json::Value>();
    let array = collaborators.as_array().expect("array");
    assert!(array.iter().any(|c| c["id"] == id));
    assert!(array.iter().any(|c| c["id"] == manager_id));

    // Update the report's name and detach the manager-flag-less fields.
    let updated = server
        .patch(&format!("/collaborators/{id}"))
        .add_header("Authorization", &auth)
        .json(&json!({ "name": "Bob Senior", "whatsapp": "+5511999999999" }))
        .await;
    updated.assert_status(StatusCode::OK);
    let updated_body = updated.json::<serde_json::Value>();
    assert_eq!(updated_body["name"], "Bob Senior");
    assert_eq!(updated_body["whatsapp"], "+5511999999999");
    // Untouched FK is preserved.
    assert_eq!(updated_body["manager_id"], manager_id);

    // Deactivate (soft delete).
    server
        .delete(&format!("/collaborators/{id}"))
        .add_header("Authorization", &auth)
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let after = server
        .get("/collaborators")
        .add_header("Authorization", &auth)
        .await;
    after.assert_status(StatusCode::OK);
    assert!(
        !after
            .json::<serde_json::Value>()
            .as_array()
            .expect("array")
            .iter()
            .any(|c| c["id"] == id),
        "a deactivated collaborator should not be listed"
    );
}

#[tokio::test]
async fn creating_with_an_unknown_sector_is_rejected() {
    let (server, _db, _provisioned) = setup().await;
    let token = token(&server, "admin@acme.test", ADMIN_PASSWORD).await;
    let auth = format!("Bearer {token}");

    let response = server
        .post("/collaborators")
        .add_header("Authorization", &auth)
        .json(&json!({
            "name": "Dangling",
            "sector_id": Uuid::new_v4().to_string()
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn creating_with_an_unknown_manager_is_rejected() {
    let (server, _db, _provisioned) = setup().await;
    let token = token(&server, "admin@acme.test", ADMIN_PASSWORD).await;
    let auth = format!("Bearer {token}");

    let response = server
        .post("/collaborators")
        .add_header("Authorization", &auth)
        .json(&json!({
            "name": "Orphan",
            "manager_id": Uuid::new_v4().to_string()
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
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
        .get("/collaborators")
        .add_header("Authorization", format!("Bearer {token}"))
        .await
        .assert_status(StatusCode::FORBIDDEN);
}
