//! Integration test for tenant provisioning. Creates an organization, its
//! dedicated PostgreSQL schema (migrated with `TenantMigrator`) and the admin
//! user, against a throwaway PostgreSQL.
//!
//! Requires Docker to be available to the test runner.

use migration::{MigratorTrait, PublicMigrator};
use sea_orm::{
    ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, EntityTrait, Statement,
};
use service::password::verify_password;
use service::provisioning::{provision_organization, NewOrganization};
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers_modules::postgres::Postgres;

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

fn input(name: &str) -> NewOrganization {
    NewOrganization {
        name: name.to_owned(),
        plan: None,
        admin_name: "Admin".to_owned(),
        admin_email: format!("admin@{name}.test"),
        admin_password: "s3cret".to_owned(),
    }
}

async fn schema_exists(db: &DatabaseConnection, schema: &str) -> bool {
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT 1 FROM information_schema.schemata WHERE schema_name = $1",
        [schema.into()],
    );
    db.query_one(stmt).await.expect("query schemata").is_some()
}

async fn table_exists(db: &DatabaseConnection, schema: &str, table: &str) -> bool {
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT 1 FROM information_schema.tables WHERE table_schema = $1 AND table_name = $2",
        [schema.into(), table.into()],
    );
    db.query_one(stmt).await.expect("query tables").is_some()
}

#[tokio::test]
async fn provisioning_creates_org_schema_and_admin() {
    let (db, url) = setup().await;

    let result = provision_organization(&db, &url, input("acme"))
        .await
        .expect("provisioning should succeed");

    // Public-schema rows.
    pretty_assertions::assert_eq!(result.organization.name, "acme");
    assert!(result.organization.is_active);
    pretty_assertions::assert_eq!(result.admin.organization_id, result.organization.id);
    assert!(result.admin.is_admin);
    assert!(verify_password("s3cret", &result.admin.password_hash));

    // The dedicated schema exists and was migrated (TenantMigrator ran against it).
    assert!(schema_exists(&db, "acme").await);
    assert!(table_exists(&db, "acme", "seaql_migrations").await);
}

#[tokio::test]
async fn provisioning_rejects_a_duplicate_name() {
    let (db, url) = setup().await;
    provision_organization(&db, &url, input("acme"))
        .await
        .expect("first provisioning");

    let duplicate = provision_organization(&db, &url, input("acme")).await;

    assert!(duplicate.is_err(), "duplicate tenant name must be rejected");
    let count = entity::organization::Entity::find()
        .all(&db)
        .await
        .expect("count organizations")
        .len();
    pretty_assertions::assert_eq!(count, 1);
}

#[tokio::test]
async fn provisioning_rejects_an_unsafe_name() {
    let (db, url) = setup().await;

    let bad = provision_organization(&db, &url, input("Bad Name!")).await;

    assert!(bad.is_err(), "unsafe schema name must be rejected");
    assert!(!schema_exists(&db, "Bad Name!").await);
}
