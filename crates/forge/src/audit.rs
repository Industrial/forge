//! Audit logging (re-exported from [forge_audit](forge_audit)).

pub use forge_audit::{
  anonymize_actor, log, retention_purge, AuditError, AuditEvent, EventKind, LogResult, Outcome,
};
