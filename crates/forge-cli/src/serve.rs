//! Production mode: build frontend, then run Rust server serving static assets.

use forge::ForgeConfig;
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
