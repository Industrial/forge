//! Temporary directory helpers that use the project `.tmp/` directory.

use std::path::PathBuf;

/// Returns the project `.tmp` directory (under workspace root). Creates it if needed.
pub fn project_tmp_dir() -> std::io::Result<PathBuf> {
  let root = workspace_root();
  let tmp = root.join(".tmp");
  std::fs::create_dir_all(&tmp)?;
  Ok(tmp)
}

/// Creates a new temporary directory inside the project `.tmp/`.
/// The directory is removed when the returned guard is dropped.
pub fn tmpdir() -> std::io::Result<tempfile::TempDir> {
  let base = project_tmp_dir()?;
  tempfile::Builder::new().prefix("e2e-").tempdir_in(base)
}

fn workspace_root() -> PathBuf {
  std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
