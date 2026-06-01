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
use uuid::Uuid;

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
        id: Set(Uuid::new_v4()),
        name: Set("acme".to_owned()),
        ..Default::default()
    }
}

fn user_for(organization_id: Uuid, email: &str) -> entity::user::ActiveModel {
    entity::user::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Admin".to_owned()),
        email: Set(email.to_owned()),
        password_hash: Set("hash".to_owned()),
        organization_id: Set(organization_id),
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

#[tokio::test]
async fn users_persist_with_defaults_and_belong_to_an_organization() {
    let db = migrated_db().await;
    let org = acme().insert(&db).await.expect("insert organization");

    let user = user_for(org.id, "admin@acme.test")
        .insert(&db)
        .await
        .expect("insert user");

    pretty_assertions::assert_eq!(user.organization_id, org.id);
    pretty_assertions::assert_eq!(user.email, "admin@acme.test");
    // Defaults defined by the migration.
    assert!(!user.is_admin);
    assert!(!user.deleted);
}

#[tokio::test]
async fn user_email_is_unique() {
    let db = migrated_db().await;
    let org = acme().insert(&db).await.expect("insert organization");

    user_for(org.id, "dup@acme.test")
        .insert(&db)
        .await
        .expect("first user");
    let duplicate = user_for(org.id, "dup@acme.test").insert(&db).await;

    assert!(duplicate.is_err(), "duplicate email must be rejected");
}

#[tokio::test]
async fn user_requires_an_existing_organization() {
    let db = migrated_db().await;

    let orphan = user_for(Uuid::new_v4(), "orphan@acme.test")
        .insert(&db)
        .await;

    assert!(
        orphan.is_err(),
        "foreign key to organizations must be enforced"
    );
}
