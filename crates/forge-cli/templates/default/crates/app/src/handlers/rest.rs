//! REST API: org-scoped routes use [ScopeFromHeaders] (X-Organization-Id, X-Role-Id); others require Bearer where applicable.
//! Epic 6: list/get responses do not embed relations (e.g. no nested memberships); relations as IDs or separate endpoints.

use crate::Error as ForgeError;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use forge_auth::token_auth::hash_password;
use forge_db::DbConnection;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use uuid::Uuid;

use crate::permissions::dashboard_permissions;
use db::models::{membership, org_role, role_permission, user, user_org_role};

// ---- Permissions (code-defined keys) ----
/// GET /api/permissions — list known permission keys (code-defined). Read-only; no auth or scope required.
pub async fn list_permissions(
  State(_db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let list: Vec<&str> = dashboard_permissions().to_vec();
  Ok(Json(serde_json::json!({ "permissions": list })).into_response())
}

// Re-export for seeds and legacy callers (impls live in db::organization).
pub use db::organization::{
  CreateOrganizationBody, create_organization_impl, ensure_organization_impl,
};

/// Body for creating an org role. Used by impls and seeds.
#[derive(Debug, Clone)]
pub struct CreateOrgRoleBody {
  pub name: String,
  pub display_name: Option<String>,
}

/// Body for adding a user to an org (user_id or email+password). Used by add_org_user_impl.
#[derive(Debug, Clone)]
pub struct AddOrgUserBody {
  pub user_id: Option<Uuid>,
  pub email: Option<String>,
  pub password: Option<String>,
}

pub async fn create_org_role_impl(
  db: &DbConnection,
  org_id: Uuid,
  payload: &CreateOrgRoleBody,
) -> Result<Uuid, ForgeError> {
  let name = payload.name.trim();
  if name.is_empty() {
    return Err(ForgeError::Generic("name is required".into()));
  }
  if org_role::Entity::find()
    .filter(org_role::Column::OrgId.eq(org_id))
    .filter(org_role::Column::Name.eq(name))
    .one(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .is_some()
  {
    return Err(ForgeError::Generic("Role name exists".into()));
  }
  let now = chrono::Utc::now().naive_utc();
  let id = Uuid::new_v4();
  let display_name = payload
    .display_name
    .as_ref()
    .map(|s: &String| s.trim())
    .filter(|s: &&str| !s.is_empty())
    .map(String::from);
  org_role::Entity::insert(org_role::ActiveModel {
    id: Set(id),
    org_id: Set(org_id),
    name: Set(name.to_string()),
    display_name: Set(display_name),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  })
  .exec(db)
  .await
  .map_err(|e| ForgeError::Generic(e.to_string()))?;
  Ok(id)
}

/// Idempotent: find org role by (org_id, name) or create. For use in seeds.
pub async fn ensure_org_role_impl(
  db: &DbConnection,
  org_id: Uuid,
  payload: &CreateOrgRoleBody,
) -> Result<Uuid, ForgeError> {
  let name = payload.name.trim();
  if name.is_empty() {
    return Err(ForgeError::Generic("name is required".into()));
  }
  if let Some(role) = org_role::Entity::find()
    .filter(org_role::Column::OrgId.eq(org_id))
    .filter(org_role::Column::Name.eq(name))
    .one(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
  {
    return Ok(role.id);
  }
  let now = chrono::Utc::now().naive_utc();
  let id = Uuid::new_v4();
  let display_name = payload
    .display_name
    .as_ref()
    .map(|s: &String| s.trim())
    .filter(|s: &&str| !s.is_empty())
    .map(String::from);
  org_role::Entity::insert(org_role::ActiveModel {
    id: Set(id),
    org_id: Set(org_id),
    name: Set(name.to_string()),
    display_name: Set(display_name),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  })
  .exec(db)
  .await
  .map_err(|e| ForgeError::Generic(e.to_string()))?;
  Ok(id)
}

/// Returns (role_id, role_name) for the org. Used by seeds to resolve role ids.
pub async fn list_org_roles_impl(
  db: &DbConnection,
  org_id: Uuid,
) -> Result<std::collections::HashMap<String, Uuid>, ForgeError> {
  let rows = org_role::Entity::find()
    .filter(org_role::Column::OrgId.eq(org_id))
    .order_by_asc(org_role::Column::Name)
    .all(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let mut map = std::collections::HashMap::new();
  for r in rows {
    map.insert(r.name.clone(), r.id);
  }
  Ok(map)
}

pub async fn add_org_user_impl(
  db: &DbConnection,
  org_id: Uuid,
  payload: &AddOrgUserBody,
) -> Result<(Uuid, bool), ForgeError> {
  let (user_id, created) = match (
    payload.user_id,
    payload.email.as_deref(),
    payload.password.as_deref(),
  ) {
    (Some(uid), _, _) => {
      let u = user::Entity::find_by_id(uid)
        .one(db)
        .await
        .map_err(|e| ForgeError::Generic(e.to_string()))?;
      if u.is_none() {
        return Err(ForgeError::Generic("User not found".into()));
      }
      if membership::Entity::find()
        .filter(membership::Column::UserId.eq(uid))
        .filter(membership::Column::OrgId.eq(org_id))
        .one(db)
        .await
        .map_err(|e| ForgeError::Generic(e.to_string()))?
        .is_some()
      {
        return Err(ForgeError::Generic("User already in organization".into()));
      }
      (uid, false)
    }
    (None, Some(email), Some(password))
      if {
        let email: &str = email;
        let password: &str = password;
        email.trim().contains('@') && password.len() >= 8
      } =>
    {
      let email = email.trim();
      if user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| ForgeError::Generic(e.to_string()))?
        .is_some()
      {
        return Err(ForgeError::Generic("Email already in use".into()));
      }
      let now = chrono::Utc::now().naive_utc();
      let user_id = Uuid::new_v4();
      let hash = hash_password(password).map_err(|e| ForgeError::Generic(e.to_string()))?;
      user::Entity::insert(user::ActiveModel {
        id: Set(user_id),
        email: Set(email.to_string()),
        password_hash: Set(hash),
        is_active: Set(true),
        is_admin: Set(false),
        current_org_id: Set(Some(org_id)),
        current_role: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      })
      .exec(db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
      membership::Entity::insert(membership::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        org_id: Set(org_id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      })
      .exec(db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
      (user_id, true)
    }
    _ => {
      return Err(ForgeError::Generic(
        "Provide user_id or email+password".into(),
      ));
    }
  };
  if membership::Entity::find()
    .filter(membership::Column::UserId.eq(user_id))
    .filter(membership::Column::OrgId.eq(org_id))
    .one(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .is_none()
  {
    let now = chrono::Utc::now().naive_utc();
    membership::Entity::insert(membership::ActiveModel {
      id: Set(Uuid::new_v4()),
      user_id: Set(user_id),
      org_id: Set(org_id),
      created_at: Set(now),
      updated_at: Set(now),
      ..Default::default()
    })
    .exec(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  }
  Ok((user_id, created))
}

/// Idempotent: get or create user by email, ensure membership in org. Returns (user_id, created_user).
/// For use in seeds. If user exists, ensures membership for org_id; does not update password.
pub async fn ensure_org_user_impl(
  db: &DbConnection,
  org_id: Uuid,
  email: &str,
  password: &str,
) -> Result<(Uuid, bool), ForgeError> {
  let email = email.trim();
  if !email.contains('@') || password.len() < 8 {
    return Err(ForgeError::Generic(
      "email and password (min 8 chars) required".into(),
    ));
  }
  let existing_user = user::Entity::find()
    .filter(user::Column::Email.eq(email))
    .one(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let (user_id, created) = match existing_user {
    Some(u) => {
      let membership_exists = membership::Entity::find()
        .filter(membership::Column::UserId.eq(u.id))
        .filter(membership::Column::OrgId.eq(org_id))
        .one(db)
        .await
        .map_err(|e| ForgeError::Generic(e.to_string()))?;
      if membership_exists.is_none() {
        let now = chrono::Utc::now().naive_utc();
        membership::Entity::insert(membership::ActiveModel {
          id: Set(Uuid::new_v4()),
          user_id: Set(u.id),
          org_id: Set(org_id),
          created_at: Set(now),
          updated_at: Set(now),
          ..Default::default()
        })
        .exec(db)
        .await
        .map_err(|e| ForgeError::Generic(e.to_string()))?;
      }
      (u.id, false)
    }
    None => {
      let now = chrono::Utc::now().naive_utc();
      let user_id = Uuid::new_v4();
      let hash = hash_password(password).map_err(|e| ForgeError::Generic(e.to_string()))?;
      user::Entity::insert(user::ActiveModel {
        id: Set(user_id),
        email: Set(email.to_string()),
        password_hash: Set(hash),
        is_active: Set(true),
        is_admin: Set(false),
        current_org_id: Set(Some(org_id)),
        current_role: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      })
      .exec(db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
      membership::Entity::insert(membership::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        org_id: Set(org_id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      })
      .exec(db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
      (user_id, true)
    }
  };
  Ok((user_id, created))
}

/// Idempotent: ensure user is a member of org (insert membership if missing). For use in seeds.
pub async fn ensure_org_membership_impl(
  db: &DbConnection,
  org_id: Uuid,
  user_id: Uuid,
) -> Result<(), ForgeError> {
  let exists = membership::Entity::find()
    .filter(membership::Column::UserId.eq(user_id))
    .filter(membership::Column::OrgId.eq(org_id))
    .one(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if exists.is_some() {
    return Ok(());
  }
  let now = chrono::Utc::now().naive_utc();
  membership::Entity::insert(membership::ActiveModel {
    id: Set(Uuid::new_v4()),
    user_id: Set(user_id),
    org_id: Set(org_id),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  })
  .exec(db)
  .await
  .map_err(|e| ForgeError::Generic(e.to_string()))?;
  Ok(())
}

pub async fn add_org_user_roles_impl(
  db: &DbConnection,
  org_id: Uuid,
  user_id: Uuid,
  role_ids: &[Uuid],
) -> Result<(), ForgeError> {
  if role_ids.is_empty() {
    return Ok(());
  }
  for rid in role_ids {
    let r = org_role::Entity::find_by_id(*rid)
      .one(db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
    if r.map(|r| r.org_id != org_id).unwrap_or(true) {
      return Err(ForgeError::Generic(
        "role must belong to organization".into(),
      ));
    }
  }
  let now = chrono::Utc::now().naive_utc();
  for rid in role_ids {
    let exists = user_org_role::Entity::find()
      .filter(user_org_role::Column::UserId.eq(user_id))
      .filter(user_org_role::Column::OrgId.eq(org_id))
      .filter(user_org_role::Column::RoleId.eq(*rid))
      .one(db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
    if exists.is_none() {
      user_org_role::Entity::insert(user_org_role::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        org_id: Set(org_id),
        role_id: Set(*rid),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      })
      .exec(db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
    }
  }
  Ok(())
}

pub async fn add_org_role_permission_impl(
  db: &DbConnection,
  org_id: Uuid,
  role_id: Uuid,
  permission_key: &str,
) -> Result<(), ForgeError> {
  let r = org_role::Entity::find_by_id(role_id)
    .one(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let Some(r) = r else {
    return Err(ForgeError::Generic("Role not found".into()));
  };
  if r.org_id != org_id {
    return Err(ForgeError::Generic("Role not in organization".into()));
  }
  let key = permission_key.trim();
  if !dashboard_permissions().contains(&key) {
    return Err(ForgeError::Generic("invalid permission_key".into()));
  }
  let exists = role_permission::Entity::find()
    .filter(role_permission::Column::Scope.eq("org"))
    .filter(role_permission::Column::OrgId.eq(org_id))
    .filter(role_permission::Column::RoleName.eq(&r.name))
    .filter(role_permission::Column::PermissionKey.eq(key))
    .one(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if exists.is_none() {
    role_permission::Entity::insert(role_permission::ActiveModel {
      id: Set(Uuid::new_v4()),
      scope: Set("org".to_string()),
      role_name: Set(r.name.clone()),
      permission_key: Set(key.to_string()),
      org_id: Set(Some(org_id)),
      ..Default::default()
    })
    .exec(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  }
  Ok(())
}
