//! E2E tests for Forge rate limiting using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first. 100% fantoccini: navigate to / 5 times and verify page loads each time.

use forge_e2e_lib::cli;

/// Single E2E test: prebuilt layout; then browser navigates to / 5 times and asserts #app each time.
#[tokio::test]
async fn e2e_prebuilt_rate_limiting_layout_and_health_ok() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  if std::env::var("E2E_WEBDRIVER_URL").is_ok() {
    forge_e2e_lib::browser::assert_app_root_loads_repeated(&base, 5)
      .await
      .expect("browser must load app root 5 times (no rate limit in dev)");
  }
}
