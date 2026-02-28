use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

use crate::config::DatabaseConfig;

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

  let db = Database::connect(opt).await?;
  Ok(db)
}
