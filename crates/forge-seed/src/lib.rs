//! Database seeding orchestrator for the Forge framework.

use forge_db::DbConnection;
use futures::future::BoxFuture;

/// Orchestrator for running database seeds.
pub struct Seeder;

impl Seeder {
  /// Run the provided seeding function.
  pub async fn run<F>(db: &DbConnection, seed_fn: F) -> Result<(), Box<dyn std::error::Error>>
  where
    F: Fn(DbConnection) -> BoxFuture<'static, Result<(), Box<dyn std::error::Error>>> + Send + Sync,
  {
    seed_fn(db.clone()).await
  }
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

  #[tokio::test]
  async fn run_invokes_seed_fn_and_returns_ok() {
    let conn = forge_db::initialize_database(&memory_db_config())
      .await
      .unwrap();
    let db = DbConnection::from(conn);
    let res = Seeder::run(&db, |_| Box::pin(async { Ok(()) })).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn run_returns_error_when_seed_fn_fails() {
    let conn = forge_db::initialize_database(&memory_db_config())
      .await
      .unwrap();
    let db = DbConnection::from(conn);
    let res = Seeder::run(&db, |_| {
      Box::pin(async { Err::<(), _>("seed failed".into()) })
    })
    .await;
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("seed failed"));
  }

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests verify seeder orchestration, error handling, and database connection passing.

    mod seeder_execution_behavior {
      use super::*;

      #[tokio::test]
      async fn should_execute_seed_function_successfully() {
        // Given: a database connection and a successful seed function
        let conn = forge_db::initialize_database(&memory_db_config())
          .await
          .unwrap();
        let db = DbConnection::from(conn);

        // When: running the seeder with a successful seed function
        let result = Seeder::run(&db, |_| Box::pin(async { Ok(()) })).await;

        // Then: should return Ok
        assert!(result.is_ok(), "Seeder should execute successfully");
      }

      #[tokio::test]
      async fn should_pass_database_connection_to_seed_function() {
        // Given: a database connection
        let conn = forge_db::initialize_database(&memory_db_config())
          .await
          .unwrap();
        let db = DbConnection::from(conn);
        let _db_clone = db.clone();

        // When: running the seeder with a function that captures the connection
        let captured_conn = std::sync::Arc::new(std::sync::Mutex::new(None));
        let captured_conn_clone = captured_conn.clone();
        let result = Seeder::run(&db, move |conn| {
          let captured_conn = captured_conn_clone.clone();
          Box::pin(async move {
            *captured_conn.lock().unwrap() = Some(conn.clone());
            Ok(())
          })
        })
        .await;

        // Then: seed function should receive the connection
        assert!(result.is_ok(), "Seeder should execute successfully");
        assert!(
          captured_conn.lock().unwrap().is_some(),
          "Seed function should receive database connection"
        );
      }

      #[tokio::test]
      async fn should_await_seed_function_completion() {
        // Given: a database connection
        let conn = forge_db::initialize_database(&memory_db_config())
          .await
          .unwrap();
        let db = DbConnection::from(conn);

        // When: running the seeder with an async function
        let execution_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag_clone = execution_flag.clone();
        let result = Seeder::run(&db, move |_| {
          let flag = flag_clone.clone();
          Box::pin(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            flag.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
          })
        })
        .await;

        // Then: seed function should complete before seeder returns
        assert!(result.is_ok(), "Seeder should execute successfully");
        assert!(
          execution_flag.load(std::sync::atomic::Ordering::SeqCst),
          "Seed function should complete execution"
        );
      }
    }

    mod seeder_error_handling_behavior {
      use super::*;

      #[tokio::test]
      async fn should_propagate_seed_function_errors() {
        // Given: a database connection and a failing seed function
        let conn = forge_db::initialize_database(&memory_db_config())
          .await
          .unwrap();
        let db = DbConnection::from(conn);

        // When: running the seeder with a failing seed function
        let result = Seeder::run(&db, |_| {
          Box::pin(async { Err::<(), _>("seed failed".into()) })
        })
        .await;

        // Then: should return error
        assert!(result.is_err(), "Seeder should propagate seed function errors");
      }

      #[tokio::test]
      async fn should_preserve_error_message_from_seed_function() {
        // Given: a database connection and a failing seed function with specific error
        let conn = forge_db::initialize_database(&memory_db_config())
          .await
          .unwrap();
        let db = DbConnection::from(conn);
        let error_message = "custom seed error message";

        // When: running the seeder with a failing seed function
        let result = Seeder::run(&db, move |_| {
          let msg = error_message.to_string();
          Box::pin(async move { Err::<(), _>(msg.into()) })
        })
        .await;

        // Then: error message should be preserved
        assert!(result.is_err(), "Seeder should return error");
        assert!(
          result.unwrap_err().to_string().contains(error_message),
          "Error message should be preserved"
        );
      }

      #[tokio::test]
      async fn should_handle_different_error_types() {
        // Given: a database connection
        let conn = forge_db::initialize_database(&memory_db_config())
          .await
          .unwrap();
        let db = DbConnection::from(conn);

        // When: running the seeder with different error types
        let result1 = Seeder::run(&db, |_| {
          Box::pin(async { Err::<(), _>("string error".into()) })
        })
        .await;
        let result2 = Seeder::run(&db, |_| {
          Box::pin(async {
            Err::<(), _>(std::io::Error::new(
              std::io::ErrorKind::Other,
              "io error",
            )
            .into())
          })
        })
        .await;

        // Then: both should return errors
        assert!(result1.is_err(), "Should handle string errors");
        assert!(result2.is_err(), "Should handle IO errors");
      }
    }

    mod seeder_concurrency_behavior {
      use super::*;

      #[tokio::test]
      async fn should_handle_concurrent_seeder_calls() {
        // Given: a database connection
        let conn = forge_db::initialize_database(&memory_db_config())
          .await
          .unwrap();
        let db = DbConnection::from(conn);

        // When: running multiple seeders concurrently
        let db1 = db.clone();
        let db2 = db.clone();
        let db3 = db.clone();

        let (r1, r2, r3) = tokio::join!(
          Seeder::run(&db1, |_| Box::pin(async { Ok(()) })),
          Seeder::run(&db2, |_| Box::pin(async { Ok(()) })),
          Seeder::run(&db3, |_| Box::pin(async { Ok(()) }))
        );

        // Then: all should complete successfully
        assert!(r1.is_ok(), "First seeder should succeed");
        assert!(r2.is_ok(), "Second seeder should succeed");
        assert!(r3.is_ok(), "Third seeder should succeed");
      }

      #[tokio::test]
      async fn should_isolate_errors_between_concurrent_calls() {
        // Given: a database connection
        let conn = forge_db::initialize_database(&memory_db_config())
          .await
          .unwrap();
        let db = DbConnection::from(conn);

        // When: running seeders concurrently with mixed success/failure
        let db1 = db.clone();
        let db2 = db.clone();
        let db3 = db.clone();

        let (r1, r2, r3) = tokio::join!(
          Seeder::run(&db1, |_| Box::pin(async { Ok(()) })),
          Seeder::run(&db2, |_| Box::pin(async { Err::<(), _>("error".into()) })),
          Seeder::run(&db3, |_| Box::pin(async { Ok(()) }))
        );

        // Then: errors should not affect other calls
        assert!(r1.is_ok(), "First seeder should succeed");
        assert!(r2.is_err(), "Second seeder should fail");
        assert!(r3.is_ok(), "Third seeder should succeed");
      }
    }

    mod seeder_api_behavior {
      use super::*;

      #[test]
      fn should_have_seeder_struct() {
        // Given: Seeder struct
        // When: checking struct existence
        // Then: should be accessible
        let _seeder = Seeder;
        // Test passes if struct compiles
      }

      #[test]
      fn should_have_run_method() {
        // Given: Seeder struct
        // When: checking method existence
        // Then: should have run method
        // Verified by compilation - run method exists
      }

      #[tokio::test]
      async fn should_accept_boxed_future_seed_function() {
        // Given: a database connection
        let conn = forge_db::initialize_database(&memory_db_config())
          .await
          .unwrap();
        let db = DbConnection::from(conn);

        // When: running seeder with BoxFuture seed function
        let result = Seeder::run(&db, |_| Box::pin(async { Ok(()) })).await;

        // Then: should accept and execute BoxFuture
        assert!(result.is_ok(), "Should accept BoxFuture seed function");
      }

      #[tokio::test]
      async fn should_require_send_sync_for_seed_function() {
        // Given: a database connection
        let conn = forge_db::initialize_database(&memory_db_config())
          .await
          .unwrap();
        let db = DbConnection::from(conn);

        // When: running seeder with Send + Sync seed function
        let result = Seeder::run(&db, |_| Box::pin(async { Ok(()) })).await;

        // Then: should compile (Send + Sync requirement satisfied)
        assert!(result.is_ok(), "Should accept Send + Sync seed function");
      }
    }
  }
}
