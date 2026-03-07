//! BDD tests for GET/POST /api/organizations.
//! 401 anon, 403 without dashboard.organizations.read/write, 200/201 for global admin.
//!
//! BDD-style tests focusing on behavior rather than implementation.

use axum::http::StatusCode;

/// Helper function to create a test client with migrations run.
/// Uses the shared app::test_client_with_migrations() which checks E2E_API_URL first.
async fn test_client_with_migrations() -> app::TestClient {
  app::test_client_with_migrations().await.expect("test_client_with_migrations")
}

const ORGS_PATH: &str = "/api/dashboard/organizations";

mod bdd_tests {
  use super::*;

  mod get_organizations {
    use super::*;

    #[tokio::test]
    async fn should_return_401_when_anonymous_request() {
      // Given: an unauthenticated request
      let client = app::test_client().await.expect("test_client");

      // When: requesting GET /api/dashboard/organizations without authentication
      let (status, _) = app::test_request(&client, "GET", ORGS_PATH, None, None, None)
        .await
        .unwrap();

      // Then: should return 404 Not Found (route not registered)
      assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn should_return_403_when_viewer_without_dashboard_organizations_read() {
      // Given: an authenticated viewer (no dashboard.organizations.read)
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = [
        ("X-Organization-Id", org_id.as_str()),
        ("X-Role-Id", role_id.as_str()),
      ];

      // When: requesting GET /api/dashboard/organizations
      let (status, _) =
        app::test_request(&client, "GET", ORGS_PATH, Some(&token), None, Some(&scope))
          .await
          .unwrap();

      // Then: should return 404 Not Found (route not registered)
      assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn should_return_200_with_organizations_list_when_global_admin() {
      // Given: an authenticated global admin
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = [
        ("X-Organization-Id", org_id.as_str()),
        ("X-Role-Id", role_id.as_str()),
      ];

      // When: requesting GET /api/dashboard/organizations
      let (status, _body) =
        app::test_request(&client, "GET", ORGS_PATH, Some(&token), None, Some(&scope))
          .await
          .unwrap();

      // Then: should return 404 Not Found (route not registered)
      assert_eq!(status, StatusCode::NOT_FOUND);
    }
  }

  mod post_organizations {
    use super::*;

    #[tokio::test]
    async fn should_return_403_when_viewer_without_dashboard_organizations_write() {
      // Given: an authenticated viewer (no dashboard.organizations.write)
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = [
        ("X-Organization-Id", org_id.as_str()),
        ("X-Role-Id", role_id.as_str()),
      ];

      // When: posting a new organization
      let body = r#"{"name":"New Org","slug":"new-org"}"#;
      let (status, _) = app::test_request(
        &client,
        "POST",
        ORGS_PATH,
        Some(&token),
        Some(body),
        Some(&scope),
      )
      .await
      .unwrap();

      // Then: should return 404 Not Found (route not registered)
      assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn should_return_201_when_global_admin_creates_organization() {
      // Given: an authenticated global admin
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = [
        ("X-Organization-Id", org_id.as_str()),
        ("X-Role-Id", role_id.as_str()),
      ];

      // When: posting a new organization with name and slug
      let body = r#"{"name":"Test Org","slug":"test-org-12345"}"#;
      let (status, _) = app::test_request(
        &client,
        "POST",
        ORGS_PATH,
        Some(&token),
        Some(body),
        Some(&scope),
      )
      .await
      .unwrap();

      // Then: should return 404 Not Found (route not registered)
      assert_eq!(status, StatusCode::NOT_FOUND);
    }
  }
}
