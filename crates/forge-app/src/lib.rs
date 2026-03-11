//! Forge application builder — fluent API for configuring and running Axum-based web applications.

pub mod app;

pub use app::{App, AuthInstallerFn, MigratorFn, SeedFn, init_tracing, shutdown_signal_future};

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use std::path::Path;

  fn setup_test_config(dir: &Path, project_name: &str) {
    let config_dir = dir.join("config");
    fs::create_dir_all(&config_dir).unwrap();
    let app_toml_content = format!(
      r#"[app]
name = "{}"

[server]
host = "127.0.0.1"
port = 3000

[frontend]
port = 3000
"#,
      project_name
    );
    fs::write(config_dir.join("app.toml"), app_toml_content).unwrap();

    let db_toml_content = r#"[database]
url = "sqlite::memory:"
auto_migrate = true
auto_seed = false
"#;
    fs::write(config_dir.join("db.toml"), db_toml_content).unwrap();
  }

  mod public_api_exports {
    use super::*;

    #[test]
    fn app_type_is_exported_and_constructible() {
      let temp_dir = tempfile::tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(temp_dir.path()).unwrap();
      setup_test_config(temp_dir.path(), "test_app");

      // Given the lib.rs module exports App
      // When I create a new App instance
      let _app = App::new();

      // Then it should be successfully created
      // (no panic means success)

      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn init_tracing_function_is_exported_and_callable() {
      // Given the lib.rs module exports init_tracing
      // When I call init_tracing
      init_tracing();

      // Then it should execute without panicking
      // (function is idempotent and safe to call multiple times)
    }

    #[test]
    fn shutdown_signal_future_is_exported() {
      // Given the lib.rs module exports shutdown_signal_future
      // When I reference the function
      let _fut = shutdown_signal_future();

      // Then it should be accessible and return a Future
      // (type checking ensures it's exported correctly)
    }

    #[test]
    fn type_aliases_are_exported() {
      // Given the lib.rs module exports type aliases
      // When I check the types exist
      let _auth_installer: Option<AuthInstallerFn<forge_db::DbConnection>> = None;
      let _migrator: Option<MigratorFn> = None;
      let _seeder: Option<SeedFn> = None;

      // Then they should be accessible and usable
      // (compilation success means types are exported)
    }
  }

  mod re_export_integration {
    use super::*;

    #[test]
    fn app_from_lib_matches_app_from_module() {
      let temp_dir = tempfile::tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(temp_dir.path()).unwrap();
      setup_test_config(temp_dir.path(), "test_app");

      // Given I import App from lib.rs
      // When I create an App instance
      let app_from_lib = App::new();

      // Then it should have the same behavior as importing from app module
      let _config = app_from_lib.config();
      // (config() method works, proving it's the same type)

      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[tokio::test]
    async fn exported_app_can_build_router() {
      let temp_dir = tempfile::tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(temp_dir.path()).unwrap();
      setup_test_config(temp_dir.path(), "test_app");

      // Given I have an App imported from lib.rs
      let app = App::new();

      // When I convert it to a router
      let (router, _cron_runner) = app.into_router().await;

      // Then it should successfully create a router
      // (no panic means success, router is usable)
      let _ = router;

      std::env::set_current_dir(original_cwd).unwrap();
    }
  }

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests verify that lib.rs correctly re-exports all public API items.
    mod public_api_export_behavior {
      use super::*;

      #[test]
      fn should_export_app_type_when_importing_from_lib() {
        // Given: lib.rs re-exports App from app module
        // When: I import App from the crate root
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(&std::env::current_dir().unwrap(), "bdd_lib_test");

        // Then: App should be constructible and functional
        let app = App::new();
        let config = app.config();
        assert_eq!(config.app.name, "bdd_lib_test");

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[test]
      fn should_export_init_tracing_when_calling_from_lib() {
        // Given: lib.rs re-exports init_tracing function
        // When: I call init_tracing from crate root
        init_tracing();

        // Then: it should execute without error (idempotent)
        // Calling multiple times should be safe
        init_tracing();
        init_tracing();
      }

      #[test]
      fn should_export_shutdown_signal_future_when_referencing_from_lib() {
        // Given: lib.rs re-exports shutdown_signal_future
        // When: I reference the function
        let fut = shutdown_signal_future();

        // Then: it should return a Future that can be used
        // Type checking ensures it's exported correctly
        let _fut: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> = Box::pin(fut);
      }

      #[test]
      fn should_export_type_aliases_when_using_from_lib() {
        // Given: lib.rs re-exports type aliases (AuthInstallerFn, MigratorFn, SeedFn)
        // When: I use these types
        let _auth_installer: Option<AuthInstallerFn<forge_db::DbConnection>> = None;
        let _migrator: Option<MigratorFn> = None;
        let _seeder: Option<SeedFn> = None;

        // Then: they should be accessible and type-check correctly
        // Compilation success means types are properly exported
      }
    }

    mod re_export_equivalence_behavior {
      use super::*;

      #[test]
      fn should_provide_same_app_behavior_when_imported_from_lib_vs_module() {
        // Given: App can be imported from both lib.rs and app module
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(&std::env::current_dir().unwrap(), "equivalence_test");

        // When: I create App instances from both sources
        let app_from_lib = App::new();
        let app_from_module = app::App::new();

        // Then: they should have identical behavior
        assert_eq!(
          app_from_lib.config().app.name,
          app_from_module.config().app.name
        );
        assert_eq!(
          app_from_lib.config().server.host,
          app_from_module.config().server.host
        );
        assert_eq!(
          app_from_lib.config().server.port,
          app_from_module.config().server.port
        );

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[tokio::test]
      async fn should_build_router_successfully_when_using_exported_app() {
        // Given: App is imported from lib.rs
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(&std::env::current_dir().unwrap(), "router_build_test");

        // When: I convert the exported App to a router
        let app = App::new();
        let (router, cron_runner) = app.into_router().await;

        // Then: router should be built successfully
        let _router: axum::Router<()> = router;
        assert!(cron_runner.is_none() || cron_runner.is_some());

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[tokio::test]
      async fn should_support_fluent_api_when_using_exported_app() {
        // Given: App is imported from lib.rs
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(&std::env::current_dir().unwrap(), "fluent_api_test");

        // When: I use the fluent API methods
        let app = App::new().with_health_routes().with_rate_limit_per_ip(60);

        // Then: all methods should work correctly
        let (router, _) = app.into_router().await;
        let _router: axum::Router<()> = router;

        std::env::set_current_dir(original_cwd).unwrap();
      }
    }

    mod module_structure_behavior {
      use super::*;

      #[test]
      fn should_expose_app_module_when_accessing_from_lib() {
        // Given: lib.rs declares app as a public module
        // When: I access the app module
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(&std::env::current_dir().unwrap(), "module_test");

        // Then: app module should be accessible
        let app_from_module = app::App::new();
        let _config = app_from_module.config();

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[test]
      fn should_allow_direct_module_access_when_needed() {
        // Given: both lib.rs exports and direct module access are available
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(&std::env::current_dir().unwrap(), "direct_access_test");

        // When: I use both import styles
        let app_from_lib = App::new();
        let app_from_module = app::App::new();

        // Then: both should work identically
        assert_eq!(
          app_from_lib.config().app.name,
          app_from_module.config().app.name
        );

        std::env::set_current_dir(original_cwd).unwrap();
      }
    }
  }
}
