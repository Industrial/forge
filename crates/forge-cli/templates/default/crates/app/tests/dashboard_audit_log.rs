//! Dashboard: GET /api/dashboard/audit-log — 401 anon, 200 with dashboard.audit.read.

use axum::http::StatusCode;

#[tokio::test]
async fn get_audit_log_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", "/api/dashboard/audit-log", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_audit_log_viewer_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) = app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = [
    ("X-Organization-Id", org_id.as_str()),
    ("X-Role-Id", role_id.as_str()),
  ];
  let (status, _) =
    app::test_request(&client, "GET", "/api/dashboard/audit-log", Some(&token), None, Some(&scope))
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}
