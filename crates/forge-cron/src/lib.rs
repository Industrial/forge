//! Cron-style schedule types and next-run calculation for the Forge framework.

use chrono::Utc;
use forge_db::DbConnection;
use std::future::Future;
use std::pin::Pin;
use tokio::time::Instant;

/// Schedule for a cron task.
#[derive(Clone, Debug)]
pub enum CronSchedule {
  Interval(std::time::Duration),
  Hourly { minute: u32 },
  Daily { hour: u32, minute: u32 },
}

/// Type-erased cron task: takes DB, returns a future.
pub type CronTaskBox = Box<
  dyn Fn(DbConnection)
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
    let daily = CronSchedule::Daily {
      hour: 9,
      minute: 0,
    };
    let _ = format!("{:?}", interval);
    let _ = format!("{:?}", hourly);
    let _ = format!("{:?}", daily);
    let _ = interval.clone();
    let _ = hourly.clone();
    let _ = daily.clone();
  }
}
