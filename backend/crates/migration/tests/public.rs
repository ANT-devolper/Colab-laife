//! Integration test for the public-schema migrations. Applies `PublicMigrator`
//! against a throwaway PostgreSQL and round-trips through the entities.
//!
//! Requires Docker to be available to the test runner.

use migration::PublicMigrator;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use sea_orm_migration::MigratorTrait;
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers_modules::postgres::Postgres;

async fn migrated_db() -> DatabaseConnection {
    let container = Postgres::default()
        .with_tag("16-alpine")
        .start()
        .await
        .expect("failed to start postgres container");
    let host = container.get_host().await.expect("host");
    let port = container.get_host_port_ipv4(5432).await.expect("port");
    // Keep the container alive for the duration of the test; process exit tears it down.
    Box::leak(Box::new(container));

    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
    let db = Database::connect(url).await.expect("connect");
    PublicMigrator::up(&db, None)
        .await
        .expect("public migrations should apply");
    db
}

fn acme() -> entity::organization::ActiveModel {
    entity::organization::ActiveModel {
        id: Set(sea_orm::prelude::Uuid::new_v4()),
        name: Set("acme".to_owned()),
        ..Default::default()
    }
}

#[tokio::test]
async fn public_migrator_creates_and_persists_organizations() {
    let db = migrated_db().await;

    let inserted = acme().insert(&db).await.expect("insert organization");

    let found = entity::organization::Entity::find_by_id(inserted.id)
        .one(&db)
        .await
        .expect("query organization")
        .expect("organization should exist");

    pretty_assertions::assert_eq!(found.name, "acme");
    // Column defaults defined by the migration apply on insert.
    pretty_assertions::assert_eq!(found.plan, "FREE");
    pretty_assertions::assert_eq!(found.employee_limit, 10);
    assert!(found.is_active);
}

#[tokio::test]
async fn organizations_name_is_unique() {
    let db = migrated_db().await;

    acme().insert(&db).await.expect("first insert");
    let duplicate = acme().insert(&db).await;

    assert!(duplicate.is_err(), "duplicate name must be rejected");
    let count = entity::organization::Entity::find()
        .all(&db)
        .await
        .expect("count organizations")
        .len();
    pretty_assertions::assert_eq!(count, 1);
}
