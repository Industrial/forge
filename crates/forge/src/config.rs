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
  /// Frontend (Vite) dev server port. Used by Inertia and `forge serve`.
  #[serde(default)]
  pub frontend: FrontendConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FrontendConfig {
  #[serde(default = "default_frontend_port")]
  pub port: u16,
}

impl Default for FrontendConfig {
  fn default() -> Self {
    Self { port: default_frontend_port() }
  }
}

fn default_frontend_port() -> u16 {
  3000
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
  pub name: String,
  pub environment: String,
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
  #[serde(default = "default_true")]
  pub auto_migrate: bool,
  #[serde(default = "default_true")]
  pub auto_seed: bool,
}

/// Default value for boolean configuration fields.
fn default_true() -> bool {
  true
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
