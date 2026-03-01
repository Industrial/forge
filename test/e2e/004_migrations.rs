//! E2E tests for Forge migrations using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first. Layout/migrations asserts stay in Rust; browser verifies app load.

use std::fs;

use forge_e2e_lib::cli;

#[tokio::test]
async fn prebuilt_project_has_migrations_layout() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  assert!(
    project_root
      .join("crates/db/src/migrations/mod.rs")
      .exists()
  );
  assert!(project_root.join("crates/db/src/models/mod.rs").exists());
  assert!(project_root.join("crates/db/src/seeds/mod.rs").exists());

  let cargo_toml = fs::read_to_string(project_root.join("Cargo.toml")).unwrap();
  assert!(
    cargo_toml.contains("[workspace]"),
    "root Cargo.toml should be a workspace"
  );

  if let Some(base) = cli::e2e_base_url()
    && std::env::var("E2E_WEBDRIVER_URL").is_ok()
  {
    forge_e2e_lib::browser::assert_app_root_loads(&base)
      .await
      .expect("browser must load app root");
  }
}
