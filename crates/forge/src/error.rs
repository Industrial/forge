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
  /// Authorization error (maps to 403 Forbidden or 404 Not Found)
  Authz(crate::authz::AuthzError),
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
      Error::Authz(ref e) => match e {
        crate::authz::AuthzError::Forbidden => (StatusCode::FORBIDDEN, e.to_string()),
        crate::authz::AuthzError::NotFound => (StatusCode::NOT_FOUND, e.to_string()),
        crate::authz::AuthzError::DatabaseError(_) => {
          (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
      },
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
      Error::Authz(err) => write!(f, "{}", err),
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
      Error::Authz(err) => Some(err),
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
    Error::Authz(err)
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

impl From<forge_core::error::Error> for Error {
  fn from(err: forge_core::error::Error) -> Self {
    match err {
      forge_core::error::Error::Io(e) => Error::Io(e),
      forge_core::error::Error::Generic(s) => Error::Generic(s),
    }
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

  /// Test suite for IntoResponse (all branches: status and body)
  mod into_response {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn io_error_returns_500_and_message() {
      let err = Error::Io(io::Error::new(io::ErrorKind::NotFound, "file not found"));
      let res = err.into_response();
      assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
      let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
      let s = String::from_utf8_lossy(&body);
      assert!(s.contains("I/O error:"));
    }

    #[tokio::test]
    async fn http_error_returns_500_and_message() {
      let err = Error::Http(axum::Error::new(io::Error::new(
        io::ErrorKind::InvalidData,
        "bad request",
      )));
      let res = err.into_response();
      assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
      let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
      let s = String::from_utf8_lossy(&body);
      assert!(s.contains("HTTP error:"));
    }

    #[tokio::test]
    async fn generic_error_returns_500_and_message() {
      let err = Error::Generic("something went wrong".to_string());
      let res = err.into_response();
      assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
      let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
      let s = String::from_utf8_lossy(&body);
      assert_eq!(s, "something went wrong");
    }

    #[tokio::test]
    async fn database_error_returns_500_and_message() {
      let err = Error::Database(sea_orm::DbErr::Custom("connection failed".into()));
      let res = err.into_response();
      assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
      let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
      let s = String::from_utf8_lossy(&body);
      assert!(s.contains("Database error:"));
    }

    #[tokio::test]
    async fn authz_forbidden_returns_403() {
      let err = Error::Authz(crate::authz::AuthzError::Forbidden);
      let res = err.into_response();
      assert_eq!(res.status(), StatusCode::FORBIDDEN);
      let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
      let s = String::from_utf8_lossy(&body);
      assert!(!s.is_empty());
    }

    #[tokio::test]
    async fn authz_not_found_returns_404() {
      let err = Error::Authz(crate::authz::AuthzError::NotFound);
      let res = err.into_response();
      assert_eq!(res.status(), StatusCode::NOT_FOUND);
      let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
      let s = String::from_utf8_lossy(&body);
      assert!(!s.is_empty());
    }

    #[tokio::test]
    async fn authz_database_error_returns_500() {
      let err = Error::Authz(crate::authz::AuthzError::DatabaseError(
        sea_orm::DbErr::Custom("db err".into()),
      ));
      let res = err.into_response();
      assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
      let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
      let s = String::from_utf8_lossy(&body);
      assert!(!s.is_empty());
    }
  }

  /// From<DbErr> and From<AuthzError>
  mod from_db_and_authz {
    use super::*;

    #[test]
    fn from_db_err_creates_database_variant() {
      let db_err = sea_orm::DbErr::Custom("test".into());
      let error: Error = db_err.into();
      match error {
        Error::Database(_) => {}
        _ => panic!("Expected Database variant"),
      }
    }

    #[test]
    fn from_authz_error_creates_authz_variant() {
      let authz_err = crate::authz::AuthzError::Forbidden;
      let error: Error = authz_err.into();
      match error {
        Error::Authz(_) => {}
        _ => panic!("Expected Authz variant"),
      }
    }
  }

  /// Display for Database and Authz
  mod display_database_authz {
    use super::*;

    #[test]
    fn database_error_displays_with_prefix() {
      let err = Error::Database(sea_orm::DbErr::Custom("conn failed".into()));
      let s = format!("{}", err);
      assert!(s.starts_with("Database error:"));
      assert!(s.contains("conn failed"));
    }

    #[test]
    fn authz_error_displays_delegate_message() {
      let err = Error::Authz(crate::authz::AuthzError::Forbidden);
      let s = format!("{}", err);
      assert!(!s.is_empty());
    }
  }

  /// source() for Http, Database, Authz
  mod source_http_database_authz {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn http_error_has_source() {
      let err = Error::Http(axum::Error::new(io::Error::new(
        io::ErrorKind::InvalidData,
        "x",
      )));
      assert!(err.source().is_some());
    }

    #[test]
    fn database_error_has_source() {
      let err = Error::Database(sea_orm::DbErr::Custom("x".into()));
      assert!(err.source().is_some());
    }

    #[test]
    fn authz_error_has_source_for_database_error() {
      let err = Error::Authz(crate::authz::AuthzError::DatabaseError(
        sea_orm::DbErr::Custom("x".into()),
      ));
      assert!(err.source().is_some());
    }
  }
}
