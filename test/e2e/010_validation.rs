//! E2E tests for Forge validation using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::fs;
use std::process::Command;
use std::time::Duration;

use forge_e2e_lib::cli;

#[test]
fn prebuilt_project_has_auth_routes_in_main() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  let main_rs = fs::read_to_string(project_root.join("crates/app/src/main.rs")).unwrap();
  assert!(
    main_rs.contains("/api/auth/register"),
    "generated main.rs must contain /api/auth/register"
  );
}

#[tokio::test]
async fn prebuilt_server_register_and_login_validation_422_for_invalid() {
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
    .args(["run", "-p", "app", "--quiet"])
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
    if client.get(format!("{}/", base)).send().await.is_ok() {
      break;
    }
    if i == 449 {
      let _ = child.kill();
      let _ = child.wait();
      panic!("server did not become ready in time");
    }
  }

  let reg_invalid = client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": "not-an-email", "password": "short" }))
    .send()
    .await
    .expect("register");
  assert_eq!(
    reg_invalid.status().as_u16(),
    422,
    "invalid register should return 422: {}",
    reg_invalid.status()
  );
  let body = reg_invalid.text().await.unwrap();
  assert!(
    body.contains("errors") || body.contains("email") || body.contains("password"),
    "422 body should contain error details: {}",
    body
  );

  let reg_ok = client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": "valid@example.com", "password": "password123" }))
    .send()
    .await
    .expect("register");
  assert!(
    reg_ok.status().is_success(),
    "valid register should succeed: {}",
    reg_ok.status()
  );

  let login_invalid = client
    .post(format!("{}/api/auth/login", base))
    .json(&serde_json::json!({ "email": "bad-email", "password": "any" }))
    .send()
    .await
    .expect("login");
  assert_eq!(
    login_invalid.status().as_u16(),
    422,
    "invalid login should return 422: {}",
    login_invalid.status()
  );

  let _ = child.kill();
  let _ = child.wait();
}
