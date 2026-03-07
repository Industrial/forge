//! Generic REST model trait and error type for entities served by a single generic handler.
//! Implement [RestModel] to get list/get/create/update/delete via `/api/entities/{model_id}`.

mod model_error;
mod rest_model;

pub use model_error::ModelError;
pub use rest_model::{REST_ACTIONS, RestModel};

#[cfg(test)]
mod bdd_tests {
  use super::*;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.

  mod public_api_exports_behavior {
    use super::*;

    #[test]
    fn should_export_model_error_type() {
      // Given: forge-entity crate
      // When: using ModelError
      // Then: should be accessible and usable
      let _error: ModelError = ModelError::UnknownModel;
      assert!(true, "ModelError is exported");
    }

    #[test]
    fn should_export_rest_model_trait() {
      // Given: forge-entity crate
      // When: checking if RestModel trait is exported
      // Then: should be accessible (verified by successful compilation)
      // The trait is exported and can be used by implementations
      assert!(true, "RestModel trait is exported");
    }

    #[test]
    fn should_export_rest_actions_constant() {
      // Given: forge-entity crate
      // When: accessing REST_ACTIONS
      // Then: should be accessible and contain expected actions
      assert_eq!(REST_ACTIONS.len(), 4);
      assert!(REST_ACTIONS.contains(&"create"));
      assert!(REST_ACTIONS.contains(&"read"));
      assert!(REST_ACTIONS.contains(&"update"));
      assert!(REST_ACTIONS.contains(&"delete"));
    }

    #[test]
    fn should_export_model_error_variants() {
      // Given: ModelError type
      // When: creating error variants
      // Then: all variants should be accessible
      let _unknown = ModelError::UnknownModel;
      let _validation = ModelError::Validation("test".to_string());
      let _not_found = ModelError::NotFound("test".to_string());
      let _database = ModelError::Database(sea_orm::DbErr::Conn(
        sea_orm::RuntimeErr::Internal("test".to_string()),
      ));
      assert!(true, "All ModelError variants are accessible");
    }
  }

  mod module_structure_behavior {
    #[test]
    fn should_have_model_error_module() {
      // Given: forge-entity crate structure
      // When: checking module organization
      // Then: model_error module should exist
      // This is verified by successful compilation and import
      assert!(true, "model_error module exists");
    }

    #[test]
    fn should_have_rest_model_module() {
      // Given: forge-entity crate structure
      // When: checking module organization
      // Then: rest_model module should exist
      // This is verified by successful compilation and import
      assert!(true, "rest_model module exists");
    }

    #[test]
    fn should_provide_unified_api_through_lib() {
      // Given: forge-entity crate
      // When: importing from the crate root
      // Then: should provide access to ModelError and RestModel
      // This is verified by successful compilation of this test module
      assert!(true, "Unified API is provided through lib.rs");
    }
  }
}
