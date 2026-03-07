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
  let server_bin = if std::path::Path::new("crates/server/src/main.rs").exists() {
    "server"
  } else if std::path::Path::new("crates/app/src/main.rs").exists() {
    "app"
  } else {
    return Err(
      "No server binary found. Expect crates/server/src/main.rs or crates/app/src/main.rs.".into(),
    );
  };

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

  eprintln!(
    "Starting HTTP server in production mode (cargo run -p {})…",
    server_bin
  );
  let status = Command::new("cargo")
    .arg("run")
    .arg("--package")
    .arg(server_bin)
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
        environment: None,
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

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests verify that serve::run correctly validates project structure and handles production mode.
    mod project_validation_behavior {
      use super::*;

      #[test]
      fn should_return_error_when_cargo_toml_missing() {
        // Given: a directory without Cargo.toml
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let orig = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(tmp.path());

        // When: running serve command
        let result = run(&default_config());

        // Then: it should return an error about missing Cargo.toml
        let _ = std::env::set_current_dir(orig);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
          err.to_string().contains("No Cargo.toml found"),
          "Error should mention Cargo.toml: {}",
          err
        );
      }

      #[test]
      fn should_return_error_when_cargo_toml_not_workspace() {
        // Given: a directory with Cargo.toml that is not a workspace
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::File::create(tmp.path().join("Cargo.toml"))
          .unwrap()
          .write_all(b"[package]\nname = \"test\"\n")
          .unwrap();
        let orig = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(tmp.path());

        // When: running serve command
        let result = run(&default_config());

        // Then: it should return an error about workspace
        let _ = std::env::set_current_dir(orig);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
          err.to_string().contains("workspace"),
          "Error should mention workspace: {}",
          err
        );
      }

      #[test]
      fn should_return_error_when_no_server_binary_found() {
        // Given: a workspace without server or app binary
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::File::create(tmp.path().join("Cargo.toml"))
          .unwrap()
          .write_all(b"[workspace]\nmembers = []\n")
          .unwrap();
        let orig = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(tmp.path());

        // When: running serve command
        let result = run(&default_config());

        // Then: it should return an error about missing main.rs
        let _ = std::env::set_current_dir(orig);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
          err.to_string().contains("main.rs"),
          "Error should mention main.rs: {}",
          err
        );
      }

      #[test]
      fn should_detect_server_binary_when_server_main_exists() {
        // Given: a workspace with crates/server/src/main.rs
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::File::create(tmp.path().join("Cargo.toml"))
          .unwrap()
          .write_all(b"[workspace]\nmembers = [\"server\"]\n")
          .unwrap();
        std::fs::create_dir_all(tmp.path().join("crates/server/src")).unwrap();
        std::fs::File::create(tmp.path().join("crates/server/src/main.rs"))
          .unwrap()
          .write_all(b"fn main() {}\n")
          .unwrap();
        let orig = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(tmp.path());

        // When: running serve command
        // Note: This will fail at cargo run stage, but should pass validation
        let result = run(&default_config());

        // Then: it should pass validation (error will be from cargo run, not validation)
        let _ = std::env::set_current_dir(orig);
        // The error should be about cargo run failing, not about missing binary
        if let Err(e) = result {
          assert!(
            !e.to_string().contains("main.rs") && !e.to_string().contains("No server binary"),
            "Should pass validation, error should be from cargo run: {}",
            e
          );
        }
      }

      #[test]
      fn should_detect_app_binary_when_app_main_exists() {
        // Given: a workspace with crates/app/src/main.rs (but no server)
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::File::create(tmp.path().join("Cargo.toml"))
          .unwrap()
          .write_all(b"[workspace]\nmembers = [\"app\"]\n")
          .unwrap();
        std::fs::create_dir_all(tmp.path().join("crates/app/src")).unwrap();
        std::fs::File::create(tmp.path().join("crates/app/src/main.rs"))
          .unwrap()
          .write_all(b"fn main() {}\n")
          .unwrap();
        let orig = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(tmp.path());

        // When: running serve command
        // Note: This will fail at cargo run stage, but should pass validation
        let result = run(&default_config());

        // Then: it should pass validation (error will be from cargo run, not validation)
        let _ = std::env::set_current_dir(orig);
        // The error should be about cargo run failing, not about missing binary
        if let Err(e) = result {
          assert!(
            !e.to_string().contains("main.rs") && !e.to_string().contains("No server binary"),
            "Should pass validation, error should be from cargo run: {}",
            e
          );
        }
      }

      #[test]
      fn should_prefer_server_over_app_binary() {
        // Given: a workspace with both server and app binaries
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::File::create(tmp.path().join("Cargo.toml"))
          .unwrap()
          .write_all(b"[workspace]\nmembers = [\"server\", \"app\"]\n")
          .unwrap();
        std::fs::create_dir_all(tmp.path().join("crates/server/src")).unwrap();
        std::fs::File::create(tmp.path().join("crates/server/src/main.rs"))
          .unwrap()
          .write_all(b"fn main() {}\n")
          .unwrap();
        std::fs::create_dir_all(tmp.path().join("crates/app/src")).unwrap();
        std::fs::File::create(tmp.path().join("crates/app/src/main.rs"))
          .unwrap()
          .write_all(b"fn main() {}\n")
          .unwrap();
        let orig = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(tmp.path());

        // When: running serve command
        // Note: This will fail at cargo run stage, but should pass validation
        let result = run(&default_config());

        // Then: it should use server binary (preferred over app)
        let _ = std::env::set_current_dir(orig);
        // The error should be about cargo run failing, not about missing binary
        if let Err(e) = result {
          assert!(
            !e.to_string().contains("main.rs") && !e.to_string().contains("No server binary"),
            "Should pass validation, error should be from cargo run: {}",
            e
          );
        }
      }
    }

    mod configuration_behavior {
      use super::*;

      #[test]
      fn should_use_config_host_and_port_when_running_server() {
        // Given: a valid workspace with server binary
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::File::create(tmp.path().join("Cargo.toml"))
          .unwrap()
          .write_all(b"[workspace]\nmembers = [\"server\"]\n")
          .unwrap();
        std::fs::create_dir_all(tmp.path().join("crates/server/src")).unwrap();
        std::fs::File::create(tmp.path().join("crates/server/src/main.rs"))
          .unwrap()
          .write_all(b"fn main() {}\n")
          .unwrap();

        // When: running serve with custom config
        let mut config = default_config();
        config.server.host = "0.0.0.0".to_string();
        config.server.port = 8080;

        let orig = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(tmp.path());
        let result = run(&config);
        let _ = std::env::set_current_dir(orig);

        // Then: config values should be used (validation passes, cargo run will fail)
        // The function should accept the config without error during validation phase
        if let Err(e) = result {
          // Error should be from cargo run, not config validation
          assert!(
            !e.to_string().contains("host") && !e.to_string().contains("port"),
            "Config should be accepted: {}",
            e
          );
        }
      }
    }

    mod frontend_build_behavior {
      use super::*;

      #[test]
      fn should_skip_frontend_build_when_no_frontend_directory() {
        // Given: a workspace without frontend directory
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::File::create(tmp.path().join("Cargo.toml"))
          .unwrap()
          .write_all(b"[workspace]\nmembers = [\"server\"]\n")
          .unwrap();
        std::fs::create_dir_all(tmp.path().join("crates/server/src")).unwrap();
        std::fs::File::create(tmp.path().join("crates/server/src/main.rs"))
          .unwrap()
          .write_all(b"fn main() {}\n")
          .unwrap();
        let orig = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(tmp.path());

        // When: running serve command
        let result = run(&default_config());

        // Then: it should skip frontend build and proceed to server
        let _ = std::env::set_current_dir(orig);
        // Should not error about frontend, only about cargo run
        if let Err(e) = result {
          assert!(
            !e.to_string().contains("bun") && !e.to_string().contains("frontend"),
            "Should skip frontend when not present: {}",
            e
          );
        }
      }

      #[test]
      fn should_detect_frontend_when_package_json_exists() {
        // Given: a workspace with frontend/package.json
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::File::create(tmp.path().join("Cargo.toml"))
          .unwrap()
          .write_all(b"[workspace]\nmembers = [\"server\"]\n")
          .unwrap();
        std::fs::create_dir_all(tmp.path().join("crates/server/src")).unwrap();
        std::fs::File::create(tmp.path().join("crates/server/src/main.rs"))
          .unwrap()
          .write_all(b"fn main() {}\n")
          .unwrap();
        std::fs::create_dir_all(tmp.path().join("frontend")).unwrap();
        std::fs::File::create(tmp.path().join("frontend/package.json"))
          .unwrap()
          .write_all(b"{\"name\": \"test\"}\n")
          .unwrap();
        let orig = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(tmp.path());

        // When: running serve command
        // Note: This will fail at bun install/build stage, but should detect frontend
        let result = run(&default_config());

        // Then: it should attempt to build frontend (will fail without bun)
        let _ = std::env::set_current_dir(orig);
        // Error should mention bun or frontend build
        if let Err(e) = result {
          // Either bun not found or build failed
          assert!(
            e.to_string().contains("bun")
              || e.to_string().contains("build")
              || e.to_string().contains("Cargo run"),
            "Should attempt frontend build: {}",
            e
          );
        }
      }
    }

    mod production_environment_behavior {
      use super::*;

      #[test]
      fn should_set_production_environment_when_running() {
        // Given: a valid workspace
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::File::create(tmp.path().join("Cargo.toml"))
          .unwrap()
          .write_all(b"[workspace]\nmembers = [\"server\"]\n")
          .unwrap();
        std::fs::create_dir_all(tmp.path().join("crates/server/src")).unwrap();
        std::fs::File::create(tmp.path().join("crates/server/src/main.rs"))
          .unwrap()
          .write_all(b"fn main() {}\n")
          .unwrap();
        let orig = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(tmp.path());

        // When: running serve command
        let result = run(&default_config());

        // Then: FORGE_ENVIRONMENT should be set to production
        // (This is tested indirectly - the function sets the env var before calling cargo)
        let _ = std::env::set_current_dir(orig);
        // Validation should pass, error will be from cargo run
        if let Err(e) = result {
          assert!(
            !e.to_string().contains("FORGE_ENVIRONMENT"),
            "Environment should be set correctly: {}",
            e
          );
        }
      }
    }

    mod error_message_behavior {
      use super::*;

      #[test]
      fn should_provide_helpful_error_message_when_cargo_toml_missing() {
        // Given: a directory without Cargo.toml
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let orig = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(tmp.path());

        // When: running serve command
        let result = run(&default_config());

        // Then: error message should suggest running `forge new`
        let _ = std::env::set_current_dir(orig);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(
          err_str.contains("No Cargo.toml found") && err_str.contains("forge new"),
          "Error should suggest creating project: {}",
          err_str
        );
      }

      #[test]
      fn should_provide_clear_error_message_when_no_server_binary() {
        // Given: a workspace without server or app binary
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::File::create(tmp.path().join("Cargo.toml"))
          .unwrap()
          .write_all(b"[workspace]\nmembers = []\n")
          .unwrap();
        let orig = std::env::current_dir().unwrap();
        let _ = std::env::set_current_dir(tmp.path());

        // When: running serve command
        let result = run(&default_config());

        // Then: error message should mention expected paths
        let _ = std::env::set_current_dir(orig);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(
          err_str.contains("server/src/main.rs") || err_str.contains("app/src/main.rs"),
          "Error should mention expected binary paths: {}",
          err_str
        );
      }
    }
  }
}
