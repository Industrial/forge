use figment::{
  providers::{Format, Toml},
  Figment,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ForgeConfig {
  pub app: AppConfig,
  pub server: ServerConfig,
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

pub fn load_config() -> Result<ForgeConfig, Box<dyn std::error::Error>> {
  let config_path = "config/app.toml";
  if !std::path::Path::new(config_path).exists() {
    return Err(format!(
      "Configuration file '{}' not found. If this is a new project, run `forge new` to generate it.",
      config_path
    )
    .into());
  }

  Figment::new()
    .merge(Toml::file(config_path))
    .extract()
    .map_err(|e| {
      format!(
        "Failed to parse configuration file '{}': {}. Ensure it contains [app] and [server] sections.",
        config_path, e
      )
      .into()
    })
}
