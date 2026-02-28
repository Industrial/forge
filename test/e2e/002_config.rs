//! End-to-End tests for Forge CLI config module

use std::fs;
use std::process::Command;

/// Helper function to get the path to the forge binary
fn get_forge_binary_path() -> std::path::PathBuf {
  if let Ok(path) = std::env::var("CARGO_BIN_EXE_forge") {
    std::path::PathBuf::from(path)
  } else {
    let mut current_dir = std::env::current_exe().unwrap();
    while current_dir.file_name().and_then(|s| s.to_str()) != Some("target") {
      if let Some(parent) = current_dir.parent() {
        current_dir = parent.to_path_buf();
      } else {
        break;
      }
    }
    let workspace_root = if current_dir.file_name().and_then(|s| s.to_str()) == Some("target") {
      current_dir.parent().unwrap().to_path_buf()
    } else {
      // Fallback to current directory if we can't find target
      std::env::current_dir().unwrap()
    };

    let release_path = workspace_root.join("target").join("release").join("forge");
    let debug_path = workspace_root.join("target").join("debug").join("forge");

    match (release_path.exists(), debug_path.exists()) {
      (true, true) => {
        let release_meta = fs::metadata(&release_path).unwrap();
        let debug_meta = fs::metadata(&debug_path).unwrap();
        if release_meta.modified().unwrap() > debug_meta.modified().unwrap() {
          release_path
        } else {
          debug_path
        }
      }
      (true, false) => release_path,
      (false, true) => debug_path,
      (false, false) => workspace_root.join("target").join("debug").join("forge"),
    }
  }
}

/// End-to-end test: Forge project creation generates config/app.toml
#[test]
fn forge_new_generates_config_file() {
  let temp_dir = tempfile::tempdir().unwrap();
  let project_name = "config_test_app";
  let project_path = temp_dir.path().join(project_name);

  let forge_binary = get_forge_binary_path();

  // Step 1: Create new Forge project using CLI
  let new_result = Command::new(&forge_binary)
    .arg("new")
    .arg(project_name)
    .current_dir(&temp_dir)
    .output()
    .expect("Failed to run forge new");

  assert!(new_result.status.success(), "forge new command failed");
  assert!(project_path.exists(), "Project directory was not created");

  // Step 2: Verify config/app.toml exists and has correct content
  let config_path = project_path.join("config").join("app.toml");
  assert!(
    config_path.exists(),
    "config/app.toml missing after forge new"
  );

  let config_content = fs::read_to_string(&config_path).expect("Could not read config/app.toml");
  assert!(!config_content.is_empty(), "config/app.toml is empty");
  assert!(
    config_content.contains("[app]"),
    "config/app.toml missing [app] section"
  );
  assert!(
    config_content.contains(&format!(r#"name = "{}""#, project_name)),
    "config/app.toml has wrong project name"
  );

  // Step 3: Verify the app can actually be initialized (library level check)
  // We can't easily run App::new() here because it depends on the CWD
  // but we can check if the file structure is correct for Figment
  println!("✅ E2E test passed: config/app.toml generated correctly with content");
}

/// End-to-end test: Forge serve uses configured host and port
#[test]
fn forge_serve_uses_config_host_and_port() {
  let temp_dir = tempfile::tempdir().unwrap();
  let project_name = "serve_config_test_app";
  let project_path = temp_dir.path().join(project_name);

  let forge_binary = get_forge_binary_path();

  // Step 1: Create new Forge project using CLI
  let new_result = Command::new(&forge_binary)
    .arg("new")
    .arg(project_name)
    .current_dir(&temp_dir)
    .output()
    .expect("Failed to run forge new");

  assert!(new_result.status.success(), "forge new command failed");
  assert!(project_path.exists(), "Project directory was not created");

  // Step 2: Start the server in the new project directory
  let mut server_command = Command::new("cargo")
    .arg("run")
    .current_dir(&project_path)
    .spawn()
    .expect("Failed to start forge serve");

  // Step 3: Poll the endpoint until it responds
  let client = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(1))
    .build()
    .unwrap();
  let mut response = None;
  let start = std::time::Instant::now();
  let timeout = std::time::Duration::from_secs(15); // Increased timeout for cargo build overhead

  while start.elapsed() < timeout {
    if let Ok(res) = client.get("http://0.0.0.0:3000/").send() {
      response = Some(res);
      break;
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
  }

  let response = response.expect("Server failed to start within timeout");

  assert!(
    response.status().is_success(),
    "Server did not return success status"
  );
  let body = response.text().unwrap();
  assert_eq!(body, "Hello from Forge!");

  // Step 4: Terminate the server process
  server_command
    .kill()
    .expect("Failed to kill server process");
  server_command
    .wait()
    .expect("Failed to wait for server process");

  println!("✅ E2E test passed: Forge serve uses configured host and port");
}

/// End-to-end test: Forge serve fails gracefully when config is missing
#[test]
fn forge_serve_fails_without_config() {
  let temp_dir = tempfile::tempdir().unwrap();
  let project_name = "no_config_app";
  let project_path = temp_dir.path().join(project_name);

  let forge_binary = get_forge_binary_path();

  // Create a minimal project structure manually WITHOUT config/app.toml
  fs::create_dir_all(project_path.join("src")).unwrap();
  fs::write(
    project_path.join("Cargo.toml"),
    format!(
      r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
forge = {{ path = "{}/crates/forge" }}
"#,
      project_name,
      std::env::current_dir().unwrap().display()
    ),
  )
  .unwrap();

  fs::write(
    project_path.join("src/main.rs"),
    r#"use forge::App;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new().serve().await
}
"#,
  )
  .unwrap();

  // Run forge serve
  let serve_result = Command::new(&forge_binary)
    .arg("serve")
    .current_dir(&project_path)
    .output()
    .expect("Failed to run forge serve");

  assert!(
    !serve_result.status.success(),
    "forge serve should have failed without config"
  );
  let stderr = String::from_utf8_lossy(&serve_result.stderr);
  assert!(
    stderr.contains("config/app.toml") && stderr.contains("not found"),
    "Error message should mention config/app.toml not found"
  );

  println!("✅ E2E test passed: forge serve fails correctly without config");
}

/// End-to-end test: Forge new works from a different working directory
#[test]
fn forge_new_works_from_different_dir() {
  let temp_dir = tempfile::tempdir().unwrap();
  let subdir = temp_dir.path().join("subdir");
  fs::create_dir(&subdir).unwrap();

  let project_name = "remote_app";
  let project_path = subdir.join(project_name);

  let forge_binary = get_forge_binary_path();

  // Run forge new from temp_dir, but target subdir/remote_app
  let new_result = Command::new(&forge_binary)
    .arg("new")
    .arg(project_path.to_str().unwrap())
    .current_dir(&temp_dir)
    .output()
    .expect("Failed to run forge new");

  assert!(
    new_result.status.success(),
    "forge new command failed when using path: {}",
    String::from_utf8_lossy(&new_result.stderr)
  );
  assert!(
    project_path.exists(),
    "Project directory was not created at {}",
    project_path.display()
  );
  assert!(
    project_path.join("config/app.toml").exists(),
    "config/app.toml missing in remote project"
  );

  println!("✅ E2E test passed: forge new works with absolute/relative paths");
}

/// End-to-end test: Forge new works with nested paths
#[test]
fn forge_new_works_with_nested_path() {
  let temp_dir = tempfile::tempdir().unwrap();
  let project_name = "nested/project/app";
  let project_path = temp_dir.path().join(project_name);

  let forge_binary = get_forge_binary_path();

  // Run forge new with a nested path
  let new_result = Command::new(&forge_binary)
    .arg("new")
    .arg(project_name)
    .current_dir(&temp_dir)
    .output()
    .expect("Failed to run forge new");

  assert!(
    new_result.status.success(),
    "forge new command failed for nested path: {}",
    String::from_utf8_lossy(&new_result.stderr)
  );
  assert!(
    project_path.exists(),
    "Project directory was not created at {}",
    project_path.display()
  );
  assert!(
    project_path.join("config/app.toml").exists(),
    "config/app.toml missing in nested project"
  );

  println!("✅ E2E test passed: forge new works with nested paths");
}
