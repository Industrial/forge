# 007_audit_logging: Phase 7 Audit Logging

## Overview

Forge provides a **structured audit log** that records who did what, when, and with what outcome. The design reuses the authorization primitives (Requester, Action, Object, Organization) from Phase 6 and is structured so that **GDPR and HIPAA alignment is achievable with minimal app code**: data minimization by default, retention-friendly schema, and access control via existing authz.

## Objectives

1. **Security and compliance**: Record authorization decisions (allow/deny) and authentication lifecycle events for accountability and forensics.
2. **Minimal payload, no PHI/PII by default**: Default schema stores only IDs, action, resource type/id, outcome, and timestamp—no free-form JSON or request bodies.
3. **Best-effort, non-blocking**: Audit write failures must not fail the user request.
4. **Multi-tenant safe**: All events are scoped by `organization_id`; reading the audit log is subject to the same authz and scoping as other resources (Ghost Mode preserved).

## Event Model

### Primitives (aligned with 006)

| Field | Type | Purpose |
|-------|------|---------|
| **actor_id** | UUID | Who performed the action (requester). |
| **subject_id** | UUID (optional) | Identity context if different from actor (e.g. impersonation). |
| **organization_id** | UUID (optional) | Tenant scope; required for org-scoped resources. |
| **action** | enum | `Create`, `Read`, `Update`, `Delete`, `Manage` (same as 006 `Action`). |
| **resource_type** | string | Entity type (e.g. `project`, `user`, `membership`). |
| **resource_id** | UUID (optional) | Target entity id when applicable. |
| **outcome** | enum | `Allowed`, `Denied` (authz) or `Success`, `Failure` (mutations). |
| **occurred_at** | DateTime | Event time (UTC). |
| **reason** | string (optional) | Short, non-PHI explanation (e.g. "export requested"). No free-form JSON in default schema. |

**Design rule**: Do not store passwords, tokens, API keys, or PHI/PII in the audit log. The optional `reason` field is for short, non-identifying text only. This keeps the log suitable for GDPR and HIPAA use with minimal policy overhead.

### Event Kinds

| Kind | When recorded | Source |
|------|----------------|--------|
| **Authz** | Every guard or scoped access allow/deny | Automatic (framework). |
| **Auth** | Login, logout, failed login, password reset requested, etc. | Automatic (framework). |
| **Mutation** | Create/update/delete on entities marked as audited | Explicit or opt-in (see below). |
| **Custom** | App-defined business events | Explicit `forge::audit::log(...)`. |

**Read actions**: Do **not** auto-audit `Read` by default (noise and volume). Sensitive reads (e.g. "export", "view PII") may be recorded via the explicit API.

## Canonical Schema

Single table, index-friendly for retention and querying by org/actor/time.

### `audit_log` table (migration in `crates/db`)

| Column | Type | Nullable | Purpose |
|--------|------|----------|---------|
| **id** | UUID | No | Primary key. |
| **event_kind** | String | No | `authz` \| `auth` \| `mutation` \| `custom`. |
| **actor_id** | UUID | No | Requester. |
| **subject_id** | UUID | Yes | Subject if different from actor. |
| **organization_id** | UUID | Yes | Tenant scope. |
| **action** | String | No | `create`, `read`, `update`, `delete`, `manage`. |
| **resource_type** | String | No | Entity type. |
| **resource_id** | UUID | Yes | Target entity id. |
| **outcome** | String | No | `allowed`, `denied`, `success`, `failure`. |
| **reason** | String | Yes | Short, non-PHI note (length limit, e.g. 512 chars). |
| **occurred_at** | DateTime (UTC) | No | Event time. |

**Indexes**:

- `(organization_id, occurred_at)` — tenant-scoped time-based queries and retention purge.
- `(actor_id, occurred_at)` — "all actions by this user" and erasure/anonymization.
- Optional: `(resource_type, resource_id)` for "history of this resource."

No `request_id`, IP, or `user_agent` in the default schema; they can be added later as opt-in columns if the app accepts the compliance and retention implications.

## Where Audit Hooks In

### Automatic (framework)

- **Authz**: When `guard()` or a scoped query results in allow/deny (or 404 from Ghost Mode), the framework writes one audit event (kind `authz`, outcome `allowed` or `denied`).
- **Auth**: On login success/failure, logout, password reset requested/completed, etc., the framework writes events (kind `auth`).

Implementation: small helpers called from existing authz and auth code paths; write is best-effort (see Failure policy).

### Explicit (app)

- **Mutation**: For entities that need a change trail, the app calls `forge::audit::log(db, event)` after create/update/delete (or the framework provides a thin wrapper that takes `Event` and writes it).
- **Custom**: For business events (e.g. "invoice approved", "export run"), the app calls `forge::audit::log(db, event)` with kind `custom`.

**API sketch**:

```rust
// forge::audit
pub struct AuditEvent {
    pub event_kind: EventKind,  // authz | auth | mutation | custom
    pub actor_id: Uuid,
    pub subject_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub action: Action,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub outcome: Outcome,
    pub reason: Option<String>,
}

pub async fn log(db: &DatabaseConnection, event: AuditEvent) -> Result<(), AuditError>;
```

`log` is fire-and-forget from the caller’s perspective: on failure, the error is logged (tracing) and optionally metrics; the function returns `Ok(())` or a non-fatal `Err` so that the request is **not** failed (see Failure policy).

## Failure Policy

- **Synchronous write**: Events are written in the request path (no required async queue in Phase 7).
- **Best-effort**: If the insert fails (e.g. DB unavailable), log the error and optionally increment a metric. **Do not** return an error to the client or abort the request. The audit log is secondary to the primary operation.
- Optional: use a dedicated small connection pool for audit writes to avoid blocking main app queries. Document this as a deployment consideration.

## Access Control for Reading the Audit Log

- **Reuse authz**: Any endpoint that returns audit log entries must use the same guard and scoping as other sensitive resources (e.g. only certain roles, org-scoped).
- **Convention**: Only return rows where `organization_id` matches the current session’s org (and optionally filter by `actor_id` for "my actions"). No cross-tenant visibility; Ghost Mode semantics apply (no leaking existence of other tenants).

No new framework primitives: document that "read audit log" is an action protected by `guard()` and that queries must be scoped by `organization_id`.

### How to expose a "view audit log" endpoint

- Protect the route with auth and a role guard (e.g. only Admin or Owner can read the audit log).
- Query the `audit_log` table **scoped by the current session’s `organization_id`** so that tenants only see their own events. Optionally filter by `actor_id` for "my actions".
- Do **not** return rows from other organizations; this preserves Ghost Mode (no cross-tenant leakage).
- Example pattern: require `auth_session.guard(Action::Read, Role::Admin)?`, then `SELECT * FROM audit_log WHERE organization_id = $current_org_id ORDER BY occurred_at DESC LIMIT N` (using your DB layer). The app crate owns the audit_log entity or raw query; the framework does not provide a built-in "list audit" handler.

## GDPR and HIPAA Alignment (Minimal Implementation)

The following design choices keep the bar low for compliance with minimal code.

### Data minimization

- Default schema contains **no free-form JSON**, **no request/response bodies**, and **no** IP/user_agent. Only structured fields and an optional short `reason`.
- Do **not** log PHI or PII in the payload; document that `reason` must remain non-PHI.

### Retention

- Schema supports time-based deletion via index on `occurred_at`.
- **No built-in scheduler** in the framework. Document: "Run a scheduled job to delete (or archive) events older than your retention window (e.g. 1 year; 6 years for HIPAA-related documentation where applicable)."
- **Optional helper**: `forge::audit::retention_purge(db, older_than: Duration) -> Result<u64>` that deletes in batches. Apps call it from their own job.

### Right to erasure (GDPR)

- Audit logs may be retained for legitimate interest or legal obligation; erasure is not always required. Where the app’s policy allows, support anonymization or deletion of events by actor.
- **Optional helper**: `forge::audit::anonymize_actor(db, actor_id, replace_with: Option<Uuid>) -> Result<u64>`. Either delete rows for that `actor_id` or set `actor_id` to a well-known "deleted user" UUID. Document: use only where policy allows; otherwise retain for compliance.

### Encryption and documentation

- **Encryption**: Framework does not implement crypto. Use TLS in front of the app and a database with encryption at rest (e.g. PostgreSQL TDE, SQLite with SQLCipher, or managed DB encryption).

### Compliance checklist (GDPR / HIPAA)

For use in regulated environments, the application is responsible for:

| Responsibility | Action |
|----------------|--------|
| **Lawful basis** | Document the lawful basis for processing audit data (e.g. legitimate interest, legal obligation) in your RoPA or privacy policy. |
| **Retention** | Define a retention window and run a scheduled job that calls `forge::audit::retention_purge(db, older_than)` (or equivalent) to delete or archive events. |
| **Access control** | Restrict who can read the audit log (e.g. guard with `Role::Admin` or `Role::Owner`) and scope queries by `organization_id`. |
| **No PHI in payload** | Do not store PHI or PII in the audit log; keep the `reason` field short and non-identifying. |
| **Encryption** | Use TLS for transport and enable encryption at rest for the database. |
| **Documentation** | Record audit logging in your Records of Processing Activities (RoPA) and, if applicable, in HIPAA policies and BAAs. |

## Implementation Plan

### 007.1: Schema and migration

- [ ] Add migration `m..._create_audit_log_table` in `crates/db` with columns and indexes above.
- [ ] No foreign keys to `user` or `organization` (audit log remains valid even if actor/org are later deleted or anonymized).

### 007.2: Forge library (`crates/forge`)

- [ ] Add `forge::audit` module with `AuditEvent`, `EventKind`, `Outcome`, and `log(db, event)`.
- [ ] Implement best-effort write: on failure, log and optionally metric; do not propagate error so that the request succeeds.
- [ ] Optional: `retention_purge(db, older_than)` and `anonymize_actor(db, actor_id, replace_with)`.

### 007.3: Integration with authz (006)

- [ ] Where guard or scoped access is evaluated, call audit::log with event_kind `authz`, outcome `allowed` or `denied`. Use existing AuthzContext for actor_id, subject_id, organization_id.
- [ ] Ensure no extra DB round-trip blocks the request; fire audit write and continue.

### 007.4: Integration with auth (005)

- [ ] On login success/failure, logout, password reset requested/completed, write events with event_kind `auth`, appropriate action and outcome.
- [ ] Do not log passwords or tokens; only event type and outcome.

### 007.5: Generated app and docs

- [ ] Document in 007 (and optionally in generated app README) how to expose a "view audit log" endpoint (guard + org-scoped query).
- [ ] Add short "Compliance" subsection to 007 (or linked doc) summarizing GDPR/HIPAA responsibilities (lawful basis, retention, access control, no PHI, encryption).

## Success Criteria

1. **Schema**: `forge new` (or migration) creates `audit_log` with the canonical columns and indexes.
2. **Automatic events**: Authz allow/deny and auth lifecycle events (login, logout, failed login) produce audit rows without app code.
3. **Explicit API**: `forge::audit::log(db, event)` writes mutation or custom events; failed writes do not fail the request.
4. **Access control**: Documented pattern for reading the log with guard + org scope; no cross-tenant leakage.
5. **Optional helpers**: If implemented, `retention_purge` and `anonymize_actor` behave as specified and are documented.
6. **E2E**: At least one test that registers/logs in, performs an action, and asserts a corresponding audit log row (e.g. authz or auth event).

## Integration with Prior Phases

- **005 (Auth)**: Audit subscribes to login, logout, failed login, password reset; uses session/user id for actor_id.
- **006 (Authz)**: Audit subscribes to guard and scoped query outcomes; uses AuthzContext for actor, subject, org, action, resource. Reading the audit log is itself an authz-protected, org-scoped operation.

## Out of Scope for Phase 7

- Async or batched write pipeline (e.g. channel + background worker).
- Built-in retention scheduler or archival.
- Redaction or encryption of payload columns.
- Sending events to external systems (webhooks, Kafka, etc.).
- IP or user_agent columns (can be added later as opt-in).
