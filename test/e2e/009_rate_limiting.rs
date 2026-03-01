//! E2E tests for Forge rate limiting using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::time::Duration;

use forge_e2e_lib::cli;

/// Single E2E test: prebuilt layout and root not rate limited in dev.
/// 1. Asserts project layout.
/// 2. Repeated GET / (5x) → 200; in dev there is no per-IP rate limit.
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
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let url = format!("{}/", base);
  for _ in 0..5 {
    let r = client.get(&url).send().await.unwrap();
    assert_eq!(
      r.status().as_u16(),
      200,
      "GET / must not be rate limited in dev"
    );
  }
}
