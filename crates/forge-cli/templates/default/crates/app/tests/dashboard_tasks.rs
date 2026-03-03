//! Dashboard: GET /api/dashboard/tasks — 401 anon, 200 any auth.

use axum::http::StatusCode;

#[tokio::test]
async fn get_tasks_anon_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(&router, "GET", "/api/dashboard/tasks", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_tasks_auth_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, org_id, role_name) = app::auth_with_profile(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = [
    ("X-Organization-Id", org_id.as_str()),
    ("X-Role-Name", role_name.as_str()),
  ];
  let (status, _) = app::test_request(&router, "GET", "/api/dashboard/tasks", Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}
