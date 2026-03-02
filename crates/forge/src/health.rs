//! Health check endpoints: /healthz, /livez, /readyz.
//!
//! No component or server details in responses (status code + minimal body only).

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::ConnectionTrait;

use crate::DbConnection;

/// **Liveness**: process is running. No dependency checks.
/// Returns 200 with minimal body. Used by orchestrators to decide whether to restart the process.
pub async fn livez() -> impl IntoResponse {
  (StatusCode::OK, "ok")
}

/// **Readiness**: service is ready to accept traffic (e.g. DB reachable).
/// Returns 200 if ready, 503 if not. Minimal body; no component details.
pub async fn readyz(State(db): State<DbConnection>) -> impl IntoResponse {
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

#[cfg(test)]
mod tests {
  use super::*;
  use axum::body::to_bytes;
  use axum::http::StatusCode;

  #[tokio::test]
  async fn livez_returns_200_ok() {
    let res = livez().await.into_response();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), b"ok");
  }

  #[tokio::test]
  async fn healthz_returns_200_ok() {
    let res = healthz().await.into_response();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), b"ok");
  }

  #[tokio::test]
  async fn readyz_returns_200_when_db_ok() {
    use sea_orm::{Database, ConnectOptions};
    let opt = ConnectOptions::new("sqlite::memory:".to_string());
    let conn = Database::connect(opt).await.unwrap();
    let db = crate::DbConnection::new(conn, sea_orm_tracing::TracingConfig::default());
    let res = readyz(axum::extract::State(db)).await.into_response();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), b"ok");
  }
}
