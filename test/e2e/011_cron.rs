//! E2E tests for Forge cron jobs (011).
//!
//! Cron is implemented as a use case of jobs: scheduler enqueues [forge::jobs::ScheduledTaskJob],
//! worker runs them by name. Covers: cron with Interval schedule runs repeatedly.

use forge::cron::CronSchedule;
use std::fs;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

/// E2E: cron task with Interval schedule is enqueued by scheduler and run by worker (cron as jobs).
#[tokio::test]
async fn cron_interval_runs_repeatedly() {
  let temp_dir = tempfile::tempdir().unwrap();
  let original_cwd = std::env::current_dir().unwrap();
  std::env::set_current_dir(temp_dir.path()).unwrap();

  let config_dir = temp_dir.path().join("config");
  fs::create_dir_all(&config_dir).unwrap();
  fs::write(
    config_dir.join("app.toml"),
    r#"[app]
name = "cron_e2e"
environment = "test"

[server]
host = "127.0.0.1"
port = 0
"#,
  )
  .unwrap();
  fs::write(
    config_dir.join("db.toml"),
    r#"[database]
url = "sqlite::memory:"
"#,
  )
  .unwrap();

  let tick_count = std::sync::Arc::new(AtomicU32::new(0));
  let tick_count_clone = tick_count.clone();

  let app = forge::App::new().with_cron(
    "tick",
    CronSchedule::Interval(Duration::from_millis(200)),
    move |_db: forge::sea_orm::DatabaseConnection| {
      let c = tick_count_clone.clone();
      async move {
        c.fetch_add(1, Ordering::Relaxed);
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
      }
    },
  );

  // serve() blocks; scheduler enqueues, worker runs (same process). Allow time for poll + runs.
  let serve_handle = tokio::spawn(async move { app.serve().await });

  tokio::time::sleep(Duration::from_millis(1500)).await;

  let count = tick_count.load(Ordering::Relaxed);
  assert!(
    count >= 2,
    "cron (as job) should have run at least 2 times in 1500ms (interval 200ms), got {}",
    count
  );

  serve_handle.abort();
  let _ = serve_handle.await;

  let _ = std::env::set_current_dir(&original_cwd);
}
