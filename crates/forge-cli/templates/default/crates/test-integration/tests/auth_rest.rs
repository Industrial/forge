//! BDD tests for Auth: logout, me, profiles, tokens, admin — 401 anon, 200 auth; admin 403 non-admin, 200 global.
//!
//! BDD-style tests focusing on behavior rather than implementation.
//! Tests are organized by feature/behavior area with descriptive names.
//! (Session and set-profile/switch-profile endpoints removed; scope is via request headers.)

use axum::http::StatusCode;

/// Helper function to create a test client with migrations run.
/// Uses the shared app::test_client_with_migrations() which checks E2E_API_URL first.
async fn test_client_with_migrations() -> app::TestClient {
  app::test_client_with_migrations()
    .await
    .expect("test_client_with_migrations")
}

async fn token_for(client: &app::TestClient, email: &str) -> String {
  app::login_as_seed_user(client, email, app::SEED_PASSWORD)
    .await
    .expect("login")
}

mod bdd_tests {
  use super::*;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.

  mod logout_behavior {
    use super::*;

    #[tokio::test]
    async fn should_allow_logout_for_anonymous_users() {
      // Given: an anonymous request
      let client = app::test_client().await.expect("test_client");

      // When: requesting logout (POST)
      let (status, _) =
        app::test_request(&client, "POST", "/api/auth/logout", None, Some("{}"), None)
          .await
          .unwrap();

      // Then: should return 200 OK
      assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn should_allow_logout_for_authenticated_users() {
      // Given: an authenticated user
      let client = test_client_with_migrations().await;
      let token = token_for(&client, "viewer@default.org").await;

      // When: requesting logout (POST)
      let (status, _) = app::test_request(
        &client,
        "POST",
        "/api/auth/logout",
        Some(&token),
        Some("{}"),
        None,
      )
      .await
      .unwrap();

      // Then: should return 200 OK
      assert_eq!(status, StatusCode::OK);
    }
  }

  mod me_endpoint_behavior {
    use super::*;

    #[tokio::test]
    async fn should_require_authentication_for_me_endpoint() {
      // Given: an anonymous request
      let client = app::test_client().await.expect("test_client");

      // When: requesting /api/auth/me without authentication
      let (status, _) = app::test_request(&client, "GET", "/api/auth/me", None, None, None)
        .await
        .unwrap();

      // Then: should return 401 Unauthorized
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_return_user_info_for_authenticated_users() {
      // Given: an authenticated user
      let client = test_client_with_migrations().await;
      let token = token_for(&client, "viewer@default.org").await;

      // When: requesting /api/auth/me with valid token
      let (status, _) = app::test_request(&client, "GET", "/api/auth/me", Some(&token), None, None)
        .await
        .unwrap();

      // Then: should return 200 OK
      assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn should_reject_invalid_or_expired_tokens() {
      // Given: an invalid or expired token
      let client = app::test_client().await.expect("test_client");

      // When: requesting /api/auth/me with invalid token
      let (status, _) = app::test_request(
        &client,
        "GET",
        "/api/auth/me",
        Some("invalid-or-expired-token"),
        None,
        None,
      )
      .await
      .unwrap();

      // Then: should return 401 Unauthorized
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
  }

  mod scopes_endpoint_behavior {
    use super::*;

    #[tokio::test]
    async fn should_require_authentication_for_scopes_endpoint() {
      // Given: an anonymous request
      let client = app::test_client().await.expect("test_client");

      // When: requesting /api/auth/scopes without authentication
      let (status, _) = app::test_request(&client, "GET", "/api/auth/scopes", None, None, None)
        .await
        .unwrap();

      // Then: should return 401 Unauthorized
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_return_scopes_for_authenticated_users() {
      // Given: an authenticated user
      let client = test_client_with_migrations().await;
      let token = token_for(&client, "viewer@default.org").await;

      // When: requesting /api/auth/scopes with valid token
      let (status, body) = app::test_request(
        &client,
        "GET",
        "/api/auth/scopes",
        Some(&token),
        None,
        None,
      )
      .await
      .unwrap();

      // Then: should return 200 OK with scopes
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      assert!(json.get("scopes").is_some());
    }
  }

  mod tokens_endpoint_behavior {
    use super::*;

    #[tokio::test]
    async fn should_require_authentication_for_tokens_endpoint() {
      // Given: an anonymous request
      let client = app::test_client().await.expect("test_client");

      // When: creating a token without authentication
      let (status, _) =
        app::test_request(&client, "POST", "/api/auth/tokens", None, Some("{}"), None)
          .await
          .unwrap();

      // Then: should return 401 Unauthorized
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_allow_authenticated_users_to_create_tokens() {
      // Given: an authenticated user
      let client = test_client_with_migrations().await;
      let token = token_for(&client, "viewer@default.org").await;

      // When: creating a token with valid authentication
      let (status, _) = app::test_request(
        &client,
        "POST",
        "/api/auth/tokens",
        Some(&token),
        Some("{}"),
        None,
      )
      .await
      .unwrap();

      // Then: should return 200 OK
      assert_eq!(status, StatusCode::OK);
    }
  }

  mod admin_endpoint_behavior {
    use super::*;

    #[tokio::test]
    async fn should_require_authentication_for_admin_endpoint() {
      // Given: an anonymous request
      let client = app::test_client().await.expect("test_client");

      // When: requesting /api/auth/admin without authentication
      let (status, _) = app::test_request(&client, "GET", "/api/auth/admin", None, None, None)
        .await
        .unwrap();

      // Then: should return 401 Unauthorized
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_forbid_non_admin_users_from_admin_endpoint() {
      // Given: an authenticated non-admin user
      let client = test_client_with_migrations().await;
      let token = token_for(&client, "viewer@default.org").await;

      // When: requesting /api/auth/admin
      let (status, _) =
        app::test_request(&client, "GET", "/api/auth/admin", Some(&token), None, None)
          .await
          .unwrap();

      // Then: should return 403 Forbidden
      assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn should_allow_global_admin_users_to_access_admin_endpoint() {
      // Given: an authenticated global admin user
      let client = test_client_with_migrations().await;
      let token = token_for(&client, "admin@admin.com").await;

      // When: requesting /api/auth/admin
      let (status, _) =
        app::test_request(&client, "GET", "/api/auth/admin", Some(&token), None, None)
          .await
          .unwrap();

      // Then: should return 200 OK
      assert_eq!(status, StatusCode::OK);
    }
  }

  mod auth_flow_flat_endpoints_behavior {
    use super::*;

    #[tokio::test]
    async fn should_require_auth_for_permissions_list() {
      let client = test_client_with_migrations().await;
      let (status, _) =
        app::test_request(&client, "GET", "/api/auth/permissions", None, None, None)
          .await
          .unwrap();
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_return_permissions_with_auth_and_scope() {
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("auth_with_profile");
      let headers = [
        ("x-organization-id", org_id.as_str()),
        ("x-role-id", role_id.as_str()),
      ];
      let (status, body) = app::test_request(
        &client,
        "GET",
        "/api/auth/permissions",
        Some(&token),
        None,
        Some(&headers),
      )
      .await
      .unwrap();
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      assert!(json.get("permissions").is_some());
    }

    #[tokio::test]
    async fn should_require_auth_for_users_list() {
      let client = test_client_with_migrations().await;
      let (status, _) = app::test_request(&client, "GET", "/api/auth/users", None, None, None)
        .await
        .unwrap();
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_return_users_with_auth_and_scope() {
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("auth_with_profile");
      let headers = [
        ("x-organization-id", org_id.as_str()),
        ("x-role-id", role_id.as_str()),
      ];
      let (status, body) = app::test_request(
        &client,
        "GET",
        "/api/auth/users",
        Some(&token),
        None,
        Some(&headers),
      )
      .await
      .unwrap();
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      assert!(json.get("users").is_some());
    }

    #[tokio::test]
    async fn should_require_auth_for_organizations_list() {
      let client = test_client_with_migrations().await;
      let (status, _) =
        app::test_request(&client, "GET", "/api/auth/organizations", None, None, None)
          .await
          .unwrap();
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_return_organizations_with_auth_and_scope() {
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("auth_with_profile");
      let headers = [
        ("x-organization-id", org_id.as_str()),
        ("x-role-id", role_id.as_str()),
      ];
      let (status, body) = app::test_request(
        &client,
        "GET",
        "/api/auth/organizations",
        Some(&token),
        None,
        Some(&headers),
      )
      .await
      .unwrap();
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      assert!(json.get("organizations").is_some());
    }

    #[tokio::test]
    async fn should_require_auth_for_roles_list() {
      let client = test_client_with_migrations().await;
      let (status, _) = app::test_request(&client, "GET", "/api/auth/roles", None, None, None)
        .await
        .unwrap();
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_return_roles_with_auth_and_scope() {
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("auth_with_profile");
      let headers = [
        ("x-organization-id", org_id.as_str()),
        ("x-role-id", role_id.as_str()),
      ];
      let (status, body) = app::test_request(
        &client,
        "GET",
        "/api/auth/roles",
        Some(&token),
        None,
        Some(&headers),
      )
      .await
      .unwrap();
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      assert!(json.get("roles").is_some());
    }

    #[tokio::test]
    async fn should_require_auth_for_role_permissions_list() {
      let client = test_client_with_migrations().await;
      let (status, _) = app::test_request(
        &client,
        "GET",
        "/api/auth/role-permissions",
        None,
        None,
        None,
      )
      .await
      .unwrap();
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_return_role_permissions_with_auth_and_scope() {
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("auth_with_profile");
      let headers = [
        ("x-organization-id", org_id.as_str()),
        ("x-role-id", role_id.as_str()),
      ];
      let (status, body) = app::test_request(
        &client,
        "GET",
        "/api/auth/role-permissions",
        Some(&token),
        None,
        Some(&headers),
      )
      .await
      .unwrap();
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      assert!(json.get("assignments").is_some());
    }

    #[tokio::test]
    async fn should_require_auth_for_global_role_assignments_list() {
      let client = test_client_with_migrations().await;
      let (status, _) = app::test_request(
        &client,
        "GET",
        "/api/auth/global-role-assignments",
        None,
        None,
        None,
      )
      .await
      .unwrap();
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_forbid_org_user_from_global_role_assignments_list() {
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("auth_with_profile");
      let headers = [
        ("x-organization-id", org_id.as_str()),
        ("x-role-id", role_id.as_str()),
      ];
      let (status, _) = app::test_request(
        &client,
        "GET",
        "/api/auth/global-role-assignments",
        Some(&token),
        None,
        Some(&headers),
      )
      .await
      .unwrap();
      assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn should_allow_global_admin_to_list_global_role_assignments() {
      let client = test_client_with_migrations().await;
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
          .await
          .expect("auth_with_profile");
      let headers = [
        ("x-organization-id", org_id.as_str()),
        ("x-role-id", role_id.as_str()),
      ];
      let (status, body) = app::test_request(
        &client,
        "GET",
        "/api/auth/global-role-assignments",
        Some(&token),
        None,
        Some(&headers),
      )
      .await
      .unwrap();
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      assert!(json.get("assignments").is_some());
    }
  }
}
