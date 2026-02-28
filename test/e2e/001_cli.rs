//! End-to-End tests for Forge CLI basic commands

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
      std::env::current_dir().unwrap()
    };

    workspace_root.join("target").join("debug").join("forge")
  }
}

#[test]
fn forge_project_creation_file_verification() {
  let temp_dir = tempfile::tempdir().unwrap();
  let project_name = "cli_test_app";
  let forge_binary = get_forge_binary_path();

  // 1. Create project
  let new_result = Command::new(&forge_binary)
    .arg("new")
    .arg(project_name)
    .current_dir(&temp_dir)
    .output()
    .expect("Failed to run forge new");

  assert!(new_result.status.success());

  let project_path = temp_dir.path().join(project_name);

  // 2. Verify files exist and have content
  assert!(project_path.join("Cargo.toml").exists());
  assert!(project_path.join("crates/app/src/main.rs").exists());
  assert!(project_path.join("crates/db/src/lib.rs").exists());
  assert!(project_path.join("config/app.toml").exists());
  assert!(project_path.join("config/db.toml").exists());
  assert!(project_path.join(".gitignore").exists());

  let main_rs = fs::read_to_string(project_path.join("crates/app/src/main.rs")).unwrap();
  assert!(main_rs.contains("App::new()"));
  assert!(main_rs.contains(".serve()"));
}
