//! Dashboard: GET/POST/PATCH/DELETE /api/dashboard/organizations — 401 anon, 403 non-global-admin, 200 global_admin.

use axum::http::StatusCode;

#[tokio::test]
async fn get_organizations_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", "/api/dashboard/organizations", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

fn scope_headers<'a>(org_id: &'a str, role_id: &'a str) -> [(&'static str, &'a str); 2] {
  [("X-Organization-Id", org_id), ("X-Role-Id", role_id)]
}

#[tokio::test]
async fn get_organizations_viewer_403() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) = app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (status, _) = app::test_request(
    &client,
    "GET",
    "/api/dashboard/organizations",
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_organizations_global_admin_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) = app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (status, body) = app::test_request(
    &client,
    "GET",
    "/api/dashboard/organizations",
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let orgs = json["organizations"].as_array().unwrap();
  assert!(!orgs.is_empty());
}

#[tokio::test]
async fn post_organizations_viewer_403() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) = app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let body = r#"{"name":"New Org","slug":"new-org"}"#;
  let (status, _) = app::test_request(
    &client,
    "POST",
    "/api/dashboard/organizations",
    Some(&token),
    Some(body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn post_organizations_global_admin_201() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) = app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let body = r#"{"name":"Test Org","slug":"test-org-12345"}"#;
  let (status, _) = app::test_request(
    &client,
    "POST",
    "/api/dashboard/organizations",
    Some(&token),
    Some(body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::CREATED);
}
