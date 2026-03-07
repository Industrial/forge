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
    let err = sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal("test failure".to_string()));
    let (status, body) = readiness_result_to_response(Err(err));
    assert_eq!(status.as_u16(), 503);
    assert_eq!(body, "unavailable");
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use forge_config::DatabaseConfig;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.

  mod liveness_endpoint_behavior {
    use super::*;

    #[tokio::test]
    async fn should_return_200_ok_when_process_is_running() {
      // Given: a running process
      // When: calling the livez endpoint
      let res = livez().await.into_response();

      // Then: should return 200 OK with "ok" body
      assert_eq!(res.status(), StatusCode::OK);
      let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
      assert_eq!(std::str::from_utf8(&bytes).unwrap(), "ok");
    }

    #[tokio::test]
    async fn should_always_succeed_regardless_of_database_state() {
      // Given: any system state (even if database is down)
      // When: calling the livez endpoint
      let res = livez().await.into_response();

      // Then: should always return 200 OK
      // (liveness doesn't depend on external services)
      assert_eq!(res.status(), StatusCode::OK);
    }
  }

  mod readiness_endpoint_behavior {
    use super::*;

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

    #[tokio::test]
    async fn should_return_200_ok_when_database_is_reachable() {
      // Given: a reachable database connection
      let config = memory_db_config();
      let db = forge_db::initialize_database(&config).await.unwrap();
      let db = DbConnection::from(db);

      // When: calling the readyz endpoint
      let res = readyz(State(db)).await.into_response();

      // Then: should return 200 OK with "ok" body
      assert_eq!(res.status(), StatusCode::OK);
      let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
      assert_eq!(std::str::from_utf8(&bytes).unwrap(), "ok");
    }

    #[tokio::test]
    async fn should_execute_database_query_to_check_readiness() {
      // Given: a database connection
      let config = memory_db_config();
      let db = forge_db::initialize_database(&config).await.unwrap();
      let db = DbConnection::from(db);

      // When: calling the readyz endpoint
      let res = readyz(State(db)).await.into_response();

      // Then: should successfully execute SELECT 1 query
      // (if query fails, status would be 503, so 200 confirms query succeeded)
      assert_eq!(res.status(), StatusCode::OK);
    }
  }

  mod readiness_result_mapping_behavior {
    use super::*;

    #[test]
    fn should_map_ok_result_to_200_status() {
      // Given: a successful readiness check result
      let result: Result<(), sea_orm::DbErr> = Ok(());

      // When: mapping the result to HTTP response
      let (status, body) = readiness_result_to_response(result);

      // Then: should return 200 OK status with "ok" body
      assert_eq!(status, StatusCode::OK);
      assert_eq!(body, "ok");
    }

    #[test]
    fn should_map_error_result_to_503_status() {
      // Given: a failed readiness check result
      let err = sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal(
        "database connection failed".to_string(),
      ));
      let result: Result<(), sea_orm::DbErr> = Err(err);

      // When: mapping the result to HTTP response
      let (status, body) = readiness_result_to_response(result);

      // Then: should return 503 SERVICE_UNAVAILABLE status with "unavailable" body
      assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
      assert_eq!(body, "unavailable");
    }

    #[test]
    fn should_map_any_database_error_to_503() {
      // Given: various types of database errors
      let errors = vec![
        sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal("connection error".to_string())),
        sea_orm::DbErr::Query(sea_orm::RuntimeErr::Internal("query error".to_string())),
        sea_orm::DbErr::Exec(sea_orm::RuntimeErr::Internal("exec error".to_string())),
      ];

      // When: mapping each error to HTTP response
      for err in errors {
        let (status, body) = readiness_result_to_response(Err(err));

        // Then: should always return 503 SERVICE_UNAVAILABLE
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body, "unavailable");
      }
    }
  }

  mod legacy_health_endpoint_behavior {
    use super::*;

    #[tokio::test]
    async fn should_return_200_ok_for_backward_compatibility() {
      // Given: a legacy health check request
      // When: calling the healthz endpoint
      let res = healthz().await.into_response();

      // Then: should return 200 OK with "ok" body
      assert_eq!(res.status(), StatusCode::OK);
      let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
      assert_eq!(std::str::from_utf8(&bytes).unwrap(), "ok");
    }

    #[tokio::test]
    async fn should_behave_identically_to_livez() {
      // Given: both healthz and livez endpoints
      // When: calling both endpoints
      let healthz_res = healthz().await.into_response();
      let livez_res = livez().await.into_response();

      // Then: should return identical responses
      assert_eq!(healthz_res.status(), livez_res.status());
      let healthz_bytes = axum::body::to_bytes(healthz_res.into_body(), usize::MAX)
        .await
        .unwrap();
      let livez_bytes = axum::body::to_bytes(livez_res.into_body(), usize::MAX)
        .await
        .unwrap();
      assert_eq!(
        std::str::from_utf8(&healthz_bytes).unwrap(),
        std::str::from_utf8(&livez_bytes).unwrap()
      );
    }
  }
}
