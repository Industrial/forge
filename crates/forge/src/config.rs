use figment::{
  Figment,
  providers::{Format, Toml},
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ForgeConfig {
  pub app: AppConfig,
  pub server: ServerConfig,
  pub database: DatabaseConfig,
  #[serde(default)]
  pub cache: Option<crate::cache::CacheConfig>,
  /// Frontend (Vite) dev server port. Used by Inertia and `forge serve`. Set in config/app.toml [frontend].
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

/// Effective environment for this process. Set by the CLI (`forge dev` / `forge serve`) via `FORGE_ENVIRONMENT`; defaults to `"development"`.
pub fn effective_environment() -> String {
  std::env::var("FORGE_ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
}

pub fn load_config() -> Result<ForgeConfig, Box<dyn std::error::Error>> {
  let app_config_path = "config/app.toml";
  let db_config_path = "config/db.toml";

  if !std::path::Path::new(app_config_path).exists() {
    return Err(format!(
      "Configuration file '{}' not found. If this is a new project, run `forge new` to generate it.",
      app_config_path
    )
    .into());
  }

  if !std::path::Path::new(db_config_path).exists() {
    return Err(format!(
      "Configuration file '{}' not found. Forge requires a database configuration. Run `forge new` to generate it.",
      db_config_path
    )
    .into());
  }

  let mut config: ForgeConfig = Figment::new()
    .merge(Toml::file(app_config_path))
    .merge(Toml::file(db_config_path))
    .extract()
    .map_err(|e| -> Box<dyn std::error::Error> {
      format!(
        "Failed to parse configuration files. Ensure they contain the required [app], [server], and [database] sections. Error: {}",
        e
      )
      .into()
    })?;

  let cache_config_path = "config/cache.toml";
  if std::path::Path::new(cache_config_path).exists() {
    config.cache = Some(
      Figment::new()
        .merge(Toml::file(cache_config_path))
        .extract::<crate::cache::CacheConfig>()
        .map_err(|e| format!("Failed to parse {}. Error: {}", cache_config_path, e))?,
    );
  }

  Ok(config)
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
    unsafe { std::env::remove_var("FORGE_ENVIRONMENT") };
    assert_eq!(effective_environment(), "development");
  }

  #[test]
  fn effective_environment_uses_env_when_set() {
    unsafe { std::env::set_var("FORGE_ENVIRONMENT", "production") };
    assert_eq!(effective_environment(), "production");
    unsafe { std::env::remove_var("FORGE_ENVIRONMENT") };
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
    let orig = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();
    let res = load_config();
    std::env::set_current_dir(orig).unwrap();
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
    // do not create db.toml
    let orig = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();
    let res = load_config();
    std::env::set_current_dir(orig).unwrap();
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
    let orig = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();
    let res = load_config();
    std::env::set_current_dir(orig).unwrap();
    assert!(res.is_ok());
    let cfg = res.unwrap();
    assert_eq!(cfg.app.name, "myapp");
    assert_eq!(cfg.server.port, 3000);
    assert_eq!(cfg.database.url, "sqlite::memory:");
    assert!(cfg.cache.is_none());
  }
}
