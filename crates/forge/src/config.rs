use figment::{
  providers::{Format, Toml},
  Figment,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ForgeConfig {
  pub app: AppConfig,
  pub server: ServerConfig,
  pub database: DatabaseConfig,
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

  Figment::new()
    .merge(Toml::file(app_config_path))
    .merge(Toml::file(db_config_path))
    .extract()
    .map_err(|e| {
      format!(
        "Failed to parse configuration files. Ensure they contain the required [app], [server], and [database] sections. Error: {}",
        e
      )
      .into()
    })
}
