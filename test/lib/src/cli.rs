//! Shared CLI helpers for E2E: forge binary path, run forge new / cargo check, assert layout.
//! Generated projects live under `.tmp/` (via tmpdir), so cargo uses `project_dir/target` by default; no CARGO_TARGET_DIR needed.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU16, Ordering};

static E2E_PORT_COUNTER: AtomicU16 = AtomicU16::new(0);

/// Returns a unique port for E2E servers (30_000–30_999) so parallel tests do not collide.
pub fn next_e2e_port() -> u16 {
  30_000u16.saturating_add(E2E_PORT_COUNTER.fetch_add(1, Ordering::Relaxed) % 1000)
}

/// Path to the `forge` binary (from `CARGO_BIN_EXE_forge` or workspace `target/debug/forge`).
pub fn forge_binary() -> std::path::PathBuf {
  if let Ok(path) = std::env::var("CARGO_BIN_EXE_forge") {
    return std::path::PathBuf::from(path);
  }
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

/// Path to the prebuilt project created by `bin/test-e2e` (`.tmp/e2e_prebuilt`). Panics if `.tmp` cannot be created.
pub fn prebuilt_project_root() -> std::path::PathBuf {
  crate::tmpdir::project_tmp_dir()
    .expect(".tmp dir")
    .join("e2e_prebuilt")
}

/// Run `forge new <name>` in `dir`; return the command output. Caller asserts success and path.
pub fn run_forge_new(dir: &Path, name: &str) -> Output {
  Command::new(forge_binary())
    .arg("new")
    .arg(name)
    .current_dir(dir)
    .output()
    .expect("failed to run forge new")
}

/// Run `cargo check` in `project_dir`. Target dir is `project_dir/target` (under `.tmp/`). Return output for assertion.
pub fn run_cargo_check(project_dir: &Path) -> Output {
  Command::new("cargo")
    .arg("check")
    .current_dir(project_dir)
    .output()
    .expect("failed to run cargo check")
}

/// Assert generated project layout and main.rs content.
pub fn assert_project_layout(project_root: &Path) {
  assert!(
    project_root.join("Cargo.toml").exists(),
    "root Cargo.toml missing"
  );
  assert!(
    project_root.join("crates/app/src/main.rs").exists(),
    "crates/app/src/main.rs missing"
  );
  assert!(
    project_root.join("crates/db/src/lib.rs").exists(),
    "crates/db/src/lib.rs missing"
  );
  assert!(
    project_root.join("config/app.toml").exists(),
    "config/app.toml missing"
  );
  assert!(
    project_root.join("config/db.toml").exists(),
    "config/db.toml missing"
  );
  assert!(
    project_root.join(".gitignore").exists(),
    ".gitignore missing"
  );

  let main_rs = fs::read_to_string(project_root.join("crates/app/src/main.rs")).unwrap();
  assert!(
    main_rs.contains("App::new()"),
    "main.rs should use App::new()"
  );
  assert!(main_rs.contains(".serve()"), "main.rs should call .serve()");
}
