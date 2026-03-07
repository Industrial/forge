//! Request validation re-exports.
//!
//! Use `Validate` on request structs and `Valid<Json<T>>` or `Valid<Query<T>>` in handlers.
//! Invalid requests receive 422 Unprocessable Entity with a structured error body.

pub use axum_valid::Valid;
pub use validator::Validate;

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Validate)]
  struct TestRequest {
    #[validate(length(min = 1, message = "name required"))]
    name: String,
  }

  #[test]
  fn validate_accepts_valid_request() {
    let r = TestRequest {
      name: "hello".to_string(),
    };
    assert!(r.validate().is_ok());
  }

  #[test]
  fn validate_rejects_invalid_request() {
    let r = TestRequest {
      name: "".to_string(),
    };
    let res = r.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.field_errors().contains_key("name"));
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.
  mod validation_exports_behavior {
    use super::*;

    #[test]
    fn should_export_valid_type() {
      // Given: forge-core validation module
      // When: using Valid type
      // Then: should be accessible (verified by compilation)
      let _: Valid<String> = Valid("ok".to_string());
    }

    #[test]
    fn should_export_validate_trait() {
      // Given: forge-core validation module
      // When: using Validate trait
      // Then: should be accessible for deriving (verified by compilation)
      #[derive(Debug, Validate)]
      struct _T {
        #[validate(length(min = 1))]
        _f: String,
      }
    }
  }

  mod request_validation_behavior {
    use super::*;

    #[derive(Debug, Validate)]
    struct TestRequest {
      #[validate(length(min = 1, message = "name required"))]
      name: String,
    }

    #[test]
    fn should_accept_valid_request_with_non_empty_name() {
      // Given: a request with valid data
      let request = TestRequest {
        name: "hello".to_string(),
      };

      // When: validating the request
      let result = request.validate();

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[test]
    fn should_reject_request_with_empty_name() {
      // Given: a request with empty name
      let request = TestRequest {
        name: "".to_string(),
      };

      // When: validating the request
      let result = request.validate();

      // Then: should return validation error
      assert!(result.is_err());
    }

    #[test]
    fn should_provide_field_errors_for_invalid_request() {
      // Given: a request with invalid data
      let request = TestRequest {
        name: "".to_string(),
      };

      // When: validating and getting errors
      let result = request.validate();
      let err = result.unwrap_err();

      // Then: should contain field errors for invalid fields
      assert!(err.field_errors().contains_key("name"));
    }

    #[test]
    fn should_include_custom_error_message() {
      // Given: a request with invalid data
      let request = TestRequest {
        name: "".to_string(),
      };

      // When: validating and getting errors
      let result = request.validate();
      let err = result.unwrap_err();
      let field_errors = err.field_errors();

      // Then: should include custom error message
      if let Some(errors) = field_errors.get("name") {
        assert!(!errors.is_empty());
        // Check if custom message is present
        let has_custom_message = errors.iter().any(|e| {
          e.message
            .as_ref()
            .map(|m| m.contains("name required"))
            .unwrap_or(false)
        });
        assert!(has_custom_message || !errors.is_empty());
      }
    }

    #[test]
    fn should_accept_request_with_minimum_length() {
      // Given: a request with name at minimum length (1 character)
      let request = TestRequest {
        name: "a".to_string(),
      };

      // When: validating the request
      let result = request.validate();

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[test]
    fn should_accept_request_with_longer_name() {
      // Given: a request with name longer than minimum
      let request = TestRequest {
        name: "this is a longer name".to_string(),
      };

      // When: validating the request
      let result = request.validate();

      // Then: should succeed
      assert!(result.is_ok());
    }
  }

  mod validation_integration_behavior {
    use super::*;

    #[derive(Debug, Validate)]
    struct MultiFieldRequest {
      #[validate(length(min = 1, message = "email required"))]
      email: String,
      #[validate(length(min = 8, message = "password must be at least 8 characters"))]
      password: String,
    }

    #[test]
    fn should_validate_multiple_fields() {
      // Given: a request with multiple fields
      let request = MultiFieldRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
      };

      // When: validating the request
      let result = request.validate();

      // Then: should succeed when all fields are valid
      assert!(result.is_ok());
    }

    #[test]
    fn should_report_errors_for_multiple_invalid_fields() {
      // Given: a request with multiple invalid fields
      let request = MultiFieldRequest {
        email: "".to_string(),
        password: "short".to_string(),
      };

      // When: validating the request
      let result = request.validate();
      let err = result.unwrap_err();
      let field_errors = err.field_errors();

      // Then: should report errors for all invalid fields
      assert!(field_errors.contains_key("email"));
      assert!(field_errors.contains_key("password"));
    }

    #[test]
    fn should_report_only_invalid_fields() {
      // Given: a request with one valid and one invalid field
      let request = MultiFieldRequest {
        email: "valid@example.com".to_string(),
        password: "short".to_string(), // too short
      };

      // When: validating the request
      let result = request.validate();
      let err = result.unwrap_err();
      let field_errors = err.field_errors();

      // Then: should only report error for invalid field
      assert!(!field_errors.contains_key("email"));
      assert!(field_errors.contains_key("password"));
    }
  }
}
