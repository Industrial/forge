//! E2E tests for Forge CLI using the prebuilt project from `bin/test-e2e`.
//!
//! **Pattern**: Use the single prebuilt project at `.tmp/e2e_prebuilt` (created and built by
//! `bin/test-e2e`). Asserts project layout then (when E2E_BASE_URL and chromedriver are
//! available) verifies the app root loads in a real browser via fantoccini.
//!
//! Requires chromedriver running (e.g. `chromedriver --port=9515`). Set `E2E_WEBDRIVER_URL`
//! to override the default `http://localhost:9515`.

use forge_e2e_lib::cli;

#[tokio::test]
async fn e2e_prebuilt_project_layout() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  if std::env::var("E2E_WEBDRIVER_URL").is_ok() {
    forge_e2e_lib::browser::assert_app_root_loads(&base)
      .await
      .expect("browser must load app root (run chromedriver --port=9515 or set E2E_WEBDRIVER_URL)");
  }
}
