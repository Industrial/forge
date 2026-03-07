//! BDD tests for GET /api/permissions.
//! Endpoint is unprotected: 200 for both anonymous and authenticated requests with permissions list.
//!
//! BDD-style tests focusing on behavior rather than implementation.

use axum::http::StatusCode;

const PERMISSIONS_PATH: &str = "/api/permissions";

mod bdd_tests {
  use super::*;

  mod get_permissions {
    use super::*;

    #[tokio::test]
    async fn should_return_200_with_permissions_list_when_anonymous() {
      // Given: an unauthenticated request
      let client = app::test_client().await.expect("test_client");

      // When: requesting GET /api/permissions without authentication
      let (status, body) = app::test_request(&client, "GET", PERMISSIONS_PATH, None, None, None)
        .await
        .unwrap();

      // Then: should return 200 OK with permissions array
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      assert!(json.get("permissions").and_then(|p| p.as_array()).is_some());
    }

    #[tokio::test]
    async fn should_return_200_with_permissions_list_when_authenticated() {
      // Given: an authenticated user (e.g. viewer) with migrations run
      use sea_orm::{ConnectionTrait, Statement};
      use sea_orm_migration::MigratorTrait;
      let (router, db_conn, guard) = app::build_router_for_test_with_db()
        .await
        .expect("build_router_for_test_with_db");
      let db_ref: &sea_orm::DatabaseConnection = db_conn.as_ref();
      migrations::Migrator::up(db_ref, None)
        .await
        .expect("migrations::Migrator::up");
      migrations::run_seeds(db_conn.clone())
        .await
        .expect("migrations::run_seeds");
      let _ = db_conn
        .execute(Statement::from_string(
          db_conn.get_database_backend(),
          "SELECT 1 FROM user LIMIT 1".to_string(),
        ))
        .await
        .expect("user table check after migration");
      let client = app::TestClient::InProcess {
        router,
        _guard: std::sync::Arc::new(std::sync::Mutex::new(Some(guard))),
      };
      let (token, _org_id, _role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");

      // When: requesting GET /api/permissions with authentication
      let (status, body) =
        app::test_request(&client, "GET", PERMISSIONS_PATH, Some(&token), None, None)
          .await
          .unwrap();

      // Then: should return 200 OK with permissions array
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      assert!(json.get("permissions").and_then(|p| p.as_array()).is_some());
    }
  }
}
