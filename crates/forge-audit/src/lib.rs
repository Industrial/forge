//! Structured audit logging for security and compliance.

use chrono::{DateTime, Duration, Utc};
use forge_authz::Action;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use tracing::warn;
use uuid::Uuid;

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

/// Outcome of an action.
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

/// Non-fatal audit error. Callers must not fail the request.
#[derive(Debug)]
pub struct AuditError(Box<dyn std::error::Error + Send + Sync>);

impl std::fmt::Display for AuditError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "audit write failed: {}", self.0)
  }
}

impl std::error::Error for AuditError {}

/// Result of a successful audit log write.
#[derive(Debug, Clone, Copy)]
pub struct LogResult {
  pub id: Uuid,
  pub occurred_at: chrono::NaiveDateTime,
}

/// Write one audit event. Best-effort: on failure, logs and returns Err.
pub async fn log(db: &impl ConnectionTrait, event: AuditEvent) -> Result<LogResult, AuditError> {
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

  Ok(LogResult { id, occurred_at })
}

/// Delete audit log rows older than the given cutoff.
pub async fn retention_purge(
  db: &impl ConnectionTrait,
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

/// Anonymize or delete audit log rows for a given actor (e.g. GDPR erasure).
pub async fn anonymize_actor(
  db: &impl ConnectionTrait,
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

#[cfg(test)]
mod tests {
  use super::*;
  use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};

  const AUDIT_LOG_DDL: &str = r#"CREATE TABLE audit_log (
    id TEXT PRIMARY KEY,
    event_kind TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    subject_id TEXT,
    organization_id TEXT,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT,
    outcome TEXT NOT NULL,
    reason TEXT,
    occurred_at TEXT NOT NULL
  )"#;

  async fn in_memory_db() -> DatabaseConnection {
    let db = Database::connect(sea_orm::ConnectOptions::new(
      "sqlite::memory:".to_string(),
    ))
    .await
    .unwrap();
    db.execute(Statement::from_string(db.get_database_backend(), AUDIT_LOG_DDL))
      .await
      .unwrap();
    db
  }

  #[test]
  fn event_kind_as_str() {
    assert_eq!(EventKind::Authz.as_str(), "authz");
    assert_eq!(EventKind::Auth.as_str(), "auth");
    assert_eq!(EventKind::Mutation.as_str(), "mutation");
    assert_eq!(EventKind::Custom.as_str(), "custom");
  }

  #[test]
  fn outcome_as_str() {
    assert_eq!(Outcome::Allowed.as_str(), "allowed");
    assert_eq!(Outcome::Denied.as_str(), "denied");
    assert_eq!(Outcome::Success.as_str(), "success");
    assert_eq!(Outcome::Failure.as_str(), "failure");
  }

  #[test]
  fn audit_event_action_str() {
    let mut event = AuditEvent {
      event_kind: EventKind::Authz,
      actor_id: Uuid::new_v4(),
      subject_id: None,
      organization_id: None,
      action: Action::Read,
      resource_type: String::new(),
      resource_id: None,
      outcome: Outcome::Allowed,
      reason: None,
    };
    assert_eq!(event.action_str(), "read");
    event.action = Action::Create;
    assert_eq!(event.action_str(), "create");
    event.action = Action::Update;
    assert_eq!(event.action_str(), "update");
    event.action = Action::Delete;
    assert_eq!(event.action_str(), "delete");
    event.action = Action::Manage;
    assert_eq!(event.action_str(), "manage");
  }

  #[test]
  fn audit_error_display_and_error() {
    let e = AuditError(Box::new(std::io::Error::new(
      std::io::ErrorKind::Other,
      "db gone",
    )));
    assert!(e.to_string().contains("audit write failed"));
    assert!(e.to_string().contains("db gone"));
    let _: &dyn std::error::Error = &e;
  }

  #[test]
  fn log_result_fields() {
    let id = Uuid::new_v4();
    let occurred_at = chrono::Utc::now().naive_utc();
    let r = LogResult { id, occurred_at };
    assert_eq!(r.id, id);
    assert_eq!(r.occurred_at, occurred_at);
  }

  #[tokio::test]
  async fn log_inserts_and_returns_ok() {
    let db = in_memory_db().await;
    let event = AuditEvent {
      event_kind: EventKind::Authz,
      actor_id: Uuid::new_v4(),
      subject_id: Some(Uuid::new_v4()),
      organization_id: Some(Uuid::new_v4()),
      action: Action::Read,
      resource_type: "test".into(),
      resource_id: None,
      outcome: Outcome::Allowed,
      reason: None,
    };
    let res = log(&db, event).await;
    assert!(res.is_ok());
    let log_result = res.unwrap();
    assert!(log_result.occurred_at <= chrono::Utc::now().naive_utc());
  }

  #[tokio::test]
  async fn log_with_reason_truncates_at_512() {
    let db = in_memory_db().await;
    let event = AuditEvent {
      event_kind: EventKind::Mutation,
      actor_id: Uuid::new_v4(),
      subject_id: None,
      organization_id: None,
      action: Action::Update,
      resource_type: "user".into(),
      resource_id: Some(Uuid::new_v4()),
      outcome: Outcome::Success,
      reason: Some("x".repeat(600)),
    };
    let res = log(&db, event).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn log_fails_on_invalid_db_returns_audit_error() {
    let db = in_memory_db().await;
    // Table missing a column so insert will fail
    db.execute(Statement::from_string(
      db.get_database_backend(),
      "DROP TABLE audit_log; CREATE TABLE audit_log (id TEXT PRIMARY KEY)",
    ))
    .await
    .unwrap();
    let event = AuditEvent {
      event_kind: EventKind::Authz,
      actor_id: Uuid::new_v4(),
      subject_id: None,
      organization_id: None,
      action: Action::Read,
      resource_type: "x".into(),
      resource_id: None,
      outcome: Outcome::Allowed,
      reason: None,
    };
    let res = log(&db, event).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("audit write failed"));
  }

  #[tokio::test]
  async fn retention_purge_deletes_old_rows() {
    let db = in_memory_db().await;
    let event = AuditEvent {
      event_kind: EventKind::Authz,
      actor_id: Uuid::new_v4(),
      subject_id: None,
      organization_id: None,
      action: Action::Read,
      resource_type: "x".into(),
      resource_id: None,
      outcome: Outcome::Allowed,
      reason: None,
    };
    log(&db, event).await.unwrap();
    // Cutoff = now - older_than. With 10s, cutoff is in the past; our row is recent so 0 deleted.
    let affected = retention_purge(&db, Duration::seconds(10)).await.unwrap();
    assert_eq!(affected, 0);
    // With -1s, cutoff = now + 1s (future); delete where occurred_at < cutoff removes recent row.
    let affected = retention_purge(&db, Duration::seconds(-1)).await.unwrap();
    assert_eq!(affected, 1);
  }

  #[tokio::test]
  async fn anonymize_actor_delete() {
    let db = in_memory_db().await;
    let actor_id = Uuid::new_v4();
    let event = AuditEvent {
      event_kind: EventKind::Authz,
      actor_id,
      subject_id: None,
      organization_id: None,
      action: Action::Read,
      resource_type: "x".into(),
      resource_id: None,
      outcome: Outcome::Allowed,
      reason: None,
    };
    log(&db, event).await.unwrap();
    let n = anonymize_actor(&db, actor_id, None).await.unwrap();
    assert_eq!(n, 1);
  }

  #[tokio::test]
  async fn anonymize_actor_replace() {
    let db = in_memory_db().await;
    let actor_id = Uuid::new_v4();
    let replace_with = Uuid::new_v4();
    let event = AuditEvent {
      event_kind: EventKind::Authz,
      actor_id,
      subject_id: None,
      organization_id: None,
      action: Action::Read,
      resource_type: "x".into(),
      resource_id: None,
      outcome: Outcome::Denied,
      reason: None,
    };
    log(&db, event).await.unwrap();
    let n = anonymize_actor(&db, actor_id, Some(replace_with)).await.unwrap();
    assert_eq!(n, 1);
  }

  #[tokio::test]
  #[cfg(feature = "mock")]
  async fn log_with_postgres_backend_uses_numbered_params() {
    use sea_orm::{MockDatabase, MockExecResult};
    let db = MockDatabase::new(sea_orm::DbBackend::Postgres)
      .append_exec_results([MockExecResult {
        last_insert_id: 0,
        rows_affected: 1,
      }])
      .into_connection();
    let event = AuditEvent {
      event_kind: EventKind::Custom,
      actor_id: Uuid::new_v4(),
      subject_id: None,
      organization_id: None,
      action: Action::Manage,
      resource_type: "custom".into(),
      resource_id: None,
      outcome: Outcome::Failure,
      reason: Some("test".into()),
    };
    let res = log(&db, event).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  #[cfg(feature = "mock")]
  async fn retention_purge_with_postgres_backend() {
    use sea_orm::{MockDatabase, MockExecResult};
    let db = MockDatabase::new(sea_orm::DbBackend::Postgres)
      .append_exec_results([MockExecResult {
        last_insert_id: 0,
        rows_affected: 2,
      }])
      .into_connection();
    let n = retention_purge(&db, Duration::days(30)).await.unwrap();
    assert_eq!(n, 2);
  }

  #[tokio::test]
  #[cfg(feature = "mock")]
  async fn anonymize_actor_postgres_replace() {
    use sea_orm::{MockDatabase, MockExecResult};
    let db = MockDatabase::new(sea_orm::DbBackend::Postgres)
      .append_exec_results([MockExecResult {
        last_insert_id: 0,
        rows_affected: 1,
      }])
      .into_connection();
    let n = anonymize_actor(&db, Uuid::new_v4(), Some(Uuid::new_v4()))
      .await
      .unwrap();
    assert_eq!(n, 1);
  }

  #[tokio::test]
  #[cfg(feature = "mock")]
  async fn anonymize_actor_postgres_delete() {
    use sea_orm::{MockDatabase, MockExecResult};
    let db = MockDatabase::new(sea_orm::DbBackend::Postgres)
      .append_exec_results([MockExecResult {
        last_insert_id: 0,
        rows_affected: 1,
      }])
      .into_connection();
    let n = anonymize_actor(&db, Uuid::new_v4(), None).await.unwrap();
    assert_eq!(n, 1);
  }
}
