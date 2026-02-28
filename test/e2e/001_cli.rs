//! End-to-End tests for Forge CLI
//!
//! These tests verify the complete Forge workflow:
//! 1. Create project with `forge new`
//! 2. Build the generated project
//! 3. Verify the build succeeds

use std::process::Command;

/// End-to-end test: Forge project creation and build
#[test]
#[ignore] // Temporarily disabled due to path resolution issues
fn forge_project_creation_and_build() {
  eprintln!("Starting E2E test...");
  println!("Starting E2E test...");
  let temp_dir = tempfile::tempdir().unwrap();
  let project_name = "e2e_test_app";
  let project_path = temp_dir.path().join(project_name);

  // Get the path to the forge binary
  // Try multiple possible locations
  let forge_binary = if let Ok(path) = std::env::var("CARGO_BIN_EXE_forge-cli") {
    println!("Using CARGO_BIN_EXE_forge-cli: {}", path);
    std::path::PathBuf::from(path)
  } else {
    // Try to find it relative to the current executable
    let current_exe = std::env::current_exe().unwrap();
    println!("Current exe: {:?}", current_exe);
    let workspace_root = current_exe
      .parent()
      .unwrap() // target/debug or target/release
      .parent()
      .unwrap() // target
      .parent()
      .unwrap(); // workspace root
    println!("Workspace root: {:?}", workspace_root);

    // Try release build first, then debug build
    let release_path = workspace_root.join("target").join("release").join("forge");
    let debug_path = workspace_root.join("target").join("debug").join("forge");

    println!("Checking release path: {:?}", release_path);
    println!("Checking debug path: {:?}", debug_path);

    if release_path.exists() {
      println!("Using release path");
      release_path
    } else if debug_path.exists() {
      println!("Using debug path");
      debug_path
    } else {
      println!("Using fallback path");
      // Fallback: try from the workspace root directly
      workspace_root.join("target").join("debug").join("forge")
    }
  };

  println!("Final forge binary path: {:?}", forge_binary);

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

  // Step 3: Try to build the generated project
  let build_result = Command::new("cargo")
    .arg("check") // Use check instead of build for faster testing
    .current_dir(&project_path)
    .output()
    .expect("Failed to run cargo check");

  if !build_result.status.success() {
    let stderr = String::from_utf8_lossy(&build_result.stderr);
    let stdout = String::from_utf8_lossy(&build_result.stdout);
    panic!(
      "Cargo check failed:\nSTDOUT: {}\nSTDERR: {}",
      stdout, stderr
    );
  }

  println!("✅ E2E test passed: Forge project created and builds successfully");
}
