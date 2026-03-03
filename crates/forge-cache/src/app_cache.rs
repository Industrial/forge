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
