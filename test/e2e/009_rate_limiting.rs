//! E2E tests for Forge rate limiting using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::time::Duration;

use forge_e2e_lib::cli;

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

#[tokio::test]
async fn prebuilt_server_health_endpoints_not_rate_limited() {
  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let url = format!("{}/healthz", base);
  for _ in 0..5 {
    let r = client.get(&url).send().await.unwrap();
    assert_eq!(r.status().as_u16(), 200, "healthz must not be rate limited");
  }
}
