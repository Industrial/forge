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
