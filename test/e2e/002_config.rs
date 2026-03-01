//! E2E tests for Forge config using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::fs;
use std::process::Command;
use std::time::Duration;

use forge_e2e_lib::cli;

#[test]
fn prebuilt_project_has_config_files_and_content() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  let app_toml = project_root.join("config/app.toml");
  let db_toml = project_root.join("config/db.toml");
  assert!(app_toml.exists(), "config/app.toml missing");
  assert!(db_toml.exists(), "config/db.toml missing");

  let app_content = fs::read_to_string(&app_toml).unwrap();
  assert!(app_content.contains("[app]"), "config/app.toml should have [app]");
  assert!(
    app_content.contains("[server]"),
    "config/app.toml should have [server]"
  );
  let db_content = fs::read_to_string(&db_toml).unwrap();
  assert!(
    db_content.contains("[database]"),
    "config/db.toml should have [database]"
  );
}

#[tokio::test]
async fn prebuilt_server_serves_and_uses_config() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );

  let port = cli::next_e2e_port();
  fs::write(
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

  for _ in 0..300 {
    tokio::time::sleep(Duration::from_millis(200)).await;
    if let Ok(resp) = client.get(&url).send().await
      && resp.status().as_u16() == 200
    {
      let body = resp.text().await.unwrap_or_default();
      assert_eq!(body.trim(), "ok");
      let _ = child.kill();
      let _ = child.wait();
      return;
    }
  }

  let _ = child.kill();
  let _ = child.wait();
  panic!("server did not respond with 200 on /healthz within 60s");
}
