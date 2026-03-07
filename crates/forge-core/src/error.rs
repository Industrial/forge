//! Minimal error type for Forge core operations.
//!
//! No dependency on HTTP, database, or auth. The main `forge` crate
//! extends this with `Authz`, `Database`, and `Http` variants and `IntoResponse`.

use std::fmt;

/// Core error variants (I/O and generic message only).
#[derive(Debug)]
pub enum Error {
  /// I/O operation failed.
  Io(std::io::Error),
  /// Generic error with message.
  Generic(String),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::Io(err) => write!(f, "I/O error: {}", err),
      Error::Generic(msg) => write!(f, "{}", msg),
    }
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Error::Io(err) => Some(err),
      Error::Generic(_) => None,
    }
  }
}

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self {
    Error::Io(err)
  }
}

impl From<String> for Error {
  fn from(msg: String) -> Self {
    Error::Generic(msg)
  }
}

impl From<&str> for Error {
  fn from(msg: &str) -> Self {
    Error::Generic(msg.to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::error::Error as StdError;
  use std::io;

  fn assert_io(error: &Error, expected_kind: io::ErrorKind) {
    match error {
      Error::Io(e) => assert_eq!(e.kind(), expected_kind),
      Error::Generic(s) => panic!("expected Io variant, got Generic: {}", s),
    }
  }

  fn assert_generic(error: &Error, expected: &str) {
    match error {
      Error::Io(_) => panic!("expected Generic variant, got Io"),
      Error::Generic(s) => assert_eq!(s, expected),
    }
  }

  #[test]
  fn io_error_variant() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error = Error::Io(io_err);
    assert_io(&error, io::ErrorKind::NotFound);
  }

  #[test]
  #[should_panic(expected = "expected Io variant")]
  fn io_error_variant_panics_on_generic() {
    assert_io(
      &Error::Generic("wrong".to_string()),
      io::ErrorKind::NotFound,
    );
  }

  #[test]
  fn generic_error_variant() {
    let error = Error::Generic("custom".to_string());
    assert_generic(&error, "custom");
  }

  #[test]
  #[should_panic(expected = "expected Generic variant")]
  fn generic_error_variant_panics_on_io() {
    let error = Error::Io(io::Error::other("wrong"));
    assert_generic(&error, "custom");
  }

  #[test]
  fn from_io_error() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
    let error: Error = io_err.into();
    assert_io(&error, io::ErrorKind::PermissionDenied);
  }

  #[test]
  fn from_str() {
    let error: Error = "message".into();
    assert_generic(&error, "message");
  }

  #[test]
  fn from_string() {
    let error: Error = "owned".to_string().into();
    assert_generic(&error, "owned");
  }

  #[test]
  fn display_io() {
    let io_err = io::Error::other("disk full");
    let error = Error::Io(io_err);
    assert_eq!(error.to_string(), "I/O error: disk full");
  }

  #[test]
  fn display_generic() {
    let error = Error::Generic("something went wrong".to_string());
    assert_eq!(error.to_string(), "something went wrong");
  }

  #[test]
  fn source_io() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "missing");
    let error = Error::Io(io_err);
    let source = StdError::source(&error).expect("Io variant has source");
    assert!(source.to_string().contains("missing"));
  }

  #[test]
  fn source_generic_none() {
    let error = Error::Generic("no source".to_string());
    assert!(StdError::source(&error).is_none());
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use std::error::Error as StdError;
  use std::io;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.

  mod error_variant_behavior {
    use super::*;

    #[test]
    fn should_create_io_error_variant() {
      // Given: an I/O error
      let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");

      // When: creating Error::Io variant
      let error = Error::Io(io_err);

      // Then: should be Io variant
      match error {
        Error::Io(_) => assert!(true),
        Error::Generic(_) => panic!("Expected Io variant"),
      }
    }

    #[test]
    fn should_create_generic_error_variant() {
      // Given: a generic error message
      let message = "custom error".to_string();

      // When: creating Error::Generic variant
      let error = Error::Generic(message.clone());

      // Then: should be Generic variant with correct message
      match error {
        Error::Generic(msg) => assert_eq!(msg, message),
        Error::Io(_) => panic!("Expected Generic variant"),
      }
    }

    #[test]
    fn should_preserve_io_error_kind() {
      // Given: an I/O error with specific kind
      let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");

      // When: creating Error::Io variant
      let error = Error::Io(io_err);

      // Then: should preserve the error kind
      match error {
        Error::Io(e) => assert_eq!(e.kind(), io::ErrorKind::PermissionDenied),
        Error::Generic(_) => panic!("Expected Io variant"),
      }
    }
  }

  mod display_formatting_behavior {
    use super::*;

    #[test]
    fn should_format_io_error_with_prefix() {
      // Given: an I/O error
      let io_err = io::Error::other("disk full");
      let error = Error::Io(io_err);

      // When: formatting as string
      let formatted = format!("{}", error);

      // Then: should include "I/O error:" prefix
      assert!(formatted.contains("I/O error:"));
      assert!(formatted.contains("disk full"));
    }

    #[test]
    fn should_format_generic_error_directly() {
      // Given: a generic error
      let error = Error::Generic("something went wrong".to_string());

      // When: formatting as string
      let formatted = format!("{}", error);

      // Then: should return the message directly
      assert_eq!(formatted, "something went wrong");
    }

    #[test]
    fn should_format_different_io_error_kinds() {
      // Given: different I/O error kinds
      let errors = vec![
        (io::ErrorKind::NotFound, "not found"),
        (io::ErrorKind::PermissionDenied, "permission denied"),
        (io::ErrorKind::AlreadyExists, "already exists"),
      ];

      // When: formatting each as string
      for (kind, msg) in errors {
        let io_err = io::Error::new(kind, msg);
        let error = Error::Io(io_err);
        let formatted = format!("{}", error);

        // Then: should format correctly
        assert!(formatted.contains("I/O error:"));
        assert!(formatted.contains(msg));
      }
    }
  }

  mod error_trait_behavior {
    use super::*;

    #[test]
    fn should_implement_std_error_trait() {
      // Given: any Error variant
      let error: Box<dyn StdError> = Box::new(Error::Generic("test".to_string()));

      // When: using Error trait methods
      // Then: should work correctly (compilation test)
      let _description = error.to_string();
      assert!(true, "Error implements std::error::Error");
    }

    #[test]
    fn should_provide_source_for_io_errors() {
      // Given: an I/O error
      let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
      let error = Error::Io(io_err);

      // When: accessing error source
      let source = StdError::source(&error);

      // Then: should provide the underlying I/O error
      assert!(source.is_some());
      let source_err = source.unwrap();
      assert!(source_err.to_string().contains("file missing"));
    }

    #[test]
    fn should_not_provide_source_for_generic_errors() {
      // Given: a generic error
      let error = Error::Generic("no source available".to_string());

      // When: accessing error source
      let source = StdError::source(&error);

      // Then: should return None
      assert!(source.is_none());
    }
  }

  mod from_conversion_behavior {
    use super::*;

    #[test]
    fn should_convert_from_io_error() {
      // Given: a std::io::Error
      let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");

      // When: converting to Error
      let error: Error = io_err.into();

      // Then: should be Io variant
      match error {
        Error::Io(e) => assert_eq!(e.kind(), io::ErrorKind::PermissionDenied),
        Error::Generic(_) => panic!("Expected Io variant"),
      }
    }

    #[test]
    fn should_convert_from_string() {
      // Given: a String message
      let message = "error message".to_string();

      // When: converting to Error
      let error: Error = message.clone().into();

      // Then: should be Generic variant with message
      match error {
        Error::Generic(msg) => assert_eq!(msg, message),
        Error::Io(_) => panic!("Expected Generic variant"),
      }
    }

    #[test]
    fn should_convert_from_str() {
      // Given: a &str message
      let message = "string slice error";

      // When: converting to Error
      let error: Error = message.into();

      // Then: should be Generic variant with converted message
      match error {
        Error::Generic(msg) => assert_eq!(msg, message),
        Error::Io(_) => panic!("Expected Generic variant"),
      }
    }

    #[test]
    fn should_preserve_io_error_details_in_conversion() {
      // Given: an I/O error with specific details
      let io_err = io::Error::new(io::ErrorKind::NotFound, "specific file missing");

      // When: converting to Error and formatting
      let error: Error = io_err.into();
      let formatted = format!("{}", error);

      // Then: should preserve error details
      assert!(formatted.contains("specific file missing"));
    }
  }

  mod error_usage_behavior {
    use super::*;

    #[test]
    fn should_be_usable_in_result_type() {
      // Given: a function that can return Error
      fn may_fail() -> Result<(), Error> {
        Err(Error::Generic("failure".to_string()))
      }

      // When: calling the function
      let result = may_fail();

      // Then: should return Err with Error
      assert!(result.is_err());
      match result.unwrap_err() {
        Error::Generic(msg) => assert_eq!(msg, "failure"),
        Error::Io(_) => panic!("Expected Generic variant"),
      }
    }

    #[test]
    fn should_be_usable_with_question_mark_operator() {
      // Given: a function that propagates errors
      fn propagate_io() -> Result<(), Error> {
        let io_err = io::Error::new(io::ErrorKind::Other, "propagated");
        Err(io_err.into())
      }

      // When: calling the function
      let result = propagate_io();

      // Then: should propagate the error correctly
      assert!(result.is_err());
      match result.unwrap_err() {
        Error::Io(_) => assert!(true),
        Error::Generic(_) => panic!("Expected Io variant"),
      }
    }

    #[test]
    fn should_be_debuggable() {
      // Given: an Error variant
      let error = Error::Generic("debug test".to_string());

      // When: formatting for debug
      let debug_str = format!("{:?}", error);

      // Then: should produce debug output
      assert!(!debug_str.is_empty());
    }
  }
}
