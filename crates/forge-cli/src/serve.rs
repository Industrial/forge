//! Production mode: build frontend, then run Rust server serving static assets.

use forge_config::ForgeConfig;
use std::process::Command;

/// Build frontend (if present), then run the app with FORGE_ENVIRONMENT=production.
pub fn run(config: &ForgeConfig) -> Result<(), Box<dyn std::error::Error>> {
  if !std::path::Path::new("Cargo.toml").exists() {
    return Err("No Cargo.toml found. Run `forge new myapp` to create a Forge project.".into());
  }
  let cargo_toml = std::fs::read_to_string("Cargo.toml")?;
  if !cargo_toml.contains("[workspace]") {
    return Err("Not a Forge workspace. Ensure your Cargo.toml has a [workspace] section.".into());
  }
  if !std::path::Path::new("crates/app/src/main.rs").exists() {
    return Err(
      "crates/app/src/main.rs not found. Ensure your project has an app crate with a main.rs."
        .into(),
    );
  }

  let port = config.server.port;
  let host = config.server.host.clone();
  let frontend_dir = std::path::Path::new("frontend");

  if frontend_dir.join("package.json").exists() {
    eprintln!("Frontend detected. Installing dependencies (bun install)…");
    let install_status = Command::new("bun")
      .arg("install")
      .current_dir(frontend_dir)
      .status();
    match install_status {
      Ok(s) if !s.success() => {
        return Err("bun install failed. Fix frontend dependencies and try again.".into());
      }
      Err(e) => {
        return Err(format!("Could not run bun install ({}). Ensure bun is in PATH.", e).into());
      }
      _ => {}
    }
    eprintln!("Building frontend (bun run build)…");
    let build_status = Command::new("bun")
      .arg("run")
      .arg("build")
      .current_dir(frontend_dir)
      .status()?;
    if !build_status.success() {
      return Err("Frontend build failed. Fix build errors and try again.".into());
    }
  }

  eprintln!("Starting HTTP server in production mode (cargo run -p app)…");
  let status = Command::new("cargo")
    .arg("run")
    .arg("--package")
    .arg("app")
    .env("FORGE_ENVIRONMENT", "production")
    .env("PORT", port.to_string())
    .env("HOST", &host)
    .status()?;

  if status.success() {
    Ok(())
  } else {
    Err(
      format!(
        "Cargo run failed with exit code: {}",
        status.code().unwrap_or(-1)
      )
      .into(),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use forge_config::{AppConfig, DatabaseConfig, ForgeConfig, FrontendConfig, ServerConfig};
  use std::io::Write;

  fn default_config() -> ForgeConfig {
    ForgeConfig {
      app: AppConfig {
        name: "test".to_string(),
      },
      server: ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 3000,
      },
      database: DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: None,
        min_connections: None,
        connect_timeout: None,
        idle_timeout: None,
        auto_migrate: true,
        auto_seed: false,
      },
      cache: None,
      frontend: FrontendConfig { port: 5173 },
    }
  }

  #[test]
  fn run_err_when_no_cargo_toml() {
    let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let orig = std::env::current_dir().unwrap();
    let _ = std::env::set_current_dir(tmp.path());
    let r = run(&default_config());
    let _ = std::env::set_current_dir(orig);
    let err = r.unwrap_err();
    assert!(err.to_string().contains("No Cargo.toml found"));
  }

  #[test]
  fn run_err_when_not_workspace() {
    let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    std::fs::File::create(tmp.path().join("Cargo.toml"))
      .unwrap()
      .write_all(b"[package]\nname = \"x\"\n")
      .unwrap();
    let orig = std::env::current_dir().unwrap();
    let _ = std::env::set_current_dir(tmp.path());
    let r = run(&default_config());
    let _ = std::env::set_current_dir(orig);
    let err = r.unwrap_err();
    assert!(err.to_string().contains("workspace"));
  }

  #[test]
  fn run_err_when_no_app_main() {
    let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    std::fs::File::create(tmp.path().join("Cargo.toml"))
      .unwrap()
      .write_all(b"[workspace]\nmembers = []\n")
      .unwrap();
    let orig = std::env::current_dir().unwrap();
    let _ = std::env::set_current_dir(tmp.path());
    let r = run(&default_config());
    let _ = std::env::set_current_dir(orig);
    let err = r.unwrap_err();
    assert!(
      err.to_string().contains("main.rs"),
      "expected error about main.rs, got: {}",
      err
    );
  }
}
