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
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates").join("default")
}

/// Directories that must not be copied into new projects (local caches, deps, etc.).
fn should_skip_dir(name: &str) -> bool {
  matches!(name, ".devenv" | "node_modules" | ".git")
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
    return Err(format!(
      "Template directory not found: {} (set FORGE_TEMPLATES_DIR to override)",
      template_base.display()
    )
    .into());
  }

  let exe_path = std::env::current_exe().unwrap();
  let forge_path_str = exe_path
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .join("crates")
    .join("forge")
    .display()
    .to_string();

  copy_template_dir(&template_base, "", project_dir, project_name, &forge_path_str)?;

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
      let is_binary = dest_path.extension().is_some_and(|e| e == "ico")
        || std::str::from_utf8(&bytes).is_err();

      if is_binary {
        fs::write(&dest_path, bytes)?;
      } else {
        let text = String::from_utf8(bytes).unwrap();
        let replaced = text
          .replace("{{PROJECT_NAME}}", project_name)
          .replace("{{FORGE_PATH}}", forge_path)
          .replace("../../../../../forge", forge_path);
        fs::write(&dest_path, replaced)?;
      }
    }
  }

  Ok(())
}
