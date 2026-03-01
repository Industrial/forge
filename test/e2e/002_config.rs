//! E2E tests for Forge config using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first. Health endpoints are tested only in 008_health.

use std::fs;

use forge_e2e_lib::cli;

#[test]
fn e2e_prebuilt_config_files_and_content() {
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
