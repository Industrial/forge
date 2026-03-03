//! Cache configuration types (from `config/cache.toml`).

use serde::Deserialize;

fn default_true() -> bool {
  true
}
fn default_max_capacity() -> u64 {
  10_000
}
fn default_ttl_secs() -> u64 {
  300
}
fn default_http_ttl_secs() -> u64 {
  60
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CacheConfig {
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default)]
  pub application: Option<ApplicationCacheConfig>,
  #[serde(default)]
  pub http_response: Option<HttpResponseCacheConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationCacheConfig {
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default = "default_max_capacity")]
  pub max_capacity: u64,
  #[serde(default = "default_ttl_secs")]
  pub default_ttl_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HttpResponseCacheConfig {
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default = "default_http_ttl_secs")]
  pub default_ttl_secs: u64,
  #[serde(default)]
  pub no_cache_paths: Option<Vec<String>>,
}
