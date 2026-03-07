//! Application error type. Handlers return `Result<_, Error>`; all variants
//! map to HTTP status and body via [IntoResponse].

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt;

use forge_auth::AuthzError;

/// Application error type for API handlers.
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
  /// Authorization error (403 Forbidden or 404 Not Found)
  Authz(AuthzError),
  /// Auth/scope error with explicit status (401, 400, 404, 403, etc.)
  Auth(StatusCode, String),
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
        AuthzError::Forbidden => (StatusCode::FORBIDDEN, e.to_string()),
        AuthzError::NotFound => (StatusCode::NOT_FOUND, e.to_string()),
        AuthzError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
      },
      Error::Auth(status, msg) => (status, msg),
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
      Error::Auth(_, msg) => write!(f, "{}", msg),
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
      Error::Auth(_, _) => None,
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

impl From<AuthzError> for Error {
  fn from(err: AuthzError) -> Self {
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

impl From<db::model_error::ModelError> for Error {
  fn from(e: db::model_error::ModelError) -> Self {
    match e {
      db::model_error::ModelError::UnknownModel => {
        Error::Auth(StatusCode::NOT_FOUND, "Unknown model".to_string())
      }
      db::model_error::ModelError::Validation(msg) => Error::Generic(msg),
      db::model_error::ModelError::NotFound(msg) => Error::Auth(StatusCode::NOT_FOUND, msg),
      db::model_error::ModelError::Database(err) => Error::Database(err),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::body::to_bytes;
  use axum::http::StatusCode;

  // --- BDD Tests ---

  mod error_creation_behavior {
    use super::*;

    #[test]
    fn should_create_io_error_from_io_error() {
      // Given: an IO error
      let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");

      // When: creating Error from IO error
      let error = Error::from(io_err);

      // Then: error should be Error::Io variant
      match error {
        Error::Io(_) => {}
        _ => panic!("Expected Error::Io variant"),
      }
    }

    #[test]
    fn should_create_generic_error_from_string() {
      // Given: a string message
      let msg = String::from("Something went wrong");

      // When: creating Error from String
      let error = Error::from(msg);

      // Then: error should be Error::Generic variant
      match error {
        Error::Generic(ref m) => assert_eq!(m, "Something went wrong"),
        _ => panic!("Expected Error::Generic variant"),
      }
    }

    #[test]
    fn should_create_generic_error_from_str() {
      // Given: a string slice
      let msg = "Error occurred";

      // When: creating Error from &str
      let error = Error::from(msg);

      // Then: error should be Error::Generic variant
      match error {
        Error::Generic(ref m) => assert_eq!(m, "Error occurred"),
        _ => panic!("Expected Error::Generic variant"),
      }
    }

    #[test]
    fn should_create_authz_error_from_authz_error() {
      // Given: an AuthzError
      let authz_err = AuthzError::Forbidden;

      // When: creating Error from AuthzError
      let error = Error::from(authz_err);

      // Then: error should be Error::Authz variant
      match error {
        Error::Authz(_) => {}
        _ => panic!("Expected Error::Authz variant"),
      }
    }

    #[test]
    fn should_create_auth_error_with_custom_status() {
      // Given: a status code and message
      let status = StatusCode::UNAUTHORIZED;
      let msg = "Authentication required";

      // When: creating Auth error
      let error = Error::Auth(status, msg.to_string());

      // Then: error should have correct status and message
      match error {
        Error::Auth(s, m) => {
          assert_eq!(s, StatusCode::UNAUTHORIZED);
          assert_eq!(m, "Authentication required");
        }
        _ => panic!("Expected Error::Auth variant"),
      }
    }
  }

  mod into_response_behavior {
    use super::*;

    #[tokio::test]
    async fn should_return_500_for_io_error() {
      // Given: an IO error
      let error = Error::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "file not found",
      ));

      // When: converting to response
      let response = error.into_response();

      // Then: response should have 500 status and error message
      assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
      let body = to_bytes(response.into_body(), 1024).await.unwrap();
      let body_str = String::from_utf8(body.to_vec()).unwrap();
      assert!(body_str.contains("I/O error"));
    }

    #[tokio::test]
    async fn should_return_500_for_http_error() {
      // Given: an HTTP error (using a generic error as axum::Error is hard to construct)
      // Note: We'll test the Generic variant which also returns 500
      let error = Error::Generic("HTTP error occurred".to_string());

      // When: converting to response
      let response = error.into_response();

      // Then: response should have 500 status
      assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
      let body = to_bytes(response.into_body(), 1024).await.unwrap();
      let body_str = String::from_utf8(body.to_vec()).unwrap();
      assert_eq!(body_str, "HTTP error occurred");
    }

    #[tokio::test]
    async fn should_return_500_for_generic_error() {
      // Given: a generic error
      let error = Error::Generic("Something went wrong".to_string());

      // When: converting to response
      let response = error.into_response();

      // Then: response should have 500 status and message
      assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
      let body = to_bytes(response.into_body(), 1024).await.unwrap();
      let body_str = String::from_utf8(body.to_vec()).unwrap();
      assert_eq!(body_str, "Something went wrong");
    }

    #[tokio::test]
    async fn should_return_500_for_database_error() {
      // Given: a database error
      let db_err = sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal(
        "Connection failed".to_string(),
      ));
      let error = Error::Database(db_err);

      // When: converting to response
      let response = error.into_response();

      // Then: response should have 500 status
      assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
      let body = to_bytes(response.into_body(), 1024).await.unwrap();
      let body_str = String::from_utf8(body.to_vec()).unwrap();
      assert!(body_str.contains("Database error"));
    }

    #[tokio::test]
    async fn should_return_403_for_forbidden_authz_error() {
      // Given: a Forbidden AuthzError
      let error = Error::Authz(AuthzError::Forbidden);

      // When: converting to response
      let response = error.into_response();

      // Then: response should have 403 status
      assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn should_return_404_for_not_found_authz_error() {
      // Given: a NotFound AuthzError
      let error = Error::Authz(AuthzError::NotFound);

      // When: converting to response
      let response = error.into_response();

      // Then: response should have 404 status
      assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn should_return_custom_status_for_auth_error() {
      // Given: an Auth error with custom status
      let error = Error::Auth(StatusCode::BAD_REQUEST, "Invalid request".to_string());

      // When: converting to response
      let response = error.into_response();

      // Then: response should have custom status and message
      assert_eq!(response.status(), StatusCode::BAD_REQUEST);
      let body = to_bytes(response.into_body(), 1024).await.unwrap();
      let body_str = String::from_utf8(body.to_vec()).unwrap();
      assert_eq!(body_str, "Invalid request");
    }

    #[tokio::test]
    async fn should_return_401_for_unauthorized_auth_error() {
      // Given: an Auth error with 401 status
      let error = Error::Auth(StatusCode::UNAUTHORIZED, "Unauthorized".to_string());

      // When: converting to response
      let response = error.into_response();

      // Then: response should have 401 status
      assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
  }

  mod display_behavior {
    use super::*;

    #[test]
    fn should_format_io_error_correctly() {
      // Given: an IO error
      let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
      let error = Error::Io(io_err);

      // When: formatting as string
      let display = format!("{}", error);

      // Then: should include "I/O error" prefix
      assert!(display.contains("I/O error"));
    }

    #[test]
    fn should_format_generic_error_correctly() {
      // Given: a generic error
      let error = Error::Generic("Custom error message".to_string());

      // When: formatting as string
      let display = format!("{}", error);

      // Then: should display the message directly
      assert_eq!(display, "Custom error message");
    }

    #[test]
    fn should_format_database_error_correctly() {
      // Given: a database error
      let db_err = sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal(
        "Connection failed".to_string(),
      ));
      let error = Error::Database(db_err);

      // When: formatting as string
      let display = format!("{}", error);

      // Then: should include "Database error" prefix
      assert!(display.contains("Database error"));
    }

    #[test]
    fn should_format_authz_error_correctly() {
      // Given: an AuthzError
      let error = Error::Authz(AuthzError::Forbidden);

      // When: formatting as string
      let display = format!("{}", error);

      // Then: should format using AuthzError's Display implementation
      assert!(!display.is_empty());
    }

    #[test]
    fn should_format_auth_error_correctly() {
      // Given: an Auth error
      let error = Error::Auth(StatusCode::UNAUTHORIZED, "Unauthorized access".to_string());

      // When: formatting as string
      let display = format!("{}", error);

      // Then: should display the message
      assert_eq!(display, "Unauthorized access");
    }
  }

  mod error_trait_behavior {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn should_provide_source_for_io_error() {
      // Given: an IO error
      let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
      let error = Error::Io(io_err);

      // When: getting source
      let source = StdError::source(&error);

      // Then: source should be Some
      assert!(source.is_some());
    }

    #[test]
    fn should_provide_source_for_database_error() {
      // Given: a database error
      let db_err = sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal(
        "Connection failed".to_string(),
      ));
      let error = Error::Database(db_err);

      // When: getting source
      let source = StdError::source(&error);

      // Then: source should be Some
      assert!(source.is_some());
    }

    #[test]
    fn should_provide_source_for_authz_error() {
      // Given: an AuthzError
      let error = Error::Authz(AuthzError::Forbidden);

      // When: getting source
      let source = StdError::source(&error);

      // Then: source should be Some
      assert!(source.is_some());
    }

    #[test]
    fn should_not_provide_source_for_generic_error() {
      // Given: a generic error
      let error = Error::Generic("Error message".to_string());

      // When: getting source
      let source = StdError::source(&error);

      // Then: source should be None
      assert!(source.is_none());
    }

    #[test]
    fn should_not_provide_source_for_auth_error() {
      // Given: an Auth error
      let error = Error::Auth(StatusCode::UNAUTHORIZED, "Unauthorized".to_string());

      // When: getting source
      let source = StdError::source(&error);

      // Then: source should be None
      assert!(source.is_none());
    }
  }

  mod from_trait_behavior {
    use super::*;

    #[test]
    fn should_convert_from_sea_orm_db_err() {
      // Given: a sea_orm::DbErr
      let db_err = sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal(
        "Connection error".to_string(),
      ));

      // When: converting to Error
      let error: Error = db_err.into();

      // Then: should be Error::Database variant
      match error {
        Error::Database(_) => {}
        _ => panic!("Expected Error::Database variant"),
      }
    }

    #[test]
    fn should_convert_from_std_io_error() {
      // Given: a std::io::Error
      let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");

      // When: converting to Error
      let error: Error = io_err.into();

      // Then: should be Error::Io variant
      match error {
        Error::Io(_) => {}
        _ => panic!("Expected Error::Io variant"),
      }
    }

    #[test]
    fn should_convert_from_authz_error() {
      // Given: an AuthzError
      let authz_err = AuthzError::NotFound;

      // When: converting to Error
      let error: Error = authz_err.into();

      // Then: should be Error::Authz variant
      match error {
        Error::Authz(_) => {}
        _ => panic!("Expected Error::Authz variant"),
      }
    }

    #[test]
    fn should_convert_from_string() {
      // Given: a String
      let msg = String::from("Error message");

      // When: converting to Error
      let error: Error = msg.into();

      // Then: should be Error::Generic variant
      match error {
        Error::Generic(ref m) => assert_eq!(m, "Error message"),
        _ => panic!("Expected Error::Generic variant"),
      }
    }

    #[test]
    fn should_convert_from_str() {
      // Given: a &str
      let msg = "Error message";

      // When: converting to Error
      let error: Error = msg.into();

      // Then: should be Error::Generic variant
      match error {
        Error::Generic(ref m) => assert_eq!(m, "Error message"),
        _ => panic!("Expected Error::Generic variant"),
      }
    }
  }
}
