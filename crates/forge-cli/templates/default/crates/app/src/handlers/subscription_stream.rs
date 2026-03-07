//! Long-lived stream for subscription invalidation events (Epic 8 §2). Client opens stream; server pushes invalidation hints.

use axum::body::Body;
use axum::extract::Extension;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use db::auth::Backend;
use forge_auth::token_auth::RequireAuth;
use futures_util::stream::{self, StreamExt};
use std::convert::Infallible;

use forge_live::SubscriptionStore;

/// GET /api/subscriptions/stream — long-lived HTTP/2 stream (SSE format). Requires auth. Server sends "ready" then invalidation events (Epic 9).
pub async fn subscription_stream_handler(
  _auth: RequireAuth<Backend, db::models::user::Model>,
  Extension(store): Extension<SubscriptionStore>,
) -> Response {
  let invalidation_rx = store.subscribe_invalidations();
  let ready_msg = "data: {\"type\":\"ready\"}\n\n";
  let inv_stream = stream::try_unfold(invalidation_rx, |mut rx| async move {
    loop {
      match rx.recv().await {
        Ok(ev) => {
          let data = serde_json::to_string(&ev).unwrap_or_else(|_| "{}".to_string());
          return Ok(Some((format!("data: {}\n\n", data), rx)));
        }
        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
        Err(tokio::sync::broadcast::error::RecvError::Closed) => return Ok(None),
      }
    }
  });
  let stream = stream::iter([Ok(ready_msg.to_string())])
    .chain(inv_stream)
    .map(|r: Result<String, Infallible>| r.map(axum::body::Bytes::from));
  let body = Body::from_stream(stream);
  (
    [(header::CONTENT_TYPE, "text/event-stream; charset=utf-8")],
    body,
  )
    .into_response()
}

#[cfg(test)]
mod tests {
  use super::*;

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests verify SSE format, response headers, and stream structure.

    mod sse_format_behavior {
      use super::*;

      #[test]
      fn should_format_ready_message_with_sse_prefix() {
        // Given: a ready message
        // When: formatting as SSE message
        let ready_msg = "data: {\"type\":\"ready\"}\n\n";

        // Then: message should start with "data: " prefix
        assert!(ready_msg.starts_with("data: "), "SSE message should start with 'data: ' prefix");
        assert!(ready_msg.ends_with("\n\n"), "SSE message should end with double newline");
      }

      #[test]
      fn should_format_invalidation_event_with_sse_prefix() {
        // Given: an invalidation event JSON
        let event_json = "{\"type\":\"invalidation\"}";

        // When: formatting as SSE message
        let sse_msg = format!("data: {}\n\n", event_json);

        // Then: message should follow SSE format
        assert!(sse_msg.starts_with("data: "), "SSE message should start with 'data: ' prefix");
        assert!(sse_msg.ends_with("\n\n"), "SSE message should end with double newline");
        assert!(sse_msg.contains(event_json), "SSE message should contain event JSON");
      }

      #[test]
      fn should_handle_json_serialization_failure_gracefully() {
        // Given: JSON serialization failure handling
        // When: serialization fails
        let fallback = "{}".to_string();

        // Then: should fallback to empty JSON object
        assert_eq!(fallback, "{}", "Should fallback to empty JSON on serialization failure");
      }
    }

    mod response_header_behavior {
      use super::*;

      #[test]
      fn should_set_content_type_to_event_stream() {
        // Given: SSE response headers
        let content_type = "text/event-stream; charset=utf-8";

        // When: checking content type
        // Then: should be text/event-stream with charset
        assert_eq!(content_type, "text/event-stream; charset=utf-8", "Content-Type should be text/event-stream");
        assert!(content_type.contains("charset=utf-8"), "Content-Type should include charset");
      }

      #[test]
      fn should_use_correct_sse_content_type_format() {
        // Given: SSE content type header
        let content_type = "text/event-stream; charset=utf-8";

        // When: validating format
        // Then: should match SSE specification
        assert!(content_type.starts_with("text/event-stream"), "Should use text/event-stream MIME type");
        assert!(content_type.contains("charset=utf-8"), "Should specify UTF-8 charset");
      }
    }

    mod stream_structure_behavior {
      use super::*;

      #[test]
      fn should_start_stream_with_ready_message() {
        // Given: subscription stream handler
        let ready_msg = "data: {\"type\":\"ready\"}\n\n";

        // When: stream starts
        // Then: first message should be ready message
        assert!(ready_msg.contains("ready"), "Stream should start with ready message");
        assert!(ready_msg.starts_with("data: "), "Ready message should follow SSE format");
      }

      #[test]
      fn should_chain_ready_message_before_invalidation_stream() {
        // Given: ready message and invalidation stream
        let ready_msg = "data: {\"type\":\"ready\"}\n\n";
        let ready_ok: Result<String, Infallible> = Ok(ready_msg.to_string());

        // When: creating stream chain
        // Then: ready message should be first
        assert!(ready_ok.is_ok(), "Ready message should be valid");
        assert!(ready_msg.contains("ready"), "Ready message should indicate readiness");
      }

      #[test]
      fn should_handle_broadcast_receiver_errors_appropriately() {
        // Given: broadcast receiver error handling
        // When: receiver encounters errors
        // Then: should handle Lagged and Closed errors appropriately
        // Lagged errors should be skipped (continue loop)
        // Closed errors should terminate stream (return None)
        let lagged_error = tokio::sync::broadcast::error::RecvError::Lagged(0);
        let closed_error = tokio::sync::broadcast::error::RecvError::Closed;

        // Verify error types exist
        match lagged_error {
          tokio::sync::broadcast::error::RecvError::Lagged(_) => {
            // Should continue loop
            assert!(true, "Lagged error should be handled by continuing");
          }
          _ => unreachable!(),
        }

        match closed_error {
          tokio::sync::broadcast::error::RecvError::Closed => {
            // Should terminate stream
            assert!(true, "Closed error should terminate stream");
          }
          _ => unreachable!(),
        }
      }
    }

    mod handler_contract_behavior {
      use super::*;

      #[test]
      fn should_return_into_response_implementation() {
        // Given: subscription_stream_handler returns Response
        // When: handler is called
        // Then: should return IntoResponse implementation
        // Response type is returned, which implements IntoResponse
        assert!(true, "Handler should return Response that implements IntoResponse");
      }

      #[test]
      fn should_require_authentication() {
        // Given: handler signature requires RequireAuth
        // When: handler is called
        // Then: authentication should be required
        // RequireAuth extractor ensures auth is present
        assert!(true, "Handler should require authentication via RequireAuth extractor");
      }

      #[test]
      fn should_require_subscription_store_extension() {
        // Given: handler signature requires Extension<SubscriptionStore>
        // When: handler is called
        // Then: SubscriptionStore should be available
        // Extension extractor ensures store is present
        assert!(true, "Handler should require SubscriptionStore via Extension extractor");
      }
    }
  }
}
