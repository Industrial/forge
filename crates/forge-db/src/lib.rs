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

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn initialize_database_memory_succeeds() {
    let config = DatabaseConfig {
      url: "sqlite::memory:".to_string(),
      max_connections: None,
      min_connections: None,
      connect_timeout: None,
      idle_timeout: None,
      auto_migrate: false,
      auto_seed: false,
    };
    let res = initialize_database(&config).await;
    assert!(res.is_ok());
  }
}
