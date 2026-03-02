//! Application cache (Moka) and optional HTTP response cache.
//! Config from `config/cache.toml`; when disabled, get returns None and set/delete no-op.

use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;
use serde::Deserialize;

/// Cache configuration from `config/cache.toml`.
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
  /// Path prefixes to exclude from response caching (e.g. \"/healthz\", \"/api/debug\").
  #[serde(default)]
  pub no_cache_paths: Option<Vec<String>>,
}

/// Default for `enabled` in cache config (true).
fn default_true() -> bool {
  true
}

/// Default max capacity for application cache entries.
fn default_max_capacity() -> u64 {
  10_000
}

/// Default TTL in seconds for application cache.
fn default_ttl_secs() -> u64 {
  300
}

/// Default TTL in seconds for HTTP response cache.
fn default_http_ttl_secs() -> u64 {
  60
}

/// In-process application cache (key-value). When cache is disabled, use [NoOpAppCache].
#[derive(Clone)]
pub struct AppCache {
  /// Moka cache backing key-value storage.
  inner: Arc<Cache<String, String>>,
}

impl AppCache {
  /// Build from config. Returns None if application cache is disabled.
  pub fn from_config(config: &CacheConfig) -> Option<Self> {
    let app = config.application.as_ref().filter(|a| a.enabled)?;
    if !config.enabled {
      return None;
    }
    let cache = Cache::builder()
      .max_capacity(app.max_capacity)
      .time_to_live(Duration::from_secs(app.default_ttl_secs))
      .build();
    Some(Self {
      inner: Arc::new(cache),
    })
  }

  /// Get value by key. Returns `None` on miss.
  pub async fn get(&self, key: &str) -> Option<String> {
    self.inner.get(key).await
  }

  /// Set key to value (uses default TTL from config).
  pub async fn set(&self, key: &str, value: String) {
    self.inner.insert(key.to_string(), value).await;
  }

  /// Remove key (manual cache busting).
  pub async fn delete(&self, key: &str) {
    self.inner.remove(key).await;
  }
}

/// No-op cache when caching is disabled: get returns None, set/delete do nothing.
#[derive(Clone, Copy, Default)]
pub struct NoOpAppCache;

impl NoOpAppCache {
  pub async fn get(&self, _key: &str) -> Option<String> {
    None
  }
  pub async fn set(&self, _key: &str, _value: String) {}
  pub async fn delete(&self, _key: &str) {}
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cache_config_default() {
    let cfg = CacheConfig::default();
    assert!(cfg.application.is_none());
    assert!(cfg.http_response.is_none());
  }

  #[test]
  fn app_cache_from_config_disabled_returns_none() {
    let cfg = CacheConfig {
      enabled: false,
      application: Some(ApplicationCacheConfig {
        enabled: true,
        max_capacity: 100,
        default_ttl_secs: 60,
      }),
      http_response: None,
    };
    assert!(AppCache::from_config(&cfg).is_none());
  }

  #[test]
  fn app_cache_from_config_no_application_returns_none() {
    let cfg = CacheConfig {
      enabled: true,
      application: None,
      http_response: None,
    };
    assert!(AppCache::from_config(&cfg).is_none());
  }

  #[test]
  fn app_cache_from_config_application_disabled_returns_none() {
    let cfg = CacheConfig {
      enabled: true,
      application: Some(ApplicationCacheConfig {
        enabled: false,
        max_capacity: 100,
        default_ttl_secs: 60,
      }),
      http_response: None,
    };
    assert!(AppCache::from_config(&cfg).is_none());
  }

  #[tokio::test]
  async fn app_cache_get_set_delete() {
    let cfg = CacheConfig {
      enabled: true,
      application: Some(ApplicationCacheConfig {
        enabled: true,
        max_capacity: 1000,
        default_ttl_secs: 300,
      }),
      http_response: None,
    };
    let cache = AppCache::from_config(&cfg).unwrap();
    assert!(cache.get("k1").await.is_none());
    cache.set("k1", "v1".to_string()).await;
    assert_eq!(cache.get("k1").await.as_deref(), Some("v1"));
    cache.delete("k1").await;
    assert!(cache.get("k1").await.is_none());
  }

  #[tokio::test]
  async fn no_op_app_cache_returns_none_and_ignores_set_delete() {
    let cache = NoOpAppCache;
    assert!(cache.get("any").await.is_none());
    cache.set("any", "value".to_string()).await;
    cache.delete("any").await;
    assert!(cache.get("any").await.is_none());
  }
}
