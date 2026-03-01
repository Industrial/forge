//! E2E tests for Forge database using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::fs;
use std::time::Duration;

use forge_e2e_lib::cli;

/// Single E2E test: prebuilt db config and server health.
/// 1. Asserts project layout and config/db.toml with [database] and sqlite URL.
/// 2. GET /healthz → 200 "ok".
#[tokio::test]
async fn e2e_prebuilt_database_and_healthz() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  let db_config = project_root.join("config/db.toml");
  assert!(db_config.exists());
  let content = fs::read_to_string(&db_config).unwrap();
  assert!(content.contains("[database]"));
  assert!(content.contains("sqlite://db.sqlite"));

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
