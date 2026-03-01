use chrono::Utc;
use forge::auth::hash_password;
use forge::DbConnection;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::info;
use uuid::Uuid;

use crate::models::{organization, membership, user};

const SEED_PASSWORD: &str = "password";

/// Ensures an organization exists by slug; returns its id.
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
      Ok(id)
    }
  }
}

/// Creates a user and their memberships if the user does not exist.
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

  for (org_id, role) in memberships.iter() {
    let membership_id = Uuid::new_v4();
    let m = membership::ActiveModel {
      id: Set(membership_id),
      user_id: Set(user_id),
      org_id: Set(*org_id),
      role: Set((*role).to_string()),
      created_at: Set(now),
      updated_at: Set(now),
    };
    membership::Entity::insert(m).exec(db).await?;
  }

  info!("Seeded user: {} (is_admin={}, role={})", email, is_admin, current_role);
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
