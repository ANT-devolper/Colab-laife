//! Integration test for credential authentication. Provisions a tenant and
//! then exercises `authenticate` against a throwaway PostgreSQL.
//!
//! Requires Docker to be available to the test runner.

use entity::organization;
use migration::{MigratorTrait, PublicMigrator};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, Set};
use service::auth::{authenticate, AuthError};
use service::provisioning::{provision_organization, NewOrganization, Provisioned};
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers_modules::postgres::Postgres;

const PASSWORD: &str = "s3cret-pass";

async fn setup() -> (DatabaseConnection, String) {
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
    (db, url)
}

/// Provisions an "acme" tenant whose admin logs in with `admin@acme.test`.
async fn provision_acme(db: &DatabaseConnection, url: &str) -> Provisioned {
    provision_organization(
        db,
        url,
        NewOrganization {
            name: "acme".to_owned(),
            plan: None,
            admin_name: "Admin".to_owned(),
            admin_email: "admin@acme.test".to_owned(),
            admin_password: PASSWORD.to_owned(),
        },
    )
    .await
    .expect("provisioning should succeed")
}

#[tokio::test]
async fn authenticate_accepts_valid_credentials() {
    let (db, url) = setup().await;
    let provisioned = provision_acme(&db, &url).await;

    let authenticated = authenticate(&db, "admin@acme.test", PASSWORD)
        .await
        .expect("valid credentials should authenticate");

    pretty_assertions::assert_eq!(authenticated.user.id, provisioned.admin.id);
    pretty_assertions::assert_eq!(authenticated.organization.name, "acme");
}

#[tokio::test]
async fn authenticate_rejects_a_wrong_password() {
    let (db, url) = setup().await;
    provision_acme(&db, &url).await;

    let result = authenticate(&db, "admin@acme.test", "wrong-password").await;

    assert!(matches!(result, Err(AuthError::InvalidCredentials)));
}

#[tokio::test]
async fn authenticate_rejects_an_unknown_email() {
    let (db, url) = setup().await;
    provision_acme(&db, &url).await;

    let result = authenticate(&db, "nobody@acme.test", PASSWORD).await;

    assert!(matches!(result, Err(AuthError::InvalidCredentials)));
}

#[tokio::test]
async fn authenticate_rejects_an_inactive_organization() {
    let (db, url) = setup().await;
    let provisioned = provision_acme(&db, &url).await;

    // Deactivate the organization the admin belongs to.
    let mut active: organization::ActiveModel = provisioned.organization.into();
    active.is_active = Set(false);
    active.update(&db).await.expect("deactivate organization");

    let result = authenticate(&db, "admin@acme.test", PASSWORD).await;

    assert!(matches!(result, Err(AuthError::OrganizationInactive)));
}
