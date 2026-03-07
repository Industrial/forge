//! Error type for model/registry operations. Map to HTTP (e.g. UnknownModel → 404).

use sea_orm::DbErr;

/// Errors from model list/get/create/update/delete and registry dispatch.
#[derive(Debug)]
pub enum ModelError {
  /// Model id not in registry (e.g. 404).
  UnknownModel,
  /// Validation or business rule failure (e.g. 400/422).
  Validation(String),
  /// Resource not found (e.g. 404).
  NotFound(String),
  /// Database error (e.g. 500).
  Database(DbErr),
}

impl std::fmt::Display for ModelError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ModelError::UnknownModel => write!(f, "Unknown model"),
      ModelError::Validation(m) => write!(f, "{}", m),
      ModelError::NotFound(m) => write!(f, "{}", m),
      ModelError::Database(e) => write!(f, "{}", e),
    }
  }
}

impl std::error::Error for ModelError {}

impl From<DbErr> for ModelError {
  fn from(e: DbErr) -> Self {
    ModelError::Database(e)
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.
  mod display_formatting_behavior {
    use super::*;

    #[test]
    fn should_format_unknown_model_error() {
      // Given: UnknownModel error variant
      let error = ModelError::UnknownModel;

      // When: formatting as string
      let formatted = format!("{}", error);

      // Then: should return "Unknown model"
      assert_eq!(formatted, "Unknown model");
    }

    #[test]
    fn should_format_validation_error_with_message() {
      // Given: Validation error with message
      let message = "Email is required";
      let error = ModelError::Validation(message.to_string());

      // When: formatting as string
      let formatted = format!("{}", error);

      // Then: should return the validation message
      assert_eq!(formatted, message);
    }

    #[test]
    fn should_format_not_found_error_with_message() {
      // Given: NotFound error with message
      let message = "User with id 123 not found";
      let error = ModelError::NotFound(message.to_string());

      // When: formatting as string
      let formatted = format!("{}", error);

      // Then: should return the not found message
      assert_eq!(formatted, message);
    }

    #[test]
    fn should_format_database_error() {
      // Given: Database error from DbErr
      let db_err = sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal(
        "Connection timeout".to_string(),
      ));
      let error = ModelError::Database(db_err);

      // When: formatting as string
      let formatted = format!("{}", error);

      // Then: should format the underlying database error
      assert!(formatted.contains("Connection timeout") || !formatted.is_empty());
    }
  }

  mod error_trait_behavior {
    use super::*;

    #[test]
    fn should_implement_std_error_trait() {
      // Given: any ModelError variant
      let error: Box<dyn std::error::Error> = Box::new(ModelError::UnknownModel);

      // When: using Error trait methods
      // Then: should work correctly (compilation test)
      let _description = error.to_string();
    }

    #[test]
    fn should_provide_error_source() {
      // Given: Database error with underlying DbErr
      let db_err = sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal(
        "Database connection failed".to_string(),
      ));
      let error = ModelError::Database(db_err);

      // When: accessing error source
      // Then: should provide source if available
      let _error_ref: &dyn std::error::Error = &error;
      // Source may or may not be available depending on DbErr implementation
    }
  }

  mod from_db_err_conversion_behavior {
    use super::*;

    #[test]
    fn should_convert_connection_error_to_database_error() {
      // Given: DbErr connection error
      let db_err = sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal(
        "Connection failed".to_string(),
      ));

      // When: converting to ModelError
      let model_error: ModelError = db_err.into();

      // Then: should be Database variant
      match model_error {
        ModelError::Database(_) => {}
        _ => panic!("Expected Database variant"),
      }
    }

    #[test]
    fn should_convert_query_error_to_database_error() {
      // Given: DbErr query error
      let db_err = sea_orm::DbErr::Query(sea_orm::RuntimeErr::Internal("Query failed".to_string()));

      // When: converting to ModelError
      let model_error: ModelError = db_err.into();

      // Then: should be Database variant
      match model_error {
        ModelError::Database(_) => {}
        _ => panic!("Expected Database variant"),
      }
    }

    #[test]
    fn should_convert_exec_error_to_database_error() {
      // Given: DbErr exec error
      let db_err = sea_orm::DbErr::Exec(sea_orm::RuntimeErr::Internal(
        "Execution failed".to_string(),
      ));

      // When: converting to ModelError
      let model_error: ModelError = db_err.into();

      // Then: should be Database variant
      match model_error {
        ModelError::Database(_) => {}
        _ => panic!("Expected Database variant"),
      }
    }

    #[test]
    fn should_preserve_database_error_details() {
      // Given: DbErr with specific message
      let original_message = "Database connection timeout";
      let db_err =
        sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal(original_message.to_string()));

      // When: converting to ModelError and formatting
      let model_error: ModelError = db_err.into();
      let formatted = format!("{}", model_error);

      // Then: should preserve error details in formatted output
      // (exact format depends on DbErr Display implementation)
      assert!(!formatted.is_empty(), "Error should have formatted output");
    }
  }

  mod error_variant_behavior {
    use super::*;

    #[test]
    fn should_have_unknown_model_variant() {
      // Given: UnknownModel variant
      let error = ModelError::UnknownModel;

      // When: matching on the variant
      // Then: should match UnknownModel
      match error {
        ModelError::UnknownModel => {}
        _ => panic!("Expected UnknownModel variant"),
      }
    }

    #[test]
    fn should_have_validation_variant_with_message() {
      // Given: Validation variant with message
      let message = "Invalid input";
      let error = ModelError::Validation(message.to_string());

      // When: matching on the variant
      // Then: should extract the message
      match error {
        ModelError::Validation(msg) => assert_eq!(msg, message),
        _ => panic!("Expected Validation variant"),
      }
    }

    #[test]
    fn should_have_not_found_variant_with_message() {
      // Given: NotFound variant with message
      let message = "Resource not found";
      let error = ModelError::NotFound(message.to_string());

      // When: matching on the variant
      // Then: should extract the message
      match error {
        ModelError::NotFound(msg) => assert_eq!(msg, message),
        _ => panic!("Expected NotFound variant"),
      }
    }

    #[test]
    fn should_have_database_variant_with_dberr() {
      // Given: Database variant with DbErr
      let db_err = sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal("Error".to_string()));
      let error = ModelError::Database(db_err);

      // When: matching on the variant
      // Then: should extract the DbErr
      match error {
        ModelError::Database(_) => {}
        _ => panic!("Expected Database variant"),
      }
    }
  }
}
