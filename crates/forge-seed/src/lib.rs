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
