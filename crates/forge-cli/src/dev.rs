//! Development mode: Vite dev server + Rust server (hot reload).

use forge::ForgeConfig;
use std::net::{TcpStream, ToSocketAddrs};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

fn wait_for_server(
  host: &str,
  port: u16,
  timeout: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
  let deadline = Instant::now() + timeout;
  let addr = (host, port)
    .to_socket_addrs()?
    .next()
    .ok_or("could not resolve server address")?;
  loop {
    if TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok() {
      return Ok(());
    }
    if Instant::now() >= deadline {
      return Err("timed out waiting for HTTP server to start".into());
    }
    std::thread::sleep(Duration::from_millis(200));
  }
}

/// Run development: backend + Vite dev server. Sets FORGE_ENVIRONMENT=development.
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
  let has_frontend = frontend_dir.join("package.json").exists();

  if has_frontend {
    eprintln!("Frontend detected. Installing dependencies (bun install)…");
    let install_status = Command::new("bun")
      .arg("install")
      .current_dir(frontend_dir)
      .status();
    if let Ok(s) = install_status {
      if !s.success() {
        eprintln!(
          "Warning: bun install failed (exit code {:?}). Continuing anyway.",
          s.code()
        );
      }
    } else {
      eprintln!("Warning: could not run bun install. Ensure bun is in PATH. Continuing.");
    }
  }

  eprintln!("Starting HTTP server (cargo run -p app)…");
  let mut cargo_child = Command::new("cargo")
    .arg("run")
    .arg("--package")
    .arg("app")
    .env("FORGE_ENVIRONMENT", "development")
    .env("PORT", port.to_string())
    .env("HOST", &host)
    .spawn()?;

  let mut vite_child: Option<Child> = None;
  if has_frontend {
    eprintln!("Waiting for HTTP server on {}:{}…", host, port);
    if let Err(e) = wait_for_server(&host, port, Duration::from_secs(90)) {
      let _ = cargo_child.kill();
      let _ = cargo_child.wait();
      return Err(e);
    }
    eprintln!("HTTP server is up. Starting Vite dev server (bun run dev)…");
    let backend_url = if host == "0.0.0.0" {
      format!("http://127.0.0.1:{}", port)
    } else {
      format!("http://{}:{}", host, port)
    };
    if let Ok(child) = Command::new("bun")
      .arg("run")
      .arg("dev")
      .current_dir(frontend_dir)
      .env("VITE_BACKEND_URL", &backend_url)
      .spawn()
    {
      vite_child = Some(child);
      eprintln!(
        "  App (open this): http://localhost:{}",
        config.frontend.port
      );
      eprintln!("  Backend (API): http://{}:{}", host, port);
    } else {
      eprintln!("Warning: could not start Vite. Run manually in frontend/ if needed.");
    }
  }

  let status = cargo_child.wait()?;
  if let Some(mut c) = vite_child {
    let _ = c.kill();
    let _ = c.wait();
  }
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
  use forge::ForgeConfig;
  use forge::config::{AppConfig, DatabaseConfig, FrontendConfig, ServerConfig};
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
    assert!(err.to_string().contains("main.rs"));
  }
}
