//! Health check endpoints: /healthz, /livez, /readyz.
//!
//! No component or server details in responses (status code + minimal body only).

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::{ConnectionTrait, DatabaseConnection};

/// **Liveness**: process is running. No dependency checks.
/// Returns 200 with minimal body. Used by orchestrators to decide whether to restart the process.
pub async fn livez() -> impl IntoResponse {
  (StatusCode::OK, "ok")
}

/// **Readiness**: service is ready to accept traffic (e.g. DB reachable).
/// Returns 200 if ready, 503 if not. Minimal body; no component details.
pub async fn readyz(State(db): State<DatabaseConnection>) -> impl IntoResponse {
  match db.execute_unprepared("SELECT 1").await {
    Ok(_) => (StatusCode::OK, "ok"),
    Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "unavailable"),
  }
}

/// **Healthz**: legacy/simple health. Returns 200 with minimal body.
/// No dependency checks or component disclosure.
pub async fn healthz() -> impl IntoResponse {
  (StatusCode::OK, "ok")
}
