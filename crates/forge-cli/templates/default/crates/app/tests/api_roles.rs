//! GET/POST /api/organizations/{id}/roles and GET/PATCH/DELETE .../roles/{role_id}. Bearer + scope headers.

use axum::http::StatusCode;

fn scope_headers<'a>(org_id: &'a str, role_id: &'a str) -> [(&'static str, &'a str); 2] {
  [("X-Organization-Id", org_id), ("X-Role-Id", role_id)]
}

const NIL_UUID: &str = "00000000-0000-0000-0000-000000000000";

#[tokio::test]
async fn get_org_roles_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(
    &client,
    "GET",
    &format!("/api/organizations/{}/roles", NIL_UUID),
    None,
    None,
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_org_roles_viewer_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (status, body) = app::test_request(
    &client,
    "GET",
    &format!("/api/organizations/{}/roles", org_id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json["roles"].as_array().is_some());
}

#[tokio::test]
async fn post_org_roles_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (_, org_id, _) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let body = r#"{"name":"custom","display_name":"Custom"}"#;
  let (status, _) = app::test_request(
    &client,
    "POST",
    &format!("/api/organizations/{}/roles", org_id),
    None,
    Some(body),
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn post_org_roles_viewer_403() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let body = r#"{"name":"custom","display_name":"Custom"}"#;
  let (status, _) = app::test_request(
    &client,
    "POST",
    &format!("/api/organizations/{}/roles", org_id),
    Some(&token),
    Some(body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn post_org_roles_editor_201() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "editor@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let unique = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis();
  let body = format!(r#"{{"name":"testrole{}","display_name":"Test Role"}}"#, unique);
  let (status, _) = app::test_request(
    &client,
    "POST",
    &format!("/api/organizations/{}/roles", org_id),
    Some(&token),
    Some(&body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn patch_org_roles_viewer_403() {
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
  let rid = roles
    .first()
    .and_then(|r| r["id"].as_str())
    .unwrap_or("00000000-0000-0000-0000-000000000000");
  let patch_body = r#"{"display_name":"Updated"}"#;
  let (status, _) = app::test_request(
    &client,
    "PATCH",
    &format!("/api/organizations/{}/roles/{}", org_id, rid),
    Some(&token),
    Some(patch_body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn patch_org_roles_editor_200() {
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
  let roles = json["roles"].as_array().unwrap();
  let rid = roles.first().and_then(|r| r["id"].as_str()).expect("at least one role");
  let patch_body = r#"{"display_name":"Updated Display"}"#;
  let (status, _) = app::test_request(
    &client,
    "PATCH",
    &format!("/api/organizations/{}/roles/{}", org_id, rid),
    Some(&token),
    Some(patch_body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn delete_org_roles_viewer_403() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let rid = "00000000-0000-0000-0000-000000000000";
  let (status, _) = app::test_request(
    &client,
    "DELETE",
    &format!("/api/organizations/{}/roles/{}", org_id, rid),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}
