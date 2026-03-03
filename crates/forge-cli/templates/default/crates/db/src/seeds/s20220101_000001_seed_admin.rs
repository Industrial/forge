//! First seed: Admin organization, admin role with all.read and all.write, and the admin user.
//! Unique to the platform admin; other orgs (Default, Other, Personal) are created in the next seed.

use chrono::Utc;
use forge_db::DbConnection;
use forge_auth::token_auth::hash_password;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::info;
use uuid::Uuid;

use crate::models::{membership, org_role, organization, role_permission, user, user_org_role};

const SEED_PASSWORD: &str = "password";
const ADMIN_ORG_NAME: &str = "Admin";
const ADMIN_ORG_SLUG: &str = "admin";
const ADMIN_ROLE_NAME: &str = "admin";
const ADMIN_EMAIL: &str = "admin@admin.com";

/// Permissions unique to the Admin org role: full read and write.
const ADMIN_PERMISSIONS: &[&str] = &["all.read", "all.write"];

pub async fn seed(db: &DbConnection) -> Result<(), Box<dyn std::error::Error>> {
  let admin_org_id = ensure_admin_org(db).await?;
  let admin_role_id = ensure_admin_role(db, admin_org_id).await?;
  ensure_admin_role_permissions(db, admin_org_id).await?;
  ensure_admin_user(db, admin_org_id, admin_role_id).await?;
  Ok(())
}

async fn ensure_admin_org(db: &DbConnection) -> Result<Uuid, Box<dyn std::error::Error>> {
  let existing = organization::Entity::find()
    .filter(organization::Column::Slug.eq(ADMIN_ORG_SLUG))
    .one(db)
    .await?;

  match existing {
    Some(org) => Ok(org.id),
    None => {
      let now = Utc::now().naive_utc();
      let id = Uuid::new_v4();
      let o = organization::ActiveModel {
        id: Set(id),
        name: Set(ADMIN_ORG_NAME.to_string()),
        slug: Set(ADMIN_ORG_SLUG.to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      };
      organization::Entity::insert(o).exec(db).await?;
      info!("Seeded organization: {} ({})", ADMIN_ORG_NAME, ADMIN_ORG_SLUG);
      Ok(id)
    }
  }
}

async fn ensure_admin_role(db: &DbConnection, org_id: Uuid) -> Result<Uuid, Box<dyn std::error::Error>> {
  let existing = org_role::Entity::find()
    .filter(org_role::Column::OrgId.eq(org_id))
    .filter(org_role::Column::Name.eq(ADMIN_ROLE_NAME))
    .one(db)
    .await?;

  match existing {
    Some(role) => Ok(role.id),
    None => {
      let now = Utc::now().naive_utc();
      let id = Uuid::new_v4();
      let model = org_role::ActiveModel {
        id: Set(id),
        org_id: Set(org_id),
        name: Set(ADMIN_ROLE_NAME.to_string()),
        display_name: Set(Some("Admin".to_string())),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      };
      org_role::Entity::insert(model).exec(db).await?;
      info!("Seeded org_role: {} in Admin org", ADMIN_ROLE_NAME);
      Ok(id)
    }
  }
}

async fn ensure_admin_role_permissions(db: &DbConnection, org_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
  for &key in ADMIN_PERMISSIONS {
    let exists = role_permission::Entity::find()
      .filter(role_permission::Column::Scope.eq("org"))
      .filter(role_permission::Column::OrgId.eq(org_id))
      .filter(role_permission::Column::RoleName.eq(ADMIN_ROLE_NAME))
      .filter(role_permission::Column::PermissionKey.eq(key))
      .one(db)
      .await?;
    if exists.is_none() {
      let id = Uuid::new_v4();
      let model = role_permission::ActiveModel {
        id: Set(id),
        scope: Set("org".to_string()),
        role_name: Set(ADMIN_ROLE_NAME.to_string()),
        permission_key: Set(key.to_string()),
        org_id: Set(Some(org_id)),
        ..Default::default()
      };
      role_permission::Entity::insert(model).exec(db).await?;
      info!("Seeded role_permission: org {} role={} key={}", org_id, ADMIN_ROLE_NAME, key);
    }
  }
  Ok(())
}

async fn ensure_admin_user(
  db: &DbConnection,
  org_id: Uuid,
  role_id: Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
  let existing = user::Entity::find()
    .filter(user::Column::Email.eq(ADMIN_EMAIL))
    .one(db)
    .await?;

  if existing.is_some() {
    return Ok(());
  }

  let now = Utc::now().naive_utc();
  let user_id = Uuid::new_v4();
  let password_hash = hash_password(SEED_PASSWORD)?;

  let user_model = user::ActiveModel {
    id: Set(user_id),
    email: Set(ADMIN_EMAIL.to_string()),
    password_hash: Set(password_hash),
    is_active: Set(true),
    is_admin: Set(true),
    current_org_id: Set(Some(org_id)),
    current_role: Set(Some(ADMIN_ROLE_NAME.to_string())),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  user::Entity::insert(user_model).exec(db).await?;

  let membership_id = Uuid::new_v4();
  let m = membership::ActiveModel {
    id: Set(membership_id),
    user_id: Set(user_id),
    org_id: Set(org_id),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  membership::Entity::insert(m).exec(db).await?;

  let uor_id = Uuid::new_v4();
  let uor = user_org_role::ActiveModel {
    id: Set(uor_id),
    user_id: Set(user_id),
    org_id: Set(org_id),
    role_id: Set(role_id),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  user_org_role::Entity::insert(uor).exec(db).await?;

  info!(
    "Seeded admin user: {} (is_admin=true, org={}, role={})",
    ADMIN_EMAIL, ADMIN_ORG_NAME, ADMIN_ROLE_NAME
  );
  Ok(())
}
