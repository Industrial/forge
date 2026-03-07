//! BDD tests for GET/POST /api/organizations/{id}/roles and GET/PATCH/DELETE .../roles/{role_id}. Bearer + scope headers.
//!
//! BDD-style tests focusing on behavior rather than implementation.
//! Tests are organized by feature/behavior area with descriptive names.

use axum::http::StatusCode;

fn scope_headers<'a>(org_id: &'a str, role_id: &'a str) -> [(&'static str, &'a str); 2] {
  [("X-Organization-Id", org_id), ("X-Role-Id", role_id)]
}

const NIL_UUID: &str = "00000000-0000-0000-0000-000000000000";

// Authentication behavior tests
#[tokio::test]
async fn should_require_authentication_for_getting_org_roles() {
  // Given: an unauthenticated request
  let client = app::test_client().await.expect("test_client");
  // When: requesting organization roles without authentication
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
  // Then: should return 401 Unauthorized
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_allow_viewer_to_list_org_roles() {
  // Given: an authenticated viewer user with proper scope headers
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  // When: requesting organization roles
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
  // Then: should return 200 OK with roles array
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json["roles"].as_array().is_some());
}

#[tokio::test]
async fn should_require_authentication_for_creating_org_roles() {
  // Given: an unauthenticated request and valid organization ID
  let client = app::test_client().await.expect("test_client");
  let (_, org_id, _) = app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let body = r#"{"name":"custom","display_name":"Custom"}"#;
  // When: creating a role without authentication
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
  // Then: should return 401 Unauthorized
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// Authorization behavior tests
#[tokio::test]
async fn should_forbid_viewer_from_creating_org_roles() {
  // Given: an authenticated viewer user with proper scope headers
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let body = r#"{"name":"custom","display_name":"Custom"}"#;
  // When: attempting to create a role
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
  // Then: should return 403 Forbidden
  assert_eq!(status, StatusCode::FORBIDDEN);
}

// Role creation behavior tests
#[tokio::test]
async fn should_allow_editor_to_create_org_roles() {
  // Given: an authenticated editor user with proper scope headers
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "editor@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  // And: a unique role name to avoid conflicts
  let unique = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis();
  let body = format!(
    r#"{{"name":"testrole{}","display_name":"Test Role"}}"#,
    unique
  );
  // When: creating a new role
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
  // Then: should return 201 Created
  assert_eq!(status, StatusCode::CREATED);
}

// Role update behavior tests
#[tokio::test]
async fn should_forbid_viewer_from_updating_org_roles() {
  // Given: an authenticated viewer user with proper scope headers
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  // And: an existing role ID
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
  // When: attempting to update a role
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
  // Then: should return 403 Forbidden
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn should_allow_editor_to_update_org_roles() {
  // Given: an authenticated editor user with proper scope headers
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "editor@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  // And: an existing role ID
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
    .expect("at least one role");
  let patch_body = r#"{"display_name":"Updated Display"}"#;
  // When: updating the role
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
  // Then: should return 200 OK
  assert_eq!(status, StatusCode::OK);
}

// Role deletion behavior tests
#[tokio::test]
async fn should_forbid_viewer_from_deleting_org_roles() {
  // Given: an authenticated viewer user with proper scope headers
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let rid = "00000000-0000-0000-0000-000000000000";
  // When: attempting to delete a role
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
  // Then: should return 403 Forbidden
  assert_eq!(status, StatusCode::FORBIDDEN);
}
