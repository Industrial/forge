//! BDD tests for GET /api/permissions.
//! Endpoint is unprotected: 200 for both anonymous and authenticated requests with permissions list.
//!
//! BDD-style tests focusing on behavior rather than implementation.

use axum::http::StatusCode;

const PERMISSIONS_PATH: &str = "/api/permissions";

mod bdd_tests {
  use super::*;

  mod get_permissions {
    use super::*;

    #[tokio::test]
    async fn should_return_200_with_permissions_list_when_anonymous() {
      // Given: an unauthenticated request
      let client = app::test_client().await.expect("test_client");

      // When: requesting GET /api/permissions without authentication
      let (status, body) = app::test_request(
        &client,
        "GET",
        PERMISSIONS_PATH,
        None,
        None,
        None,
      )
      .await
      .unwrap();

      // Then: should return 200 OK with permissions array
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      assert!(json.get("permissions").and_then(|p| p.as_array()).is_some());
    }

    #[tokio::test]
    async fn should_return_200_with_permissions_list_when_authenticated() {
      // Given: an authenticated user (e.g. viewer)
      let client = app::test_client().await.expect("test_client");
      let (token, _org_id, _role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");

      // When: requesting GET /api/permissions with authentication
      let (status, body) = app::test_request(
        &client,
        "GET",
        PERMISSIONS_PATH,
        Some(&token),
        None,
        None,
      )
      .await
      .unwrap();

      // Then: should return 200 OK with permissions array
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      assert!(json.get("permissions").and_then(|p| p.as_array()).is_some());
    }
  }
}
