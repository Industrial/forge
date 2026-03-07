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
