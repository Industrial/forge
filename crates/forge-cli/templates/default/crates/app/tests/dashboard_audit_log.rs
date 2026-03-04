//! GET /api/audit-log — 401 anon, 200 with valid token.

use axum::http::StatusCode;

#[tokio::test]
async fn get_audit_log_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", "/api/audit-log", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_audit_log_viewer_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, _org_id, _role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let (status, _) = app::test_request(&client, "GET", "/api/audit-log", Some(&token), None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}
