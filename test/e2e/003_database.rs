//! E2E tests for Forge database using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first. db.toml content asserts stay in Rust; browser verifies app load.

use std::fs;

use forge_e2e_lib::cli;

#[tokio::test]
async fn prebuilt_project_has_db_config() {
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
  assert!(
    content.contains("sqlite"),
    "db.toml should contain a sqlite URL (default or e2e TMPDIR path)"
  );

  if let Some(base) = cli::e2e_base_url()
    && std::env::var("E2E_WEBDRIVER_URL").is_ok()
  {
    forge_e2e_lib::browser::assert_app_root_loads(&base)
      .await
      .expect("browser must load app root");
  }
}
