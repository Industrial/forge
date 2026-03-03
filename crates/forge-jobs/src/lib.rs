//! Background jobs via Apalis + SQLite for scheduled tasks.

use apalis::prelude::*;
use apalis_sqlite::SqliteStorage;
use forge_cron::{next_daily_run, next_hourly_run, next_interval_run, CronSchedule, CronTaskBox};
use forge_db::DbConnection;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

/// Single job type for all scheduled tasks.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ScheduledTaskJob {
  pub task_name: String,
}

fn normalize_job_pool_url(url: &str) -> &str {
  if url == "sqlite::memory:" {
    "sqlite://:memory:"
  } else {
    url
  }
}

/// Runs the scheduler loops and the worker. Uses SQLite URL for the job queue.
pub async fn run_scheduler_and_worker(
  db_url: &str,
  db: DbConnection,
  tasks: Vec<(String, CronSchedule, CronTaskBox)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  use apalis_sqlite::SqlitePool;
  use tokio::time::{sleep_until, Instant};

  let url = normalize_job_pool_url(db_url);
  let pool = SqlitePool::connect(url).await?;
  SqliteStorage::<(), (), ()>::setup(&pool).await?;

  let storage_for_worker = SqliteStorage::<ScheduledTaskJob, (), ()>::new(&pool);
  let storage_for_scheduler = storage_for_worker.clone();

  let mut registry = HashMap::new();
  let mut schedule_list = Vec::new();
  for (name, schedule, task) in tasks {
    registry.insert(name.clone(), task);
    schedule_list.push((name, schedule));
  }
  let registry: Arc<HashMap<String, CronTaskBox>> = Arc::new(registry);

  for (name, schedule) in schedule_list {
    let name = name.clone();
    let mut storage = storage_for_scheduler.clone();
    let mut last_interval: Option<Instant> = None;
    tokio::spawn(async move {
      loop {
        let next_instant = match &schedule {
          CronSchedule::Interval(d) => {
            let (next, new) = next_interval_run(*d, last_interval);
            last_interval = new;
            next
          }
          CronSchedule::Hourly { minute } => next_hourly_run(*minute),
          CronSchedule::Daily { hour, minute } => next_daily_run(*hour, *minute),
        };
        sleep_until(next_instant).await;
        if let Err(e) = storage
          .push(ScheduledTaskJob {
            task_name: name.clone(),
          })
          .await
        {
          error!(cron = %name, error = %e, "failed to enqueue scheduled task");
        } else {
          info!(cron = %name, "enqueued scheduled task");
        }
      }
    });
  }

  let registry_worker = registry.clone();
  let worker = WorkerBuilder::new("forge-scheduled-tasks")
    .backend(storage_for_worker)
    .data((registry_worker, db))
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
  data: Data<(Arc<HashMap<String, CronTaskBox>>, DbConnection)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let (registry, db): &(Arc<HashMap<String, CronTaskBox>>, DbConnection) = &data;
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
  use forge_cron::CronTaskBox;
  use sea_orm::{ConnectOptions, Database};
  use sea_orm_tracing::TracedConnection;
  use std::pin::Pin;

  fn ok_task() -> CronTaskBox {
    Box::new(|_db: forge_db::DbConnection| {
      Box::pin(async move { Ok(()) }) as Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>
    })
  }

  fn err_task() -> CronTaskBox {
    Box::new(|_db: forge_db::DbConnection| {
      Box::pin(async move {
        Err("task failed".into())
      })
    })
  }

  async fn test_db() -> forge_db::DbConnection {
    let conn = Database::connect(ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap();
    TracedConnection::wrap(conn)
  }

  #[test]
  fn normalize_job_pool_url_normalizes_sqlite_memory() {
    assert_eq!(normalize_job_pool_url("sqlite::memory:"), "sqlite://:memory:");
  }

  #[test]
  fn normalize_job_pool_url_passthrough_for_other_urls() {
    assert_eq!(normalize_job_pool_url("sqlite:///path/to/db.sqlite"), "sqlite:///path/to/db.sqlite");
    assert_eq!(normalize_job_pool_url("sqlite://:memory:"), "sqlite://:memory:");
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
  async fn run_scheduler_and_worker_starts_with_empty_tasks() {
    let db = test_db().await;
    let res = run_scheduler_and_worker("sqlite::memory:", db, vec![]).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn run_scheduler_and_worker_starts_with_one_task() {
    let db = test_db().await;
    let tasks = vec![(
      "test-cron".to_string(),
      forge_cron::CronSchedule::Interval(std::time::Duration::from_millis(10)),
      ok_task(),
    )];
    let res = run_scheduler_and_worker("sqlite::memory:", db, tasks).await;
    assert!(res.is_ok());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
  }

  #[tokio::test]
  async fn run_scheduler_and_worker_with_hourly_schedule() {
    let db = test_db().await;
    let tasks = vec![(
      "hourly".to_string(),
      forge_cron::CronSchedule::Hourly { minute: 0 },
      ok_task(),
    )];
    let res = run_scheduler_and_worker("sqlite::memory:", db, tasks).await;
    assert!(res.is_ok());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
  }

  #[tokio::test]
  async fn run_scheduler_and_worker_with_daily_schedule() {
    let db = test_db().await;
    let tasks = vec![(
      "daily".to_string(),
      forge_cron::CronSchedule::Daily {
        hour: 12,
        minute: 0,
      },
      ok_task(),
    )];
    let res = run_scheduler_and_worker("sqlite::memory:", db, tasks).await;
    assert!(res.is_ok());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
  }

  #[tokio::test]
  async fn run_scheduler_and_worker_fails_on_invalid_db_url() {
    let db = test_db().await;
    let res = run_scheduler_and_worker("invalid://bad", db, vec![]).await;
    assert!(res.is_err());
  }

  #[tokio::test]
  #[cfg(unix)]
  async fn run_scheduler_and_worker_fails_when_setup_cannot_write() {
    use std::io::Write;
    let db = test_db().await;
    let dir = std::env::temp_dir().join("forge_jobs_readonly_test");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("readonly.sqlite");
    std::fs::File::create(&path).unwrap().write_all(b"").unwrap();
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&path, perms).unwrap();
    let url = format!("sqlite://{}", path.display());
    let res = run_scheduler_and_worker(&url, db, vec![]).await;
    assert!(res.is_err());
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
  }
}
