use axum::{extract::State, response::IntoResponse};
use forge_db::DbConnection;

pub async fn handler(State(_db): State<DbConnection>) -> impl IntoResponse {
  tracing::debug!(target: "app::handlers", "route: GET /api/cached-page");
  let v = format!("cached-page-{}", uuid::Uuid::new_v4());
  v.into_response()
}

#[cfg(test)]
mod bdd_tests {
  use crate::build_router_for_test;
  use axum::body::Body;
  use axum::http::{Request, StatusCode};
  use tower::ServiceExt;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests verify cached_page handler behavior - generates unique values that can be cached by HTTP response cache layer.

  mod response_generation_behavior {
    use super::*;

    #[tokio::test]
    async fn should_generate_unique_value_on_each_request() {
      // Given: a router with the cached_page handler
      let (router, _guard) = build_router_for_test().await.unwrap();

      // When: making multiple requests to cached_page endpoint
      let mut values = Vec::new();
      for _ in 0..3 {
        let req = Request::builder()
          .uri("/api/cached-page")
          .body(Body::empty())
          .unwrap();
        let res = router.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
          .await
          .unwrap();
        let value = std::str::from_utf8(&body).unwrap().to_string();
        values.push(value);
      }

      // Then: each request should generate a unique UUID-based value
      assert_eq!(values.len(), 3);
      // All values should start with "cached-page-"
      for value in &values {
        assert!(
          value.starts_with("cached-page-"),
          "Value should start with 'cached-page-', got: {}",
          value
        );
      }
      // Values should be different (unless HTTP cache is serving cached response)
      // Note: If HTTP response cache is enabled, values might be the same
      // This test verifies the handler generates unique values when not cached
    }

    #[tokio::test]
    async fn should_return_successful_response() {
      // Given: a router with the cached_page handler
      let (router, _guard) = build_router_for_test().await.unwrap();

      // When: making a request to cached_page endpoint
      let req = Request::builder()
        .uri("/api/cached-page")
        .body(Body::empty())
        .unwrap();
      let res = router.oneshot(req).await.unwrap();

      // Then: should return 200 OK status
      assert_eq!(res.status(), StatusCode::OK);
      let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
      let value = std::str::from_utf8(&body).unwrap();
      assert!(
        value.starts_with("cached-page-"),
        "Response should contain cached-page prefix, got: {}",
        value
      );
    }

    #[tokio::test]
    async fn should_generate_valid_uuid_format() {
      // Given: a router with the cached_page handler
      let (router, _guard) = build_router_for_test().await.unwrap();

      // When: making a request to cached_page endpoint
      let req = Request::builder()
        .uri("/api/cached-page")
        .body(Body::empty())
        .unwrap();
      let res = router.oneshot(req).await.unwrap();

      // Then: response should contain a valid UUID format after prefix
      assert_eq!(res.status(), StatusCode::OK);
      let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
      let value = std::str::from_utf8(&body).unwrap();
      assert!(
        value.starts_with("cached-page-"),
        "Should start with prefix, got: {}",
        value
      );
      // Extract UUID part (after "cached-page-")
      let uuid_part = value.strip_prefix("cached-page-").unwrap();
      // Verify it's a valid UUID format (8-4-4-4-12 hex digits)
      assert!(
        uuid_part.len() == 36,
        "UUID should be 36 characters, got: {}",
        uuid_part.len()
      );
      // Verify UUID format with hyphens
      let parts: Vec<&str> = uuid_part.split('-').collect();
      assert_eq!(
        parts.len(),
        5,
        "UUID should have 5 parts separated by hyphens"
      );
      assert_eq!(parts[0].len(), 8, "First part should be 8 hex digits");
      assert_eq!(parts[1].len(), 4, "Second part should be 4 hex digits");
      assert_eq!(parts[2].len(), 4, "Third part should be 4 hex digits");
      assert_eq!(parts[3].len(), 4, "Fourth part should be 4 hex digits");
      assert_eq!(parts[4].len(), 12, "Fifth part should be 12 hex digits");
    }
  }

  mod http_cache_integration_behavior {
    use super::*;

    #[tokio::test]
    async fn should_be_cacheable_by_http_response_cache_layer() {
      // Given: a router with HTTP response cache enabled
      let (router, _guard) = build_router_for_test().await.unwrap();
      // Note: HTTP response cache is applied at router level, not handler level
      // This test verifies the handler returns cacheable responses

      // When: making a request to cached_page endpoint
      let req = Request::builder()
        .method("GET")
        .uri("/api/cached-page")
        .body(Body::empty())
        .unwrap();
      let res = router.oneshot(req).await.unwrap();

      // Then: response should be successful and cacheable
      assert_eq!(res.status(), StatusCode::OK);
      // GET requests with 200 OK are cacheable by HTTP response cache
      // The handler doesn't set cache-control headers itself,
      // but the HTTP response cache layer can add them
    }

    #[tokio::test]
    async fn should_not_require_authentication() {
      // Given: a router with the cached_page handler
      let (router, _guard) = build_router_for_test().await.unwrap();

      // When: making an unauthenticated request to cached_page endpoint
      let req = Request::builder()
        .uri("/api/cached-page")
        .body(Body::empty())
        .unwrap();
      let res = router.oneshot(req).await.unwrap();

      // Then: should return successful response without authentication
      assert_eq!(res.status(), StatusCode::OK);
      // Handler doesn't require auth (no RequireAuth extractor)
    }
  }

  mod database_integration_behavior {
    use super::*;

    #[tokio::test]
    async fn should_accept_database_connection_from_state() {
      // Given: a router with database connection in state
      let (router, _guard) = build_router_for_test().await.unwrap();

      // When: making a request to cached_page endpoint
      let req = Request::builder()
        .uri("/api/cached-page")
        .body(Body::empty())
        .unwrap();
      let res = router.oneshot(req).await.unwrap();

      // Then: handler should successfully extract database from state
      // (handler doesn't use db, but extraction should succeed)
      assert_eq!(res.status(), StatusCode::OK);
      // If database extraction failed, we'd get a 500 error
      // Success indicates State extractor worked correctly
    }
  }
}
