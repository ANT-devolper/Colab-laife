pub use sea_orm_migration::prelude::*;

mod public;

/// Migrations for the cross-tenant `public` schema (organizations, users).
/// Run day-to-day via `cargo run -p migration`.
pub struct PublicMigrator;

impl MigratorTrait for PublicMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            public::m20260601_000001_create_organizations::Migration,
        )]
    }
}

/// Migrations applied inside each tenant's dedicated schema. Run by the tenant
/// provisioning flow against the new schema's `search_path`.
pub struct TenantMigrator;

impl MigratorTrait for TenantMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Tenant-schema tables are appended here as the domain model grows.
        vec![]
    }
}
