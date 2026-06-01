use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    // CLI: `cargo run -p migration -- <up|down|status|...>`.
    // Reads the DATABASE_URL environment variable.
    cli::run_cli(migration::Migrator).await;
}
