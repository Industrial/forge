use forge::DbConnection;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::info;
use uuid::Uuid;

use crate::models::role_permission;

/// Permission keys (must match app's DASHBOARD_PERMISSIONS).
const PERMISSIONS: &[&str] = &[
  "dashboard",
  "dashboard.organizations",
  "dashboard.users",
  "dashboard.permissions.manage",
];

/// Org-scoped permissions for owner and admin (all except dashboard.organizations).
const ORG_OWNER_ADMIN: &[&str] = &[
  "dashboard",
  "dashboard.users",
  "dashboard.permissions.manage",
];

/// Org-scoped for editor.
const ORG_EDITOR: &[&str] = &["dashboard", "dashboard.users"];

/// Org-scoped for viewer.
const ORG_VIEWER: &[&str] = &["dashboard"];

async fn ensure_role_permission(
  db: &DbConnection,
  scope: &str,
  role_name: &str,
  permission_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let existing = role_permission::Entity::find()
    .filter(role_permission::Column::Scope.eq(scope))
    .filter(role_permission::Column::RoleName.eq(role_name))
    .filter(role_permission::Column::PermissionKey.eq(permission_key))
    .one(db)
    .await?;

  if existing.is_some() {
    return Ok(());
  }

  let id = Uuid::new_v4();
  let model = role_permission::ActiveModel {
    id: Set(id),
    scope: Set(scope.to_string()),
    role_name: Set(role_name.to_string()),
    permission_key: Set(permission_key.to_string()),
    ..Default::default()
  };
  role_permission::Entity::insert(model).exec(db).await?;
  info!(
    "Seeded role_permission: scope={} role_name={} permission_key={}",
    scope, role_name, permission_key
  );
  Ok(())
}

pub async fn seed(db: &DbConnection) -> Result<(), Box<dyn std::error::Error>> {
  // Global: platform_admin has all permissions (is_admin users get all via resolve_permissions; seeds for UI consistency).
  for key in PERMISSIONS {
    ensure_role_permission(db, "global", "platform_admin", key).await?;
  }

  // Org: owner and admin get dashboard, dashboard.users, dashboard.permissions.manage.
  for key in ORG_OWNER_ADMIN {
    ensure_role_permission(db, "org", "owner", key).await?;
    ensure_role_permission(db, "org", "admin", key).await?;
  }

  // Org: editor gets dashboard, dashboard.users.
  for key in ORG_EDITOR {
    ensure_role_permission(db, "org", "editor", key).await?;
  }

  // Org: viewer gets dashboard only.
  for key in ORG_VIEWER {
    ensure_role_permission(db, "org", "viewer", key).await?;
  }

  Ok(())
}
