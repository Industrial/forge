use sea_orm::{ConnectOptions, Database, DatabaseConnection, SqlxSqliteConnector};
use sqlx::sqlite::SqlitePoolOptions;
use std::time::Duration;

use crate::config::DatabaseConfig;

/// Initialize the database connection from configuration.
pub async fn initialize_database(
  config: &DatabaseConfig,
) -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
  if config.url.starts_with("sqlite") {
    let mut pool_options = SqlitePoolOptions::new();

    if let Some(max) = config.max_connections {
      pool_options = pool_options.max_connections(max);
    }

    if let Some(min) = config.min_connections {
      pool_options = pool_options.min_connections(min);
    }

    if let Some(timeout) = config.connect_timeout {
      pool_options = pool_options.acquire_timeout(Duration::from_secs(timeout));
    }

    if let Some(timeout) = config.idle_timeout {
      pool_options = pool_options.idle_timeout(Some(Duration::from_secs(timeout)));
    }

    let pool = pool_options.connect(&config.url).await?;
    let db = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
    Ok(db)
  } else {
    // For other databases, fallback to default SeaORM connection
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

    let db = Database::connect(opt).await?;
    Ok(db)
  }
}
