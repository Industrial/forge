//! E2E tests for Forge auth using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::fs;
use std::process::Command;
use std::time::Duration;

use forge_e2e_lib::cli;

#[test]
fn prebuilt_project_has_auth_layout() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  assert!(
    project_root.join("crates/db/src/auth.rs").exists(),
    "crates/db/src/auth.rs missing"
  );
  assert!(project_root.join("crates/db/src/models/organization.rs").exists());
  assert!(project_root.join("crates/db/src/models/membership.rs").exists());

  let user_model = fs::read_to_string(project_root.join("crates/db/src/models/user.rs")).unwrap();
  assert!(user_model.contains("current_org_id") && user_model.contains("current_role"));
  assert!(user_model.contains("impl AuthzContext"));

  let auth_handlers =
    fs::read_to_string(project_root.join("crates/app/src/handlers/auth.rs")).unwrap();
  assert!(
    auth_handlers.contains("guard_and_audit")
      || auth_handlers.contains("guard(Action::Manage, Role::Owner)")
  );

  let main_rs = fs::read_to_string(project_root.join("crates/app/src/main.rs")).unwrap();
  assert!(main_rs.contains("post_route") && main_rs.contains("/api/auth/admin"));
  assert!(!main_rs.contains("forge::prelude"));
}

#[tokio::test]
async fn prebuilt_server_auth_flow_register_login_protected_route() {
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
    .cookie_store(true)
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let base = format!("http://127.0.0.1:{}", port);

  for _ in 0..300 {
    tokio::time::sleep(Duration::from_millis(200)).await;
    if client.get(format!("{}/", base)).send().await.is_ok() {
      break;
    }
  }

  let reg = client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": "auth-e2e@test.com", "password": "password123" }))
    .send()
    .await
    .expect("register");
  assert!(
    reg.status().is_success(),
    "register should succeed: {}",
    reg.status()
  );

  let login = client
    .post(format!("{}/api/auth/login", base))
    .json(&serde_json::json!({ "email": "auth-e2e@test.com", "password": "password123" }))
    .send()
    .await
    .expect("login");
  assert!(
    login.status().is_success(),
    "login should succeed: {}",
    login.status()
  );

  let admin = client
    .get(format!("{}/api/auth/admin", base))
    .send()
    .await
    .expect("admin");
  assert!(
    admin.status().as_u16() == 200,
    "GET /api/auth/admin with session should be 200: {}",
    admin.status()
  );

  let _ = child.kill();
  let _ = child.wait();
}
