//! E2E test for Forge background jobs (011): layout, config, cron registration; browser verifies #app and Pages/Home.
//! Run via `bin/test-e2e`. 100% fantoccini for server check: navigate to / and verify #app and Pages/Home content.

use std::fs;

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
  if std::env::var("E2E_WEBDRIVER_URL").is_ok() {
    forge_e2e_lib::browser::assert_app_root_loads_and_contains(&base, "Pages/Home")
      .await
      .expect("browser must load app root with Pages/Home");
  }
}
