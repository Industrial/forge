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
