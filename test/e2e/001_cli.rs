//! E2E tests for Forge CLI using the prebuilt project from `bin/test-e2e`.
//!
//! **Pattern**: Use the single prebuilt project at `.tmp/e2e_prebuilt` (created and built by
//! `bin/test-e2e`). Start the server and run tests against it. No `forge new` or `cargo build`
//! in the test; run `bin/test-e2e` first.

use std::time::Duration;

use forge_e2e_lib::cli;

// --- Tests (run against prebuilt server) ---

/// Assert the prebuilt project exists and has the expected layout.
#[test]
fn prebuilt_project_has_correct_layout() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);
}

/// GET /healthz → 200 "ok" (uses shared server when run via bin/test-e2e).
#[tokio::test]
async fn prebuilt_server_healthz_ok() {
  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let url = format!("{}/healthz", base);
  let resp = client.get(&url).send().await.expect("request");
  assert_eq!(resp.status().as_u16(), 200, "GET /healthz");
  let body = resp.text().await.unwrap_or_default();
  assert_eq!(body.trim(), "ok", "GET /healthz body should be 'ok'");
}
