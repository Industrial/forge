//! BDD tests for Auth: logout, me, profiles, tokens, admin — 401 anon, 200 auth; admin 403 non-admin, 200 global.
//!
//! BDD-style tests focusing on behavior rather than implementation.
//! Tests are organized by feature/behavior area with descriptive names.
//! (Session and set-profile/switch-profile endpoints removed; scope is via request headers.)

use axum::http::StatusCode;

/// Helper function to create a test client with migrations run.
/// This is needed because migrations don't run automatically for integration tests
/// due to #[cfg(test)] conditional compilation in build_router_for_test_with_db.
async fn test_client_with_migrations() -> app::TestClient {
  use sea_orm::{ConnectionTrait, Statement};
  use sea_orm_migration::MigratorTrait;

  // Build router with database connection
  let (router, db_conn, guard) = app::build_router_for_test_with_db()
    .await
    .expect("build_router_for_test_with_db");

  // Run migrations manually (available when compiling test binaries)
  let db_ref: &sea_orm::DatabaseConnection = db_conn.as_ref();
  migrations::Migrator::up(db_ref, None)
    .await
    .expect("migrations::Migrator::up");
  migrations::run_seeds(db_conn.clone())
    .await
    .expect("migrations::run_seeds");

  // Verify migrations ran successfully
  let _ = db_conn
    .execute(Statement::from_string(
      db_conn.get_database_backend(),
      "SELECT 1 FROM user LIMIT 1".to_string(),
    ))
    .await
    .expect("user table check after migration");

  // Return TestClient with the router (router already has state attached)
  app::TestClient::InProcess {
    router,
    _guard: std::sync::Arc::new(std::sync::Mutex::new(Some(guard))),
  }
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

      // When: requesting logout
      let (status, _) = app::test_request(&client, "GET", "/api/auth/logout", None, None, None)
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

      // When: requesting logout
      let (status, _) =
        app::test_request(&client, "GET", "/api/auth/logout", Some(&token), None, None)
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

  mod profiles_endpoint_behavior {
    use super::*;

    #[tokio::test]
    async fn should_require_authentication_for_profiles_endpoint() {
      // Given: an anonymous request
      let client = app::test_client().await.expect("test_client");

      // When: requesting /api/auth/profiles without authentication
      let (status, _) = app::test_request(&client, "GET", "/api/auth/profiles", None, None, None)
        .await
        .unwrap();

      // Then: should return 401 Unauthorized
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_return_profiles_for_authenticated_users() {
      // Given: an authenticated user
      let client = test_client_with_migrations().await;
      let token = token_for(&client, "viewer@default.org").await;

      // When: requesting /api/auth/profiles with valid token
      let (status, body) = app::test_request(
        &client,
        "GET",
        "/api/auth/profiles",
        Some(&token),
        None,
        None,
      )
      .await
      .unwrap();

      // Then: should return 200 OK with profiles
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      assert!(json.get("profiles").is_some());
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
}
