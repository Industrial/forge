//! E2E test for Forge health observability (017): distributed tracing, W3C trace context propagation,
//! and trace-id endpoint. Run via `bin/test-e2e`.

use std::time::Duration;

use forge_e2e_lib::cli;

/// Single E2E test: prebuilt has observability (tracing) enabled; GET /api/observability/trace-id
/// returns current trace id; sending Traceparent header propagates that trace id into the response.
#[tokio::test]
async fn e2e_prebuilt_observability_trace_propagation_and_trace_id() {
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

  let trace_id_url = format!("{}/api/observability/trace-id", base);

  // Without Traceparent: server generates a new trace; response must be 200 and body a non-empty trace id (32 hex chars or similar)
  let r0 = client
    .get(&trace_id_url)
    .send()
    .await
    .expect("GET /api/observability/trace-id without header");
  assert!(
    r0.status().is_success(),
    "GET /api/observability/trace-id must succeed without header; got {}",
    r0.status()
  );
  let body0 = r0.text().await.expect("body");
  assert!(
    !body0.trim().is_empty(),
    "trace-id response must not be empty; got {:?}",
    body0
  );
  let trace_id_0 = body0.trim();
  assert!(
    trace_id_0.len() >= 16 && trace_id_0.chars().all(|c| c.is_ascii_hexdigit()),
    "trace-id should be hex string (e.g. 32 chars); got {:?}",
    trace_id_0
  );

  // With Traceparent: server uses our trace id; response body must contain that trace id
  let expected_trace_id = "0af7651916cd43dd8448eb211c80319c";
  let traceparent = format!("00-{}-00f067aa0ba902b7-01", expected_trace_id);
  let r1 = client
    .get(&trace_id_url)
    .header("traceparent", traceparent)
    .send()
    .await
    .expect("GET /api/observability/trace-id with traceparent");
  assert!(
    r1.status().is_success(),
    "GET /api/observability/trace-id with traceparent must succeed; got {}",
    r1.status()
  );
  let body1 = r1.text().await.expect("body");
  assert!(
    body1.trim().contains(expected_trace_id),
    "response must contain propagated trace id {}; got {:?}",
    expected_trace_id,
    body1
  );

  // Response may include traceparent/tracestate (W3C propagation); at least we verified same trace id in body

  if std::env::var("E2E_WEBDRIVER_URL").is_ok() {
    forge_e2e_lib::browser::assert_app_root_loads(&base)
      .await
      .expect("browser must load app root");
  }
}
