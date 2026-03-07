//! BDD tests for GET/POST /api/organizations.
//! 401 anon, 403 without dashboard.organizations.read/write, 200/201 for global admin.
//!
//! BDD-style tests focusing on behavior rather than implementation.

use axum::http::StatusCode;

const ORGS_PATH: &str = "/api/organizations";

mod bdd_tests {
  use super::*;

  mod get_organizations {
    use super::*;

    #[tokio::test]
    async fn should_return_401_when_anonymous_request() {
      // Given: an unauthenticated request
      let client = app::test_client().await.expect("test_client");

      // When: requesting GET /api/organizations without authentication
      let (status, _) = app::test_request(&client, "GET", ORGS_PATH, None, None, None)
        .await
        .unwrap();

      // Then: should return 401 Unauthorized
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_return_403_when_viewer_without_dashboard_organizations_read() {
      // Given: an authenticated viewer (no dashboard.organizations.read)
      let client = app::test_client().await.expect("test_client");
      let (token, _org_id, _role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");

      // When: requesting GET /api/organizations
      let (status, _) = app::test_request(&client, "GET", ORGS_PATH, Some(&token), None, None)
        .await
        .unwrap();

      // Then: should return 403 Forbidden
      assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn should_return_200_with_organizations_list_when_global_admin() {
      // Given: an authenticated global admin
      let client = app::test_client().await.expect("test_client");
      let (token, _org_id, _role_id) =
        app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
          .await
          .expect("login");

      // When: requesting GET /api/organizations
      let (status, body) = app::test_request(&client, "GET", ORGS_PATH, Some(&token), None, None)
        .await
        .unwrap();

      // Then: should return 200 OK with non-empty organizations array
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      let orgs = json["organizations"].as_array().unwrap();
      assert!(!orgs.is_empty());
    }
  }

  mod post_organizations {
    use super::*;

    #[tokio::test]
    async fn should_return_403_when_viewer_without_dashboard_organizations_write() {
      // Given: an authenticated viewer (no dashboard.organizations.write)
      let client = app::test_client().await.expect("test_client");
      let (token, _org_id, _role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");

      // When: posting a new organization
      let body = r#"{"name":"New Org","slug":"new-org"}"#;
      let (status, _) =
        app::test_request(&client, "POST", ORGS_PATH, Some(&token), Some(body), None)
          .await
          .unwrap();

      // Then: should return 403 Forbidden
      assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn should_return_201_when_global_admin_creates_organization() {
      // Given: an authenticated global admin
      let client = app::test_client().await.expect("test_client");
      let (token, _org_id, _role_id) =
        app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
          .await
          .expect("login");

      // When: posting a new organization with name and slug
      let body = r#"{"name":"Test Org","slug":"test-org-12345"}"#;
      let (status, _) =
        app::test_request(&client, "POST", ORGS_PATH, Some(&token), Some(body), None)
          .await
          .unwrap();

      // Then: should return 201 Created
      assert_eq!(status, StatusCode::CREATED);
    }
  }
}
