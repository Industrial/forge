//! E2E tests for Forge health endpoints using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::process::Command;
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
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );

  let port = cli::next_e2e_port();
  std::fs::write(
    project_root.join("config/app.toml"),
    format!(
      r#"[app]
name = "e2e_prebuilt"
environment = "development"

[server]
host = "127.0.0.1"
port = {}
"#,
      port
    ),
  )
  .unwrap();

  let mut child = Command::new("cargo")
    .args(["run", "--quiet"])
    .current_dir(&project_root)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .expect("spawn cargo run");

  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let base = format!("http://127.0.0.1:{}", port);

  for i in 0..450 {
    tokio::time::sleep(Duration::from_millis(200)).await;
    if client.get(format!("{}/healthz", base)).send().await.is_ok() {
      assert_health_endpoint(&client, &format!("{}/healthz", base), 200, "ok").await;
      assert_health_endpoint(&client, &format!("{}/livez", base), 200, "ok").await;
      assert_health_endpoint(&client, &format!("{}/readyz", base), 200, "ok").await;
      let _ = child.kill();
      let _ = child.wait();
      return;
    }
    if i == 449 {
      let _ = child.kill();
      let _ = child.wait();
      panic!("server did not become ready in time");
    }
  }
  let _ = child.wait();
}
