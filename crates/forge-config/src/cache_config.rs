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

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use figment::{
    Figment,
    providers::{Format, Toml},
  };

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.
  mod cache_config_behavior {
    use super::*;

    #[test]
    fn should_default_to_enabled_when_not_specified() {
      // Given: CacheConfig with enabled not specified
      let toml = r#""#;
      let config: CacheConfig = Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking enabled field
      // Then: should default to true
      assert!(config.enabled, "Cache should default to enabled");
    }

    #[test]
    fn should_allow_disabling_cache() {
      // Given: CacheConfig with enabled = false
      let toml = r#"enabled = false"#;
      let config: CacheConfig = Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking enabled field
      // Then: should be false
      assert!(
        !config.enabled,
        "Cache should be disabled when set to false"
      );
    }

    #[test]
    fn should_have_optional_application_cache_config() {
      // Given: CacheConfig without application section
      let toml = r#"enabled = true"#;
      let config: CacheConfig = Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking application field
      // Then: should be None
      assert!(
        config.application.is_none(),
        "Application cache should be optional"
      );
    }

    #[test]
    fn should_have_optional_http_response_cache_config() {
      // Given: CacheConfig without http_response section
      let toml = r#"enabled = true"#;
      let config: CacheConfig = Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking http_response field
      // Then: should be None
      assert!(
        config.http_response.is_none(),
        "HTTP response cache should be optional"
      );
    }

    #[test]
    fn should_load_application_cache_config_when_provided() {
      // Given: CacheConfig with application section
      let toml = r#"
enabled = true
[application]
enabled = true
max_capacity = 5000
default_ttl_secs = 120
"#;
      let config: CacheConfig = Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking application config
      // Then: should be Some with correct values
      let app_cache = config.application.as_ref().unwrap();
      assert!(app_cache.enabled);
      assert_eq!(app_cache.max_capacity, 5000);
      assert_eq!(app_cache.default_ttl_secs, 120);
    }

    #[test]
    fn should_load_http_response_cache_config_when_provided() {
      // Given: CacheConfig with http_response section
      let toml = r#"
enabled = true
[http_response]
enabled = true
default_ttl_secs = 90
no_cache_paths = ["/health", "/ready"]
"#;
      let config: CacheConfig = Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking http_response config
      // Then: should be Some with correct values
      let http_cache = config.http_response.as_ref().unwrap();
      assert!(http_cache.enabled);
      assert_eq!(http_cache.default_ttl_secs, 90);
      assert_eq!(
        http_cache.no_cache_paths.as_deref(),
        Some(&["/health".to_string(), "/ready".to_string()][..])
      );
    }

    #[test]
    fn should_be_cloneable() {
      // Given: a CacheConfig instance
      let toml = r#"
enabled = true
[application]
enabled = true
"#;
      let config: CacheConfig = Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: cloning it
      let cloned = config.clone();

      // Then: should have same values
      assert_eq!(cloned.enabled, config.enabled);
      assert_eq!(cloned.application.is_some(), config.application.is_some());
    }

    #[test]
    fn should_be_debuggable() {
      // Given: a CacheConfig instance
      let toml = r#"enabled = true"#;
      let config: CacheConfig = Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: formatting for debug
      let debug_str = format!("{:?}", config);

      // Then: should produce debug output
      assert!(!debug_str.is_empty(), "Should produce debug output");
    }

    #[test]
    fn should_implement_default_trait() {
      // Given: Default trait implementation
      // When: creating default CacheConfig
      let config = CacheConfig::default();

      // Then: should have default values
      // Default for bool is false, but we use default_true() which returns true
      // Actually, let's check what Default::default() does
      // Since we have #[derive(Default)], it uses the type's default
      // But we also have #[serde(default = "default_true")] which only applies during deserialization
      // So Default::default() will use bool::default() which is false
      assert!(
        !config.enabled,
        "Default should use bool::default() == false"
      );
      assert!(config.application.is_none());
      assert!(config.http_response.is_none());
    }
  }

  mod application_cache_config_behavior {
    use super::*;

    #[test]
    fn should_default_to_enabled_when_not_specified() {
      // Given: ApplicationCacheConfig with enabled not specified
      let toml = r#"
max_capacity = 1000
default_ttl_secs = 200
"#;
      let config: ApplicationCacheConfig =
        Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking enabled field
      // Then: should default to true
      assert!(
        config.enabled,
        "Application cache should default to enabled"
      );
    }

    #[test]
    fn should_use_default_max_capacity_when_not_specified() {
      // Given: ApplicationCacheConfig without max_capacity
      let toml = r#"
enabled = true
default_ttl_secs = 200
"#;
      let config: ApplicationCacheConfig =
        Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking max_capacity
      // Then: should default to 10_000
      assert_eq!(config.max_capacity, 10_000, "Should default to 10_000");
    }

    #[test]
    fn should_use_default_ttl_when_not_specified() {
      // Given: ApplicationCacheConfig without default_ttl_secs
      let toml = r#"
enabled = true
max_capacity = 5000
"#;
      let config: ApplicationCacheConfig =
        Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking default_ttl_secs
      // Then: should default to 300
      assert_eq!(
        config.default_ttl_secs, 300,
        "Should default to 300 seconds"
      );
    }

    #[test]
    fn should_load_all_fields_when_provided() {
      // Given: ApplicationCacheConfig with all fields
      let toml = r#"
enabled = false
max_capacity = 20000
default_ttl_secs = 600
"#;
      let config: ApplicationCacheConfig =
        Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking all fields
      // Then: should have correct values
      assert!(!config.enabled);
      assert_eq!(config.max_capacity, 20000);
      assert_eq!(config.default_ttl_secs, 600);
    }

    #[test]
    fn should_be_cloneable() {
      // Given: an ApplicationCacheConfig instance
      let toml = r#"
enabled = true
max_capacity = 5000
default_ttl_secs = 120
"#;
      let config: ApplicationCacheConfig =
        Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: cloning it
      let cloned = config.clone();

      // Then: should have same values
      assert_eq!(cloned.enabled, config.enabled);
      assert_eq!(cloned.max_capacity, config.max_capacity);
      assert_eq!(cloned.default_ttl_secs, config.default_ttl_secs);
    }

    #[test]
    fn should_be_debuggable() {
      // Given: an ApplicationCacheConfig instance
      let toml = r#"enabled = true"#;
      let config: ApplicationCacheConfig =
        Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: formatting for debug
      let debug_str = format!("{:?}", config);

      // Then: should produce debug output
      assert!(!debug_str.is_empty(), "Should produce debug output");
    }
  }

  mod http_response_cache_config_behavior {
    use super::*;

    #[test]
    fn should_default_to_enabled_when_not_specified() {
      // Given: HttpResponseCacheConfig with enabled not specified
      let toml = r#"
default_ttl_secs = 90
"#;
      let config: HttpResponseCacheConfig =
        Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking enabled field
      // Then: should default to true
      assert!(
        config.enabled,
        "HTTP response cache should default to enabled"
      );
    }

    #[test]
    fn should_use_default_http_ttl_when_not_specified() {
      // Given: HttpResponseCacheConfig without default_ttl_secs
      let toml = r#"enabled = true"#;
      let config: HttpResponseCacheConfig =
        Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking default_ttl_secs
      // Then: should default to 60
      assert_eq!(config.default_ttl_secs, 60, "Should default to 60 seconds");
    }

    #[test]
    fn should_have_optional_no_cache_paths() {
      // Given: HttpResponseCacheConfig without no_cache_paths
      let toml = r#"
enabled = true
default_ttl_secs = 90
"#;
      let config: HttpResponseCacheConfig =
        Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking no_cache_paths
      // Then: should be None
      assert!(
        config.no_cache_paths.is_none(),
        "no_cache_paths should be optional"
      );
    }

    #[test]
    fn should_load_no_cache_paths_when_provided() {
      // Given: HttpResponseCacheConfig with no_cache_paths
      let toml = r#"
enabled = true
default_ttl_secs = 90
no_cache_paths = ["/health", "/ready", "/metrics"]
"#;
      let config: HttpResponseCacheConfig =
        Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking no_cache_paths
      // Then: should contain specified paths
      let paths = config.no_cache_paths.as_ref().unwrap();
      assert_eq!(paths.len(), 3);
      assert!(paths.contains(&"/health".to_string()));
      assert!(paths.contains(&"/ready".to_string()));
      assert!(paths.contains(&"/metrics".to_string()));
    }

    #[test]
    fn should_load_all_fields_when_provided() {
      // Given: HttpResponseCacheConfig with all fields
      let toml = r#"
enabled = false
default_ttl_secs = 120
no_cache_paths = ["/api/health"]
"#;
      let config: HttpResponseCacheConfig =
        Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: checking all fields
      // Then: should have correct values
      assert!(!config.enabled);
      assert_eq!(config.default_ttl_secs, 120);
      assert_eq!(
        config.no_cache_paths.as_deref(),
        Some(&["/api/health".to_string()][..])
      );
    }

    #[test]
    fn should_be_cloneable() {
      // Given: an HttpResponseCacheConfig instance
      let toml = r#"
enabled = true
default_ttl_secs = 90
no_cache_paths = ["/health"]
"#;
      let config: HttpResponseCacheConfig =
        Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: cloning it
      let cloned = config.clone();

      // Then: should have same values
      assert_eq!(cloned.enabled, config.enabled);
      assert_eq!(cloned.default_ttl_secs, config.default_ttl_secs);
      assert_eq!(cloned.no_cache_paths, config.no_cache_paths);
    }

    #[test]
    fn should_be_debuggable() {
      // Given: an HttpResponseCacheConfig instance
      let toml = r#"enabled = true"#;
      let config: HttpResponseCacheConfig =
        Figment::new().merge(Toml::string(toml)).extract().unwrap();

      // When: formatting for debug
      let debug_str = format!("{:?}", config);

      // Then: should produce debug output
      assert!(!debug_str.is_empty(), "Should produce debug output");
    }
  }

  mod default_function_behavior {
    use super::*;

    #[test]
    fn should_provide_default_true() {
      // Given: default_true function
      // When: calling it
      let value = default_true();

      // Then: should return true
      assert!(value, "default_true should return true");
    }

    #[test]
    fn should_provide_default_max_capacity() {
      // Given: default_max_capacity function
      // When: calling it
      let value = default_max_capacity();

      // Then: should return 10_000
      assert_eq!(value, 10_000, "default_max_capacity should return 10_000");
    }

    #[test]
    fn should_provide_default_ttl_secs() {
      // Given: default_ttl_secs function
      // When: calling it
      let value = default_ttl_secs();

      // Then: should return 300
      assert_eq!(value, 300, "default_ttl_secs should return 300");
    }

    #[test]
    fn should_provide_default_http_ttl_secs() {
      // Given: default_http_ttl_secs function
      // When: calling it
      let value = default_http_ttl_secs();

      // Then: should return 60
      assert_eq!(value, 60, "default_http_ttl_secs should return 60");
    }
  }
}
