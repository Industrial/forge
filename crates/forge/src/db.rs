use log::LevelFilter;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

use crate::config::DatabaseConfig;

/// Initialize the database connection from configuration.
/// Uses SeaORM [ConnectOptions] for all backends (SQLite, PostgreSQL, etc.) so that
/// DB-level OpenTelemetry (e.g. the sea-orm-tracing crate wrapping the connection)
/// can apply to SQLite and non-SQLite alike.
///
/// SQL statement logging is enabled when the `FORGE_SQL_DEBUG` env var is set (e.g. `1` or `true`).
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

  // DB-level OTel: SeaORM 1.1 does not expose set_auto_tracing on ConnectOptions.
  // For SQLite and other backends, you can add spans via sea-orm-tracing (wrap the
  // returned DatabaseConnection with TracedConnection) or a future SeaORM release.

  let db = Database::connect(opt).await?;
  Ok(db)
}
