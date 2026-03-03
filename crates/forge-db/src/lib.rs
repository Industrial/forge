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
}
