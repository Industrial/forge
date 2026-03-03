use forge::DbConnection;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::info;
use uuid::Uuid;

use crate::models::{user, user_global_role};

const PLATFORM_ADMIN_EMAIL: &str = "admin@admin.com";
const PLATFORM_ADMIN_ROLE: &str = "platform_admin";

pub async fn seed(db: &DbConnection) -> Result<(), Box<dyn std::error::Error>> {
  let admin = user::Entity::find()
    .filter(user::Column::Email.eq(PLATFORM_ADMIN_EMAIL))
    .one(db)
    .await?;

  let Some(admin) = admin else {
    return Ok(());
  };

  let existing = user_global_role::Entity::find()
    .filter(user_global_role::Column::UserId.eq(admin.id))
    .filter(user_global_role::Column::RoleName.eq(PLATFORM_ADMIN_ROLE))
    .one(db)
    .await?;

  if existing.is_some() {
    return Ok(());
  }

  let id = Uuid::new_v4();
  let model = user_global_role::ActiveModel {
    id: Set(id),
    user_id: Set(admin.id),
    role_name: Set(PLATFORM_ADMIN_ROLE.to_string()),
    ..Default::default()
  };
  user_global_role::Entity::insert(model).exec(db).await?;
  info!(
    "Seeded user_global_role: user_id={} role_name={}",
    admin.id, PLATFORM_ADMIN_ROLE
  );
  Ok(())
}
