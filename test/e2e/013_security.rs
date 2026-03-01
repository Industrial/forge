//! E2E test for Forge security additions (013): OWASP-aligned headers on all responses.
//! Run via `bin/test-e2e`. Asserts X-Content-Type-Options, X-Frame-Options, Referrer-Policy,
//! Content-Security-Policy (frame-ancestors), Permissions-Policy, Cross-Origin-Resource-Policy.

use std::time::Duration;

use forge_e2e_lib::cli;

fn assert_header(resp: &reqwest::Response, name: &str, expected_substr: &str) {
  let value = resp
    .headers()
    .get(name)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");
  assert!(
    value
      .to_lowercase()
      .contains(&expected_substr.to_lowercase()),
    "header {} should contain {:?}, got {:?}",
    name,
    expected_substr,
    value
  );
}

/// Single E2E test: prebuilt layout and security headers on responses.
/// GET /healthz and optionally GET / and assert OWASP-recommended headers are set.
#[tokio::test]
async fn e2e_prebuilt_security_headers() {
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

  let resp = client
    .get(format!("{}/healthz", base))
    .send()
    .await
    .expect("GET /healthz");
  assert!(resp.status().is_success(), "healthz must succeed");

  assert_header(&resp, "x-content-type-options", "nosniff");
  assert_header(&resp, "x-frame-options", "DENY");
  assert_header(&resp, "referrer-policy", "strict-origin-when-cross-origin");
  assert_header(&resp, "content-security-policy", "frame-ancestors");
  assert_header(&resp, "permissions-policy", "geolocation=()");
  assert_header(&resp, "cross-origin-resource-policy", "same-site");

  if std::env::var("E2E_WEBDRIVER_URL").is_ok() {
    forge_e2e_lib::browser::assert_app_root_loads(&base)
      .await
      .expect("browser must load app root");
  }
}
