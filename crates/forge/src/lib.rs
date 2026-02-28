//! Forge — a full-stack web framework for Rust.
//!
//! Core library; CLI lives in the `forge-cli` crate.

pub mod app;
pub mod error;

pub use app::App;
pub use error::Error;

#[cfg(test)]
mod tests {
  use super::*;

  /// Test suite for Forge library public API
  mod public_api {
    use super::*;

    #[test]
    fn app_type_is_publicly_accessible() {
      // Given: Forge library is imported
      // When: Accessing the App type
      // Then: It should be available without compilation errors
      let _app: App;
    }

    #[test]
    fn error_type_is_publicly_accessible() {
      // Given: Forge library is imported
      // When: Accessing the Error type
      // Then: It should be available without compilation errors
      let _error: Error;
    }

    #[test]
    fn app_can_be_created_via_public_api() {
      // Given: Access to public App API
      // When: Creating a new App instance
      // Then: It should succeed without errors
      let _app = App::new();
    }

    #[test]
    fn app_default_implementation_available() {
      // Given: Access to public App API
      // When: Using Default trait
      // Then: It should work
      let _app = App::default();
    }
  }

  /// Test suite for module structure
  mod module_structure {
    #[test]
    fn app_module_exists() {
      // Given: Forge library structure
      // When: Checking for app module
      // Then: Module should be accessible
      // Note: This is tested by successful compilation
    }

    #[test]
    fn error_module_exists() {
      // Given: Forge library structure
      // When: Checking for error module
      // Then: Module should be accessible
      // Note: This is tested by successful compilation
    }
  }
}
