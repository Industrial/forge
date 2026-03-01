//! Forge — a full-stack web framework for Rust.
//!
//! Core library; CLI lives in the `forge-cli` crate.

pub mod app;
pub mod audit;
pub mod auth;
pub mod authz;
pub mod cache;
pub mod cache_http_layer;
pub mod config;
pub mod cron;
pub mod db;
pub mod error;
pub mod health;
pub mod jobs;
pub mod rate_limit;
pub mod security_headers;
pub mod seed;
pub mod token_auth;
pub mod validation;

pub use forge_macros::*;

pub use app::App;
pub use audit::{AuditError, AuditEvent, EventKind, Outcome};
pub use cache::{AppCache, CacheConfig};
pub use config::ForgeConfig;
pub use cron::CronSchedule;
pub use db::initialize_database;
pub use error::Error;
pub use jobs::ScheduledTaskJob;
pub use rate_limit::RequesterOrgKey;
pub use seed::Seeder;
pub use token_auth::{OptionalRequireAuth, RequireAuth, TokenAuthLayer, TokenLookupFn, TokenUser};
pub use validation::{Valid, Validate};

// Re-exports for a unified API (Phase 3)
pub use async_trait;
pub use axum;
pub use axum::http;
pub use axum_login;
pub use chrono;
pub use sea_orm;
pub use sea_orm_migration;
pub use serde;
pub use serde_json;
pub use tokio;
pub use tower_sessions;
pub use tower_sessions_sqlx_store;
pub use uuid;

/// Re-exported axum extractors for convenience
pub mod extract {
  pub use axum::extract::*;
}

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

    let db_toml_content = r#"[database]
url = "sqlite::memory:"
"#;
    fs::write(config_dir.join("db.toml"), db_toml_content).unwrap();
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
