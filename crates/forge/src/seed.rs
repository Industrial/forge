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
