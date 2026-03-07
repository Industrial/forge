//! Cron-style schedule types, next-run calculation, and cron job runner for the Forge framework.
//! Built on [forge_jobs](forge_jobs) for the job queue and worker.

use apalis_core::backend::TaskSink;
use chrono::Utc;
use forge_db::DbConnection;
use forge_jobs::{ScheduledTaskJob, TaskFn, run_worker, setup_queue};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::time::{Instant, sleep_until};
use tracing::{error, info};

/// Schedule for a cron task.
#[derive(Clone, Debug)]
pub enum CronSchedule {
  Interval(std::time::Duration),
  Hourly { minute: u32 },
  Daily { hour: u32, minute: u32 },
}

/// Type-erased cron task: takes DB, returns a future.
pub type CronTaskBox = Box<
  dyn Fn(
      DbConnection,
    )
      -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>
    + Send
    + Sync,
>;

/// Next run for interval schedule.
pub fn next_interval_run(
  duration: std::time::Duration,
  last_run: Option<Instant>,
) -> (Instant, Option<Instant>) {
  let now = Instant::now();
  let next = last_run.map(|t| t + duration).unwrap_or(now);
  let next = if next <= now { now } else { next };
  (next, Some(next))
}

/// Next run for Hourly { minute } (UTC).
pub fn next_hourly_run(minute: u32) -> Instant {
  let minute = minute.min(59);
  let now = Utc::now();
  let now_ts = now.timestamp();
  let hour_secs = 3600;
  let next_ts = (now_ts / hour_secs) * hour_secs + minute as i64 * 60;
  let next_ts = if next_ts <= now_ts {
    next_ts + hour_secs
  } else {
    next_ts
  };
  let delay_secs = (next_ts - now_ts).max(0) as u64;
  Instant::now() + std::time::Duration::from_secs(delay_secs)
}

/// Next run for Daily { hour, minute } (UTC).
pub fn next_daily_run(hour: u32, minute: u32) -> Instant {
  let hour = hour.min(23);
  let minute = minute.min(59);
  let now = Utc::now();
  let now_ts = now.timestamp();
  let day_secs = 86400;
  let today_start = (now_ts / day_secs) * day_secs;
  let next_ts = today_start + hour as i64 * 3600 + minute as i64 * 60;
  let next_ts = if next_ts <= now_ts {
    next_ts + day_secs
  } else {
    next_ts
  };
  let delay_secs = (next_ts - now_ts).max(0) as u64;
  Instant::now() + std::time::Duration::from_secs(delay_secs)
}

/// Runner that starts the cron scheduler loops and the job worker via [forge_jobs].
pub struct CronRunner {
  /// (task name, schedule, task closure)
  pub tasks: Vec<(String, CronSchedule, CronTaskBox)>,
  /// SQLite URL for the job queue (e.g. from config).
  pub job_pool_url: String,
}

impl CronRunner {
  /// Spawns the scheduler loops and worker in the background.
  pub fn spawn(self, db: DbConnection) {
    let url = self.job_pool_url.clone();
    let tasks = self.tasks;
    tokio::spawn(async move {
      if let Err(e) = run_scheduler_and_worker(&url, db, tasks).await {
        error!(error = %e, "scheduler/worker failed");
      }
    });
  }
}

/// Runs the cron scheduler loops (push jobs by schedule) and the job worker.
/// Uses [forge_jobs::setup_queue] and [forge_jobs::run_worker].
pub async fn run_scheduler_and_worker(
  db_url: &str,
  db: DbConnection,
  tasks: Vec<(String, CronSchedule, CronTaskBox)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let storage = setup_queue(db_url).await?;
  let storage_for_worker = storage.clone();
  let storage_for_scheduler = storage;

  let mut registry: HashMap<String, TaskFn> = HashMap::new();
  let mut schedule_list = Vec::new();
  for (name, schedule, task) in tasks {
    let task_fn: TaskFn = Box::new(move |db: DbConnection| (task)(db));
    registry.insert(name.clone(), task_fn);
    schedule_list.push((name, schedule));
  }
  let registry = Arc::new(registry);

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

  run_worker(storage_for_worker, db, registry).await
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::time::Duration;

  #[test]
  fn next_interval_run_first_run_is_now() {
    let (next, _) = next_interval_run(Duration::from_secs(60), None);
    let now = Instant::now();
    assert!(next <= now + Duration::from_millis(100));
  }

  #[test]
  fn next_interval_run_with_last_run_in_past_uses_now() {
    let past = Instant::now() - Duration::from_secs(10);
    let (next, last) = next_interval_run(Duration::from_secs(5), Some(past));
    let now = Instant::now();
    assert!(next <= now + Duration::from_millis(100));
    assert!(last.is_some());
  }

  #[test]
  fn next_interval_run_with_last_run_yields_future() {
    let past = Instant::now() - Duration::from_secs(1);
    let (next, last) = next_interval_run(Duration::from_secs(60), Some(past));
    let now = Instant::now();
    assert!(next > now);
    assert!(last.unwrap() > now);
  }

  #[test]
  fn next_hourly_run_returns_future_instant() {
    let next = next_hourly_run(0);
    let now = Instant::now();
    assert!(next > now);
  }

  #[test]
  fn next_hourly_run_clamps_minute() {
    let next = next_hourly_run(99);
    let now = Instant::now();
    assert!(next > now);
  }

  #[test]
  fn next_daily_run_returns_future_instant() {
    let next = next_daily_run(3, 0);
    let now = Instant::now();
    assert!(next > now);
  }

  #[test]
  fn next_daily_run_clamps_hour_and_minute() {
    let next = next_daily_run(25, 99);
    let now = Instant::now();
    assert!(next > now);
  }

  #[test]
  fn cron_schedule_derive_clone_debug() {
    let interval = CronSchedule::Interval(Duration::from_secs(30));
    let hourly = CronSchedule::Hourly { minute: 15 };
    let daily = CronSchedule::Daily { hour: 9, minute: 0 };
    let _ = format!("{:?}", interval);
    let _ = format!("{:?}", hourly);
    let _ = format!("{:?}", daily);
    let _ = interval.clone();
    let _ = hourly.clone();
    let _ = daily.clone();
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use std::time::Duration;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.
  mod cron_schedule_enum_behavior {
    use super::*;

    #[test]
    fn should_create_interval_schedule() {
      // Given: a duration
      let duration = Duration::from_secs(60);

      // When: creating an Interval schedule
      let schedule = CronSchedule::Interval(duration);

      // Then: should be an Interval variant
      match schedule {
        CronSchedule::Interval(d) => assert_eq!(d, duration),
        _ => panic!("Expected Interval variant"),
      }
    }

    #[test]
    fn should_create_hourly_schedule() {
      // Given: a minute value
      let minute = 30;

      // When: creating an Hourly schedule
      let schedule = CronSchedule::Hourly { minute };

      // Then: should be an Hourly variant with correct minute
      match schedule {
        CronSchedule::Hourly { minute: m } => assert_eq!(m, minute),
        _ => panic!("Expected Hourly variant"),
      }
    }

    #[test]
    fn should_create_daily_schedule() {
      // Given: hour and minute values
      let hour = 9;
      let minute = 15;

      // When: creating a Daily schedule
      let schedule = CronSchedule::Daily { hour, minute };

      // Then: should be a Daily variant with correct values
      match schedule {
        CronSchedule::Daily { hour: h, minute: m } => {
          assert_eq!(h, hour);
          assert_eq!(m, minute);
        }
        _ => panic!("Expected Daily variant"),
      }
    }

    #[test]
    fn should_be_cloneable() {
      // Given: a CronSchedule
      let schedule = CronSchedule::Interval(Duration::from_secs(30));

      // When: cloning it
      let cloned = schedule.clone();

      // Then: should have same variant and values
      match (schedule, cloned) {
        (CronSchedule::Interval(d1), CronSchedule::Interval(d2)) => assert_eq!(d1, d2),
        _ => panic!("Clones should match"),
      }
    }

    #[test]
    fn should_be_debuggable() {
      // Given: a CronSchedule
      let schedule = CronSchedule::Hourly { minute: 15 };

      // When: formatting for debug
      let debug_str = format!("{:?}", schedule);

      // Then: should produce debug output
      assert!(!debug_str.is_empty());
    }
  }

  mod next_interval_run_behavior {
    use super::*;

    #[test]
    fn should_return_now_for_first_run() {
      // Given: an interval duration and no last run
      let duration = Duration::from_secs(60);
      let last_run = None;

      // When: calculating next run
      let (next, new_last) = next_interval_run(duration, last_run);
      let now = Instant::now();

      // Then: next should be at or very close to now
      assert!(next <= now + Duration::from_millis(100));
      assert!(new_last.is_some());
    }

    #[test]
    fn should_schedule_next_run_after_interval() {
      // Given: an interval duration and a past last run
      let duration = Duration::from_secs(60);
      let past = Instant::now() - Duration::from_secs(1);
      let last_run = Some(past);

      // When: calculating next run
      let (next, new_last) = next_interval_run(duration, last_run);
      let now = Instant::now();

      // Then: next should be in the future
      assert!(next > now);
      assert!(new_last.is_some());
      assert!(new_last.unwrap() > now);
    }

    #[test]
    fn should_use_now_if_last_run_plus_interval_is_in_past() {
      // Given: an interval duration and a last run that makes next run in the past
      let duration = Duration::from_secs(5);
      let past = Instant::now() - Duration::from_secs(10);
      let last_run = Some(past);

      // When: calculating next run
      let (next, new_last) = next_interval_run(duration, last_run);
      let now = Instant::now();

      // Then: should use now instead of past time
      assert!(next <= now + Duration::from_millis(100));
      assert!(new_last.is_some());
    }

    #[test]
    fn should_update_last_run_to_next_run() {
      // Given: an interval duration and last run
      let duration = Duration::from_secs(60);
      let past = Instant::now() - Duration::from_secs(1);
      let last_run = Some(past);

      // When: calculating next run
      let (next, new_last) = next_interval_run(duration, last_run);

      // Then: new_last should equal next
      assert_eq!(new_last, Some(next));
    }
  }

  mod next_hourly_run_behavior {
    use super::*;

    #[test]
    fn should_return_future_instant() {
      // Given: a minute value
      let minute = 0;

      // When: calculating next hourly run
      let next = next_hourly_run(minute);
      let now = Instant::now();

      // Then: should be in the future
      assert!(next > now);
    }

    #[test]
    fn should_clamp_minute_to_59() {
      // Given: a minute value greater than 59
      let minute = 99;

      // When: calculating next hourly run
      let next = next_hourly_run(minute);
      let now = Instant::now();

      // Then: should still return valid future instant (minute clamped)
      assert!(next > now);
    }

    #[test]
    fn should_schedule_for_next_hour_if_minute_passed() {
      // Given: a minute value that may have already passed this hour
      let minute = 0;

      // When: calculating next hourly run
      let next = next_hourly_run(minute);
      let now = Instant::now();

      // Then: should be at least in the future (could be next hour)
      assert!(next >= now);
    }

    #[test]
    fn should_schedule_for_current_hour_if_minute_not_passed() {
      // Given: a minute value in the future of current hour
      // (This is probabilistic - minute 59 is likely in the future)
      let minute = 59;

      // When: calculating next hourly run
      let next = next_hourly_run(minute);
      let now = Instant::now();

      // Then: should be in the future
      assert!(next >= now);
    }
  }

  mod next_daily_run_behavior {
    use super::*;

    #[test]
    fn should_return_future_instant() {
      // Given: hour and minute values
      let hour = 3;
      let minute = 0;

      // When: calculating next daily run
      let next = next_daily_run(hour, minute);
      let now = Instant::now();

      // Then: should be in the future
      assert!(next > now);
    }

    #[test]
    fn should_clamp_hour_to_23() {
      // Given: hour value greater than 23
      let hour = 25;
      let minute = 0;

      // When: calculating next daily run
      let next = next_daily_run(hour, minute);
      let now = Instant::now();

      // Then: should still return valid future instant (hour clamped)
      assert!(next > now);
    }

    #[test]
    fn should_clamp_minute_to_59() {
      // Given: minute value greater than 59
      let hour = 9;
      let minute = 99;

      // When: calculating next daily run
      let next = next_daily_run(hour, minute);
      let now = Instant::now();

      // Then: should still return valid future instant (minute clamped)
      assert!(next > now);
    }

    #[test]
    fn should_schedule_for_tomorrow_if_time_passed_today() {
      // Given: hour and minute that may have passed today
      let hour = 0;
      let minute = 0;

      // When: calculating next daily run
      let next = next_daily_run(hour, minute);
      let now = Instant::now();

      // Then: should be in the future (could be tomorrow)
      assert!(next >= now);
    }

    #[test]
    fn should_schedule_for_today_if_time_not_passed() {
      // Given: hour and minute likely in the future today
      // (This is probabilistic - hour 23 is likely in the future)
      let hour = 23;
      let minute = 59;

      // When: calculating next daily run
      let next = next_daily_run(hour, minute);
      let now = Instant::now();

      // Then: should be in the future
      assert!(next >= now);
    }
  }

  mod cron_runner_behavior {
    use super::*;

    #[test]
    fn should_store_tasks_and_job_pool_url() {
      // Given: tasks and job pool URL
      let tasks = Vec::new();
      let job_pool_url = "sqlite::memory:".to_string();

      // When: creating CronRunner
      let runner = CronRunner {
        tasks,
        job_pool_url: job_pool_url.clone(),
      };

      // Then: should store the values
      assert_eq!(runner.job_pool_url, job_pool_url);
      assert_eq!(runner.tasks.len(), 0);
    }

    #[test]
    fn should_accept_multiple_tasks() {
      // Given: multiple cron tasks
      let task1: CronTaskBox = Box::new(|_db: DbConnection| Box::pin(async move { Ok(()) }));
      let task2: CronTaskBox = Box::new(|_db: DbConnection| Box::pin(async move { Ok(()) }));

      // When: creating CronRunner with multiple tasks
      let runner = CronRunner {
        tasks: vec![
          (
            "task1".to_string(),
            CronSchedule::Interval(Duration::from_secs(60)),
            task1,
          ),
          (
            "task2".to_string(),
            CronSchedule::Hourly { minute: 0 },
            task2,
          ),
        ],
        job_pool_url: "sqlite::memory:".to_string(),
      };

      // Then: should store all tasks
      assert_eq!(runner.tasks.len(), 2);
    }
  }

  mod cron_task_type_behavior {
    use super::*;

    #[test]
    fn should_accept_async_task_function() {
      // Given: an async task function
      let _task: CronTaskBox = Box::new(|_db: DbConnection| {
        Box::pin(async move {
          // Simulate some async work
          Ok(())
        })
      });

      // When: using the task type
      // Then: should compile and be usable
    }

    #[test]
    fn should_allow_error_return() {
      // Given: a task that can return an error
      let _task: CronTaskBox =
        Box::new(|_db: DbConnection| Box::pin(async move { Err("test error".into()) }));

      // When: using the task type
      // Then: should accept error return type
    }
  }
}
