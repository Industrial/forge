//! BDD tests for login and GET profile (template auth).
//! Uses prebuilt server when E2E_API_URL is set.
//!
//! BDD-style tests focusing on behavior rather than implementation.
//! Tests are organized by feature/behavior area with descriptive names.

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

mod bdd_tests {
  use super::*;

  mod login_behavior {
    use super::*;

    #[tokio::test]
    async fn should_return_200_for_profile_after_successful_login() {
      // Given: a test client
      let client = test_client_with_migrations().await;
      // When: logging in as a seed user and then requesting GET /api/auth/me with the token
      let token = app::login_as_seed_user(&client, "viewer@default.org", app::SEED_PASSWORD)
        .await
        .expect("login as viewer@default.org");
      let (status, _) = app::test_request(&client, "GET", "/api/auth/me", Some(&token), None, None)
        .await
        .unwrap();
      // Then: should return 200 OK
      assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn should_return_401_for_login_with_invalid_credentials() {
      // Given: a test client and wrong password
      let client = test_client_with_migrations().await;
      let body = r#"{"email":"viewer@default.org","password":"wrongpassword"}"#;
      // When: posting to /api/auth/login with invalid credentials
      let (status, _) =
        app::test_request(&client, "POST", "/api/auth/login", None, Some(body), None)
          .await
          .unwrap();
      // Then: should return 401 Unauthorized
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
  }

  mod profile_behavior {
    use super::*;

    #[tokio::test]
    async fn should_return_401_for_profile_without_token() {
      // Given: a test client without authentication
      let client = app::test_client().await.expect("test_client");
      // When: requesting GET /api/auth/me without a token
      let (status, _) = app::test_request(&client, "GET", "/api/auth/me", None, None, None)
        .await
        .unwrap();
      // Then: should return 401 Unauthorized
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
  }
}
