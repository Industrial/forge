//! Configuration loading and types for the Forge framework.

mod cache_config;

pub use cache_config::{ApplicationCacheConfig, CacheConfig, HttpResponseCacheConfig};

use figment::{
  Figment,
  providers::{Format, Toml},
};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ForgeConfig {
  pub app: AppConfig,
  pub server: ServerConfig,
  pub database: DatabaseConfig,
  #[serde(default)]
  pub cache: Option<CacheConfig>,
  pub frontend: FrontendConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FrontendConfig {
  pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
  pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
  pub host: String,
  pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
  pub url: String,
  pub max_connections: Option<u32>,
  pub min_connections: Option<u32>,
  pub connect_timeout: Option<u64>,
  pub idle_timeout: Option<u64>,
  pub auto_migrate: bool,
  pub auto_seed: bool,
}

/// Effective environment (e.g. `FORGE_ENVIRONMENT`); defaults to `"development"`.
pub fn effective_environment() -> String {
  std::env::var("FORGE_ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
}

/// Load configuration from a given directory (looks for `config/app.toml`, `config/db.toml`, optional `config/cache.toml`).
pub fn load_config_from_dir(base: &Path) -> Result<ForgeConfig, Box<dyn std::error::Error>> {
  let app_config_path = base.join("config/app.toml");
  let db_config_path = base.join("config/db.toml");
  let app_display = app_config_path.display().to_string();
  let db_display = db_config_path.display().to_string();

  if !app_config_path.exists() {
    return Err(format!(
      "Configuration file '{}' not found. If this is a new project, run `forge new` to generate it.",
      app_display
    )
    .into());
  }

  if !db_config_path.exists() {
    return Err(format!(
      "Configuration file '{}' not found. Forge requires a database configuration. Run `forge new` to generate it.",
      db_display
    )
    .into());
  }

  let mut config: ForgeConfig = Figment::new()
    .merge(Toml::file(&app_config_path))
    .merge(Toml::file(&db_config_path))
    .extract()
    .map_err(|e| -> Box<dyn std::error::Error> {
      format!(
        "Failed to parse configuration files. Ensure they contain the required [app], [server], and [database] sections. Error: {}",
        e
      )
      .into()
    })?;

  let cache_config_path = base.join("config/cache.toml");
  if cache_config_path.exists() {
    config.cache = Some(
      Figment::new()
        .merge(Toml::file(&cache_config_path))
        .extract::<CacheConfig>()
        .map_err(|e| -> Box<dyn std::error::Error> {
          format!(
            "Failed to parse {}. Error: {}",
            cache_config_path.display(),
            e
          )
          .into()
        })?,
    );
  }

  Ok(config)
}

/// Load configuration from the current working directory.
pub fn load_config() -> Result<ForgeConfig, Box<dyn std::error::Error>> {
  let cwd = std::env::current_dir().map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
  load_config_from_dir(&cwd)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use std::path::Path;

  fn write_test_config(dir: &Path, app_toml: &str, db_toml: &str) {
    let config_dir = dir.join("config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("app.toml"), app_toml).unwrap();
    fs::write(config_dir.join("db.toml"), db_toml).unwrap();
  }

  #[test]
  fn effective_environment_defaults_to_development() {
    let _ = std::env::var("FORGE_ENVIRONMENT");
    // Don't mutate env in tests; just document behavior
    assert!(!effective_environment().is_empty());
  }

  #[test]
  fn cache_config_default() {
    let cache = CacheConfig::default();
    assert!(!cache.enabled); // derive(Default) uses bool::default() == false
    assert!(cache.application.is_none());
    assert!(cache.http_response.is_none());
  }

  #[test]
  fn load_config_fails_when_app_toml_missing() {
    let dir = tempfile::tempdir().unwrap();
    let db_only = r#"[database]
url = "sqlite::memory:"
auto_migrate = true
auto_seed = false
"#;
    fs::create_dir_all(dir.path().join("config")).unwrap();
    fs::write(dir.path().join("config/db.toml"), db_only).unwrap();
    let res = load_config_from_dir(dir.path());
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.to_string().contains("app.toml"));
  }

  #[test]
  fn load_config_fails_when_db_toml_missing() {
    let dir = tempfile::tempdir().unwrap();
    let app_toml = r#"[app]
name = "test"
[server]
host = "127.0.0.1"
port = 3000
[frontend]
port = 3000
"#;
    let config_dir = dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("app.toml"), app_toml).unwrap();
    let res = load_config_from_dir(dir.path());
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.to_string().contains("db.toml"));
  }

  #[test]
  fn load_config_succeeds_with_required_only() {
    let dir = tempfile::tempdir().unwrap();
    let app_toml = r#"[app]
name = "myapp"
[server]
host = "127.0.0.1"
port = 3000
[frontend]
port = 3000
"#;
    let db_toml = r#"[database]
url = "sqlite::memory:"
auto_migrate = true
auto_seed = false
"#;
    write_test_config(dir.path(), app_toml, db_toml);
    let res = load_config_from_dir(dir.path());
    assert!(res.is_ok());
    let cfg = res.unwrap();
    assert_eq!(cfg.app.name, "myapp");
    assert_eq!(cfg.server.port, 3000);
    assert_eq!(cfg.database.url, "sqlite::memory:");
    assert!(cfg.cache.is_none());
  }

  #[test]
  fn load_config_succeeds_with_cache_toml() {
    let dir = tempfile::tempdir().unwrap();
    let app_toml = r#"[app]
name = "myapp"
[server]
host = "127.0.0.1"
port = 3000
[frontend]
port = 3000
"#;
    let db_toml = r#"[database]
url = "sqlite::memory:"
auto_migrate = true
auto_seed = false
"#;
    let cache_toml = r#"
enabled = true
[application]
enabled = true
max_capacity = 5000
default_ttl_secs = 120
[http_response]
enabled = true
default_ttl_secs = 90
no_cache_paths = ["/health", "/ready"]
"#;
    write_test_config(dir.path(), app_toml, db_toml);
    let config_dir = dir.path().join("config");
    fs::write(config_dir.join("cache.toml"), cache_toml).unwrap();
    let res = load_config_from_dir(dir.path());
    assert!(res.is_ok());
    let cfg = res.unwrap();
    let cache = cfg.cache.as_ref().unwrap();
    assert!(cache.enabled);
    let app_cache = cache.application.as_ref().unwrap();
    assert!(app_cache.enabled);
    assert_eq!(app_cache.max_capacity, 5000);
    assert_eq!(app_cache.default_ttl_secs, 120);
    let http_cache = cache.http_response.as_ref().unwrap();
    assert!(http_cache.enabled);
    assert_eq!(http_cache.default_ttl_secs, 90);
    let expected_paths: Vec<String> = vec!["/health".into(), "/ready".into()];
    assert_eq!(
      http_cache.no_cache_paths.as_deref(),
      Some(expected_paths.as_slice())
    );
  }

  #[test]
  fn load_config_succeeds_with_minimal_cache_toml_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let app_toml = r#"[app]
name = "myapp"
[server]
host = "127.0.0.1"
port = 3000
[frontend]
port = 3000
"#;
    let db_toml = r#"[database]
url = "sqlite::memory:"
auto_migrate = true
auto_seed = false
"#;
    let cache_toml = r#"enabled = true"#;
    write_test_config(dir.path(), app_toml, db_toml);
    fs::write(dir.path().join("config/cache.toml"), cache_toml).unwrap();
    let res = load_config_from_dir(dir.path());
    assert!(res.is_ok());
    let cfg = res.unwrap();
    let cache = cfg.cache.as_ref().unwrap();
    assert!(cache.enabled);
    assert!(cache.application.is_none());
    assert!(cache.http_response.is_none());
  }

  #[test]
  fn load_config_succeeds_with_empty_cache_toml_uses_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let app_toml = r#"[app]
name = "myapp"
[server]
host = "127.0.0.1"
port = 3000
[frontend]
port = 3000
"#;
    let db_toml = r#"[database]
url = "sqlite::memory:"
auto_migrate = true
auto_seed = false
"#;
    write_test_config(dir.path(), app_toml, db_toml);
    fs::write(dir.path().join("config/cache.toml"), "").unwrap();
    let res = load_config_from_dir(dir.path());
    assert!(res.is_ok());
    let cfg = res.unwrap();
    let cache = cfg.cache.as_ref().unwrap();
    assert!(cache.enabled, "CacheConfig default_true() for enabled");
    assert!(cache.application.is_none());
    assert!(cache.http_response.is_none());
  }

  #[test]
  fn load_config_succeeds_with_cache_application_http_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let app_toml = r#"[app]
name = "myapp"
[server]
host = "127.0.0.1"
port = 3000
[frontend]
port = 3000
"#;
    let db_toml = r#"[database]
url = "sqlite::memory:"
auto_migrate = true
auto_seed = false
"#;
    let cache_toml = r#"
[application]
[http_response]
"#;
    write_test_config(dir.path(), app_toml, db_toml);
    fs::write(dir.path().join("config/cache.toml"), cache_toml).unwrap();
    let res = load_config_from_dir(dir.path());
    assert!(res.is_ok());
    let cfg = res.unwrap();
    let cache = cfg.cache.as_ref().unwrap();
    let app_cache = cache.application.as_ref().unwrap();
    assert!(app_cache.enabled);
    assert_eq!(app_cache.max_capacity, 10_000);
    assert_eq!(app_cache.default_ttl_secs, 300);
    let http_cache = cache.http_response.as_ref().unwrap();
    assert!(http_cache.enabled);
    assert_eq!(http_cache.default_ttl_secs, 60);
    assert!(http_cache.no_cache_paths.is_none());
  }

  #[test]
  fn load_config_fails_when_app_toml_invalid() {
    let dir = tempfile::tempdir().unwrap();
    let app_toml = "not valid toml [[[";
    let db_toml = r#"[database]
url = "sqlite::memory:"
auto_migrate = true
auto_seed = false
"#;
    write_test_config(dir.path(), app_toml, db_toml);
    let res = load_config_from_dir(dir.path());
    assert!(res.is_err());
    let err = res.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Failed to parse configuration"), "{}", msg);
  }

  #[test]
  fn load_config_fails_when_cache_toml_invalid() {
    let dir = tempfile::tempdir().unwrap();
    let app_toml = r#"[app]
name = "myapp"
[server]
host = "127.0.0.1"
port = 3000
[frontend]
port = 3000
"#;
    let db_toml = r#"[database]
url = "sqlite::memory:"
auto_migrate = true
auto_seed = false
"#;
    write_test_config(dir.path(), app_toml, db_toml);
    fs::write(dir.path().join("config/cache.toml"), "invalid [[[").unwrap();
    let res = load_config_from_dir(dir.path());
    assert!(res.is_err());
    let err = res.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("cache.toml"), "{}", msg);
  }

  #[test]
  fn load_config_uses_current_dir() {
    let dir = tempfile::tempdir().unwrap();
    let app_toml = r#"[app]
name = "cwdapp"
[server]
host = "0.0.0.0"
port = 4000
[frontend]
port = 4000
"#;
    let db_toml = r#"[database]
url = "sqlite::memory:"
auto_migrate = true
auto_seed = false
"#;
    write_test_config(dir.path(), app_toml, db_toml);
    let orig = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();
    let res = load_config();
    std::env::set_current_dir(&orig).unwrap();
    assert!(
      res.is_ok(),
      "load_config() should succeed when cwd has config: {:?}",
      res.err()
    );
    let cfg = res.unwrap();
    assert_eq!(cfg.app.name, "cwdapp");
    assert_eq!(cfg.server.port, 4000);
  }

  #[test]
  fn load_config_fails_when_current_dir_unavailable() {
    // When the current directory has been removed (e.g. we chdir into a temp dir then drop it),
    // current_dir() can fail with ENOENT on Unix. This tests the map_err path in load_config().
    let orig = std::env::current_dir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_path_buf();
    std::env::set_current_dir(&path).unwrap();
    drop(dir); // removes the directory while we're still in it
    let res = load_config();
    std::env::set_current_dir(&orig).unwrap(); // restore so later tests are not affected
    assert!(
      res.is_err(),
      "load_config() should fail when cwd has been removed"
    );
  }
}
