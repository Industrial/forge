//! BDD tests for POST /api/auth/register (template auth).
//! Public registration: success (201) and validation 4xx.
//!
//! BDD-style tests focusing on behavior rather than implementation.
//! Tests are organized by feature/behavior area with descriptive names.

use axum::http::StatusCode;

/// Helper function to create a test client with migrations run.
/// Uses the shared app::test_client_with_migrations() which checks E2E_API_URL first.
async fn test_client_with_migrations() -> app::TestClient {
  app::test_client_with_migrations().await.expect("test_client_with_migrations")
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
