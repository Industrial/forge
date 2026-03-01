//! Create a new Forge project (scaffold) from the templates/default directory.

use include_dir::{Dir, DirEntry};
use std::fs;
use std::path::Path;
use std::process::Command;

static TEMPLATE: Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/templates/default");

pub fn create_new_project(name: &str) -> Result<(), Box<dyn std::error::Error>> {
  let project_dir = Path::new(name);

  if project_dir.exists() {
    return Err(format!("Directory '{}' already exists", name).into());
  }

  let project_name = project_dir
    .file_name()
    .and_then(|p| p.to_str())
    .unwrap_or(name);

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

  // entry.path() is the full path from the template root (see include_dir Doc)
  copy_template_dir(
    TEMPLATE.entries(),
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
  entries: &[DirEntry<'_>],
  dest: &Path,
  project_name: &str,
  forge_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  for entry in entries {
    let rel_path = entry.path();
    let dest_path = dest.join(rel_path);

    match entry {
      DirEntry::Dir(d) => {
        fs::create_dir_all(&dest_path)?;
        copy_template_dir(d.entries(), dest, project_name, forge_path)?;
      }
      DirEntry::File(f) => {
        if let Some(parent) = dest_path.parent() {
          fs::create_dir_all(parent)?;
        }

        let is_binary = rel_path.extension().is_some_and(|e| e == "ico");
        if is_binary {
          fs::write(&dest_path, f.contents())?;
        } else {
          let text =
            std::str::from_utf8(f.contents()).map_err(|_| "template file is not valid UTF-8")?;
          let replaced = text
            .replace("{{PROJECT_NAME}}", project_name)
            .replace("{{FORGE_PATH}}", forge_path);
          fs::write(&dest_path, replaced)?;
        }
      }
    }
  }
  Ok(())
}
