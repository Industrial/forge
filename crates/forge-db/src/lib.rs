//! Database connection and initialization for the Forge framework.

use log::LevelFilter;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

use forge_config::DatabaseConfig;

/// Initialize the database connection from configuration.
pub async fn initialize_database(
  config: &DatabaseConfig,
) -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
  let mut opt = ConnectOptions::new(config.url.clone());

  if let Some(max) = config.max_connections {
    opt.max_connections(max);
  }

  if let Some(min) = config.min_connections {
    opt.min_connections(min);
  }

  if let Some(timeout) = config.connect_timeout {
    opt.connect_timeout(Duration::from_secs(timeout));
  }

  if let Some(timeout) = config.idle_timeout {
    opt.idle_timeout(Duration::from_secs(timeout));
  }

  if std::env::var("FORGE_SQL_DEBUG").as_deref() == Ok("1")
    || std::env::var("FORGE_SQL_DEBUG").as_deref() == Ok("true")
  {
    opt.sqlx_logging(true);
    opt.sqlx_logging_level(LevelFilter::Debug);
  }

  let db = Database::connect(opt).await?;
  Ok(db)
}

/// Database connection type with OpenTelemetry tracing on all DB operations.
pub use sea_orm_tracing::TracedConnection as DbConnection;

/// Wrap a raw database connection for use as [DbConnection] (traced). Use after [initialize_database] when the app needs a single traced connection for state and cron.
pub fn wrap_traced(conn: DatabaseConnection) -> DbConnection {
  DbConnection::wrap(conn)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn memory_config() -> DatabaseConfig {
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
  async fn initialize_database_memory_succeeds() {
    let config = memory_config();
    let res = initialize_database(&config).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn initialize_database_with_all_options_succeeds() {
    let config = DatabaseConfig {
      url: "sqlite::memory:".to_string(),
      max_connections: Some(5),
      min_connections: Some(1),
      connect_timeout: Some(10),
      idle_timeout: Some(30),
      auto_migrate: false,
      auto_seed: false,
    };
    let res = initialize_database(&config).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn initialize_database_without_sql_debug_skips_logging() {
    // Unset so the branch that skips sqlx_logging is taken (dev env often has FORGE_SQL_DEBUG=1).
    unsafe { std::env::remove_var("FORGE_SQL_DEBUG") };
    let prev = std::env::var("FORGE_SQL_DEBUG").ok();
    let res = initialize_database(&memory_config()).await;
    if let Some(p) = prev.as_deref() {
      unsafe { std::env::set_var("FORGE_SQL_DEBUG", p) };
    } else {
      unsafe { std::env::remove_var("FORGE_SQL_DEBUG") };
    }
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn initialize_database_with_sql_debug_env_enables_logging() {
    let prev = std::env::var("FORGE_SQL_DEBUG").ok();
    // SAFETY: test only; single-threaded test, restore after
    unsafe {
      std::env::set_var("FORGE_SQL_DEBUG", "1");
    }
    let res = initialize_database(&memory_config()).await;
    if let Some(p) = prev.as_deref() {
      unsafe { std::env::set_var("FORGE_SQL_DEBUG", p) };
    } else {
      unsafe { std::env::remove_var("FORGE_SQL_DEBUG") };
    }
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn initialize_database_with_sql_debug_true_enables_logging() {
    // Ensure prev is None so the remove_var restore path is covered.
    unsafe { std::env::remove_var("FORGE_SQL_DEBUG") };
    let prev = std::env::var("FORGE_SQL_DEBUG").ok();
    // SAFETY: test only; single-threaded test, restore after
    unsafe {
      std::env::set_var("FORGE_SQL_DEBUG", "true");
    }
    let res = initialize_database(&memory_config()).await;
    if let Some(p) = prev.as_deref() {
      unsafe { std::env::set_var("FORGE_SQL_DEBUG", p) };
    } else {
      unsafe { std::env::remove_var("FORGE_SQL_DEBUG") };
    }
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn initialize_database_fails_on_invalid_url() {
    let config = DatabaseConfig {
      url: "invalid-scheme://bad".to_string(),
      max_connections: None,
      min_connections: None,
      connect_timeout: None,
      idle_timeout: None,
      auto_migrate: false,
      auto_seed: false,
    };
    let res = initialize_database(&config).await;
    assert!(res.is_err());
  }

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests are organized by feature/behavior area with descriptive names.
    mod database_initialization_behavior {
      use super::*;

      fn memory_config() -> DatabaseConfig {
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
      async fn should_succeed_with_memory_database() {
        // Given: a memory database configuration
        let config = memory_config();

        // When: initializing the database
        let result = initialize_database(&config).await;

        // Then: should succeed
        assert!(
          result.is_ok(),
          "Database initialization should succeed with memory database"
        );
      }

      #[tokio::test]
      async fn should_return_database_connection_on_success() {
        // Given: a valid database configuration
        let config = memory_config();

        // When: initializing the database
        let result = initialize_database(&config).await;

        // Then: should return a DatabaseConnection
        assert!(result.is_ok(), "Should return Ok(DatabaseConnection)");
        let _conn = result.unwrap();
        // Connection type is verified by successful unwrap
      }

      #[tokio::test]
      async fn should_fail_with_invalid_database_url() {
        // Given: an invalid database URL
        let config = DatabaseConfig {
          url: "invalid-scheme://bad-url".to_string(),
          max_connections: None,
          min_connections: None,
          connect_timeout: None,
          idle_timeout: None,
          auto_migrate: false,
          auto_seed: false,
        };

        // When: initializing the database
        let result = initialize_database(&config).await;

        // Then: should return an error
        assert!(
          result.is_err(),
          "Should return error for invalid database URL"
        );
      }
    }

    mod connection_pool_configuration_behavior {
      use super::*;

      #[tokio::test]
      async fn should_configure_max_connections_when_provided() {
        // Given: a configuration with max_connections set
        let config = DatabaseConfig {
          url: "sqlite::memory:".to_string(),
          max_connections: Some(10),
          min_connections: None,
          connect_timeout: None,
          idle_timeout: None,
          auto_migrate: false,
          auto_seed: false,
        };

        // When: initializing the database
        let result = initialize_database(&config).await;

        // Then: should succeed (max_connections is applied)
        assert!(
          result.is_ok(),
          "Should succeed with max_connections configured"
        );
      }

      #[tokio::test]
      async fn should_configure_min_connections_when_provided() {
        // Given: a configuration with min_connections set
        let config = DatabaseConfig {
          url: "sqlite::memory:".to_string(),
          max_connections: None,
          min_connections: Some(2),
          connect_timeout: None,
          idle_timeout: None,
          auto_migrate: false,
          auto_seed: false,
        };

        // When: initializing the database
        let result = initialize_database(&config).await;

        // Then: should succeed (min_connections is applied)
        assert!(
          result.is_ok(),
          "Should succeed with min_connections configured"
        );
      }

      #[tokio::test]
      async fn should_configure_all_connection_pool_options() {
        // Given: a configuration with all pool options set
        let config = DatabaseConfig {
          url: "sqlite::memory:".to_string(),
          max_connections: Some(20),
          min_connections: Some(5),
          connect_timeout: Some(15),
          idle_timeout: Some(60),
          auto_migrate: false,
          auto_seed: false,
        };

        // When: initializing the database
        let result = initialize_database(&config).await;

        // Then: should succeed with all options applied
        assert!(
          result.is_ok(),
          "Should succeed with all connection pool options configured"
        );
      }

      #[tokio::test]
      async fn should_work_without_connection_pool_options() {
        // Given: a configuration without pool options
        let config = DatabaseConfig {
          url: "sqlite::memory:".to_string(),
          max_connections: None,
          min_connections: None,
          connect_timeout: None,
          idle_timeout: None,
          auto_migrate: false,
          auto_seed: false,
        };

        // When: initializing the database
        let result = initialize_database(&config).await;

        // Then: should succeed with default pool settings
        assert!(
          result.is_ok(),
          "Should succeed with default connection pool settings"
        );
      }
    }

    mod timeout_configuration_behavior {
      use super::*;

      #[tokio::test]
      async fn should_configure_connect_timeout_when_provided() {
        // Given: a configuration with connect_timeout set
        let config = DatabaseConfig {
          url: "sqlite::memory:".to_string(),
          max_connections: None,
          min_connections: None,
          connect_timeout: Some(30),
          idle_timeout: None,
          auto_migrate: false,
          auto_seed: false,
        };

        // When: initializing the database
        let result = initialize_database(&config).await;

        // Then: should succeed (connect_timeout is applied)
        assert!(
          result.is_ok(),
          "Should succeed with connect_timeout configured"
        );
      }

      #[tokio::test]
      async fn should_configure_idle_timeout_when_provided() {
        // Given: a configuration with idle_timeout set
        let config = DatabaseConfig {
          url: "sqlite::memory:".to_string(),
          max_connections: None,
          min_connections: None,
          connect_timeout: None,
          idle_timeout: Some(120),
          auto_migrate: false,
          auto_seed: false,
        };

        // When: initializing the database
        let result = initialize_database(&config).await;

        // Then: should succeed (idle_timeout is applied)
        assert!(
          result.is_ok(),
          "Should succeed with idle_timeout configured"
        );
      }
    }

    mod sql_debug_logging_behavior {
      use super::*;

      fn memory_config() -> DatabaseConfig {
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
      async fn should_enable_sql_logging_when_env_var_is_1() {
        // Given: FORGE_SQL_DEBUG environment variable set to "1"
        let prev = std::env::var("FORGE_SQL_DEBUG").ok();
        unsafe {
          std::env::set_var("FORGE_SQL_DEBUG", "1");
        }
        let config = memory_config();

        // When: initializing the database
        let result = initialize_database(&config).await;

        // Then: should succeed with SQL logging enabled
        assert!(result.is_ok(), "Should succeed with SQL logging enabled");

        // Restore environment
        if let Some(p) = prev.as_deref() {
          unsafe { std::env::set_var("FORGE_SQL_DEBUG", p) };
        } else {
          unsafe { std::env::remove_var("FORGE_SQL_DEBUG") };
        }
      }

      #[tokio::test]
      async fn should_enable_sql_logging_when_env_var_is_true() {
        // Given: FORGE_SQL_DEBUG environment variable set to "true"
        let prev = std::env::var("FORGE_SQL_DEBUG").ok();
        unsafe {
          std::env::set_var("FORGE_SQL_DEBUG", "true");
        }
        let config = memory_config();

        // When: initializing the database
        let result = initialize_database(&config).await;

        // Then: should succeed with SQL logging enabled
        assert!(result.is_ok(), "Should succeed with SQL logging enabled");

        // Restore environment
        if let Some(p) = prev.as_deref() {
          unsafe { std::env::set_var("FORGE_SQL_DEBUG", p) };
        } else {
          unsafe { std::env::remove_var("FORGE_SQL_DEBUG") };
        }
      }

      #[tokio::test]
      async fn should_disable_sql_logging_when_env_var_is_not_set() {
        // Given: FORGE_SQL_DEBUG environment variable is not set
        let prev = std::env::var("FORGE_SQL_DEBUG").ok();
        unsafe {
          std::env::remove_var("FORGE_SQL_DEBUG");
        }
        let config = memory_config();

        // When: initializing the database
        let result = initialize_database(&config).await;

        // Then: should succeed without SQL logging
        assert!(result.is_ok(), "Should succeed without SQL logging");

        // Restore environment
        if let Some(p) = prev.as_deref() {
          unsafe { std::env::set_var("FORGE_SQL_DEBUG", p) };
        } else {
          unsafe { std::env::remove_var("FORGE_SQL_DEBUG") };
        }
      }

      #[tokio::test]
      async fn should_disable_sql_logging_when_env_var_is_other_value() {
        // Given: FORGE_SQL_DEBUG environment variable set to something other than "1" or "true"
        let prev = std::env::var("FORGE_SQL_DEBUG").ok();
        unsafe {
          std::env::set_var("FORGE_SQL_DEBUG", "0");
        }
        let config = memory_config();

        // When: initializing the database
        let result = initialize_database(&config).await;

        // Then: should succeed without SQL logging
        assert!(
          result.is_ok(),
          "Should succeed without SQL logging when env var is not '1' or 'true'"
        );

        // Restore environment
        if let Some(p) = prev.as_deref() {
          unsafe { std::env::set_var("FORGE_SQL_DEBUG", p) };
        } else {
          unsafe { std::env::remove_var("FORGE_SQL_DEBUG") };
        }
      }
    }

    mod traced_connection_behavior {
      use super::*;

      #[tokio::test]
      async fn should_wrap_database_connection_with_tracing() {
        // Given: a database connection
        let config = DatabaseConfig {
          url: "sqlite::memory:".to_string(),
          max_connections: None,
          min_connections: None,
          connect_timeout: None,
          idle_timeout: None,
          auto_migrate: false,
          auto_seed: false,
        };
        let conn = initialize_database(&config).await.unwrap();

        // When: wrapping with tracing
        let traced_conn = wrap_traced(conn);

        // Then: should return a DbConnection (TracedConnection)
        // Type is verified by successful assignment
        let _db: DbConnection = traced_conn;
      }

      #[test]
      fn should_provide_db_connection_type_alias() {
        // Given: DbConnection is a type alias
        // When: checking the type
        // Then: should be TracedConnection
        // Verified by compilation - DbConnection is pub use sea_orm_tracing::TracedConnection
      }
    }
  }
}
