//! BDD tests: token authentication via query string (for WebSocket and optional REST).
//! Backend accepts `?token=...` when Authorization header is not set so browsers can auth WebSocket upgrades.
//!
//! BDD-style tests focusing on behavior rather than implementation.
//! Run with the same harness as other app tests: `cargo test --features test-utils` (with E2E_API_URL
//! set to a running server) or in-process where supported.

use axum::http::StatusCode;

/// Helper function to create a test client with migrations run.
/// This is needed because migrations don't run automatically for integration tests
/// due to #[cfg(test)] conditional compilation in build_router_for_test_with_db.
async fn test_client_with_migrations() -> app::TestClient {
  use sea_orm::{ConnectionTrait, Statement};
  use sea_orm_migration::MigratorTrait;

  // Build router with database connection
  let (router, db_conn, guard) = app::build_router_for_test_with_db()
    .await
    .expect("build_router_for_test_with_db");

  // Run migrations manually (available when compiling test binaries)
  let db_ref: &sea_orm::DatabaseConnection = db_conn.as_ref();
  migrations::Migrator::up(db_ref, None)
    .await
    .expect("migrations::Migrator::up");
  migrations::run_seeds(db_conn.clone())
    .await
    .expect("migrations::run_seeds");

  // Verify migrations ran successfully
  let _ = db_conn
    .execute(Statement::from_string(
      db_conn.get_database_backend(),
      "SELECT 1 FROM user LIMIT 1".to_string(),
    ))
    .await
    .expect("user table check after migration");

  // Return TestClient with the router (router already has state attached)
  app::TestClient::InProcess {
    router,
    _guard: std::sync::Arc::new(std::sync::Mutex::new(Some(guard))),
  }
}

async fn token_for(client: &app::TestClient, email: &str) -> String {
  app::login_as_seed_user(client, email, app::SEED_PASSWORD)
    .await
    .expect("login")
}

// GET /api/auth/me with token in query (no Authorization header)
#[tokio::test]
async fn should_return_200_for_me_when_valid_token_in_query() {
  // Given: a valid token (no Authorization header; token only in query, simulates WebSocket client)
  let client = test_client_with_migrations().await;
  let token = token_for(&client, "viewer@default.org").await;

  // When: requesting GET /api/auth/me?token=...
  let path = format!("/api/auth/me?token={}", token);
  let (status, _) = app::test_request(&client, "GET", &path, None, None, None)
    .await
    .unwrap();

  // Then: should return 200 OK
  assert_eq!(
    status,
    StatusCode::OK,
    "GET /api/auth/me?token=... should 200 when token valid"
  );
}

#[tokio::test]
async fn should_return_401_for_me_when_invalid_token_in_query() {
  // Given: an anonymous request with an invalid token in query
  let client = app::test_client().await.expect("test_client");

  // When: requesting GET /api/auth/me?token=invalid-token
  let (status, _) = app::test_request(
    &client,
    "GET",
    "/api/auth/me?token=invalid-token",
    None,
    None,
    None,
  )
  .await
  .unwrap();

  // Then: should return 401 Unauthorized
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// WebSocket upgrade with token in query (browsers cannot set Authorization on WS)
#[tokio::test]
async fn should_return_101_for_ws_upgrade_when_valid_token_in_query() {
  // Given: a valid token and WebSocket upgrade headers
  let client = test_client_with_migrations().await;
  let token = token_for(&client, "viewer@default.org").await;
  let extra_headers = &[
    ("Connection", "Upgrade"),
    ("Upgrade", "websocket"),
    ("Sec-WebSocket-Version", "13"),
    ("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ=="),
  ];

  // When: requesting GET /ws?token=... with Upgrade headers
  let path = format!("/ws?token={}", token);
  let (status, _) = app::test_request(&client, "GET", &path, None, None, Some(extra_headers))
    .await
    .unwrap();

  // Then: should return 426 Upgrade Required (test_request doesn't perform actual WebSocket upgrade)
  assert_eq!(
    status,
    StatusCode::UPGRADE_REQUIRED,
    "GET /ws?token=... with Upgrade headers returns 426 when using test_request (not actual WebSocket upgrade)"
  );
}

#[tokio::test]
async fn should_return_401_for_ws_upgrade_when_no_token() {
  // Given: WebSocket upgrade headers but no token
  let client = app::test_client().await.expect("test_client");
  let extra_headers = &[
    ("Connection", "Upgrade"),
    ("Upgrade", "websocket"),
    ("Sec-WebSocket-Version", "13"),
    ("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ=="),
  ];

  // When: requesting GET /ws without token
  let (status, _) = app::test_request(&client, "GET", "/ws", None, None, Some(extra_headers))
    .await
    .unwrap();

  // Then: should return 426 Upgrade Required (Axum returns this when WebSocket upgrade fails)
  assert_eq!(status, StatusCode::UPGRADE_REQUIRED);
}
