//! Forge — a full-stack web framework for Rust.
//!
//! Core library; CLI lives in the `forge-cli` crate.

pub mod app;
pub mod config;
pub mod error;

pub use app::App;
pub use config::ForgeConfig;
pub use error::Error;

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use std::path::Path;

  fn setup_test_config(dir: &Path, project_name: &str) {
    let config_dir = dir.join("config");
    fs::create_dir_all(&config_dir).unwrap();
    let app_toml_content = format!(
      r#"[app]
name = "{}"
environment = "test"

[server]
host = "127.0.0.1"
port = 3000
"#,
      project_name
    );
    fs::write(config_dir.join("app.toml"), app_toml_content).unwrap();
  }

  #[allow(unused_variables)]
  /// Test suite for Forge library public API
  mod public_api {
    use super::*;

    #[test]
    fn app_type_is_publicly_accessible() {
      // Given: Forge library is imported
      // When: Accessing the App type
      // Then: It should be available without compilation errors
      let _app: App;
    }

    #[test]
    fn error_type_is_publicly_accessible() {
      // Given: Forge library is imported
      // When: Accessing the Error type
      // Then: It should be available without compilation errors
      let _error: Error;
    }

    #[test]
    fn app_can_be_created_via_public_api() {
      let temp_dir = tempfile::tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(&temp_dir).unwrap();
      setup_test_config(temp_dir.path(), "test_app");

      // Given: Access to public App API
      // When: Creating a new App instance
      // Then: It should succeed without errors
      let _app = App::new();

      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn app_default_implementation_available() {
      let temp_dir = tempfile::tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(&temp_dir).unwrap();
      setup_test_config(temp_dir.path(), "test_app");

      // Given: Access to public App API
      // When: Using Default trait
      // Then: It should work
      let _app = App::default();

      std::env::set_current_dir(original_cwd).unwrap();
    }
  }

  /// Test suite for module structure
  mod module_structure {
    #[test]
    fn app_module_exists() {
      // Given: Forge library structure
      // When: Checking for app module
      // Then: Module should be accessible
      // Note: This is tested by successful compilation
    }

    #[test]
    fn error_module_exists() {
      // Given: Forge library structure
      // When: Checking for error module
      // Then: Module should be accessible
      // Note: This is tested by successful compilation
    }
  }
}
