//! E2E tests for Forge health endpoints using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first. 100% fantoccini: navigate to /healthz, /livez, /readyz and assert body contains ok.

use forge_e2e_lib::cli;

/// Single E2E test: prebuilt layout; then browser navigates to healthz, livez, readyz and asserts body contains "ok".
#[tokio::test]
async fn e2e_prebuilt_health_layout_and_endpoints() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  if std::env::var("E2E_WEBDRIVER_URL").is_err() {
    return;
  }
  let b = base.trim_end_matches('/');
  forge_e2e_lib::browser::assert_health_ok(&format!("{}/healthz", b))
    .await
    .expect("browser must load healthz and show ok");
  forge_e2e_lib::browser::assert_health_ok(&format!("{}/livez", b))
    .await
    .expect("browser must load livez and show ok");
  forge_e2e_lib::browser::assert_health_ok(&format!("{}/readyz", b))
    .await
    .expect("browser must load readyz and show ok");
}
