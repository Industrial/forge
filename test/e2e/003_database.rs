//! E2E tests for Forge database using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first. Health endpoints are tested only in 008_health.

use std::fs;

use forge_e2e_lib::cli;

#[test]
fn prebuilt_project_has_db_config() {
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
}
