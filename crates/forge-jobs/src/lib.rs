//! Background jobs via Apalis + SQLite. Generic job queue and worker; no scheduling.

use apalis::prelude::*;
use apalis_sqlite::SqliteStorage;
use forge_db::DbConnection;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{error, info};

/// Single job type: identified by task name for dispatch.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ScheduledTaskJob {
  pub task_name: String,
}

/// Type-erased task: takes DB, returns a future. Used by the worker to run jobs by name.
pub type TaskFn = Box<
  dyn Fn(DbConnection)
    -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>
    + Send
    + Sync,
>;

fn normalize_job_pool_url(url: &str) -> &str {
  if url == "sqlite::memory:" {
    "sqlite://:memory:"
  } else {
    url
  }
}

/// Storage handle for the job queue (Apalis SqliteStorage with default codec and fetcher).
pub type JobQueueStorage =
  SqliteStorage<ScheduledTaskJob, apalis_codec::json::JsonCodec<Vec<u8>>, apalis_sqlite::fetcher::SqliteFetcher>;

/// Sets up the job queue storage. Returns a cloneable storage: use one clone for pushing jobs,
/// pass another to [run_worker].
pub async fn setup_queue(
  db_url: &str,
) -> Result<JobQueueStorage, Box<dyn std::error::Error + Send + Sync>> {
  use apalis_sqlite::SqlitePool;
  let url = normalize_job_pool_url(db_url);
  let pool = SqlitePool::connect(url).await?;
  SqliteStorage::<(), (), ()>::setup(&pool).await?;
  Ok(SqliteStorage::new(&pool))
}

/// Runs the worker: pulls jobs from `storage` and dispatches by name using `registry`.
/// Spawns the worker in the background and returns immediately.
pub async fn run_worker(
  storage: JobQueueStorage,
  db: DbConnection,
  registry: Arc<HashMap<String, TaskFn>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let worker = WorkerBuilder::new("forge-scheduled-tasks")
    .backend(storage)
    .data((registry, db))
    .build(run_scheduled_task);
  tokio::spawn(async move {
    if let Err(e) = worker.run().await {
      error!(error = %e, "forge scheduled tasks worker exited with error");
    }
  });
  Ok(())
}

async fn run_scheduled_task(
  job: ScheduledTaskJob,
  data: Data<(Arc<HashMap<String, TaskFn>>, DbConnection)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let (registry, db): &(Arc<HashMap<String, TaskFn>>, DbConnection) = &data;
  if let Some(task) = registry.get(&job.task_name) {
    if let Err(e) = (task)(db.clone()).await {
      error!(task = %job.task_name, error = %e, "scheduled task failed");
      return Err(e);
    }
    info!(task = %job.task_name, "scheduled task completed");
  } else {
    error!(task = %job.task_name, "no handler registered for scheduled task");
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use apalis::prelude::Data;
  use sea_orm::{ConnectOptions, Database};
  use sea_orm_tracing::TracedConnection;
  use std::time::Duration;

  fn ok_task() -> TaskFn {
    Box::new(|_db: DbConnection| {
      Box::pin(async move { Ok(()) })
        as Pin<
          Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>,
        >
    })
  }

  fn err_task() -> TaskFn {
    Box::new(|_db: DbConnection| Box::pin(async move { Err("task failed".into()) }))
  }

  async fn test_db() -> DbConnection {
    let conn = Database::connect(ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap();
    TracedConnection::wrap(conn)
  }

  #[test]
  fn normalize_job_pool_url_normalizes_sqlite_memory() {
    assert_eq!(
      normalize_job_pool_url("sqlite::memory:"),
      "sqlite://:memory:"
    );
  }

  #[test]
  fn normalize_job_pool_url_passthrough_for_other_urls() {
    assert_eq!(
      normalize_job_pool_url("sqlite:///path/to/db.sqlite"),
      "sqlite:///path/to/db.sqlite"
    );
    assert_eq!(
      normalize_job_pool_url("sqlite://:memory:"),
      "sqlite://:memory:"
    );
  }

  #[test]
  fn scheduled_task_job_serde_roundtrip() {
    let job = ScheduledTaskJob {
      task_name: "test-task".to_string(),
    };
    let json = serde_json::to_string(&job).unwrap();
    let parsed: ScheduledTaskJob = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.task_name, job.task_name);
  }

  #[tokio::test]
  async fn run_scheduled_task_found_and_succeeds() {
    let db = test_db().await;
    let mut reg = HashMap::new();
    reg.insert("ok-task".to_string(), ok_task());
    let registry = Arc::new(reg);
    let data = Data::new((registry, db));
    let job = ScheduledTaskJob {
      task_name: "ok-task".to_string(),
    };
    let res = run_scheduled_task(job, data).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn run_scheduled_task_found_and_returns_err() {
    let db = test_db().await;
    let mut reg = HashMap::new();
    reg.insert("err-task".to_string(), err_task());
    let registry = Arc::new(reg);
    let data = Data::new((registry, db));
    let job = ScheduledTaskJob {
      task_name: "err-task".to_string(),
    };
    let res = run_scheduled_task(job, data).await;
    assert!(res.is_err());
  }

  #[tokio::test]
  async fn run_scheduled_task_not_in_registry() {
    let db = test_db().await;
    let registry = Arc::new(HashMap::new());
    let data = Data::new((registry, db));
    let job = ScheduledTaskJob {
      task_name: "missing".to_string(),
    };
    let res = run_scheduled_task(job, data).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn setup_queue_succeeds_with_memory_url() {
    let res = setup_queue("sqlite::memory:").await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn setup_queue_fails_on_invalid_db_url() {
    let res = setup_queue("invalid://bad").await;
    assert!(res.is_err());
  }

  #[tokio::test]
  async fn run_worker_starts_with_empty_registry() {
    let db = test_db().await;
    let storage = setup_queue("sqlite::memory:").await.unwrap();
    let registry = Arc::new(HashMap::new());
    let res = run_worker(storage, db, registry).await;
    assert!(res.is_ok());
    tokio::time::sleep(Duration::from_millis(50)).await;
  }

  #[tokio::test]
  async fn run_worker_starts_with_one_task() {
    let db = test_db().await;
    let storage = setup_queue("sqlite::memory:").await.unwrap();
    let mut reg = HashMap::new();
    reg.insert("test-task".to_string(), ok_task());
    let registry = Arc::new(reg);
    let res = run_worker(storage, db, registry).await;
    assert!(res.is_ok());
    tokio::time::sleep(Duration::from_millis(50)).await;
  }
}
