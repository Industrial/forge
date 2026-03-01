//! E2E test for Forge background jobs (011): one test for layout, config, cron registration, and server-with-worker running.
//! Run via `bin/test-e2e`. Health endpoints are tested only in 008_health.

use std::fs;
use std::time::Duration;

use forge_e2e_lib::cli;

#[tokio::test]
async fn jobs_layout_config_cron_and_server_with_worker() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  let app_toml = project_root.join("config/app.toml");
  let db_toml = project_root.join("config/db.toml");
  assert!(app_toml.exists(), "config/app.toml missing");
  assert!(db_toml.exists(), "config/db.toml missing");
  let app_content = fs::read_to_string(&app_toml).unwrap();
  let db_content = fs::read_to_string(&db_toml).unwrap();
  assert!(
    app_content.contains("[app]"),
    "config/app.toml should have [app]"
  );
  assert!(
    app_content.contains("[server]"),
    "config/app.toml should have [server]"
  );
  assert!(
    db_content.contains("[database]"),
    "config/db.toml should have [database]"
  );
  assert!(
    db_content.contains("sqlite"),
    "jobs use app DB; config/db.toml should have sqlite url"
  );

  let main_rs = fs::read_to_string(project_root.join("crates/app/src/main.rs")).unwrap();
  assert!(
    main_rs.contains("with_cron"),
    "main.rs should register a cron task (with_cron)"
  );
  assert!(
    main_rs.contains("CronSchedule"),
    "main.rs should use CronSchedule for schedule type"
  );
  assert!(
    main_rs.contains("Interval"),
    "main.rs should use Interval (or Hourly/Daily) for schedule"
  );

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let resp = client
    .get(format!("{}/", base))
    .send()
    .await
    .expect("request");
  assert_eq!(
    resp.status().as_u16(),
    200,
    "server with cron worker must be up"
  );
  let body = resp.text().await.unwrap_or_default();
  assert!(
    body.contains("Hello"),
    "root route should return Hello from Forge; got: {}",
    body
  );
}
