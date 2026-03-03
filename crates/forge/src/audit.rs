//! Audit logging (re-exported from [forge_audit](forge_audit)).

pub use forge_audit::{
  AuditError, AuditEvent, EventKind, LogResult, Outcome, anonymize_actor, log, retention_purge,
};
