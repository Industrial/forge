//! E2E tests for Forge config using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::fs;
use std::time::Duration;

use forge_e2e_lib::cli;

#[test]
fn prebuilt_project_has_config_files_and_content() {
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
  assert!(
    app_content.contains("[app]"),
    "config/app.toml should have [app]"
  );
  assert!(
    app_content.contains("[server]"),
    "config/app.toml should have [server]"
  );
  let db_content = fs::read_to_string(&db_toml).unwrap();
  assert!(
    db_content.contains("[database]"),
    "config/db.toml should have [database]"
  );
}

#[tokio::test]
async fn prebuilt_server_serves_and_uses_config() {
  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let resp = client
    .get(format!("{}/healthz", base))
    .send()
    .await
    .expect("request");
  assert_eq!(resp.status().as_u16(), 200);
  assert_eq!(resp.text().await.unwrap_or_default().trim(), "ok");
}
