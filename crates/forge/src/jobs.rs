//! Background jobs via Apalis + SQLite. Cron is implemented by enqueuing [ScheduledTaskJob] on a schedule.

use apalis::prelude::*;
use apalis_sqlite::SqliteStorage;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

use crate::cron::{CronSchedule, CronTaskBox, next_daily_run, next_hourly_run, next_interval_run};

/// Single job type for all scheduled tasks: worker dispatches by [ScheduledTaskJob::task_name].
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ScheduledTaskJob {
  pub task_name: String,
}

/// Normalize DB URL for sqlx (apalis-sqlite uses sqlx). SeaORM uses "sqlite::memory:", sqlx uses "sqlite://:memory:".
fn normalize_job_pool_url(url: &str) -> &str {
  if url == "sqlite::memory:" {
    "sqlite://:memory:"
  } else {
    url
  }
}

/// Runs the scheduler loops (enqueue [ScheduledTaskJob] on schedule) and the worker (run by name from registry).
/// Uses the same SQLite URL as the app DB for the job queue; [SqliteStorage::setup] creates job tables.
pub async fn run_scheduler_and_worker(
  db_url: &str,
  db: DatabaseConnection,
  tasks: Vec<(String, CronSchedule, CronTaskBox)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  use apalis_sqlite::SqlitePool;
  use tokio::time::{Instant, sleep_until};

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

/// Runs a single scheduled task job by name from the registry.
async fn run_scheduled_task(
  job: ScheduledTaskJob,
  data: Data<(Arc<HashMap<String, CronTaskBox>>, DatabaseConnection)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let (registry, db): &(Arc<HashMap<String, CronTaskBox>>, DatabaseConnection) = &data;
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

  #[test]
  fn normalize_job_pool_url_normalizes_sqlite_memory() {
    assert_eq!(
      normalize_job_pool_url("sqlite::memory:"),
      "sqlite://:memory:"
    );
    assert_eq!(
      normalize_job_pool_url("sqlite:///path/to/db.sqlite"),
      "sqlite:///path/to/db.sqlite"
    );
  }
}
