use forge_db::DbConnection;

pub mod auth;
pub mod model_error;
pub mod models;
pub mod organization;
pub mod query_spec;
pub mod registry;
pub mod rest_model;

/// Default permission keys per org role name. Used by [seed_role_permissions_for_org] and by migrations seeds.
const ORG_OWNER_ADMIN: &[&str] = &[
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
  "all.read",
  "all.write",
];
const ORG_EDITOR: &[&str] = &[
  "dashboard",
  "dashboard.users.read",
  "dashboard.users.write",
  "dashboard.permissions.read",
  "dashboard.permissions.write",
  "dashboard.roles.read",
  "dashboard.roles.write",
];
const ORG_VIEWER: &[&str] = &[
  "dashboard",
  "dashboard.users.read",
  "dashboard.audit.read",
  "dashboard.permissions.read",
  "dashboard.roles.read",
];

/// Inserts default role_permission rows for an org (owner, admin, editor, viewer).
pub async fn seed_role_permissions_for_org<C: sea_orm::ConnectionTrait>(
  db: &C,
  org_id: uuid::Uuid,
) -> Result<(), sea_orm::DbErr> {
  use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
  let roles = crate::models::org_role::Entity::find()
    .filter(crate::models::org_role::Column::OrgId.eq(org_id))
    .all(db)
    .await?;
  for role in roles {
    let keys: &[&str] = match role.name.as_str() {
      "owner" | "admin" => ORG_OWNER_ADMIN,
      "editor" => ORG_EDITOR,
      "viewer" => ORG_VIEWER,
      _ => continue,
    };
    for key in keys {
      let exists = crate::models::role_permission::Entity::find()
        .filter(crate::models::role_permission::Column::OrgId.eq(Some(org_id)))
        .filter(crate::models::role_permission::Column::RoleName.eq(&role.name))
        .filter(crate::models::role_permission::Column::PermissionKey.eq(*key))
        .one(db)
        .await?;
      if exists.is_some() {
        continue;
      }
      let id = uuid::Uuid::new_v4();
      let row = crate::models::role_permission::ActiveModel {
        id: Set(id),
        scope: Set("org".to_string()),
        role_name: Set(role.name.clone()),
        permission_key: Set((*key).to_string()),
        org_id: Set(Some(org_id)),
        ..Default::default()
      };
      crate::models::role_permission::Entity::insert(row)
        .exec(db)
        .await?;
    }
  }
  Ok(())
}

/// Look up user id by raw API token (Bearer). Returns None if token invalid or expired.
pub async fn token_lookup(db: DbConnection, raw_token: String) -> Option<uuid::Uuid> {
  use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
  let hash = forge_auth::token_auth::hash_api_token(&raw_token);
  let row = crate::models::api_token::Entity::find()
    .filter(crate::models::api_token::Column::TokenHash.eq(hash))
    .one(&db)
    .await
    .ok()
    .flatten()?;
  if let Some(exp) = row.expires_at {
    if exp < chrono::Utc::now().naive_utc() {
      return None;
    }
  }
  Some(row.user_id)
}

#[cfg(test)]
mod tests {
  use super::*;
  use sea_orm::{Database, EntityTrait, Set};

  mod bdd_tests {
    use super::*;
    use chrono::Utc;
    use sea_orm_migration::MigratorTrait;
    use uuid::Uuid;

    async fn test_db() -> forge_db::DbConnection {
      let conn = Database::connect(sea_orm::ConnectOptions::new(
        "sqlite::memory:".to_string(),
      ))
      .await
      .unwrap();
      migrations::Migrator::up(&conn, None).await.expect("migrate");
      forge_db::wrap_traced(conn)
    }

    mod seed_role_permissions_for_org_behavior {
      use super::*;

      #[tokio::test]
      async fn should_insert_role_permissions_for_owner_admin_editor_viewer() {
        // Given: a DB with an organization and org_roles owner, admin, editor, viewer
        let db = test_db().await;
        let org_id = Uuid::new_v4();
        let now = Utc::now().naive_utc();
        crate::models::organization::Entity::insert(
          crate::models::organization::ActiveModel {
            id: Set(org_id),
            name: Set("Test Org".to_string()),
            slug: Set("test-org".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
          },
        )
        .exec(&db)
        .await
        .expect("insert org");
        for name in ["owner", "admin", "editor", "viewer"] {
          crate::models::org_role::Entity::insert(
            crate::models::org_role::ActiveModel {
              id: Set(Uuid::new_v4()),
              org_id: Set(org_id),
              name: Set(name.to_string()),
              display_name: Set(None),
              created_at: Set(now),
              updated_at: Set(now),
              ..Default::default()
            },
          )
          .exec(&db)
          .await
          .expect("insert org_role");
        }
        // When: calling seed_role_permissions_for_org
        seed_role_permissions_for_org(&db, org_id)
          .await
          .expect("seed");
        // Then: role_permission rows exist for that org (e.g. owner has "dashboard")
        let perms = crate::models::role_permission::Entity::find()
          .filter(crate::models::role_permission::Column::OrgId.eq(Some(org_id)))
          .filter(crate::models::role_permission::Column::RoleName.eq("owner"))
          .all(&db)
          .await
          .expect("find");
        let keys: std::collections::HashSet<_> =
          perms.iter().map(|p| p.permission_key.as_str()).collect();
        assert!(keys.contains("dashboard"), "owner should have dashboard");
        assert!(
          keys.contains("dashboard.organizations.read"),
          "owner should have dashboard.organizations.read"
        );
      }

      #[tokio::test]
      async fn should_not_duplicate_permissions_on_second_call() {
        // Given: a DB with org and roles, and seed already run once
        let db = test_db().await;
        let org_id = Uuid::new_v4();
        let now = Utc::now().naive_utc();
        crate::models::organization::Entity::insert(
          crate::models::organization::ActiveModel {
            id: Set(org_id),
            name: Set("Test Org".to_string()),
            slug: Set("test-org".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
          },
        )
        .exec(&db)
        .await
        .expect("insert org");
        for name in ["owner", "admin", "editor", "viewer"] {
          crate::models::org_role::Entity::insert(
            crate::models::org_role::ActiveModel {
              id: Set(Uuid::new_v4()),
              org_id: Set(org_id),
              name: Set(name.to_string()),
              display_name: Set(None),
              created_at: Set(now),
              updated_at: Set(now),
              ..Default::default()
            },
          )
          .exec(&db)
          .await
          .expect("insert org_role");
        }
        seed_role_permissions_for_org(&db, org_id)
          .await
          .expect("seed first");
        let count_before = crate::models::role_permission::Entity::find()
          .filter(crate::models::role_permission::Column::OrgId.eq(Some(org_id)))
          .all(&db)
          .await
          .expect("find")
          .len();
        // When: calling seed_role_permissions_for_org again
        seed_role_permissions_for_org(&db, org_id)
          .await
          .expect("seed second");
        let count_after = crate::models::role_permission::Entity::find()
          .filter(crate::models::role_permission::Column::OrgId.eq(Some(org_id)))
          .all(&db)
          .await
          .expect("find")
          .len();
        // Then: count unchanged (idempotent)
        assert_eq!(count_before, count_after);
      }
    }

    mod token_lookup_behavior {
      use super::*;
      use crate::models::api_token;

      #[tokio::test]
      async fn should_return_some_user_id_when_token_exists_and_not_expired() {
        // Given: a DB with an api_token row for a raw token
        let db = test_db().await;
        let user_id = Uuid::new_v4();
        let raw_token = "test-token-secret-12345";
        let hash = forge_auth::token_auth::hash_api_token(raw_token);
        let now = Utc::now().naive_utc();
        api_token::Entity::insert(api_token::ActiveModel {
          id: Set(Uuid::new_v4()),
          user_id: Set(user_id),
          token_hash: Set(hash),
          name: Set(Some("test".to_string())),
          last_used_at: Set(None),
          expires_at: Set(None),
          created_at: Set(now),
          updated_at: Set(now),
          ..Default::default()
        })
        .exec(&db)
        .await
        .expect("insert token");
        // When: calling token_lookup with that raw token
        let result = token_lookup(db.clone(), raw_token.to_string()).await;
        // Then: returns Some(user_id)
        assert_eq!(result, Some(user_id));
      }

      #[tokio::test]
      async fn should_return_none_when_token_expired() {
        // Given: a DB with an api_token row with expires_at in the past
        let db = test_db().await;
        let user_id = Uuid::new_v4();
        let raw_token = "expired-token";
        let hash = forge_auth::token_auth::hash_api_token(raw_token);
        let now = Utc::now().naive_utc();
        let past = now - chrono::Duration::days(1);
        api_token::Entity::insert(api_token::ActiveModel {
          id: Set(Uuid::new_v4()),
          user_id: Set(user_id),
          token_hash: Set(hash),
          name: Set(Some("test".to_string())),
          last_used_at: Set(None),
          expires_at: Set(Some(past)),
          created_at: Set(now),
          updated_at: Set(now),
          ..Default::default()
        })
        .exec(&db)
        .await
        .expect("insert token");
        // When: calling token_lookup with that raw token
        let result = token_lookup(db.clone(), raw_token.to_string()).await;
        // Then: returns None
        assert_eq!(result, None);
      }

      #[tokio::test]
      async fn should_return_none_when_token_unknown() {
        // Given: a DB with no matching api_token
        let db = test_db().await;
        // When: calling token_lookup with an unknown raw token
        let result =
          token_lookup(db.clone(), "unknown-token-never-inserted".to_string()).await;
        // Then: returns None
        assert_eq!(result, None);
      }
    }
  }
}
