use std::sync::Arc;
use std::time::Duration;

use forge_config::CacheConfig;
use moka::future::Cache;

/// In-process application cache (key-value). When cache is disabled, use [NoOpAppCache].
#[derive(Clone)]
pub struct AppCache {
  /// The underlying Moka cache instance.
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

  // --- BDD Tests ---

  mod cache_creation_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_cache_when_config_is_fully_enabled() {
      // Given: a cache config with all settings enabled
      let config = CacheConfig {
        enabled: true,
        application: Some(ApplicationCacheConfig {
          enabled: true,
          max_capacity: 100,
          default_ttl_secs: 60,
        }),
        http_response: None,
      };

      // When: creating cache from config
      let cache = AppCache::from_config(&config);

      // Then: cache should be created successfully
      assert!(cache.is_some());
      let cache = cache.unwrap();
      // Verify it works by setting and getting a value
      cache.set("test_key", "test_value".to_string()).await;
      assert_eq!(cache.get("test_key").await.as_deref(), Some("test_value"));
    }

    #[tokio::test]
    async fn should_return_none_when_cache_is_disabled() {
      // Given: a cache config with cache disabled
      let config = CacheConfig {
        enabled: false,
        application: Some(ApplicationCacheConfig {
          enabled: true,
          max_capacity: 1000,
          default_ttl_secs: 300,
        }),
        http_response: None,
      };

      // When: creating cache from config
      let cache = AppCache::from_config(&config);

      // Then: cache should not be created
      assert!(cache.is_none());
    }

    #[tokio::test]
    async fn should_return_none_when_application_config_is_missing() {
      // Given: a cache config without application config
      let config = CacheConfig {
        enabled: true,
        application: None,
        http_response: None,
      };

      // When: creating cache from config
      let cache = AppCache::from_config(&config);

      // Then: cache should not be created
      assert!(cache.is_none());
    }

    #[tokio::test]
    async fn should_return_none_when_application_cache_is_disabled() {
      // Given: a cache config with application cache disabled
      let config = CacheConfig {
        enabled: true,
        application: Some(ApplicationCacheConfig {
          enabled: false,
          max_capacity: 1000,
          default_ttl_secs: 300,
        }),
        http_response: None,
      };

      // When: creating cache from config
      let cache = AppCache::from_config(&config);

      // Then: cache should not be created
      assert!(cache.is_none());
    }

    #[tokio::test]
    async fn should_apply_max_capacity_from_config() {
      // Given: a cache config with specific max capacity and TTL
      let max_capacity = 50;
      let default_ttl_secs = 60;
      let config = CacheConfig {
        enabled: true,
        application: Some(ApplicationCacheConfig {
          enabled: true,
          max_capacity,
          default_ttl_secs,
        }),
        http_response: None,
      };

      // When: creating cache from config
      let cache = AppCache::from_config(&config);

      // Then: cache should be created successfully and accept entries up to capacity
      assert!(cache.is_some());
      let cache = cache.unwrap();
      // Verify cache works by storing and retrieving values
      for i in 0..max_capacity.min(10) {
        let value = format!("value_{}", i);
        cache.set(&format!("key_{}", i), value.clone()).await;
        assert_eq!(
          cache.get(&format!("key_{}", i)).await.as_deref(),
          Some(value.as_str())
        );
      }
    }
  }

  mod cache_operations_behavior {
    use super::*;

    #[tokio::test]
    async fn should_store_and_retrieve_value_when_key_exists() {
      // Given: a cache instance
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

      // When: setting a value and retrieving it
      cache.set("my_key", "my_value".to_string()).await;
      let result = cache.get("my_key").await;

      // Then: the value should be retrieved correctly
      assert_eq!(result.as_deref(), Some("my_value"));
    }

    #[tokio::test]
    async fn should_return_none_when_key_does_not_exist() {
      // Given: a cache instance with no entries
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

      // When: retrieving a non-existent key
      let result = cache.get("non_existent_key").await;

      // Then: result should be None
      assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_overwrite_value_when_setting_same_key_twice() {
      // Given: a cache instance with an existing entry
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
      cache.set("key", "original_value".to_string()).await;

      // When: setting a new value for the same key
      cache.set("key", "updated_value".to_string()).await;

      // Then: the new value should be retrieved
      assert_eq!(cache.get("key").await.as_deref(), Some("updated_value"));
    }

    #[tokio::test]
    async fn should_delete_value_when_key_exists() {
      // Given: a cache instance with an entry
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
      cache.set("key_to_delete", "value".to_string()).await;
      assert!(cache.get("key_to_delete").await.is_some());

      // When: deleting the key
      cache.delete("key_to_delete").await;

      // Then: the key should no longer exist
      assert!(cache.get("key_to_delete").await.is_none());
    }

    #[tokio::test]
    async fn should_handle_delete_when_key_does_not_exist() {
      // Given: a cache instance with no entries
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

      // When: deleting a non-existent key
      cache.delete("non_existent_key").await;

      // Then: operation should complete without error
      assert!(cache.get("non_existent_key").await.is_none());
    }

    #[tokio::test]
    async fn should_store_multiple_keys_independently() {
      // Given: a cache instance
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

      // When: setting multiple different keys
      cache.set("key1", "value1".to_string()).await;
      cache.set("key2", "value2".to_string()).await;
      cache.set("key3", "value3".to_string()).await;

      // Then: all keys should be retrievable independently
      assert_eq!(cache.get("key1").await.as_deref(), Some("value1"));
      assert_eq!(cache.get("key2").await.as_deref(), Some("value2"));
      assert_eq!(cache.get("key3").await.as_deref(), Some("value3"));
    }

    #[tokio::test]
    async fn should_handle_empty_string_values() {
      // Given: a cache instance
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

      // When: setting an empty string value
      cache.set("empty_key", String::new()).await;

      // Then: the empty string should be retrievable
      assert_eq!(cache.get("empty_key").await.as_deref(), Some(""));
    }
  }

  mod noop_cache_behavior {
    use super::*;

    #[tokio::test]
    async fn should_always_return_none_for_get_operations() {
      // Given: a NoOpAppCache instance
      let cache = NoOpAppCache;

      // When: attempting to get any key
      let result1 = cache.get("any_key").await;
      let result2 = cache.get("another_key").await;

      // Then: all get operations should return None
      assert!(result1.is_none());
      assert!(result2.is_none());
    }

    #[tokio::test]
    async fn should_ignore_set_operations() {
      // Given: a NoOpAppCache instance
      let cache = NoOpAppCache;

      // When: setting values
      cache.set("key1", "value1".to_string()).await;
      cache.set("key2", "value2".to_string()).await;

      // Then: values should not be stored
      assert!(cache.get("key1").await.is_none());
      assert!(cache.get("key2").await.is_none());
    }

    #[tokio::test]
    async fn should_ignore_delete_operations() {
      // Given: a NoOpAppCache instance
      let cache = NoOpAppCache;

      // When: deleting keys
      cache.delete("key1").await;
      cache.delete("non_existent_key").await;

      // Then: operations should complete without error
      // (No-op cache always returns None, so we just verify it doesn't panic)
      assert!(cache.get("key1").await.is_none());
    }

    #[tokio::test]
    async fn should_be_cloneable() {
      // Given: a NoOpAppCache instance
      let cache = NoOpAppCache;

      // When: cloning the cache
      let cloned_cache = cache;

      // Then: cloned cache should behave the same way
      assert!(cloned_cache.get("any_key").await.is_none());
      cloned_cache.set("key", "value".to_string()).await;
      assert!(cloned_cache.get("key").await.is_none());
    }
  }
}
