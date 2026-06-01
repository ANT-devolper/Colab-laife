//! Integration test for `TenantMigrator`. Applies the tenant-schema migrations
//! to a dedicated schema on a throwaway PostgreSQL and checks the RBAC tables
//! land there.
//!
//! Requires Docker to be available to the test runner.

use migration::{MigratorTrait, TenantMigrator};
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement,
};
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers_modules::postgres::Postgres;

const SCHEMA: &str = "tenant_acme";

/// Spins PostgreSQL, creates a tenant schema, migrates it with `TenantMigrator`
/// (over a `search_path` connection), and returns an admin connection for
/// inspection.
async fn setup() -> DatabaseConnection {
    let container = Postgres::default()
        .with_tag("16-alpine")
        .start()
        .await
        .expect("failed to start postgres container");
    let host = container.get_host().await.expect("host");
    let port = container.get_host_port_ipv4(5432).await.expect("port");
    Box::leak(Box::new(container));

    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
    let admin = Database::connect(&url).await.expect("connect");
    admin
        .execute_unprepared(&format!("CREATE SCHEMA \"{SCHEMA}\""))
        .await
        .expect("create schema");

    let mut options = ConnectOptions::new(url);
    options.set_schema_search_path(SCHEMA.to_owned());
    let tenant = Database::connect(options).await.expect("tenant connect");
    TenantMigrator::up(&tenant, None)
        .await
        .expect("tenant migrations should apply");

    admin
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
async fn tenant_migrations_create_the_rbac_tables() {
    let db = setup().await;

    for table in [
        "permission_resources",
        "permission_tasks",
        "permission_task_resources",
        "permission_profiles",
        "permission_profile_tasks",
        "permission_profile_users",
    ] {
        assert!(
            table_exists(&db, SCHEMA, table).await,
            "expected tenant table `{table}` to exist"
        );
    }
}
