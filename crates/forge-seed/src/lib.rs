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
}
