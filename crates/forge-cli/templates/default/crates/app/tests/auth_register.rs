//! BDD tests for POST /api/auth/register (template auth).
//! Public registration: success (201) and validation 4xx.
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

  mod register_behavior {
    use super::*;

    #[tokio::test]
    async fn should_return_201_for_valid_registration() {
      // Given: a test client and valid email and password
      let client = test_client_with_migrations().await;
      let body = r#"{"email":"newuser@example.com","password":"password123"}"#;
      // When: posting to /api/auth/register with valid payload
      let (status, _) = app::test_request(
        &client,
        "POST",
        "/api/auth/register",
        None,
        Some(body),
        None,
      )
      .await
      .unwrap();
      // Then: should return 201 Created
      assert_eq!(status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn should_return_422_for_invalid_email() {
      // Given: a test client and payload with invalid email format
      let client = test_client_with_migrations().await;
      let body = r#"{"email":"not-an-email","password":"password123"}"#;
      // When: posting to /api/auth/register
      let (status, _) = app::test_request(
        &client,
        "POST",
        "/api/auth/register",
        None,
        Some(body),
        None,
      )
      .await
      .unwrap();
      // Then: should return 422 Unprocessable Entity
      assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn should_return_422_for_short_password() {
      // Given: a test client and payload with password too short
      let client = test_client_with_migrations().await;
      let body = r#"{"email":"u@example.com","password":"short"}"#;
      // When: posting to /api/auth/register
      let (status, _) = app::test_request(
        &client,
        "POST",
        "/api/auth/register",
        None,
        Some(body),
        None,
      )
      .await
      .unwrap();
      // Then: should return 422 Unprocessable Entity
      assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
  }
}
