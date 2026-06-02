//! Integration test for the RBAC-guarded `/expectation-items` CRUD routes. An
//! expectation-contract item lives in the tenant schema and references a feedback;
//! `kind` discriminates a goal from a behavior. Provisions a tenant, drives the
//! routes over real HTTP, and covers the authorized admin (including validation of
//! the referenced feedback and the kind) and a member that lacks the permission.
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

/// Creates a collaborator and a feedback for it, returning the feedback id.
async fn create_feedback(server: &TestServer, auth: &str) -> String {
    let collaborator = server
        .post("/collaborators")
        .add_header("Authorization", auth)
        .json(&json!({ "name": "Bob Report" }))
        .await;
    collaborator.assert_status(StatusCode::CREATED);
    let collaborator_id = collaborator.json::<serde_json::Value>()["id"]
        .as_str()
        .expect("collaborator id")
        .to_owned();

    let feedback = server
        .post("/feedbacks")
        .add_header("Authorization", auth)
        .json(&json!({
            "collaborator_id": collaborator_id,
            "feedback_date": "2026-06-01T10:00:00Z"
        }))
        .await;
    feedback.assert_status(StatusCode::CREATED);
    feedback.json::<serde_json::Value>()["id"]
        .as_str()
        .expect("feedback id")
        .to_owned()
}

#[tokio::test]
async fn admin_can_create_list_update_and_deactivate_expectation_items() {
    let (server, _db, _provisioned) = setup().await;
    let token = token(&server, "admin@acme.test", ADMIN_PASSWORD).await;
    let auth = format!("Bearer {token}");

    let feedback_id = create_feedback(&server, &auth).await;

    // Create a goal.
    let created = server
        .post("/expectation-items")
        .add_header("Authorization", &auth)
        .json(&json!({
            "feedback_id": feedback_id,
            "kind": "goal",
            "description": "Ship the feature"
        }))
        .await;
    created.assert_status(StatusCode::CREATED);
    let body = created.json::<serde_json::Value>();
    let id = body["id"].as_str().expect("id").to_owned();
    assert_eq!(body["feedback_id"], feedback_id);
    assert_eq!(body["kind"], "goal");
    assert_eq!(body["done"], false);
    assert_eq!(body["active"], true);

    // Also create a behavior under the same feedback.
    server
        .post("/expectation-items")
        .add_header("Authorization", &auth)
        .json(&json!({
            "feedback_id": feedback_id,
            "kind": "behavior",
            "description": "Communicate early"
        }))
        .await
        .assert_status(StatusCode::CREATED);

    // List filtered by feedback shows both; filtered by kind shows one.
    let all = server
        .get(&format!("/expectation-items?feedback_id={feedback_id}"))
        .add_header("Authorization", &auth)
        .await;
    all.assert_status(StatusCode::OK);
    assert_eq!(all.json::<serde_json::Value>().as_array().unwrap().len(), 2);

    let goals = server
        .get(&format!(
            "/expectation-items?feedback_id={feedback_id}&kind=goal"
        ))
        .add_header("Authorization", &auth)
        .await;
    goals.assert_status(StatusCode::OK);
    let goals_body = goals.json::<serde_json::Value>();
    assert_eq!(goals_body.as_array().unwrap().len(), 1);
    assert_eq!(goals_body[0]["kind"], "goal");

    // Mark the goal done.
    let updated = server
        .patch(&format!("/expectation-items/{id}"))
        .add_header("Authorization", &auth)
        .json(&json!({ "done": true }))
        .await;
    updated.assert_status(StatusCode::OK);
    assert_eq!(updated.json::<serde_json::Value>()["done"], true);

    // Deactivate (soft delete).
    server
        .delete(&format!("/expectation-items/{id}"))
        .add_header("Authorization", &auth)
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let after = server
        .get(&format!("/expectation-items?feedback_id={feedback_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(
        after.json::<serde_json::Value>().as_array().unwrap().len(),
        1,
        "the deactivated item should no longer be listed"
    );
}

#[tokio::test]
async fn creating_with_an_invalid_kind_is_rejected() {
    let (server, _db, _provisioned) = setup().await;
    let token = token(&server, "admin@acme.test", ADMIN_PASSWORD).await;
    let auth = format!("Bearer {token}");
    let feedback_id = create_feedback(&server, &auth).await;

    server
        .post("/expectation-items")
        .add_header("Authorization", &auth)
        .json(&json!({ "feedback_id": feedback_id, "kind": "nonsense" }))
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn creating_for_an_unknown_feedback_is_rejected() {
    let (server, _db, _provisioned) = setup().await;
    let token = token(&server, "admin@acme.test", ADMIN_PASSWORD).await;
    let auth = format!("Bearer {token}");

    server
        .post("/expectation-items")
        .add_header("Authorization", &auth)
        .json(&json!({ "feedback_id": Uuid::new_v4().to_string(), "kind": "goal" }))
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);
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
        .get("/expectation-items")
        .add_header("Authorization", format!("Bearer {token}"))
        .await
        .assert_status(StatusCode::FORBIDDEN);
}
