//! Create a new Forge project (scaffold) from the templates/default directory.
//! Templates are read from the filesystem at runtime; the binary does not embed them.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Resolve the path to the default template directory.
/// Prefer FORGE_TEMPLATES_DIR env (path to templates/default or its parent).
/// Otherwise use the crate's templates at build-time path (works when run from repo).
fn template_dir() -> PathBuf {
  if let Ok(dir) = std::env::var("FORGE_TEMPLATES_DIR") {
    let p = PathBuf::from(dir);
    if p.join("default").is_dir() {
      return p.join("default");
    }
    if p.is_dir() {
      return p;
    }
  }
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("templates")
    .join("default")
}

/// Directories that must not be copied into new projects (local caches, deps, etc.).
pub(crate) fn should_skip_dir(name: &str) -> bool {
  matches!(name, ".devenv" | "node_modules" | ".git" | "target")
}

pub fn create_new_project(name: &str) -> Result<(), Box<dyn std::error::Error>> {
  let project_dir = Path::new(name);

  if project_dir.exists() {
    return Err(format!("Directory '{}' already exists", name).into());
  }

  let project_name = project_dir
    .file_name()
    .and_then(|p| p.to_str())
    .unwrap_or(name);

  let template_base = template_dir();
  if !template_base.is_dir() {
    return Err(
      format!(
        "Template directory not found: {} (set FORGE_TEMPLATES_DIR to override)",
        template_base.display()
      )
      .into(),
    );
  }

  let exe_path = std::env::current_exe().unwrap();
  // Workspace root (when run from repo target/debug/forge) for path replacements in template Cargo.toml.
  let forge_path_str = exe_path
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .display()
    .to_string();

  copy_template_dir(
    &template_base,
    "",
    project_dir,
    project_name,
    &forge_path_str,
  )?;

  let _ = Command::new("git")
    .arg("init")
    .current_dir(project_dir)
    .status();

  Ok(())
}

fn copy_template_dir(
  source_root: &Path,
  rel: &str,
  dest_root: &Path,
  project_name: &str,
  forge_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let source_path = source_root.join(rel);
  let entries = fs::read_dir(&source_path)?;

  for entry in entries {
    let entry = entry?;
    let name = entry.file_name();
    let name_str = name.to_string_lossy();

    if should_skip_dir(&name_str) {
      continue;
    }

    let rel_ent = if rel.is_empty() {
      name_str.to_string()
    } else {
      format!("{}/{}", rel, name_str)
    };
    let dest_path = dest_root.join(&rel_ent);

    if entry.file_type()?.is_dir() {
      fs::create_dir_all(&dest_path)?;
      copy_template_dir(source_root, &rel_ent, dest_root, project_name, forge_path)?;
    } else {
      if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
      }

      let bytes = fs::read(entry.path())?;
      let is_binary =
        dest_path.extension().is_some_and(|e| e == "ico") || std::str::from_utf8(&bytes).is_err();

      if is_binary {
        fs::write(&dest_path, bytes)?;
      } else {
        let text = String::from_utf8(bytes).unwrap();
        let replaced = text
          .replace("{{PROJECT_NAME}}", project_name)
          .replace("{{FORGE_PATH}}", forge_path)
          .replace("../../../../../forge-app", &format!("{}/crates/forge-app", forge_path))
          .replace("../../../../../forge-auth", &format!("{}/crates/forge-auth", forge_path))
          .replace("../../../../../forge-audit", &format!("{}/crates/forge-audit", forge_path))
          .replace("../../../../../forge-cache", &format!("{}/crates/forge-cache", forge_path))
          .replace("../../../../../forge-config", &format!("{}/crates/forge-config", forge_path))
          .replace("../../../../../forge-core", &format!("{}/crates/forge-core", forge_path))
          .replace("../../../../../forge-cron", &format!("{}/crates/forge-cron", forge_path))
          .replace("../../../../../forge-db", &format!("{}/crates/forge-db", forge_path))
          .replace("../../../../../forge-live", &format!("{}/crates/forge-live", forge_path))
          .replace("../../../../../forge-observability", &format!("{}/crates/forge-observability", forge_path));
        fs::write(&dest_path, replaced)?;
      }
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;

  #[test]
  fn should_skip_dir_skips_devenv_node_modules_git_target() {
    assert!(should_skip_dir(".devenv"));
    assert!(should_skip_dir("node_modules"));
    assert!(should_skip_dir(".git"));
    assert!(should_skip_dir("target"));
  }

  #[test]
  fn should_skip_dir_does_not_skip_other_dirs() {
    assert!(!should_skip_dir("src"));
    assert!(!should_skip_dir("crates"));
    assert!(!should_skip_dir("templates"));
    assert!(!should_skip_dir(""));
  }

  #[test]
  fn create_new_project_err_when_dir_exists() {
    let tmp = tempfile::tempdir().unwrap();
    let existing = tmp.path().join("existing");
    std::fs::create_dir_all(&existing).unwrap();
    let err = create_new_project(existing.to_str().unwrap()).unwrap_err();
    let msg = err.to_string();
    assert!(
      msg.contains("already exists"),
      "expected 'already exists', got: {}",
      msg
    );
  }

  /// When FORGE_TEMPLATES_DIR points to a nonexistent path, create_new_project returns Err.
  /// Ignored by default because it mutates process env and can race with other tests.
  #[test]
  #[ignore]
  fn create_new_project_err_when_template_dir_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("nonexistent");
    unsafe {
      std::env::set_var("FORGE_TEMPLATES_DIR", missing.as_os_str());
    }
    let out_dir = tmp.path().join("out");
    std::fs::create_dir_all(&out_dir).unwrap();
    let project_path = out_dir.join("myapp");
    let result = create_new_project(project_path.to_str().unwrap());
    unsafe {
      std::env::remove_var("FORGE_TEMPLATES_DIR");
    }
    assert!(result.is_err(), "expected Err when template dir is missing");
  }

  #[test]
  fn create_new_project_success_and_replaces_placeholders() {
    let tmp = tempfile::tempdir().unwrap();
    let template_root = tmp.path().join("templates").join("default");
    std::fs::create_dir_all(&template_root).unwrap();
    let f = template_root.join("Cargo.toml");
    let mut f = std::fs::File::create(&f).unwrap();
    f.write_all(b"name = \"{{PROJECT_NAME}}\"\npath = \"{{FORGE_PATH}}\"")
      .unwrap();
    f.sync_all().unwrap();
    drop(f);
    unsafe {
      std::env::set_var("FORGE_TEMPLATES_DIR", tmp.path().join("templates"));
    }
    let out_dir = tmp.path().join("out");
    std::fs::create_dir_all(&out_dir).unwrap();
    let project_path = out_dir.join("myapp");
    let result = create_new_project(project_path.to_str().unwrap());
    unsafe {
      std::env::remove_var("FORGE_TEMPLATES_DIR");
    }
    result.expect("create_new_project should succeed");
    let generated = std::fs::read_to_string(project_path.join("Cargo.toml")).unwrap();
    assert!(
      generated.contains("myapp"),
      "expected PROJECT_NAME replacement: {}",
      generated
    );
  }
}
