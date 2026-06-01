//! Integration test for `TenantRegistry`. Verifies that a resolved connection
//! is pinned to the requested schema and that the registry caches one
//! connection per schema, against a throwaway PostgreSQL.
//!
//! Requires Docker to be available to the test runner.

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};
use service::tenant::{TenantError, TenantRegistry};
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
    (db, url)
}

#[tokio::test]
async fn connection_is_pinned_to_the_requested_schema() {
    let (admin, url) = setup().await;
    admin
        .execute_unprepared("CREATE SCHEMA \"acme\"")
        .await
        .expect("create schema");

    let registry = TenantRegistry::new(url);
    let conn = registry.connection("acme").await.expect("resolve acme");

    // The search_path resolves to the tenant schema.
    let row = conn
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT current_schema() AS schema",
        ))
        .await
        .expect("query current_schema")
        .expect("one row");
    let current: String = row.try_get("", "schema").expect("read schema");
    pretty_assertions::assert_eq!(current, "acme");
}

#[tokio::test]
async fn repeated_resolution_reuses_the_same_connection() {
    let (admin, url) = setup().await;
    admin
        .execute_unprepared("CREATE SCHEMA \"acme\"")
        .await
        .expect("create schema");

    let registry = TenantRegistry::new(url);
    let first = registry.connection("acme").await.expect("first");
    let second = registry.connection("acme").await.expect("second");

    assert!(
        std::sync::Arc::ptr_eq(&first, &second),
        "the registry should cache one connection per schema"
    );
}

#[tokio::test]
async fn rejects_an_invalid_schema_name() {
    let (_admin, url) = setup().await;
    let registry = TenantRegistry::new(url);

    let result = registry.connection("Bad Name!").await;

    assert!(matches!(result, Err(TenantError::InvalidSchema)));
}
