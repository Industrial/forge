//! Structured audit logging for security and compliance (Phase 7).
//!
//! Records who did what, when, and with what outcome. Best-effort writes:
//! audit failures must not fail the user request.

use chrono::{DateTime, Duration, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use tracing::warn;
use uuid::Uuid;

use crate::authz::Action;

/// Event kind for the audit log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
  Authz,
  Auth,
  Mutation,
  Custom,
}

impl EventKind {
  pub fn as_str(&self) -> &'static str {
    match self {
      EventKind::Authz => "authz",
      EventKind::Auth => "auth",
      EventKind::Mutation => "mutation",
      EventKind::Custom => "custom",
    }
  }
}

/// Outcome of an action (authz allow/deny or mutation success/failure).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
  Allowed,
  Denied,
  Success,
  Failure,
}

impl Outcome {
  pub fn as_str(&self) -> &'static str {
    match self {
      Outcome::Allowed => "allowed",
      Outcome::Denied => "denied",
      Outcome::Success => "success",
      Outcome::Failure => "failure",
    }
  }
}

/// A single audit event (minimal payload, no PHI/PII).
#[derive(Debug, Clone)]
pub struct AuditEvent {
  pub event_kind: EventKind,
  pub actor_id: Uuid,
  pub subject_id: Option<Uuid>,
  pub organization_id: Option<Uuid>,
  pub action: Action,
  pub resource_type: String,
  pub resource_id: Option<Uuid>,
  pub outcome: Outcome,
  pub reason: Option<String>,
}

impl AuditEvent {
  /// Action as string for storage (lowercase).
  pub fn action_str(&self) -> &'static str {
    match self.action {
      Action::Read => "read",
      Action::Create => "create",
      Action::Update => "update",
      Action::Delete => "delete",
      Action::Manage => "manage",
    }
  }
}

/// Non-fatal audit error (e.g. DB unavailable). Callers must not fail the request.
#[derive(Debug)]
pub struct AuditError(Box<dyn std::error::Error + Send + Sync>);

impl std::fmt::Display for AuditError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "audit write failed: {}", self.0)
  }
}

impl std::error::Error for AuditError {}

/// Write one audit event. Best-effort: on failure, logs and returns Ok(()) so the request is not failed.
pub async fn log(db: &DatabaseConnection, event: AuditEvent) -> Result<(), AuditError> {
  let id = Uuid::new_v4();
  let occurred_at = Utc::now().naive_utc();

  let reason = event.reason.as_deref().unwrap_or("");
  let reason_len = reason.len().min(512);
  let reason = &reason[..reason_len];

  let backend = db.get_database_backend();
  let sql = match backend {
    DbBackend::Sqlite => {
      r#"INSERT INTO audit_log (id, event_kind, actor_id, subject_id, organization_id, action, resource_type, resource_id, outcome, reason, occurred_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
    }
    _ => {
      r#"INSERT INTO audit_log (id, event_kind, actor_id, subject_id, organization_id, action, resource_type, resource_id, outcome, reason, occurred_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#
    }
  };

  let values: Vec<sea_orm::Value> = vec![
    id.into(),
    event.event_kind.as_str().into(),
    event.actor_id.into(),
    event.subject_id.into(),
    event.organization_id.into(),
    event.action_str().into(),
    event.resource_type.into(),
    event.resource_id.into(),
    event.outcome.as_str().into(),
    reason.into(),
    occurred_at.into(),
  ];

  let stmt = Statement::from_sql_and_values(backend, sql, values);

  if let Err(e) = db.execute(stmt).await {
    warn!(error = %e, "audit log write failed (best-effort)");
    return Err(AuditError(Box::new(e)));
  }

  Ok(())
}

/// Delete audit log rows older than the given cutoff (for retention policy).
/// Returns the number of rows deleted. Run from a scheduled job.
pub async fn retention_purge(
  db: &DatabaseConnection,
  older_than: Duration,
) -> Result<u64, sea_orm::DbErr> {
  let cutoff: DateTime<Utc> = Utc::now() - older_than;
  let cutoff_naive = cutoff.naive_utc();

  let backend = db.get_database_backend();
  let (sql, values) = match backend {
    DbBackend::Sqlite => (
      "DELETE FROM audit_log WHERE occurred_at < ?",
      vec![cutoff_naive.into()],
    ),
    _ => (
      "DELETE FROM audit_log WHERE occurred_at < $1",
      vec![cutoff_naive.into()],
    ),
  };
  let stmt = Statement::from_sql_and_values(backend, sql, values);
  let result = db.execute(stmt).await?;
  Ok(result.rows_affected())
}

/// Anonymize or delete audit log rows for a given actor (e.g. for GDPR erasure).
/// If `replace_with` is Some(uuid), sets actor_id to that UUID; if None, deletes the rows.
/// Returns the number of rows updated or deleted.
pub async fn anonymize_actor(
  db: &DatabaseConnection,
  actor_id: Uuid,
  replace_with: Option<Uuid>,
) -> Result<u64, sea_orm::DbErr> {
  let backend = db.get_database_backend();
  let stmt = match (backend, replace_with) {
    (DbBackend::Sqlite, Some(replace)) => Statement::from_sql_and_values(
      backend,
      "UPDATE audit_log SET actor_id = ? WHERE actor_id = ?",
      vec![replace.into(), actor_id.into()],
    ),
    (DbBackend::Sqlite, None) => Statement::from_sql_and_values(
      backend,
      "DELETE FROM audit_log WHERE actor_id = ?",
      vec![actor_id.into()],
    ),
    (_, Some(replace)) => Statement::from_sql_and_values(
      backend,
      "UPDATE audit_log SET actor_id = $1 WHERE actor_id = $2",
      vec![replace.into(), actor_id.into()],
    ),
    (_, None) => Statement::from_sql_and_values(
      backend,
      "DELETE FROM audit_log WHERE actor_id = $1",
      vec![actor_id.into()],
    ),
  };
  let result = db.execute(stmt).await?;
  Ok(result.rows_affected())
}
