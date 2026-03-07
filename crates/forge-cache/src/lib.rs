//! Application cache (Moka) and HTTP response cache layer for the Forge framework.

/// Application cache implementation.
mod app_cache;
mod http_layer;

pub use app_cache::{AppCache, NoOpAppCache};
pub use forge_config::{ApplicationCacheConfig, CacheConfig, HttpResponseCacheConfig};
pub use http_layer::HttpResponseCacheLayer;

#[cfg(test)]
mod bdd_tests {
  use super::*;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests verify that public API re-exports are accessible and functional.
  mod public_api_exports_behavior {
    use super::*;

    #[test]
    fn should_export_app_cache_types_when_imported_from_crate_root() {
      // Given: importing from crate root
      // When: using AppCache and NoOpAppCache types
      let _app_cache_type: Option<AppCache> = None;
      let _noop_cache: NoOpAppCache = NoOpAppCache;

      // Then: types should be accessible and usable
    }

    #[test]
    fn should_export_config_types_when_imported_from_crate_root() {
      // Given: importing from crate root
      // When: using config types
      let _cache_config: CacheConfig = CacheConfig {
        enabled: true,
        application: None,
        http_response: None,
      };
      let _app_cache_config: Option<ApplicationCacheConfig> = None;
      let _http_cache_config: Option<HttpResponseCacheConfig> = None;

      // Then: config types should be accessible and constructible
    }

    #[test]
    fn should_export_http_cache_layer_when_imported_from_crate_root() {
      // Given: importing from crate root
      // When: using HttpResponseCacheLayer type
      let _layer_type: Option<HttpResponseCacheLayer> = None;

      // Then: HttpResponseCacheLayer should be accessible
    }

    #[tokio::test]
    async fn should_create_app_cache_from_exported_config_type() {
      // Given: CacheConfig imported from crate root
      let config = CacheConfig {
        enabled: true,
        application: Some(ApplicationCacheConfig {
          enabled: true,
          max_capacity: 100,
          default_ttl_secs: 60,
        }),
        http_response: None,
      };

      // When: creating AppCache from config
      let cache = AppCache::from_config(&config);

      // Then: cache should be created successfully
      assert!(
        cache.is_some(),
        "AppCache should be created from exported config type"
      );
    }

    #[tokio::test]
    async fn should_create_http_cache_layer_from_exported_config_type() {
      // Given: CacheConfig imported from crate root
      let config = CacheConfig {
        enabled: true,
        application: None,
        http_response: Some(HttpResponseCacheConfig {
          enabled: true,
          default_ttl_secs: 120,
          no_cache_paths: None,
        }),
      };

      // When: creating HttpResponseCacheLayer from config
      let layer = HttpResponseCacheLayer::from_config(&config);

      // Then: layer should be created successfully
      assert!(
        layer.is_some(),
        "HttpResponseCacheLayer should be created from exported config type"
      );
    }

    #[tokio::test]
    async fn should_use_noop_cache_when_imported_from_crate_root() {
      // Given: NoOpAppCache imported from crate root
      let cache = NoOpAppCache;

      // When: using NoOpAppCache methods
      let result = cache.get("test-key").await;

      // Then: should return None (no-op behavior)
      assert!(
        result.is_none(),
        "NoOpAppCache should return None for all keys"
      );
    }

    #[test]
    fn should_allow_constructing_cache_config_with_exported_types() {
      // Given: all config types imported from crate root
      // When: constructing a complete CacheConfig
      let config = CacheConfig {
        enabled: true,
        application: Some(ApplicationCacheConfig {
          enabled: true,
          max_capacity: 1000,
          default_ttl_secs: 300,
        }),
        http_response: Some(HttpResponseCacheConfig {
          enabled: true,
          default_ttl_secs: 60,
          no_cache_paths: Some(vec!["/healthz".into()]),
        }),
      };

      // Then: config should be valid and usable
      assert!(config.enabled, "Config should be enabled");
      assert!(
        config.application.is_some(),
        "Application config should be present"
      );
      assert!(
        config.http_response.is_some(),
        "HTTP response config should be present"
      );
    }
  }
}
