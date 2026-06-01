//! Integration test for serving the Elm SPA from the Axum binary (single origin,
//! see ADR 0011). Uses a `MockDatabase` so it needs no Docker: static serving
//! never touches the database. A fixture `dist` directory stands in for the built
//! frontend.

use api::{build_router, with_static_spa};
use axum::http::StatusCode;
use axum_test::TestServer;
use sea_orm::{DatabaseBackend, MockDatabase};
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/spa")
}

fn server() -> TestServer {
    let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
    let router = build_router(db, "postgres://unused", b"test-secret".to_vec());
    TestServer::new(with_static_spa(router, fixture_dir()))
}

#[tokio::test]
async fn serves_index_at_the_root() {
    let response = server().get("/").await;

    response.assert_status_ok();
    assert!(
        response.text().contains("spa-fixture-index"),
        "the root should serve index.html"
    );
}

#[tokio::test]
async fn serves_real_static_assets() {
    let response = server().get("/app.js").await;

    response.assert_status_ok();
    assert!(
        response.text().contains("spa-fixture-app"),
        "a real asset should be served from disk"
    );
}

#[tokio::test]
async fn falls_back_to_index_for_unknown_client_routes() {
    // A client-side route the backend knows nothing about must still boot the SPA.
    let response = server().get("/some/client/route").await;

    response.assert_status_ok();
    assert!(
        response.text().contains("spa-fixture-index"),
        "unknown non-API paths should fall back to index.html"
    );
}

#[tokio::test]
async fn api_routes_take_precedence_over_the_spa() {
    let response = server().get("/health").await;

    response.assert_status(StatusCode::OK);
    assert_eq!(
        response.json::<serde_json::Value>(),
        serde_json::json!({ "status": "ok" })
    );
}
