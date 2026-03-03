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

/// Maps readiness check result to HTTP response (used by readyz and tests).
pub(crate) fn readiness_result_to_response(
  r: Result<(), sea_orm::DbErr>,
) -> (StatusCode, &'static str) {
  match r {
    Ok(()) => (StatusCode::OK, "ok"),
    Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "unavailable"),
  }
}

/// Readiness: DB reachable.
pub async fn readyz(State(db): State<DbConnection>) -> impl IntoResponse {
  let r = db.execute_unprepared("SELECT 1").await.map(drop);
  readiness_result_to_response(r)
}

/// Legacy health. Returns 200.
pub async fn healthz() -> impl IntoResponse {
  (StatusCode::OK, "ok")
}

#[cfg(test)]
mod tests {
  use super::*;
  use forge_config::DatabaseConfig;

  fn memory_db_config() -> DatabaseConfig {
    DatabaseConfig {
      url: "sqlite::memory:".to_string(),
      max_connections: None,
      min_connections: None,
      connect_timeout: None,
      idle_timeout: None,
      auto_migrate: false,
      auto_seed: false,
    }
  }

  async fn assert_status_body(res: axum::response::Response, status: u16, body: &str) {
    assert_eq!(res.status().as_u16(), status);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
      .await
      .unwrap();
    assert_eq!(std::str::from_utf8(&bytes).unwrap(), body);
  }

  #[tokio::test]
  async fn livez_returns_200_ok() {
    let res = livez().await.into_response();
    assert_status_body(res, 200, "ok").await;
  }

  #[tokio::test]
  async fn healthz_returns_200_ok() {
    let res = healthz().await.into_response();
    assert_status_body(res, 200, "ok").await;
  }

  #[tokio::test]
  async fn readyz_returns_200_when_db_ok() {
    let db = forge_db::initialize_database(&memory_db_config())
      .await
      .unwrap();
    let db = DbConnection::from(db);
    let res = readyz(State(db)).await.into_response();
    assert_status_body(res, 200, "ok").await;
  }

  #[test]
  fn readiness_result_to_response_ok_returns_200() {
    let (status, body) = readiness_result_to_response(Ok(()));
    assert_eq!(status.as_u16(), 200);
    assert_eq!(body, "ok");
  }

  #[test]
  fn readiness_result_to_response_err_returns_503() {
    let err = sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal(
      "test failure".to_string(),
    ));
    let (status, body) = readiness_result_to_response(Err(err));
    assert_eq!(status.as_u16(), 503);
    assert_eq!(body, "unavailable");
  }
}
