//! Dashboard: GET /api/dashboard/permissions — 401 anon, 200 any auth.

use axum::http::StatusCode;

#[tokio::test]
async fn get_permissions_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", "/api/dashboard/permissions", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

fn scope_headers<'a>(org_id: &'a str, role_id: &'a str) -> [(&'static str, &'a str); 2] {
  [("X-Organization-Id", org_id), ("X-Role-Id", role_id)]
}

#[tokio::test]
async fn get_permissions_viewer_default_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) = app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (status, _) =
    app::test_request(&client, "GET", "/api/dashboard/permissions", Some(&token), None, Some(&scope))
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_permissions_editor_default_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) = app::auth_with_profile(&client, "editor@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (status, _) =
    app::test_request(&client, "GET", "/api/dashboard/permissions", Some(&token), None, Some(&scope))
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_permissions_admin_default_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) = app::auth_with_profile(&client, "orgadmin@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (status, _) =
    app::test_request(&client, "GET", "/api/dashboard/permissions", Some(&token), None, Some(&scope))
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_permissions_owner_default_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) = app::auth_with_profile(&client, "owner@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (status, _) =
    app::test_request(&client, "GET", "/api/dashboard/permissions", Some(&token), None, Some(&scope))
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_permissions_global_admin_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) = app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (status, _) =
    app::test_request(&client, "GET", "/api/dashboard/permissions", Some(&token), None, Some(&scope))
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}
