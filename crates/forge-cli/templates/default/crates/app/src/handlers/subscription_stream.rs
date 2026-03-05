//! Long-lived stream for subscription invalidation events (Epic 8 §2). Client opens stream; server pushes invalidation hints.

use axum::body::Body;
use axum::extract::Extension;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use db::auth::Backend;
use forge_auth::token_auth::RequireAuth;
use futures_util::stream::{self, StreamExt};
use std::convert::Infallible;

use crate::subscriptions::SubscriptionStore;

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
    [(
      header::CONTENT_TYPE,
      "text/event-stream; charset=utf-8",
    )],
    body,
  )
    .into_response()
}
