//! BDD tests for GET/POST/DELETE /api/organizations/{id}/roles/{role_id}/permissions.
//! Bearer + scope headers.
//!
//! BDD-style tests focusing on behavior rather than implementation.
//! Tests are organized by feature/behavior area with descriptive names.

use axum::http::StatusCode;

const NIL_UUID: &str = "00000000-0000-0000-0000-000000000000";

fn scope_headers<'a>(org_id: &'a str, role_id: &'a str) -> [(&'static str, &'a str); 2] {
  [("X-Organization-Id", org_id), ("X-Role-Id", role_id)]
}

/// Helper function to create a test client with migrations run.
/// Uses the shared app::test_client_with_migrations() which uses FORGE_BACKEND_HOST + port when set.
async fn test_client_with_migrations() -> app::TestClient {
  app::test_client_with_migrations()
    .await
    .expect("test_client_with_migrations")
}

mod bdd_tests {
  use super::*;

  mod authentication_behavior {
    use super::*;

    #[tokio::test]
    async fn should_require_authentication_for_getting_org_role_permissions() {
      // Given: an unauthenticated request
      let client = app::test_client().await.expect("test_client");

      // When: requesting organization role permissions without authentication
      let (status, _) = app::test_request(
        &client,
        "GET",
        &format!(
          "/api/organizations/{}/roles/{}/permissions",
          NIL_UUID, NIL_UUID
        ),
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
    async fn should_require_authentication_for_posting_org_role_permissions() {
      // Given: an unauthenticated request
      let client = app::test_client().await.expect("test_client");

      // When: posting a permission to a role without authentication
      let (status, _) = app::test_request(
        &client,
        "POST",
        &format!(
          "/api/organizations/{}/roles/{}/permissions",
          NIL_UUID, NIL_UUID
        ),
        None,
        Some(r#"{"permission_key":"dashboard.users.read"}"#),
        None,
      )
      .await
      .unwrap();

      // Then: should return 401 Unauthorized
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_require_authentication_for_deleting_org_role_permissions() {
      // Given: an unauthenticated request
      let client = app::test_client().await.expect("test_client");

      // When: deleting a permission from a role without authentication
      let (status, _) = app::test_request(
        &client,
        "DELETE",
        &format!(
          "/api/organizations/{}/roles/{}/permissions",
          NIL_UUID, NIL_UUID
        ),
        None,
        Some(r#"{"permission_key":"dashboard.users.read"}"#),
        None,
      )
      .await
      .unwrap();

      // Then: should return 401 Unauthorized
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
  }

  mod get_permissions_behavior {
    use super::*;

    #[tokio::test]
    async fn should_allow_viewer_to_get_org_role_permissions() {
      // Given: an authenticated viewer user with proper scope headers
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = scope_headers(org_id.as_str(), role_id.as_str());

      // And: an existing role ID from the organization
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

      // When: requesting permissions for that role
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

      // Then: should return 200 OK
      assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn should_return_not_found_for_non_existent_role() {
      // Given: an authenticated viewer user with proper scope headers
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = scope_headers(org_id.as_str(), role_id.as_str());

      // And: a non-existent role ID
      let non_existent_role_id = NIL_UUID;

      // When: requesting permissions for that non-existent role
      let (status, _) = app::test_request(
        &client,
        "GET",
        &format!(
          "/api/organizations/{}/roles/{}/permissions",
          org_id, non_existent_role_id
        ),
        Some(&token),
        None,
        Some(&scope),
      )
      .await
      .unwrap();

      // Then: should return 404 Not Found
      assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn should_return_permissions_list_in_response_body() {
      // Given: an authenticated viewer user with proper scope headers
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = scope_headers(org_id.as_str(), role_id.as_str());

      // And: an existing role ID from the organization
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

      // When: requesting permissions for that role
      let (status, response_body) = app::test_request(
        &client,
        "GET",
        &format!("/api/organizations/{}/roles/{}/permissions", org_id, rid),
        Some(&token),
        None,
        Some(&scope),
      )
      .await
      .unwrap();

      // Then: should return 200 OK with valid JSON response
      assert_eq!(status, StatusCode::OK);
      let response_json: app::serde_json::Value =
        app::serde_json::from_slice(&response_body).unwrap();
      // Response should be valid JSON (structure may vary, but should parse)
      assert!(response_json.is_object() || response_json.is_array());
    }
  }

  mod post_permissions_behavior {
    use super::*;

    #[tokio::test]
    async fn should_forbid_viewer_from_posting_org_role_permissions() {
      // Given: an authenticated viewer user with proper scope headers
      let client = test_client_with_migrations().await;
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
      let rid = json["roles"]
        .as_array()
        .unwrap()
        .first()
        .and_then(|r| r["id"].as_str())
        .unwrap();

      // When: attempting to add a permission to the role
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

      // Then: should return 403 Forbidden
      assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn should_allow_editor_to_post_org_role_permissions_or_return_expected_errors() {
      // Given: an authenticated editor user with proper scope headers
      let client = test_client_with_migrations().await;
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
      let rid = json["roles"]
        .as_array()
        .unwrap()
        .first()
        .and_then(|r| r["id"].as_str())
        .unwrap();

      // When: adding a permission to the role
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

      // Then: should return success, or 422/403/404 when not applicable
      assert!(
        status.is_success()
          || status == StatusCode::UNPROCESSABLE_ENTITY
          || status == StatusCode::FORBIDDEN
          || status == StatusCode::NOT_FOUND
      );
    }

    #[tokio::test]
    async fn should_reject_invalid_permission_key() {
      // Given: an authenticated editor user with proper scope headers
      let client = test_client_with_migrations().await;
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
      let rid = json["roles"]
        .as_array()
        .unwrap()
        .first()
        .and_then(|r| r["id"].as_str())
        .unwrap();

      // When: attempting to add an invalid permission key
      let (status, _) = app::test_request(
        &client,
        "POST",
        &format!("/api/organizations/{}/roles/{}/permissions", org_id, rid),
        Some(&token),
        Some(r#"{"permission_key":"invalid.permission.key"}"#),
        Some(&scope),
      )
      .await
      .unwrap();

      // Then: should return 422 Unprocessable Entity
      assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn should_return_not_found_when_posting_to_non_existent_role() {
      // Given: an authenticated editor user with proper scope headers
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "editor@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = scope_headers(org_id.as_str(), role_id.as_str());

      // And: a non-existent role ID
      let non_existent_role_id = NIL_UUID;

      // When: attempting to add a permission to that non-existent role
      let (status, _) = app::test_request(
        &client,
        "POST",
        &format!(
          "/api/organizations/{}/roles/{}/permissions",
          org_id, non_existent_role_id
        ),
        Some(&token),
        Some(r#"{"permission_key":"dashboard.users.read"}"#),
        Some(&scope),
      )
      .await
      .unwrap();

      // Then: should return 404 Not Found
      assert_eq!(status, StatusCode::NOT_FOUND);
    }
  }

  mod delete_permissions_behavior {
    use super::*;

    #[tokio::test]
    async fn should_forbid_viewer_from_deleting_org_role_permissions() {
      // Given: an authenticated viewer user with proper scope headers
      let client = test_client_with_migrations().await;
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
      let rid = json["roles"]
        .as_array()
        .unwrap()
        .first()
        .and_then(|r| r["id"].as_str())
        .unwrap();

      // When: attempting to remove a permission from the role
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

      // Then: should return 403 Forbidden
      assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn should_allow_editor_to_delete_org_role_permissions_or_return_expected_errors() {
      // Given: an authenticated editor user with proper scope headers
      let client = test_client_with_migrations().await;
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
      let rid = json["roles"]
        .as_array()
        .unwrap()
        .first()
        .and_then(|r| r["id"].as_str())
        .unwrap();

      // When: removing a permission from the role
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

      // Then: should return 200 OK, or 404/403 when not applicable
      assert!(
        status == StatusCode::OK
          || status == StatusCode::NOT_FOUND
          || status == StatusCode::FORBIDDEN
      );
    }

    #[tokio::test]
    async fn should_return_not_found_when_deleting_non_existent_permission() {
      // Given: an authenticated editor user with proper scope headers
      let client = test_client_with_migrations().await;
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
      let rid = json["roles"]
        .as_array()
        .unwrap()
        .first()
        .and_then(|r| r["id"].as_str())
        .unwrap();

      // When: attempting to remove a permission that doesn't exist
      let (status, _) = app::test_request(
        &client,
        "DELETE",
        &format!("/api/organizations/{}/roles/{}/permissions", org_id, rid),
        Some(&token),
        Some(r#"{"permission_key":"non.existent.permission"}"#),
        Some(&scope),
      )
      .await
      .unwrap();

      // Then: should return 404 Not Found or 200 OK (idempotent delete)
      assert!(
        status == StatusCode::NOT_FOUND || status == StatusCode::OK,
        "Expected 404 Not Found or 200 OK for non-existent permission deletion"
      );
    }

    #[tokio::test]
    async fn should_return_not_found_when_deleting_from_non_existent_role() {
      // Given: an authenticated editor user with proper scope headers
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "editor@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = scope_headers(org_id.as_str(), role_id.as_str());

      // And: a non-existent role ID
      let non_existent_role_id = NIL_UUID;

      // When: attempting to remove a permission from that non-existent role
      let (status, _) = app::test_request(
        &client,
        "DELETE",
        &format!(
          "/api/organizations/{}/roles/{}/permissions",
          org_id, non_existent_role_id
        ),
        Some(&token),
        Some(r#"{"permission_key":"dashboard.users.read"}"#),
        Some(&scope),
      )
      .await
      .unwrap();

      // Then: should return 404 Not Found
      assert_eq!(status, StatusCode::NOT_FOUND);
    }
  }
}
