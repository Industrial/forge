//! Dashboard: GET /api/dashboard/audit-log — 401 anon, 200 with dashboard.audit.read.

use axum::http::StatusCode;

#[tokio::test]
async fn get_audit_log_anon_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(&router, "GET", "/api/dashboard/audit-log", None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_audit_log_viewer_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, _) =
    app::test_request(&router, "GET", "/api/dashboard/audit-log", Some(&cookie), None)
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}
