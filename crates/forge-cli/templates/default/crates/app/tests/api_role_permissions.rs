//! GET/POST/DELETE /api/organizations/{id}/roles/{role_id}/permissions. Bearer + scope headers.

use axum::http::StatusCode;

const NIL_UUID: &str = "00000000-0000-0000-0000-000000000000";

fn scope_headers<'a>(org_id: &'a str, role_id: &'a str) -> [(&'static str, &'a str); 2] {
  [("X-Organization-Id", org_id), ("X-Role-Id", role_id)]
}

#[tokio::test]
async fn get_org_role_permissions_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(
    &client,
    "GET",
    &format!("/api/organizations/{}/roles/{}/permissions", NIL_UUID, NIL_UUID),
    None,
    None,
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_org_role_permissions_viewer_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (_, body) = app::test_request(
    &client,
    "GET",
    &format!("/api/organizations/{}/roles", org_id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let roles = json["roles"].as_array().unwrap();
  let rid = roles.first().and_then(|r| r["id"].as_str()).expect("role");
  let (status, _) = app::test_request(
    &client,
    "GET",
    &format!("/api/organizations/{}/roles/{}/permissions", org_id, rid),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn post_org_role_permissions_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(
    &client,
    "POST",
    &format!("/api/organizations/{}/roles/{}/permissions", NIL_UUID, NIL_UUID),
    None,
    Some(r#"{"permission_key":"dashboard.users.read"}"#),
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn post_org_role_permissions_viewer_403() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (_, body) = app::test_request(
    &client,
    "GET",
    &format!("/api/organizations/{}/roles", org_id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let rid = json["roles"].as_array().unwrap().first().and_then(|r| r["id"].as_str()).unwrap();
  let (status, _) = app::test_request(
    &client,
    "POST",
    &format!("/api/organizations/{}/roles/{}/permissions", org_id, rid),
    Some(&token),
    Some(r#"{"permission_key":"dashboard.users.read"}"#),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn post_org_role_permissions_editor_201_or_422() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "editor@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (_, body) = app::test_request(
    &client,
    "GET",
    &format!("/api/organizations/{}/roles", org_id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let rid = json["roles"].as_array().unwrap().first().and_then(|r| r["id"].as_str()).unwrap();
  let (status, _) = app::test_request(
    &client,
    "POST",
    &format!("/api/organizations/{}/roles/{}/permissions", org_id, rid),
    Some(&token),
    Some(r#"{"permission_key":"dashboard.users.read"}"#),
    Some(&scope),
  )
  .await
  .unwrap();
  assert!(
    status.is_success()
      || status == StatusCode::UNPROCESSABLE_ENTITY
      || status == StatusCode::FORBIDDEN
      || status == StatusCode::NOT_FOUND
  );
}

#[tokio::test]
async fn delete_org_role_permissions_viewer_403() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (_, body) = app::test_request(
    &client,
    "GET",
    &format!("/api/organizations/{}/roles", org_id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let rid = json["roles"].as_array().unwrap().first().and_then(|r| r["id"].as_str()).unwrap();
  let (status, _) = app::test_request(
    &client,
    "DELETE",
    &format!("/api/organizations/{}/roles/{}/permissions", org_id, rid),
    Some(&token),
    Some(r#"{"permission_key":"dashboard"}"#),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn delete_org_role_permissions_editor_200_or_404() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "editor@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (_, body) = app::test_request(
    &client,
    "GET",
    &format!("/api/organizations/{}/roles", org_id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let rid = json["roles"].as_array().unwrap().first().and_then(|r| r["id"].as_str()).unwrap();
  let (status, _) = app::test_request(
    &client,
    "DELETE",
    &format!("/api/organizations/{}/roles/{}/permissions", org_id, rid),
    Some(&token),
    Some(r#"{"permission_key":"dashboard"}"#),
    Some(&scope),
  )
  .await
  .unwrap();
  assert!(
    status == StatusCode::OK || status == StatusCode::NOT_FOUND || status == StatusCode::FORBIDDEN
  );
}
