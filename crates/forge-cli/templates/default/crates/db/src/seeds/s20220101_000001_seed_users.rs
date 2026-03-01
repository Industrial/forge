use chrono::Utc;
use forge::auth::hash_password;
use forge::DbConnection;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::info;
use uuid::Uuid;

use crate::models::{organization, membership, user};

pub async fn seed(db: &DbConnection) -> Result<(), Box<dyn std::error::Error>> {
  let email = "root@localhost";

  let existing = user::Entity::find()
    .filter(user::Column::Email.eq(email))
    .one(db)
    .await?;

  if existing.is_none() {
    let now = Utc::now().naive_utc();
    let user_id = Uuid::new_v4();
    let org_id = Uuid::new_v4();
    let membership_id = Uuid::new_v4();
    let password_hash = hash_password("password123")?;

    let root_user = user::ActiveModel {
      id: Set(user_id),
      email: Set(email.to_owned()),
      password_hash: Set(password_hash),
      is_active: Set(true),
      is_admin: Set(true),
      current_org_id: Set(Some(org_id)),
      current_role: Set(Some("owner".to_string())),
      created_at: Set(now),
      updated_at: Set(now),
    };
    user::Entity::insert(root_user).exec(db).await?;

    let default_org = organization::ActiveModel {
      id: Set(org_id),
      name: Set("Default".to_string()),
      slug: Set("default".to_string()),
      created_at: Set(now),
      updated_at: Set(now),
    };
    organization::Entity::insert(default_org).exec(db).await?;

    let root_membership = membership::ActiveModel {
      id: Set(membership_id),
      user_id: Set(user_id),
      org_id: Set(org_id),
      role: Set("owner".to_string()),
      created_at: Set(now),
      updated_at: Set(now),
    };
    membership::Entity::insert(root_membership).exec(db).await?;

    info!("Seeded root user: {} with org and Owner membership", email);
  }

  Ok(())
}
