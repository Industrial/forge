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
  use std::io;

  #[test]
  fn io_error_variant() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error = Error::Io(io_err);
    match error {
      Error::Io(e) => assert_eq!(e.kind(), io::ErrorKind::NotFound),
      _ => panic!("expected Io variant"),
    }
  }

  #[test]
  fn generic_error_variant() {
    let error = Error::Generic("custom".to_string());
    match error {
      Error::Generic(s) => assert_eq!(s, "custom"),
      _ => panic!("expected Generic variant"),
    }
  }

  #[test]
  fn from_io_error() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
    let error: Error = io_err.into();
    match error {
      Error::Io(_) => {}
      _ => panic!("expected Io"),
    }
  }

  #[test]
  fn from_str() {
    let error: Error = "message".into();
    match error {
      Error::Generic(s) => assert_eq!(s, "message"),
      _ => panic!("expected Generic"),
    }
  }
}
