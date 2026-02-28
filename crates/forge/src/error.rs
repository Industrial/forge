//! Error types for the Forge framework.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt;

/// Custom error type for Forge operations.
#[derive(Debug)]
pub enum Error {
  /// I/O operation failed
  Io(std::io::Error),
  /// HTTP server error
  Http(axum::Error),
  /// Generic error with message
  Generic(String),
  /// Database error
  Database(sea_orm::DbErr),
}

impl IntoResponse for Error {
  fn into_response(self) -> Response {
    let (status, message) = match self {
      Error::Io(err) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("I/O error: {}", err),
      ),
      Error::Http(err) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("HTTP error: {}", err),
      ),
      Error::Generic(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
      Error::Database(err) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {}", err),
      ),
    };

    (status, message).into_response()
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::Io(err) => write!(f, "I/O error: {}", err),
      Error::Http(err) => write!(f, "HTTP error: {}", err),
      Error::Generic(msg) => write!(f, "{}", msg),
      Error::Database(err) => write!(f, "Database error: {}", err),
    }
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Error::Io(err) => Some(err),
      Error::Http(err) => Some(err),
      Error::Generic(_) => None,
      Error::Database(err) => Some(err),
    }
  }
}

impl From<sea_orm::DbErr> for Error {
  fn from(err: sea_orm::DbErr) -> Self {
    Error::Database(err)
  }
}

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self {
    Error::Io(err)
  }
}

impl From<axum::Error> for Error {
  fn from(err: axum::Error) -> Self {
    Error::Http(err)
  }
}

impl From<crate::authz::AuthzError> for Error {
  fn from(err: crate::authz::AuthzError) -> Self {
    Error::Generic(err.to_string())
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
  use std::io;

  /// Test suite for Error enum variants
  mod error_variants {
    use super::*;

    #[test]
    fn io_error_variant_stores_io_error() {
      // Given: An IO error
      let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");

      // When: Creating Error::Io variant
      let error = Error::Io(io_err);

      // Then: It should store the IO error
      match error {
        Error::Io(stored_err) => {
          assert_eq!(stored_err.kind(), io::ErrorKind::NotFound);
        }
        _ => panic!("Expected Io variant"),
      }
    }

    #[test]
    fn http_error_variant_stores_axum_error() {
      // Given: An axum error (simulated)
      let http_err = axum::Error::new(io::Error::new(io::ErrorKind::InvalidData, "bad request"));

      // When: Creating Error::Http variant
      let error = Error::Http(http_err);

      // Then: It should store the HTTP error
      match error {
        Error::Http(_) => {
          // Error type confirmed
        }
        _ => panic!("Expected Http variant"),
      }
    }

    #[test]
    fn generic_error_variant_stores_string_message() {
      // Given: A string message
      let message = "custom error message";

      // When: Creating Error::Generic variant
      let error = Error::Generic(message.to_string());

      // Then: It should store the message
      match error {
        Error::Generic(stored_msg) => {
          assert_eq!(stored_msg, message);
        }
        _ => panic!("Expected Generic variant"),
      }
    }
  }

  /// Test suite for Display implementation
  mod display_implementation {
    use super::*;

    #[test]
    fn io_error_displays_with_prefix() {
      // Given: An IO error
      let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
      let error = Error::Io(io_err);

      // When: Formatting for display
      let display_str = format!("{}", error);

      // Then: It should include "I/O error:" prefix
      assert!(display_str.starts_with("I/O error:"));
      assert!(display_str.contains("file not found"));
    }

    #[test]
    fn http_error_displays_with_prefix() {
      // Given: An HTTP error
      let http_err = axum::Error::new(io::Error::new(io::ErrorKind::InvalidData, "bad request"));
      let error = Error::Http(http_err);

      // When: Formatting for display
      let display_str = format!("{}", error);

      // Then: It should include "HTTP error:" prefix
      assert!(display_str.starts_with("HTTP error:"));
    }

    #[test]
    fn generic_error_displays_message_directly() {
      // Given: A generic error with message
      let message = "something went wrong";
      let error = Error::Generic(message.to_string());

      // When: Formatting for display
      let display_str = format!("{}", error);

      // Then: It should display the message directly
      assert_eq!(display_str, message);
    }
  }

  /// Test suite for Error trait implementation
  mod error_trait {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn error_implements_std_error_trait() {
      // Given: Any Error variant
      let error = Error::Generic("test".to_string());

      // When: Using as std::error::Error
      let std_error: &dyn StdError = &error;

      // Then: It should work (compilation test)
      let _ = std_error;
    }

    #[test]
    fn error_provides_source_for_io_errors() {
      // Given: An IO error
      let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
      let error = Error::Io(io_err);

      // When: Getting source
      let source = error.source();

      // Then: It should return the underlying IO error
      assert!(source.is_some());
    }

    #[test]
    fn generic_error_has_no_source() {
      // Given: A generic error
      let error = Error::Generic("message".to_string());

      // When: Getting source
      let source = error.source();

      // Then: It should return None
      assert!(source.is_none());
    }
  }

  /// Test suite for From trait implementations
  mod from_trait_implementations {
    use super::*;

    #[test]
    fn from_io_error_creates_io_variant() {
      // Given: An std::io::Error
      let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");

      // When: Converting via From trait
      let error: Error = io_err.into();

      // Then: It should create Io variant
      match error {
        Error::Io(_) => {
          // Correct variant
        }
        _ => panic!("Expected Io variant"),
      }
    }

    #[test]
    fn from_axum_error_creates_http_variant() {
      // Given: An axum::Error
      let axum_err = axum::Error::new(io::Error::new(io::ErrorKind::InvalidData, "bad data"));

      // When: Converting via From trait
      let error: Error = axum_err.into();

      // Then: It should create Http variant
      match error {
        Error::Http(_) => {
          // Correct variant
        }
        _ => panic!("Expected Http variant"),
      }
    }

    #[test]
    fn from_string_creates_generic_variant() {
      // Given: A String
      let message = String::from("error message");

      // When: Converting via From trait
      let error: Error = message.into();

      // Then: It should create Generic variant
      match error {
        Error::Generic(msg) => {
          assert_eq!(msg, "error message");
        }
        _ => panic!("Expected Generic variant"),
      }
    }

    #[test]
    fn from_str_creates_generic_variant() {
      // Given: A &str
      let message = "error message";

      // When: Converting via From trait
      let error: Error = message.into();

      // Then: It should create Generic variant
      match error {
        Error::Generic(msg) => {
          assert_eq!(msg, "error message");
        }
        _ => panic!("Expected Generic variant"),
      }
    }
  }

  /// Test suite for Debug implementation (automatically derived)
  mod debug_implementation {
    use super::*;

    #[test]
    fn error_implements_debug_trait() {
      // Given: Any Error variant
      let error = Error::Generic("debug test".to_string());

      // When: Using Debug formatting
      let debug_str = format!("{:?}", error);

      // Then: It should produce debug output
      assert!(debug_str.contains("Generic"));
      assert!(debug_str.contains("debug test"));
    }
  }
}
