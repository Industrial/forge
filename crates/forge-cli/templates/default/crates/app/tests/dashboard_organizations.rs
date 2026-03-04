//! GET/POST /api/organizations — 401 anon, 403 without dashboard.organizations.read/write, 200/201 for global admin.

use axum::http::StatusCode;

#[tokio::test]
async fn get_organizations_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", "/api/organizations", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_organizations_viewer_403() {
  let client = app::test_client().await.expect("test_client");
  let (token, _org_id, _role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let (status, _) = app::test_request(
    &client,
    "GET",
    "/api/organizations",
    Some(&token),
    None,
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_organizations_global_admin_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, _org_id, _role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let (status, body) = app::test_request(
    &client,
    "GET",
    "/api/organizations",
    Some(&token),
    None,
    None,
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
  let (token, _org_id, _role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let body = r#"{"name":"New Org","slug":"new-org"}"#;
  let (status, _) = app::test_request(
    &client,
    "POST",
    "/api/organizations",
    Some(&token),
    Some(body),
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn post_organizations_global_admin_201() {
  let client = app::test_client().await.expect("test_client");
  let (token, _org_id, _role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let body = r#"{"name":"Test Org","slug":"test-org-12345"}"#;
  let (status, _) = app::test_request(
    &client,
    "POST",
    "/api/organizations",
    Some(&token),
    Some(body),
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::CREATED);
}
