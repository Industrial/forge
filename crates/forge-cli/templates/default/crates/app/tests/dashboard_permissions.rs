//! Dashboard: GET /api/dashboard/permissions — 401 anon, 200 any auth.

use axum::http::StatusCode;

#[tokio::test]
async fn get_permissions_anon_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(&router, "GET", "/api/dashboard/permissions", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

fn scope_headers(org_id: &str, role_name: &str) -> [(&'static str, &str); 2] {
  [("X-Organization-Id", org_id), ("X-Role-Name", role_name)]
}

#[tokio::test]
async fn get_permissions_viewer_default_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, org_id, role_name) = app::auth_with_profile(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_name.as_str());
  let (status, _) =
    app::test_request(&router, "GET", "/api/dashboard/permissions", Some(&token), None, Some(&scope))
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_permissions_editor_default_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, org_id, role_name) = app::auth_with_profile(&router, "editor@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_name.as_str());
  let (status, _) =
    app::test_request(&router, "GET", "/api/dashboard/permissions", Some(&token), None, Some(&scope))
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_permissions_admin_default_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, org_id, role_name) = app::auth_with_profile(&router, "orgadmin@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_name.as_str());
  let (status, _) =
    app::test_request(&router, "GET", "/api/dashboard/permissions", Some(&token), None, Some(&scope))
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_permissions_owner_default_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, org_id, role_name) = app::auth_with_profile(&router, "owner@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_name.as_str());
  let (status, _) =
    app::test_request(&router, "GET", "/api/dashboard/permissions", Some(&token), None, Some(&scope))
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_permissions_global_admin_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, org_id, role_name) = app::auth_with_profile(&router, "admin@admin.com", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_name.as_str());
  let (status, _) =
    app::test_request(&router, "GET", "/api/dashboard/permissions", Some(&token), None, Some(&scope))
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}
