pub use sea_orm_migration::prelude::*;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Migrations are appended here as the schema evolves.
        vec![]
    }
}
