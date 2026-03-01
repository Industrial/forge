//! E2E tests for Forge health endpoints using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::time::Duration;

use forge_e2e_lib::cli;

async fn assert_health_endpoint(
  client: &reqwest::Client,
  url: &str,
  expected_status: u16,
  expected_body: &str,
) {
  let resp = client.get(url).send().await.expect("request");
  assert_eq!(
    resp.status().as_u16(),
    expected_status,
    "{} should return {}",
    url,
    expected_status
  );
  let body = resp.text().await.expect("body");
  assert_eq!(
    body.trim(),
    expected_body,
    "{} body should be {:?}, got {:?}",
    url,
    expected_body,
    body
  );
  assert!(
    !body.contains("components") && !body.contains("database"),
    "response must not disclose components: {:?}",
    body
  );
}

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
async fn prebuilt_server_health_endpoints_200_minimal_body() {
  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  assert_health_endpoint(&client, &format!("{}/healthz", base), 200, "ok").await;
  assert_health_endpoint(&client, &format!("{}/livez", base), 200, "ok").await;
  assert_health_endpoint(&client, &format!("{}/readyz", base), 200, "ok").await;
}
