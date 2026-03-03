//! Health check endpoints: /healthz, /livez, /readyz.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use forge_db::DbConnection;
use sea_orm::ConnectionTrait;

/// Liveness: process is running.
pub async fn livez() -> impl IntoResponse {
  (StatusCode::OK, "ok")
}

/// Readiness: DB reachable.
pub async fn readyz(State(db): State<DbConnection>) -> impl IntoResponse {
  match db.execute_unprepared("SELECT 1").await {
    Ok(_) => (StatusCode::OK, "ok"),
    Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "unavailable"),
  }
}

/// Legacy health. Returns 200.
pub async fn healthz() -> impl IntoResponse {
  (StatusCode::OK, "ok")
}
