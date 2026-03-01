//! E2E test for Forge WebSocket support (014): prebuilt app exposes /ws-demo; verify echo via browser.
//! Run via `bin/test-e2e`. 100% fantoccini: open /ws-demo and verify echo via page UI.

use forge_e2e_lib::cli;

/// Single E2E test: prebuilt layout; then browser opens /ws-demo, sends "ping", asserts message appears in page.
#[tokio::test]
async fn e2e_prebuilt_websocket_echo() {
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
  forge_e2e_lib::browser::assert_ws_demo_echo(&base, "ping")
    .await
    .expect("browser must load ws-demo and see echo");
}
