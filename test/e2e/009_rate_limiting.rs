//! E2E tests for Forge rate limiting using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::process::Command;
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
  let base = format!("http://127.0.0.1:{}/healthz", port);

  for i in 0..450 {
    tokio::time::sleep(Duration::from_millis(200)).await;
    if client.get(&base).send().await.is_ok() {
      for _ in 0..5 {
        let r = client.get(&base).send().await.unwrap();
        assert_eq!(r.status().as_u16(), 200, "healthz must not be rate limited");
      }
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
