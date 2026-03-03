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
    let error = Error::Io(io::Error::new(io::ErrorKind::Other, "wrong"));
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
    let io_err = io::Error::new(io::ErrorKind::Other, "disk full");
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
