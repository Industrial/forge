//! Cron-style scheduled tasks (re-export from [forge_cron](forge_cron) + CronRunner).

use tracing::error;

use crate::DbConnection;
pub use forge_cron::{
  CronSchedule, CronTaskBox, next_daily_run, next_hourly_run, next_interval_run,
};

/// Runner that starts the job-based scheduler and worker.
pub struct CronRunner {
  pub(crate) tasks: Vec<(String, CronSchedule, CronTaskBox)>,
  pub(crate) job_pool_url: String,
}

impl CronRunner {
  /// Spawns the scheduler and worker via [forge_jobs::run_scheduler_and_worker].
  pub fn spawn(self, db: DbConnection) {
    let url = self.job_pool_url.clone();
    let tasks = self.tasks;
    tokio::spawn(async move {
      if let Err(e) = forge_jobs::run_scheduler_and_worker(&url, db, tasks).await {
        error!(error = %e, "scheduler/worker failed");
      }
    });
  }
}
