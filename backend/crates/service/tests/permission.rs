//! Integration test for the RBAC permission check. Provisions a tenant and
//! walks the seeded profile → task → resource chain for its admin, against a
//! throwaway PostgreSQL.
//!
//! Requires Docker to be available to the test runner.

use migration::{MigratorTrait, PublicMigrator};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use service::permission::{has_permission, Resource};
use service::provisioning::{provision_organization, NewOrganization};
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers_modules::postgres::Postgres;
use uuid::Uuid;

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

async fn tenant_conn(url: &str, schema: &str) -> DatabaseConnection {
    let mut options = ConnectOptions::new(url.to_owned());
    options.set_schema_search_path(schema.to_owned());
    Database::connect(options).await.expect("tenant connect")
}

#[tokio::test]
async fn admin_is_granted_catalog_resources_through_its_profile() {
    let (db, url) = setup().await;
    let provisioned = provision_organization(
        &db,
        &url,
        NewOrganization {
            name: "acme".to_owned(),
            plan: None,
            admin_name: "Admin".to_owned(),
            admin_email: "admin@acme.test".to_owned(),
            admin_password: "s3cret".to_owned(),
        },
    )
    .await
    .expect("provisioning should succeed");
    let tenant = tenant_conn(&url, "acme").await;

    // The admin holds the "administrator" profile, which grants every resource.
    assert!(
        has_permission(&tenant, provisioned.admin.id, Resource::UserRead)
            .await
            .expect("check")
    );
    assert!(
        has_permission(&tenant, provisioned.admin.id, Resource::ProfileManage)
            .await
            .expect("check")
    );
}

#[tokio::test]
async fn an_unlinked_user_is_granted_nothing() {
    let (db, url) = setup().await;
    provision_organization(
        &db,
        &url,
        NewOrganization {
            name: "acme".to_owned(),
            plan: None,
            admin_name: "Admin".to_owned(),
            admin_email: "admin@acme.test".to_owned(),
            admin_password: "s3cret".to_owned(),
        },
    )
    .await
    .expect("provisioning should succeed");
    let tenant = tenant_conn(&url, "acme").await;

    // A user with no profile reaches no resource.
    assert!(!has_permission(&tenant, Uuid::new_v4(), Resource::UserRead)
        .await
        .expect("check"));
}
