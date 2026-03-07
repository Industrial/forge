//! Re-export model error from forge-entity.

pub use forge_entity::ModelError;

#[cfg(test)]
mod bdd_tests {
  use super::*;

  mod re_export_behavior {
    use super::*;

    #[test]
    fn should_re_export_model_error_from_forge_entity() {
      // Given: ModelError from this module
      // When: creating error variants
      // Then: all variants should be accessible
      let _unknown = ModelError::UnknownModel;
      let _validation = ModelError::Validation("test".to_string());
      let _not_found = ModelError::NotFound("test".to_string());
      let _database = ModelError::Database(sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal(
        "test".to_string(),
      )));
    }

    #[test]
    fn should_have_same_error_variants_as_forge_entity() {
      // Given: ModelError re-exported from forge-entity
      // When: matching on variants
      // Then: should match all expected variants
      match ModelError::UnknownModel {
        ModelError::UnknownModel => {}
        _ => panic!("Expected UnknownModel variant"),
      }
      match ModelError::Validation("test".to_string()) {
        ModelError::Validation(_) => {}
        _ => panic!("Expected Validation variant"),
      }
      match ModelError::NotFound("test".to_string()) {
        ModelError::NotFound(_) => {}
        _ => panic!("Expected NotFound variant"),
      }
    }
  }
}
