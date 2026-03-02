use futures::future::BoxFuture;

use crate::DbConnection;

/// Orchestrator for running database seeds.
///
/// In Forge, seeds are simple idempotent functions that run on every startup
/// if configured. Forge does not track seed execution history; idempotency
/// is the responsibility of the seed function implementation.
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
  use sea_orm::Database;

  #[tokio::test]
  async fn seeder_run_invokes_closure_and_returns_ok() {
    let conn = Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap();
    let db = crate::DbConnection::new(conn, sea_orm_tracing::TracingConfig::default());
    let res = Seeder::run(&db, |_| Box::pin(async { Ok(()) })).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn seeder_run_propagates_error() {
    let conn = Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap();
    let db = crate::DbConnection::new(conn, sea_orm_tracing::TracingConfig::default());
    let res = Seeder::run(&db, |_| {
      Box::pin(async { Err("seed failed".into()) })
    })
    .await;
    assert!(res.is_err());
  }
}
