use sea_orm::{ConnectionTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::info;
use uuid::Uuid;

use crate::models::{organization, role_permission};

/// Permission keys (must match app's DASHBOARD_PERMISSIONS). .read = view/list, .write = create/update/delete.
const PERMISSIONS: &[&str] = &[
  "dashboard",
  "dashboard.organizations.read",
  "dashboard.organizations.write",
  "dashboard.users.read",
  "dashboard.users.write",
  "dashboard.roles.read",
  "dashboard.roles.write",
  "dashboard.permissions.read",
  "dashboard.permissions.write",
  "dashboard.audit.read",
];

/// Org-scoped: owner and admin get users, roles, permissions read+write, and audit read (organizations list is platform-only).
const ORG_OWNER_ADMIN: &[&str] = &[
  "dashboard",
  "dashboard.users.read",
  "dashboard.users.write",
  "dashboard.roles.read",
  "dashboard.roles.write",
  "dashboard.permissions.read",
  "dashboard.permissions.write",
  "dashboard.audit.read",
];

/// Org-scoped: editor can read and write users, roles, and role-permissions.
const ORG_EDITOR: &[&str] = &[
  "dashboard",
  "dashboard.users.read",
  "dashboard.users.write",
  "dashboard.permissions.read",
  "dashboard.permissions.write",
  "dashboard.roles.read",
  "dashboard.roles.write",
];

/// Org-scoped: viewer can read users list, audit log, permissions, and roles.
const ORG_VIEWER: &[&str] = &[
  "dashboard",
  "dashboard.users.read",
  "dashboard.audit.read",
  "dashboard.permissions.read",
  "dashboard.roles.read",
];

async fn ensure_role_permission<C: ConnectionTrait>(
  db: &C,
  scope: &str,
  role_name: &str,
  permission_key: &str,
  org_id: Option<Uuid>,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut q = role_permission::Entity::find()
    .filter(role_permission::Column::Scope.eq(scope))
    .filter(role_permission::Column::RoleName.eq(role_name))
    .filter(role_permission::Column::PermissionKey.eq(permission_key));
  match org_id {
    Some(id) => q = q.filter(role_permission::Column::OrgId.eq(id)),
    None => q = q.filter(role_permission::Column::OrgId.is_null()),
  }
  let existing = q.one(db).await?;

  if existing.is_some() {
    return Ok(());
  }

  let id = Uuid::new_v4();
  let mut model = role_permission::ActiveModel {
    id: Set(id),
    scope: Set(scope.to_string()),
    role_name: Set(role_name.to_string()),
    permission_key: Set(permission_key.to_string()),
    ..Default::default()
  };
  model.org_id = Set(org_id);
  role_permission::Entity::insert(model).exec(db).await?;
  info!(
    "Seeded role_permission: scope={} role_name={} permission_key={} org_id={:?}",
    scope, role_name, permission_key, org_id
  );
  Ok(())
}

/// Legacy permission keys that were replaced by .read/.write. Maps old key -> [new keys].
const LEGACY_MIGRATION: &[(&str, &[&str])] = &[
  ("dashboard.organizations", &["dashboard.organizations.read", "dashboard.organizations.write"]),
  ("dashboard.users", &["dashboard.users.read", "dashboard.users.write"]),
  ("dashboard.permissions.manage", &["dashboard.permissions.read", "dashboard.permissions.write"]),
];

/// Migrate legacy role_permission rows to .read/.write keys (idempotent).
async fn migrate_legacy_permissions<C: ConnectionTrait>(db: &C) -> Result<(), Box<dyn std::error::Error>> {
  for (old_key, new_keys) in LEGACY_MIGRATION {
    let rows = role_permission::Entity::find()
      .filter(role_permission::Column::PermissionKey.eq(*old_key))
      .all(db)
      .await?;
    for row in rows {
      for &new_key in *new_keys {
        ensure_role_permission(
          db,
          &row.scope,
          &row.role_name,
          new_key,
          row.org_id,
        )
        .await?;
      }
      role_permission::Entity::delete_by_id(row.id).exec(db).await?;
    }
  }
  Ok(())
}

pub async fn seed(db: &impl ConnectionTrait) -> Result<(), Box<dyn std::error::Error>> {
  // Migrate any legacy permission keys to .read/.write (no-op if already migrated).
  migrate_legacy_permissions(db).await?;

  // Global: platform_admin has all permissions (org_id = null).
  for key in PERMISSIONS {
    ensure_role_permission(db, "global", "platform_admin", key, None).await?;
  }

  // Per-org: seed role_permission for each organization in the DB.
  let orgs = organization::Entity::find().all(db).await?;
  for org in orgs {
    let org_id = org.id;
    for key in ORG_OWNER_ADMIN {
      ensure_role_permission(db, "org", "owner", key, Some(org_id)).await?;
      ensure_role_permission(db, "org", "admin", key, Some(org_id)).await?;
    }
    for key in ORG_EDITOR {
      ensure_role_permission(db, "org", "editor", key, Some(org_id)).await?;
    }
    for key in ORG_VIEWER {
      ensure_role_permission(db, "org", "viewer", key, Some(org_id)).await?;
    }
  }

  Ok(())
}

/// Seeds role_permission rows for the four template roles of one org. Call after creating org_roles for a new org.
pub async fn seed_role_permissions_for_org<C: ConnectionTrait>(
  db: &C,
  org_id: Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
  for key in ORG_OWNER_ADMIN {
    ensure_role_permission(db, "org", "owner", key, Some(org_id)).await?;
    ensure_role_permission(db, "org", "admin", key, Some(org_id)).await?;
  }
  for key in ORG_EDITOR {
    ensure_role_permission(db, "org", "editor", key, Some(org_id)).await?;
  }
  for key in ORG_VIEWER {
    ensure_role_permission(db, "org", "viewer", key, Some(org_id)).await?;
  }
  Ok(())
}
