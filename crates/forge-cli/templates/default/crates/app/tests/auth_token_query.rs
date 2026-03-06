//! Integration tests: token authentication via query string (for WebSocket and optional REST).
//! Backend accepts `?token=...` when Authorization header is not set so browsers can auth WebSocket upgrades.
//!
//! Run with the same harness as other app tests: `cargo test --features test-utils` (with E2E_API_URL
//! set to a running server) or in-process where supported.

use axum::http::StatusCode;

#[tokio::test]
async fn get_me_with_token_in_query_200() {
  let client = app::test_client().await.expect("test_client");
  let token = app::login_as_seed_user(&client, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");

  // No Authorization header; token only in query (simulates WebSocket client)
  let path = format!("/api/auth/me?token={}", token);
  let (status, _) = app::test_request(&client, "GET", &path, None, None, None)
    .await
    .unwrap();
  assert_eq!(
    status,
    StatusCode::OK,
    "GET /api/auth/me?token=... should 200 when token valid"
  );
}

#[tokio::test]
async fn get_me_with_invalid_token_in_query_401() {
  let client = app::test_client().await.expect("test_client");

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
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn ws_upgrade_with_token_in_query_101() {
  let client = app::test_client().await.expect("test_client");
  let token = app::login_as_seed_user(&client, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");

  // WebSocket upgrade: token in query (browsers cannot set Authorization on WS), required headers
  let path = format!("/ws?token={}", token);
  let extra_headers = &[
    ("Connection", "Upgrade"),
    ("Upgrade", "websocket"),
    ("Sec-WebSocket-Version", "13"),
    ("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ=="),
  ];
  let (status, _) = app::test_request(&client, "GET", &path, None, None, Some(extra_headers))
    .await
    .unwrap();
  assert_eq!(
    status,
    StatusCode::SWITCHING_PROTOCOLS,
    "GET /ws?token=... with Upgrade headers should 101 when token valid"
  );
}

#[tokio::test]
async fn ws_upgrade_without_token_401() {
  let client = app::test_client().await.expect("test_client");

  let extra_headers = &[
    ("Connection", "Upgrade"),
    ("Upgrade", "websocket"),
    ("Sec-WebSocket-Version", "13"),
    ("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ=="),
  ];
  let (status, _) = app::test_request(&client, "GET", "/ws", None, None, Some(extra_headers))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}
