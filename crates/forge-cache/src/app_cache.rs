use std::sync::Arc;
use std::time::Duration;

use forge_config::CacheConfig;
use moka::future::Cache;

/// In-process application cache (key-value). When cache is disabled, use [NoOpAppCache].
#[derive(Clone)]
pub struct AppCache {
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

  pub async fn get(&self, key: &str) -> Option<String> {
    self.inner.get(key).await
  }

  pub async fn set(&self, key: &str, value: String) {
    self.inner.insert(key.to_string(), value).await;
  }

  pub async fn delete(&self, key: &str) {
    self.inner.remove(key).await;
  }
}

/// No-op cache when caching is disabled.
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
  use forge_config::ApplicationCacheConfig;

  #[tokio::test]
  async fn from_config_returns_none_when_disabled() {
    let config = CacheConfig {
      enabled: false,
      application: Some(ApplicationCacheConfig {
        enabled: true,
        max_capacity: 1000,
        default_ttl_secs: 300,
      }),
      http_response: None,
    };
    assert!(AppCache::from_config(&config).is_none());
  }

  #[tokio::test]
  async fn from_config_returns_none_when_no_application_config() {
    let config = CacheConfig {
      enabled: true,
      application: None,
      http_response: None,
    };
    assert!(AppCache::from_config(&config).is_none());
  }

  #[tokio::test]
  async fn from_config_returns_none_when_application_disabled() {
    let config = CacheConfig {
      enabled: true,
      application: Some(ApplicationCacheConfig {
        enabled: false,
        max_capacity: 1000,
        default_ttl_secs: 300,
      }),
      http_response: None,
    };
    assert!(AppCache::from_config(&config).is_none());
  }

  #[tokio::test]
  async fn from_config_returns_some_when_enabled() {
    let config = CacheConfig {
      enabled: true,
      application: Some(ApplicationCacheConfig {
        enabled: true,
        max_capacity: 100,
        default_ttl_secs: 60,
      }),
      http_response: None,
    };
    let cache = AppCache::from_config(&config).unwrap();
    cache.set("k", "v".to_string()).await;
    assert_eq!(cache.get("k").await.as_deref(), Some("v"));
    cache.delete("k").await;
    assert!(cache.get("k").await.is_none());
  }

  #[tokio::test]
  async fn noop_app_cache_returns_none_and_ignores_write() {
    let cache = NoOpAppCache;
    assert!(cache.get("any").await.is_none());
    cache.set("k", "v".to_string()).await;
    cache.delete("k").await;
    assert!(cache.get("k").await.is_none());
  }
}
