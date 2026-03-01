//! E2E tests for Forge migrations using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::fs;
use std::time::Duration;

use forge_e2e_lib::cli;

#[test]
fn prebuilt_project_has_migrations_layout() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  assert!(project_root.join("crates/db/src/migrations/mod.rs").exists());
  assert!(project_root.join("crates/db/src/models/mod.rs").exists());
  assert!(project_root.join("crates/db/src/seeds/mod.rs").exists());

  let cargo_toml = fs::read_to_string(project_root.join("Cargo.toml")).unwrap();
  assert!(
    cargo_toml.contains("[workspace]"),
    "root Cargo.toml should be a workspace"
  );
}

#[tokio::test]
async fn prebuilt_server_readyz_after_migrations() {
  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let resp = client.get(format!("{}/readyz", base)).send().await.expect("request");
  assert_eq!(resp.status().as_u16(), 200);
  assert_eq!(resp.text().await.unwrap_or_default().trim(), "ok");
}
