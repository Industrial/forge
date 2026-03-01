//! E2E test for Forge caching (016): config/cache.toml, application cache, HTTP response cache.
//! Run via `bin/test-e2e`. One test: layout, cache config, app cache, response cache (same body + Cache-Control).

use std::time::Duration;

use forge_e2e_lib::cli;

/// Single E2E test: prebuilt has config/cache.toml; app cache and HTTP response cache both work.
#[tokio::test]
async fn e2e_prebuilt_caching_config_and_app_cache() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  let cache_config = project_root.join("config/cache.toml");
  assert!(
    cache_config.exists(),
    "config/cache.toml missing at {}",
    cache_config.display()
  );
  let content = std::fs::read_to_string(&cache_config).expect("read cache.toml");
  assert!(
    content.contains("enabled") && content.contains("[application]") && content.contains("[http_response]"),
    "config/cache.toml should define enabled and [application] and [http_response]; got {:?}",
    content
  );

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();

  // Application cache: GET /api/cache-demo twice returns identical body
  let url = format!("{}/api/cache-demo", base);
  let r1 = client.get(&url).send().await.expect("GET /api/cache-demo first");
  assert!(r1.status().is_success(), "first GET must succeed");
  let body1 = r1.text().await.expect("body");
  assert!(!body1.is_empty(), "first response body must not be empty");

  let r2 = client.get(&url).send().await.expect("GET /api/cache-demo second");
  assert!(r2.status().is_success(), "second GET must succeed");
  let body2 = r2.text().await.expect("body");
  assert_eq!(
    body1, body2,
    "second request must return same body as first (served from app cache); got {:?} vs {:?}",
    body1, body2
  );

  // HTTP response cache: GET /api/cached-page twice returns identical body (full response cached)
  let cached_url = format!("{}/api/cached-page", base);
  let resp1 = client.get(&cached_url).send().await.expect("GET /api/cached-page first");
  assert!(resp1.status().is_success(), "first GET /api/cached-page must succeed");
  let page1 = resp1.text().await.expect("body");
  assert!(
    page1.starts_with("cached-page-"),
    "response should look like cached-page-{{uuid}}; got {:?}",
    page1
  );

  let resp2 = client.get(&cached_url).send().await.expect("GET /api/cached-page second");
  assert!(resp2.status().is_success(), "second GET /api/cached-page must succeed");
  let cache_control = resp2
    .headers()
    .get("cache-control")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("")
    .to_string();
  let page2 = resp2.text().await.expect("body");
  assert_eq!(
    page1, page2,
    "second request must return same body (served from HTTP response cache); got {:?} vs {:?}",
    page1, page2
  );
  assert!(
    cache_control.to_lowercase().contains("max-age") || cache_control.to_lowercase().contains("public"),
    "cached response should have Cache-Control (max-age or public); got {:?}",
    cache_control
  );
}
