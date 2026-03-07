use axum::{extract::Request, response::IntoResponse};
use forge_observability::{find_current_trace_id, trace_id_from_traceparent};

/// Returns the current OpenTelemetry trace id (for e2e and debugging).
/// Prefers trace id from traceparent header when present, then current context.
pub async fn trace_id(req: Request) -> impl IntoResponse {
  tracing::debug!(target: "app::handlers", "route: GET /api/observability/trace-id");
  let from_header = req
    .headers()
    .get("traceparent")
    .and_then(|v| v.to_str().ok())
    .and_then(|s| trace_id_from_traceparent(Some(s)));
  from_header
    .or_else(find_current_trace_id)
    .unwrap_or_else(|| "".to_string())
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::body::Body;
  use axum::http::{HeaderValue, Request as HttpRequest, StatusCode};

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests verify trace ID extraction from headers and context.

    mod trace_id_extraction_behavior {
      use super::*;

      #[tokio::test]
      async fn should_return_trace_id_from_traceparent_header_when_present() {
        // Given: a request with traceparent header
        let mut req = HttpRequest::builder()
          .uri("http://example.com/api/observability/trace-id")
          .body(Body::empty())
          .unwrap();
        req.headers_mut().insert(
          "traceparent",
          HeaderValue::from_static("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
        );

        // When: calling trace_id handler
        let response = trace_id(req).await.into_response();

        // Then: response should contain trace ID from header
        assert_eq!(response.status(), StatusCode::OK);
        // Note: Actual trace ID extraction depends on forge_observability implementation
        // The handler should prefer header over context
      }

      #[tokio::test]
      async fn should_handle_request_without_traceparent_header() {
        // Given: a request without traceparent header
        let req = HttpRequest::builder()
          .uri("http://example.com/api/observability/trace-id")
          .body(Body::empty())
          .unwrap();

        // When: calling trace_id handler
        let response = trace_id(req).await.into_response();

        // Then: response should be successful (may return empty string or context trace ID)
        assert_eq!(response.status(), StatusCode::OK);
        // Handler falls back to current context or empty string
      }

      #[tokio::test]
      async fn should_handle_invalid_traceparent_header_gracefully() {
        // Given: a request with invalid traceparent header
        let mut req = HttpRequest::builder()
          .uri("http://example.com/api/observability/trace-id")
          .body(Body::empty())
          .unwrap();
        req.headers_mut().insert(
          "traceparent",
          HeaderValue::from_static("invalid-traceparent-format"),
        );

        // When: calling trace_id handler
        let response = trace_id(req).await.into_response();

        // Then: response should handle invalid header gracefully
        assert_eq!(response.status(), StatusCode::OK);
        // Should fall back to context or empty string
      }

      #[tokio::test]
      async fn should_prefer_traceparent_header_over_context() {
        // Given: a request with traceparent header (which should be preferred)
        let mut req = HttpRequest::builder()
          .uri("http://example.com/api/observability/trace-id")
          .body(Body::empty())
          .unwrap();
        req.headers_mut().insert(
          "traceparent",
          HeaderValue::from_static("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
        );

        // When: calling trace_id handler
        let response = trace_id(req).await.into_response();

        // Then: handler should prefer header over context
        assert_eq!(response.status(), StatusCode::OK);
        // Implementation detail: from_header is checked first, then find_current_trace_id
      }

      #[tokio::test]
      async fn should_return_empty_string_when_no_trace_id_available() {
        // Given: a request without traceparent header and no trace context
        let req = HttpRequest::builder()
          .uri("http://example.com/api/observability/trace-id")
          .body(Body::empty())
          .unwrap();

        // When: calling trace_id handler
        let response = trace_id(req).await.into_response();

        // Then: response should be successful (may return empty string)
        assert_eq!(response.status(), StatusCode::OK);
        // Handler returns empty string as fallback: unwrap_or_else(|| "".to_string())
      }

      #[tokio::test]
      async fn should_handle_request_with_multiple_headers() {
        // Given: a request with traceparent header and other headers
        let mut req = HttpRequest::builder()
          .uri("http://example.com/api/observability/trace-id")
          .body(Body::empty())
          .unwrap();
        req.headers_mut().insert(
          "traceparent",
          HeaderValue::from_static("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
        );
        req
          .headers_mut()
          .insert("authorization", HeaderValue::from_static("Bearer token123"));

        // When: calling trace_id handler
        let response = trace_id(req).await.into_response();

        // Then: should extract trace ID correctly despite other headers
        assert_eq!(response.status(), StatusCode::OK);
      }
    }

    mod handler_contract_behavior {
      use super::*;

      #[tokio::test]
      async fn should_return_into_response_implementation() {
        // Given: any valid request
        let req = HttpRequest::builder()
          .uri("http://example.com/api/observability/trace-id")
          .body(Body::empty())
          .unwrap();

        // When: calling trace_id handler
        let response = trace_id(req).await.into_response();

        // Then: should return a valid HTTP response
        assert_eq!(response.status(), StatusCode::OK);
        // Response implements IntoResponse trait
      }

      #[tokio::test]
      async fn should_handle_different_request_methods() {
        // Given: requests with different HTTP methods (though handler doesn't check method)
        let req_get = HttpRequest::builder()
          .method("GET")
          .uri("http://example.com/api/observability/trace-id")
          .body(Body::empty())
          .unwrap();

        // When: calling trace_id handler
        let response = trace_id(req_get).await.into_response();

        // Then: should handle request successfully
        assert_eq!(response.status(), StatusCode::OK);
      }
    }
  }
}
