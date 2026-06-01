use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    // CLI: `cargo run -p migration -- <up|down|status|...>`.
    // Reads the DATABASE_URL environment variable. Operates on the public schema;
    // tenant schemas are migrated by the provisioning flow.
    cli::run_cli(migration::PublicMigrator).await;
}
