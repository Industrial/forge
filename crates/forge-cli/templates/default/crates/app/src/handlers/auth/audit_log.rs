//! GET /api/auth/audit-log — list audit log entries. Requires audit.read. Non-global: only current org.

use axum::{Json, extract::Query, response::IntoResponse};
use chrono::NaiveDateTime;
use forge_auth::token_auth::RequireAuth;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::Deserialize;
use uuid::Uuid;

use db::auth::Backend;
use db::models::{audit_log, user};

use crate::Error as ForgeError;
use crate::scoped_query::WithScope;

use super::shared::{
  DbFromScope, PERMISSION_AUDIT_READ, ScopeFromHeaders, has_global_scope, require_permission,
};

fn default_limit() -> u64 {
  50
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListAuditLogQuery {
  pub from: Option<String>,
  pub to: Option<String>,
  pub actor_id: Option<Uuid>,
  pub outcome: Option<String>,
  pub event_kind: Option<String>,
  pub resource_type: Option<String>,
  pub action: Option<String>,
  pub reason: Option<String>,
  #[serde(default = "default_limit")]
  pub limit: u64,
  #[serde(default)]
  pub offset: u64,
}

/// GET /api/auth/audit-log — list audit log entries. Requires audit.read. Non-global: only current org.
pub async fn list_audit_log(
  ScopeFromHeaders(scope, _): ScopeFromHeaders<user::Model>,
  auth: RequireAuth<Backend, user::Model>,
  DbFromScope(db): DbFromScope,
  Query(q): Query<ListAuditLogQuery>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_AUDIT_READ, Some(&scope)).await {
    return Ok(resp);
  }
  let limit = q.limit.min(200);
  let offset = q.offset;

  let scope_opt: Option<&forge_auth::RequestScope> =
    if has_global_scope(&db, user, PERMISSION_AUDIT_READ).await {
      None
    } else {
      Some(&scope)
    };
  let mut query = audit_log::Entity::find().with_scope(scope_opt);
  if let Some(ref from) = q.from {
    if let Ok(naive) = NaiveDateTime::parse_from_str(from, "%Y-%m-%dT%H:%M:%S%.fZ") {
      query = query.filter(audit_log::Column::OccurredAt.gte(naive));
    } else if let Ok(naive) = NaiveDateTime::parse_from_str(from, "%Y-%m-%d") {
      query = query.filter(audit_log::Column::OccurredAt.gte(naive));
    }
  }
  if let Some(ref to) = q.to {
    if let Ok(naive) = NaiveDateTime::parse_from_str(to, "%Y-%m-%dT%H:%M:%S%.fZ") {
      query = query.filter(audit_log::Column::OccurredAt.lte(naive));
    } else if let Ok(naive) = NaiveDateTime::parse_from_str(to, "%Y-%m-%d") {
      let end = naive + chrono::Duration::days(1);
      query = query.filter(audit_log::Column::OccurredAt.lt(end));
    }
  }
  if let Some(actor_id) = q.actor_id {
    query = query.filter(audit_log::Column::ActorId.eq(actor_id));
  }
  if let Some(ref outcome) = q.outcome {
    query = query.filter(audit_log::Column::Outcome.eq(outcome.as_str()));
  }
  if let Some(ref event_kind) = q.event_kind {
    query = query.filter(audit_log::Column::EventKind.eq(event_kind.as_str()));
  }
  if let Some(ref resource_type) = q.resource_type {
    query = query.filter(audit_log::Column::ResourceType.eq(resource_type.as_str()));
  }
  if let Some(ref action) = q.action {
    query = query.filter(audit_log::Column::Action.eq(action.as_str()));
  }
  if let Some(ref reason) = q.reason.filter(|r| !r.is_empty()) {
    query = query.filter(audit_log::Column::Reason.contains(reason.as_str()));
  }

  let total = query
    .clone()
    .count(&db)
    .await
    .map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?;
  let rows = query
    .order_by_desc(audit_log::Column::OccurredAt)
    .limit(limit)
    .offset(offset)
    .all(&db)
    .await
    .map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?;

  let entries: Vec<serde_json::Value> = rows
    .into_iter()
    .map(|r| {
      serde_json::json!({
        "id": r.id.to_string(),
        "event_kind": r.event_kind,
        "actor_id": r.actor_id.to_string(),
        "subject_id": r.subject_id.map(|u: Uuid| u.to_string()),
        "organization_id": r.organization_id.map(|u: Uuid| u.to_string()),
        "action": r.action,
        "resource_type": r.resource_type,
        "resource_id": r.resource_id.map(|u: Uuid| u.to_string()),
        "outcome": r.outcome,
        "reason": r.reason,
        "occurred_at": r.occurred_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      })
    })
    .collect();

  Ok(Json(serde_json::json!({ "entries": entries, "total": total })).into_response())
}
