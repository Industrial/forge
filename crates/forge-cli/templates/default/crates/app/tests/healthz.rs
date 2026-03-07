//! BDD tests for health endpoints (forge layer).
//! Uses prebuilt server when E2E_API_URL is set.
//!
//! BDD-style tests focusing on behavior of /healthz, /livez, /readyz.

use axum::http::StatusCode;

mod bdd_tests {
  use super::*;

  mod health_endpoints_behavior {
    use super::*;

    #[tokio::test]
    async fn should_return_200_for_get_healthz() {
      // Given: a test client
      let client = app::test_client().await.expect("test_client");
      // When: requesting GET /healthz
      let (status, _) = app::test_request(&client, "GET", "/healthz", None, None, None)
        .await
        .unwrap();
      // Then: should return 200 OK
      assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn should_return_200_for_get_livez() {
      // Given: a test client
      let client = app::test_client().await.expect("test_client");
      // When: requesting GET /livez
      let (status, _) = app::test_request(&client, "GET", "/livez", None, None, None)
        .await
        .unwrap();
      // Then: should return 200 OK
      assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn should_return_200_for_get_readyz() {
      // Given: a test client
      let client = app::test_client().await.expect("test_client");
      // When: requesting GET /readyz
      let (status, _) = app::test_request(&client, "GET", "/readyz", None, None, None)
        .await
        .unwrap();
      // Then: should return 200 OK
      assert_eq!(status, StatusCode::OK);
    }
  }
}
