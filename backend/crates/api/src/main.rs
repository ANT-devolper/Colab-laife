use std::net::SocketAddr;

use sea_orm::Database;
use tracing_subscriber::EnvFilter;

// Axum application entry point. Routes live in the library crate (`api::build_router`)
// so they can be driven directly from tests; `main` only wires up the runtime.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL").map_err(|_| {
        "DATABASE_URL must be set (e.g. postgres://colab:colab@localhost:5432/colab_life)"
    })?;
    let db = Database::connect(database_url.clone()).await?;

    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| "JWT_SECRET must be set (a strong random secret signs session tokens)")?;

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let mut router = api::build_router(db, database_url, jwt_secret);

    // Serve the built Elm SPA from the same origin when its dist directory is
    // configured (see ADR 0011); otherwise run as an API-only server.
    if let Ok(dist) = std::env::var("FRONTEND_DIST") {
        tracing::info!("serving SPA from {dist}");
        router = api::with_static_spa(router, dist);
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, router).await?;

    Ok(())
}
