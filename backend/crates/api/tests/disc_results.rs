//! Integration test for the RBAC-guarded `/disc-results` routes. A DISC result
//! lives in the tenant schema and references a collaborator. Provisions a tenant,
//! drives the routes over real HTTP, and covers the authorized admin (including the
//! derived primary/secondary profile and referenced-collaborator validation) and a
//! member that lacks the permission.
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

async fn create_collaborator(server: &TestServer, auth: &str, name: &str) -> String {
    let response = server
        .post("/collaborators")
        .add_header("Authorization", auth)
        .json(&json!({ "name": name }))
        .await;
    response.assert_status(StatusCode::CREATED);
    response.json::<serde_json::Value>()["id"]
        .as_str()
        .expect("collaborator id")
        .to_owned()
}

#[tokio::test]
async fn admin_can_record_list_and_delete_a_disc_result() {
    let (server, _db, _provisioned) = setup().await;
    let token = token(&server, "admin@acme.test", ADMIN_PASSWORD).await;
    let auth = format!("Bearer {token}");

    let collaborator_id = create_collaborator(&server, &auth, "Bob Report").await;

    // Record a result; the highest score is the communicator, the second the analyst.
    let created = server
        .post("/disc-results")
        .add_header("Authorization", &auth)
        .json(&json!({
            "collaborator_id": collaborator_id,
            "executor": 10,
            "communicator": 25,
            "planner": 5,
            "analyst": 18
        }))
        .await;
    created.assert_status(StatusCode::CREATED);
    let body = created.json::<serde_json::Value>();
    let id = body["id"].as_str().expect("id").to_owned();
    assert_eq!(body["collaborator_id"], collaborator_id);
    assert_eq!(body["primary_profile"], "communicator");
    assert_eq!(body["secondary_profile"], "analyst");

    // List (filtered by collaborator) shows it with the derived profile.
    let listed = server
        .get(&format!("/disc-results?collaborator_id={collaborator_id}"))
        .add_header("Authorization", &auth)
        .await;
    listed.assert_status(StatusCode::OK);
    let results = listed.json::<serde_json::Value>();
    assert!(
        results
            .as_array()
            .expect("array")
            .iter()
            .any(|r| r["id"] == id),
        "the recorded result should be listed"
    );

    // Delete it (hard delete — results are immutable history).
    server
        .delete(&format!("/disc-results/{id}"))
        .add_header("Authorization", &auth)
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let after = server
        .get("/disc-results")
        .add_header("Authorization", &auth)
        .await;
    after.assert_status(StatusCode::OK);
    assert!(
        !after
            .json::<serde_json::Value>()
            .as_array()
            .expect("array")
            .iter()
            .any(|r| r["id"] == id),
        "a deleted result should not be listed"
    );
}

#[tokio::test]
async fn recording_for_an_unknown_collaborator_is_rejected() {
    let (server, _db, _provisioned) = setup().await;
    let token = token(&server, "admin@acme.test", ADMIN_PASSWORD).await;
    let auth = format!("Bearer {token}");

    let response = server
        .post("/disc-results")
        .add_header("Authorization", &auth)
        .json(&json!({
            "collaborator_id": Uuid::new_v4().to_string(),
            "executor": 1,
            "communicator": 2,
            "planner": 3,
            "analyst": 4
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_user_without_the_permission_is_forbidden() {
    let (server, db, provisioned) = setup().await;

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
        .get("/disc-results")
        .add_header("Authorization", format!("Bearer {token}"))
        .await
        .assert_status(StatusCode::FORBIDDEN);
}
