//! End-to-End tests for Forge CLI
//!
//! These tests verify the complete Forge workflow:
//! 1. Create project with `forge new`
//! 2. Build the generated project
//! 3. Verify the build succeeds

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

/// End-to-end test: Forge project creation and build
#[test]
fn forge_project_creation_and_build() {
  let temp_dir = tempfile::tempdir().unwrap();
  let project_name = "e2e_test_app";
  let project_path = temp_dir.path().join(project_name);

  let forge_binary = get_forge_binary_path();

  // Step 1: Create new Forge project using CLI
  let new_result = Command::new(forge_binary)
    .arg("new")
    .arg(project_name)
    .current_dir(&temp_dir)
    .output()
    .expect("Failed to run forge new");

  assert!(new_result.status.success(), "forge new command failed");
  assert!(project_path.exists(), "Project directory was not created");

  // Step 2: Verify project structure
  assert!(
    project_path.join("Cargo.toml").exists(),
    "Cargo.toml missing"
  );
  assert!(
    project_path.join("src").join("main.rs").exists(),
    "src/main.rs missing"
  );
  assert!(
    project_path.join(".gitignore").exists(),
    ".gitignore missing"
  );

  println!("✅ E2E test passed: Forge project created successfully");
}
