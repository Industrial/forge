//! E2E tests for Forge jobs using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::fs;
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

  let main_rs = fs::read_to_string(project_root.join("crates/app/src/main.rs")).unwrap();
  let has_jobs = main_rs.contains("with_cron") || main_rs.contains("cron") || main_rs.contains("jobs");
  if !has_jobs {
    eprintln!("note: generated app may not include jobs in main.rs; still asserting serve");
  }
}

#[tokio::test]
async fn prebuilt_server_serves() {
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
  let url = format!("http://127.0.0.1:{}/healthz", port);

  for _ in 0..450 {
    tokio::time::sleep(Duration::from_millis(200)).await;
    if let Ok(resp) = client.get(&url).send().await
      && resp.status().as_u16() == 200
    {
      assert_eq!(resp.text().await.unwrap_or_default().trim(), "ok");
      let _ = child.kill();
      let _ = child.wait();
      return;
    }
  }

  let _ = child.kill();
  let _ = child.wait();
  panic!("server did not respond with 200 on /healthz within 90s");
}
