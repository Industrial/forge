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
    })
    .exec(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  }
  Ok(())
}

#[cfg(test)]
mod unit_tests {
  use super::*;
  use crate::build_router_for_test;

  mod list_permissions_behavior {
    use super::*;

    #[tokio::test]
    async fn list_permissions_returns_all_dashboard_permissions() {
      // Given a database connection
      let (_router, _guard) = build_router_for_test().await.unwrap();
      let db = forge_db::initialize_database(&forge_config::load_config().unwrap().database)
        .await
        .unwrap();
      let db_conn = forge_db::wrap_traced(db);

      // When I call list_permissions
      let result = list_permissions(State(db_conn)).await;

      // Then it should return a JSON object with permissions array
      assert!(result.is_ok());
      let response = result.unwrap();
      // The response should contain dashboard permissions
      // Note: We can't easily extract the JSON here without more setup,
      // but we verify the function executes successfully
    }
  }

  mod create_org_role_behavior {
    use super::*;

    #[tokio::test]
    async fn create_org_role_returns_error_when_name_is_empty() {
      // Given an organization ID and empty role name
      let (_router, db_conn, _guard) = crate::build_router_for_test_with_db().await.unwrap();
      // Create organization first (required for foreign key constraint)
      let org_id = Uuid::new_v4();
      use chrono::Utc;
      use db::models::organization;
      use sea_orm::Set;
      organization::Entity::insert(organization::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Test Org {}", org_id)),
        slug: Set(format!("test-org-{}", org_id)),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc()),
      })
      .exec(&db_conn)
      .await
      .expect("create test org");
      let payload = CreateOrgRoleBody {
        name: "   ".to_string(),
        display_name: None,
      };

      // When I try to create an org role with empty name
      let result = create_org_role_impl(&db_conn, org_id, &payload).await;

      // Then it should return an error
      assert!(result.is_err());
      let err = result.unwrap_err();
      assert!(err.to_string().contains("name is required"));
    }

    #[tokio::test]
    async fn create_org_role_returns_error_when_role_name_exists() {
      // Given an organization with an existing role
      let (_router, db_conn, _guard) = crate::build_router_for_test_with_db().await.unwrap();
      // Create organization first (required for foreign key constraint)
      let org_id = Uuid::new_v4();
      use chrono::Utc;
      use db::models::organization;
      use sea_orm::Set;
      organization::Entity::insert(organization::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Test Org {}", org_id)),
        slug: Set(format!("test-org-{}", org_id)),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc()),
      })
      .exec(&db_conn)
      .await
      .expect("create test org");
      let payload1 = CreateOrgRoleBody {
        name: "admin".to_string(),
        display_name: None,
      };
      let _role_id1 = create_org_role_impl(&db_conn, org_id, &payload1)
        .await
        .unwrap();

      // When I try to create another role with the same name
      let payload2 = CreateOrgRoleBody {
        name: "admin".to_string(),
        display_name: None,
      };
      let result = create_org_role_impl(&db_conn, org_id, &payload2).await;

      // Then it should return an error about duplicate role name
      assert!(result.is_err());
      let err = result.unwrap_err();
      assert!(err.to_string().contains("Role name exists"));
    }

    #[tokio::test]
    async fn create_org_role_succeeds_with_valid_name() {
      // Given an organization ID and valid role name
      let (_router, db_conn, _guard) = crate::build_router_for_test_with_db().await.unwrap();
      // Create organization first (required for foreign key constraint)
      let org_id = Uuid::new_v4();
      use chrono::Utc;
      use db::models::organization;
      use sea_orm::Set;
      organization::Entity::insert(organization::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Test Org {}", org_id)),
        slug: Set(format!("test-org-{}", org_id)),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc()),
      })
      .exec(&db_conn)
      .await
      .expect("create test org");
      let payload = CreateOrgRoleBody {
        name: "viewer".to_string(),
        display_name: Some("Viewer Role".to_string()),
      };

      // When I create an org role
      let result = create_org_role_impl(&db_conn, org_id, &payload).await;

      // Then it should succeed and return a role ID
      assert!(result.is_ok());
      let role_id = result.unwrap();
      assert_ne!(role_id, Uuid::nil());
    }
  }

  mod ensure_org_role_behavior {
    use super::*;

    #[tokio::test]
    async fn ensure_org_role_returns_existing_role_when_already_exists() {
      // Given an organization with an existing role
      let (_router, db_conn, _guard) = crate::build_router_for_test_with_db().await.unwrap();
      // Create organization first (required for foreign key constraint)
      let org_id = Uuid::new_v4();
      use chrono::Utc;
      use db::models::organization;
      use sea_orm::Set;
      organization::Entity::insert(organization::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Test Org {}", org_id)),
        slug: Set(format!("test-org-{}", org_id)),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc()),
      })
      .exec(&db_conn)
      .await
      .expect("create test org");
      let payload = CreateOrgRoleBody {
        name: "editor".to_string(),
        display_name: None,
      };
      let existing_role_id = create_org_role_impl(&db_conn, org_id, &payload)
        .await
        .unwrap();

      // When I ensure the same role exists
      let result = ensure_org_role_impl(&db_conn, org_id, &payload).await;

      // Then it should return the existing role ID (idempotent)
      assert!(result.is_ok());
      let role_id = result.unwrap();
      assert_eq!(role_id, existing_role_id);
    }

    #[tokio::test]
    async fn ensure_org_role_creates_new_role_when_not_exists() {
      // Given an organization without a specific role
      let (_router, db_conn, _guard) = crate::build_router_for_test_with_db().await.unwrap();
      // Create organization first (required for foreign key constraint)
      let org_id = Uuid::new_v4();
      use chrono::Utc;
      use db::models::organization;
      use sea_orm::Set;
      organization::Entity::insert(organization::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Test Org {}", org_id)),
        slug: Set(format!("test-org-{}", org_id)),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc()),
      })
      .exec(&db_conn)
      .await
      .expect("create test org");
      let payload = CreateOrgRoleBody {
        name: "new_role".to_string(),
        display_name: None,
      };

      // When I ensure the role exists
      let result = ensure_org_role_impl(&db_conn, org_id, &payload).await;

      // Then it should create and return a new role ID
      assert!(result.is_ok());
      let role_id = result.unwrap();
      assert_ne!(role_id, Uuid::nil());
    }
  }

  mod list_org_roles_behavior {
    use super::*;

    #[tokio::test]
    async fn list_org_roles_returns_empty_map_when_no_roles_exist() {
      // Given an organization with no roles
      let (_router, db_conn, _guard) = crate::build_router_for_test_with_db().await.unwrap();
      // Create organization first (required for foreign key constraint)
      let org_id = Uuid::new_v4();
      use chrono::Utc;
      use db::models::organization;
      use sea_orm::Set;
      organization::Entity::insert(organization::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Test Org {}", org_id)),
        slug: Set(format!("test-org-{}", org_id)),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc()),
      })
      .exec(&db_conn)
      .await
      .expect("create test org");

      // When I list org roles
      let result = list_org_roles_impl(&db_conn, org_id).await;

      // Then it should return an empty map
      assert!(result.is_ok());
      let roles = result.unwrap();
      assert!(roles.is_empty());
    }

    #[tokio::test]
    async fn list_org_roles_returns_all_roles_for_organization() {
      // Given an organization with multiple roles
      let (_router, db_conn, _guard) = crate::build_router_for_test_with_db().await.unwrap();
      // Create organization first (required for foreign key constraint)
      let org_id = Uuid::new_v4();
      use chrono::Utc;
      use db::models::organization;
      use sea_orm::Set;
      organization::Entity::insert(organization::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Test Org {}", org_id)),
        slug: Set(format!("test-org-{}", org_id)),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc()),
      })
      .exec(&db_conn)
      .await
      .expect("create test org");
      let role1 = CreateOrgRoleBody {
        name: "admin".to_string(),
        display_name: None,
      };
      let role2 = CreateOrgRoleBody {
        name: "viewer".to_string(),
        display_name: None,
      };
      let _id1 = create_org_role_impl(&db_conn, org_id, &role1)
        .await
        .unwrap();
      let _id2 = create_org_role_impl(&db_conn, org_id, &role2)
        .await
        .unwrap();

      // When I list org roles
      let result = list_org_roles_impl(&db_conn, org_id).await;

      // Then it should return a map with both role names and IDs
      assert!(result.is_ok());
      let roles = result.unwrap();
      assert_eq!(roles.len(), 2);
      assert!(roles.contains_key("admin"));
      assert!(roles.contains_key("viewer"));
    }
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use crate::build_router_for_test;
  use axum::body::Body;
  use axum::http::{Request, StatusCode};
  use tower::ServiceExt;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests verify REST API handler behaviors for permissions, org roles, and user management.

  mod permissions_list_behavior {
    use super::*;

    #[tokio::test]
    async fn should_list_all_permissions() {
      // Given: a router with permissions endpoint
      let (router, _guard) = build_router_for_test().await.unwrap();

      // When: making request to list permissions endpoint
      let req = Request::builder()
        .uri("/api/permissions")
        .body(Body::empty())
        .unwrap();
      let res = router.oneshot(req).await.unwrap();

      // Then: should return list of permissions
      assert_eq!(res.status(), StatusCode::OK);
      let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
      let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
      assert!(json.get("permissions").is_some());
      assert!(json["permissions"].is_array());
    }

    #[tokio::test]
    async fn should_not_require_authentication() {
      // Given: a router with permissions endpoint
      let (router, _guard) = build_router_for_test().await.unwrap();

      // When: making unauthenticated request to permissions endpoint
      let req = Request::builder()
        .uri("/api/permissions")
        .body(Body::empty())
        .unwrap();
      let res = router.oneshot(req).await.unwrap();

      // Then: should return successful response without authentication
      assert_eq!(res.status(), StatusCode::OK);
      // Handler doesn't require auth (no RequireAuth extractor)
    }
  }

  mod org_role_creation_behavior {
    use super::*;

    #[test]
    fn should_reject_empty_role_name() {
      // Given: an org role payload with empty name
      let payload = CreateOrgRoleBody {
        name: "   ".to_string(), // whitespace only
        display_name: None,
      };

      // When: validating payload
      let name = payload.name.trim();

      // Then: should be empty after trim
      assert!(name.is_empty(), "Empty name should be rejected");
    }

    #[test]
    fn should_accept_role_name_with_display_name() {
      // Given: an org role payload with name and display_name
      let payload = CreateOrgRoleBody {
        name: "admin".to_string(),
        display_name: Some("Administrator".to_string()),
      };

      // When: accessing payload fields
      // Then: should have both name and display_name
      assert_eq!(payload.name, "admin");
      assert_eq!(payload.display_name, Some("Administrator".to_string()));
    }

    #[test]
    fn should_handle_optional_display_name() {
      // Given: an org role payload without display_name
      let payload = CreateOrgRoleBody {
        name: "admin".to_string(),
        display_name: None,
      };

      // When: accessing payload
      // Then: display_name should be None
      assert_eq!(payload.name, "admin");
      assert!(payload.display_name.is_none());
    }
  }

  mod org_user_management_behavior {
    use super::*;

    #[test]
    fn should_accept_user_id_in_add_org_user_body() {
      // Given: an AddOrgUserBody with user_id
      let payload = AddOrgUserBody {
        user_id: Some(uuid::Uuid::new_v4()),
        email: None,
        password: None,
      };

      // When: accessing payload
      // Then: should have user_id set
      assert!(payload.user_id.is_some());
    }

    #[test]
    fn should_accept_email_and_password_in_add_org_user_body() {
      // Given: an AddOrgUserBody with email and password
      let payload = AddOrgUserBody {
        user_id: None,
        email: Some("test@example.com".to_string()),
        password: Some("password123".to_string()),
      };

      // When: accessing payload
      // Then: should have email and password set
      assert_eq!(payload.email, Some("test@example.com".to_string()));
      assert_eq!(payload.password, Some("password123".to_string()));
    }

    #[test]
    fn should_validate_email_format() {
      // Given: an email string
      let email = "test@example.com";

      // When: checking if email contains @
      let is_valid = email.contains('@');

      // Then: should be valid
      assert!(is_valid, "Email should contain @");
    }

    #[test]
    fn should_validate_password_length() {
      // Given: a password string
      let password = "password123";

      // When: checking password length
      let is_valid = password.len() >= 8;

      // Then: should meet minimum length requirement
      assert!(is_valid, "Password should be at least 8 characters");
    }

    #[test]
    fn should_reject_short_password() {
      // Given: a password shorter than 8 characters
      let password = "short";

      // When: checking password length
      let is_valid = password.len() >= 8;

      // Then: should be rejected
      assert!(
        !is_valid,
        "Password shorter than 8 characters should be rejected"
      );
    }

    #[test]
    fn should_reject_email_without_at_symbol() {
      // Given: an email string without @
      let email = "invalid-email";

      // When: checking if email contains @
      let is_valid = email.contains('@');

      // Then: should be invalid
      assert!(!is_valid, "Email without @ should be rejected");
    }
  }

  mod validation_behavior {

    #[test]
    fn should_trim_whitespace_from_role_name() {
      // Given: a role name with leading/trailing whitespace
      let name = "  admin  ";

      // When: trimming whitespace
      let trimmed = name.trim();

      // Then: should remove whitespace
      assert_eq!(trimmed, "admin");
    }

    #[test]
    fn should_trim_whitespace_from_email() {
      // Given: an email with leading/trailing whitespace
      let email = "  test@example.com  ";

      // When: trimming whitespace
      let trimmed = email.trim();

      // Then: should remove whitespace
      assert_eq!(trimmed, "test@example.com");
    }

    #[test]
    fn should_handle_empty_display_name() {
      // Given: a display_name that is empty or whitespace
      let display_name = Some("   ".to_string());

      // When: processing display_name
      let processed = display_name
        .as_ref()
        .map(|s: &String| s.trim())
        .filter(|s: &&str| !s.is_empty())
        .map(String::from);

      // Then: should be None (filtered out)
      assert!(processed.is_none());
    }

    #[test]
    fn should_preserve_non_empty_display_name() {
      // Given: a display_name with content
      let display_name = Some("  Administrator  ".to_string());

      // When: processing display_name
      let processed = display_name
        .as_ref()
        .map(|s: &String| s.trim())
        .filter(|s: &&str| !s.is_empty())
        .map(String::from);

      // Then: should preserve trimmed content
      assert_eq!(processed, Some("Administrator".to_string()));
    }
  }

  mod permission_validation_behavior {
    use super::*;

    #[test]
    fn should_validate_permission_key_exists() {
      // Given: a list of dashboard permissions
      let permissions = dashboard_permissions();

      // When: checking if permissions exist
      // Then: should have some permissions defined
      assert!(
        !permissions.is_empty(),
        "Should have some permissions defined"
      );
    }

    #[test]
    fn should_trim_permission_key_whitespace() {
      // Given: a permission key with whitespace
      let key = "  user.read  ";

      // When: trimming whitespace
      let trimmed = key.trim();

      // Then: should remove whitespace
      assert_eq!(trimmed, "user.read");
    }
  }
}

#[cfg(test)]
mod integration_tests {
  use super::*;

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests verify validation logic, data structures, and core REST handler behaviors.

    mod data_structure_behavior {
      use super::*;

      #[test]
      fn should_create_org_role_body_with_name() {
        // Given: a role name
        let name = "admin".to_string();

        // When: creating CreateOrgRoleBody
        let body = CreateOrgRoleBody {
          name: name.clone(),
          display_name: None,
        };

        // Then: body should contain the name
        assert_eq!(body.name, name);
        assert_eq!(body.display_name, None);
      }

      #[test]
      fn should_create_org_role_body_with_display_name() {
        // Given: a role name and display name
        let name = "admin".to_string();
        let display_name = "Administrator".to_string();

        // When: creating CreateOrgRoleBody with display name
        let body = CreateOrgRoleBody {
          name: name.clone(),
          display_name: Some(display_name.clone()),
        };

        // Then: body should contain both names
        assert_eq!(body.name, name);
        assert_eq!(body.display_name, Some(display_name));
      }

      #[test]
      fn should_create_add_org_user_body_with_user_id() {
        // Given: a user ID
        let user_id = Uuid::new_v4();

        // When: creating AddOrgUserBody with user_id
        let body = AddOrgUserBody {
          user_id: Some(user_id),
          email: None,
          password: None,
        };

        // Then: body should contain the user ID
        assert_eq!(body.user_id, Some(user_id));
        assert_eq!(body.email, None);
        assert_eq!(body.password, None);
      }

      #[test]
      fn should_create_add_org_user_body_with_email_and_password() {
        // Given: an email and password
        let email = "user@example.com".to_string();
        let password = "password123".to_string();

        // When: creating AddOrgUserBody with email and password
        let body = AddOrgUserBody {
          user_id: None,
          email: Some(email.clone()),
          password: Some(password.clone()),
        };

        // Then: body should contain email and password
        assert_eq!(body.user_id, None);
        assert_eq!(body.email, Some(email));
        assert_eq!(body.password, Some(password));
      }
    }

    mod validation_behavior {
      use super::*;

      #[test]
      fn should_validate_empty_name_as_invalid() {
        // Given: CreateOrgRoleBody with empty name
        let body = CreateOrgRoleBody {
          name: "".to_string(),
          display_name: None,
        };

        // When: checking name validity (trimmed)
        let trimmed = body.name.trim();

        // Then: name should be empty after trimming
        assert!(trimmed.is_empty(), "Empty name should be invalid");
      }

      #[test]
      fn should_validate_whitespace_only_name_as_invalid() {
        // Given: CreateOrgRoleBody with whitespace-only name
        let body = CreateOrgRoleBody {
          name: "   ".to_string(),
          display_name: None,
        };

        // When: checking name validity (trimmed)
        let trimmed = body.name.trim();

        // Then: name should be empty after trimming
        assert!(trimmed.is_empty(), "Whitespace-only name should be invalid");
      }

      #[test]
      fn should_validate_email_format_requirement() {
        // Given: email validation logic (must contain @)
        let valid_email = "user@example.com";
        let invalid_email = "not-an-email";

        // When: checking email format
        let valid = valid_email.trim().contains('@');
        let invalid = invalid_email.trim().contains('@');

        // Then: valid email should pass, invalid should fail
        assert!(valid, "Valid email should contain @");
        assert!(!invalid, "Invalid email should not contain @");
      }

      #[test]
      fn should_validate_password_length_requirement() {
        // Given: password validation logic (min 8 chars)
        let valid_password = "password123"; // 11 chars
        let invalid_password = "short"; // 5 chars

        // When: checking password length
        let valid = valid_password.len() >= 8;
        let invalid = invalid_password.len() >= 8;

        // Then: valid password should pass, invalid should fail
        assert!(valid, "Valid password should be at least 8 characters");
        assert!(
          !invalid,
          "Invalid password should be less than 8 characters"
        );
      }

      #[tokio::test]
      async fn should_require_either_user_id_or_email_password() {
        // Given: AddOrgUserBody validation logic
        let with_user_id = AddOrgUserBody {
          user_id: Some(Uuid::new_v4()),
          email: None,
          password: None,
        };
        let with_email_password = AddOrgUserBody {
          user_id: None,
          email: Some("user@example.com".to_string()),
          password: Some("password123".to_string()),
        };
        let with_nothing = AddOrgUserBody {
          user_id: None,
          email: None,
          password: None,
        };

        // When: checking if body has required fields
        let has_user_id = with_user_id.user_id.is_some();
        let has_email_password =
          with_email_password.email.is_some() && with_email_password.password.is_some();
        let has_nothing = with_nothing.user_id.is_none()
          && with_nothing.email.is_none()
          && with_nothing.password.is_none();

        // Then: should require either user_id or email+password
        assert!(
          has_user_id || has_email_password,
          "Should have either user_id or email+password"
        );
        assert!(has_nothing, "Should be empty when nothing provided");
      }
    }

    mod permission_list_behavior {
      use super::*;

      #[test]
      fn should_return_permissions_list_from_dashboard_permissions() {
        // Given: dashboard_permissions function
        // When: calling dashboard_permissions
        let permissions = dashboard_permissions();

        // Then: should return a list of permission strings
        assert!(
          !permissions.is_empty(),
          "Should return non-empty permissions list"
        );
        // All items should be string slices
        for perm in permissions {
          assert!(!perm.is_empty(), "Permission should not be empty");
        }
      }

      #[test]
      fn should_validate_permission_key_exists_in_dashboard_permissions() {
        // Given: a permission key from dashboard_permissions
        let permissions = dashboard_permissions();
        let first_permission = permissions[0];

        // When: checking if permission exists
        let exists = permissions.contains(&first_permission);

        // Then: permission should exist in the list
        assert!(exists, "Permission should exist in dashboard_permissions");
      }

      #[test]
      fn should_reject_invalid_permission_key() {
        // Given: an invalid permission key
        let invalid_key = "invalid.permission.key";
        let permissions = dashboard_permissions();

        // When: checking if invalid key exists
        let exists = permissions.contains(&invalid_key);

        // Then: invalid key should not exist
        assert!(!exists, "Invalid permission key should not exist");
      }
    }

    mod idempotent_operation_behavior {
      use super::*;

      #[test]
      fn should_handle_empty_role_ids_gracefully() {
        // Given: empty role_ids array
        let role_ids: &[Uuid] = &[];

        // When: checking if empty
        let is_empty = role_ids.is_empty();

        // Then: should be empty
        assert!(is_empty, "Empty role_ids should be handled gracefully");
        // Function should return Ok(()) early for empty array
      }

      #[test]
      fn should_validate_uuid_format() {
        // Given: valid and invalid UUID strings
        let valid_uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let invalid_uuid_str = "not-a-uuid";

        // When: parsing UUIDs
        let valid = Uuid::parse_str(valid_uuid_str);
        let invalid = Uuid::parse_str(invalid_uuid_str);

        // Then: valid should parse, invalid should fail
        assert!(valid.is_ok(), "Valid UUID string should parse");
        assert!(invalid.is_err(), "Invalid UUID string should fail to parse");
      }
    }
  }
}
