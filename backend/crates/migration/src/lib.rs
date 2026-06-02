pub use sea_orm_migration::prelude::*;

mod public;
mod tenant;

/// Migrations for the cross-tenant `public` schema (organizations, users).
/// Run day-to-day via `cargo run -p migration`.
pub struct PublicMigrator;

impl MigratorTrait for PublicMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(public::m20260601_000001_create_organizations::Migration),
            Box::new(public::m20260601_000002_create_users::Migration),
        ]
    }
}

/// Migrations applied inside each tenant's dedicated schema. Run by the tenant
/// provisioning flow against the new schema's `search_path`.
pub struct TenantMigrator;

impl MigratorTrait for TenantMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(tenant::m20260601_000003_create_permissions::Migration),
            Box::new(tenant::m20260601_000004_create_sector::Migration),
            Box::new(tenant::m20260601_000005_create_role::Migration),
            Box::new(tenant::m20260601_000006_create_collaborator::Migration),
            Box::new(tenant::m20260601_000007_create_feedback::Migration),
            Box::new(tenant::m20260601_000008_create_expectation_contract_item::Migration),
            Box::new(tenant::m20260601_000009_create_feedback_behavior::Migration),
            Box::new(tenant::m20260601_000010_create_annotation::Migration),
        ]
    }
}
