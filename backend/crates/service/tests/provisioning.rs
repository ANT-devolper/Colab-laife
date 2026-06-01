//! Integration test for tenant provisioning. Creates an organization, its
//! dedicated PostgreSQL schema (migrated with `TenantMigrator`) and the admin
//! user, against a throwaway PostgreSQL.
//!
//! Requires Docker to be available to the test runner.

use migration::{MigratorTrait, PublicMigrator};
use sea_orm::{
    ColumnTrait, ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection,
    EntityTrait, QueryFilter, Statement,
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

/// Opens a connection whose `search_path` targets `schema`, to inspect the
/// tenant's RBAC tables directly.
async fn tenant_conn(url: &str, schema: &str) -> DatabaseConnection {
    let mut options = ConnectOptions::new(url.to_owned());
    options.set_schema_search_path(schema.to_owned());
    Database::connect(options).await.expect("tenant connect")
}

#[tokio::test]
async fn provisioning_seeds_rbac_and_links_the_admin() {
    let (db, url) = setup().await;

    let provisioned = provision_organization(&db, &url, input("acme"))
        .await
        .expect("provisioning should succeed");
    let tenant = tenant_conn(&url, "acme").await;

    // The minimal resource catalog is seeded in the tenant schema.
    let resources = entity::permission::resource::Entity::find()
        .all(&tenant)
        .await
        .expect("query resources");
    assert!(!resources.is_empty(), "expected a seeded resource catalog");

    // An "administrator" profile exists and the admin holds exactly it.
    let links = entity::permission::profile_user::Entity::find()
        .filter(entity::permission::profile_user::Column::UserId.eq(provisioned.admin.id))
        .all(&tenant)
        .await
        .expect("query profile_users");
    pretty_assertions::assert_eq!(links.len(), 1, "admin should hold exactly one profile");
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
