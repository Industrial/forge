//! Cron-style scheduled tasks (interval, hourly, daily) without a cron library.
//!
//! Uses tokio + chrono for next-run calculation; tasks run in-process alongside the server.

use chrono::Utc;
use sea_orm::DatabaseConnection;
use std::future::Future;
use std::pin::Pin;
use tokio::time::Instant;
use tracing::error;

/// Schedule for a cron task. No external cron crate; minimal DSL.
#[derive(Clone, Debug)]
pub enum CronSchedule {
  /// Every fixed duration (e.g. every 5 minutes).
  Interval(std::time::Duration),
  /// Every hour at the given minute (0..60).
  Hourly { minute: u32 },
  /// Once per day at the given hour and minute (UTC).
  Daily { hour: u32, minute: u32 },
}

/// Type-erased cron task: takes DB, returns a future.
pub type CronTaskBox = Box<
  dyn Fn(
      DatabaseConnection,
    )
      -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>
    + Send
    + Sync,
>;

/// Computes the next run instant for an interval schedule (first run = now, then last_run + duration).
pub(crate) fn next_interval_run(
  duration: std::time::Duration,
  last_run: Option<Instant>,
) -> (Instant, Option<Instant>) {
  let now = Instant::now();
  let next = last_run.map(|t| t + duration).unwrap_or(now);
  let next = if next <= now { now } else { next };
  (next, Some(next))
}

/// Computes the next run instant for Hourly { minute } (UTC): next occurrence of :minute.
pub(crate) fn next_hourly_run(minute: u32) -> Instant {
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

/// Computes the next run instant for Daily { hour, minute } (UTC): next occurrence of HH:MM.
pub(crate) fn next_daily_run(hour: u32, minute: u32) -> Instant {
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

/// Runner that starts the job-based scheduler and worker (cron as a use case of jobs).
pub struct CronRunner {
  pub(crate) tasks: Vec<(String, CronSchedule, CronTaskBox)>,
  pub(crate) job_pool_url: String,
}

impl CronRunner {
  /// Spawns the scheduler (enqueues [crate::jobs::ScheduledTaskJob] on schedule) and worker (runs jobs by name).
  pub fn spawn(self, db: DatabaseConnection) {
    let url = self.job_pool_url.clone();
    let tasks = self.tasks;
    tokio::spawn(async move {
      if let Err(e) = crate::jobs::run_scheduler_and_worker(&url, db, tasks).await {
        error!(error = %e, "scheduler/worker failed");
      }
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::time::Duration;

  #[test]
  fn next_interval_run_first_run_is_now() {
    let (next, _) = next_interval_run(Duration::from_secs(60), None);
    let now = Instant::now();
    assert!(
      next <= now + Duration::from_millis(100),
      "first run should be ~now"
    );
  }

  #[test]
  fn next_hourly_run_returns_future_instant() {
    let next = next_hourly_run(0);
    let now = Instant::now();
    assert!(next > now, "next hourly should be in the future");
  }

  #[test]
  fn next_daily_run_returns_future_instant() {
    let next = next_daily_run(3, 0);
    let now = Instant::now();
    assert!(next > now, "next daily should be in the future");
  }
}
