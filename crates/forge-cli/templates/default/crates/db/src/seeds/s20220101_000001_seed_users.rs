use chrono::Utc;
use forge::DbConnection;
use forge::auth::hash_password;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tracing::info;
use uuid::Uuid;

use crate::models::{membership, org_role, organization, user, user_org_role};

const SEED_PASSWORD: &str = "password";
const TEMPLATE_ROLES: &[&str] = &["owner", "admin", "editor", "viewer"];

/// Ensures the four template org_roles exist for an org (e.g. after creating the org).
async fn ensure_org_roles(db: &DbConnection, org_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
  let count = org_role::Entity::find()
    .filter(org_role::Column::OrgId.eq(org_id))
    .count(db)
    .await?;
  if count > 0 {
    return Ok(());
  }
  let now = Utc::now().naive_utc();
  for &name in TEMPLATE_ROLES {
    let id = Uuid::new_v4();
    let model = org_role::ActiveModel {
      id: Set(id),
      org_id: Set(org_id),
      name: Set(name.to_string()),
      display_name: Set(None),
      created_at: Set(now),
      updated_at: Set(now),
      ..Default::default()
    };
    org_role::Entity::insert(model).exec(db).await?;
  }
  Ok(())
}

/// Ensures an organization exists by slug; returns its id. Creates org_roles when creating the org.
async fn ensure_org(
  db: &DbConnection,
  name: &str,
  slug: &str,
) -> Result<Uuid, Box<dyn std::error::Error>> {
  let existing = organization::Entity::find()
    .filter(organization::Column::Slug.eq(slug))
    .one(db)
    .await?;

  match existing {
    Some(org) => Ok(org.id),
    None => {
      let now = Utc::now().naive_utc();
      let id = Uuid::new_v4();
      let o = organization::ActiveModel {
        id: Set(id),
        name: Set(name.to_string()),
        slug: Set(slug.to_string()),
        created_at: Set(now),
        updated_at: Set(now),
      };
      organization::Entity::insert(o).exec(db).await?;
      ensure_org_roles(db, id).await?;
      Ok(id)
    }
  }
}

/// Creates a user and their memberships + role assignments if the user does not exist.
/// Every user has at least one org (first in `memberships` = personal org). Session profile is source of truth for active org/role.
async fn ensure_user(
  db: &DbConnection,
  email: &str,
  is_admin: bool,
  current_org_id: Uuid,
  current_role: &str,
  memberships: &[(Uuid, &str)],
) -> Result<(), Box<dyn std::error::Error>> {
  let existing = user::Entity::find()
    .filter(user::Column::Email.eq(email))
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
    email: Set(email.to_string()),
    password_hash: Set(password_hash),
    is_active: Set(true),
    is_admin: Set(is_admin),
    current_org_id: Set(Some(current_org_id)),
    current_role: Set(Some(current_role.to_string())),
    created_at: Set(now),
    updated_at: Set(now),
  };
  user::Entity::insert(user_model).exec(db).await?;

  for (org_id, role_name) in memberships.iter() {
    let membership_exists = membership::Entity::find()
      .filter(membership::Column::UserId.eq(user_id))
      .filter(membership::Column::OrgId.eq(*org_id))
      .one(db)
      .await?;
    if membership_exists.is_none() {
      let membership_id = Uuid::new_v4();
      let m = membership::ActiveModel {
        id: Set(membership_id),
        user_id: Set(user_id),
        org_id: Set(*org_id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      };
      membership::Entity::insert(m).exec(db).await?;
    }

    let role_row = org_role::Entity::find()
      .filter(org_role::Column::OrgId.eq(*org_id))
      .filter(org_role::Column::Name.eq(*role_name))
      .one(db)
      .await?;
    if let Some(role) = role_row {
      let uor_id = Uuid::new_v4();
      let uor = user_org_role::ActiveModel {
        id: Set(uor_id),
        user_id: Set(user_id),
        org_id: Set(*org_id),
        role_id: Set(role.id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      };
      user_org_role::Entity::insert(uor).exec(db).await?;
    }
  }

  info!(
    "Seeded user: {} (is_admin={}, role={})",
    email, is_admin, current_role
  );
  Ok(())
}

pub async fn seed(db: &DbConnection) -> Result<(), Box<dyn std::error::Error>> {
  let default_org_id = ensure_org(db, "Default", "default").await?;
  let other_org_id = ensure_org(db, "Other", "other").await?;

  // Global admin: can access app-wide admin panel; owner in Default org.
  ensure_user(
    db,
    "admin@admin.com",
    true,
    default_org_id,
    "owner",
    &[(default_org_id, "owner")],
  )
  .await?;

  // Default org: owner, orgadmin, editor, viewer.
  ensure_user(
    db,
    "owner@default.org",
    false,
    default_org_id,
    "owner",
    &[(default_org_id, "owner")],
  )
  .await?;

  ensure_user(
    db,
    "orgadmin@default.org",
    false,
    default_org_id,
    "admin",
    &[(default_org_id, "admin")],
  )
  .await?;

  ensure_user(
    db,
    "editor@default.org",
    false,
    default_org_id,
    "editor",
    &[(default_org_id, "editor")],
  )
  .await?;

  ensure_user(
    db,
    "viewer@default.org",
    false,
    default_org_id,
    "viewer",
    &[(default_org_id, "viewer")],
  )
  .await?;

  // Other org: owner, orgadmin, viewer (for testing second org and multi-org switching).
  ensure_user(
    db,
    "owner@other.org",
    false,
    other_org_id,
    "owner",
    &[(other_org_id, "owner")],
  )
  .await?;

  ensure_user(
    db,
    "orgadmin@other.org",
    false,
    other_org_id,
    "admin",
    &[(other_org_id, "admin")],
  )
  .await?;

  ensure_user(
    db,
    "viewer@other.org",
    false,
    other_org_id,
    "viewer",
    &[(other_org_id, "viewer")],
  )
  .await?;

  // Multi-org: viewer in Default, editor in Other (tests org switching).
  ensure_user(
    db,
    "multi@email.com",
    false,
    default_org_id,
    "viewer",
    &[(default_org_id, "viewer"), (other_org_id, "editor")],
  )
  .await?;

  Ok(())
}
