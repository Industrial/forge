//! Multi-org user: one user (multi@email.com) in both Default and CoolOrg — editor in Default, viewer in CoolOrg.

use forge_db::DbConnection;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use app::handlers::rest::{
  AddOrgUserBody, add_org_user_impl, add_org_user_roles_impl, list_org_roles_impl,
};
use db::models::organization;

const SEED_PASSWORD: &str = "password";
const MULTI_EMAIL: &str = "multi@email.com";

pub async fn seed(db: &DbConnection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let default_org = organization::Entity::find()
    .filter(organization::Column::Slug.eq("default"))
    .one(db)
    .await?
    .ok_or("organization 'default' not found")?;
  let coolorg_org = organization::Entity::find()
    .filter(organization::Column::Slug.eq("coolorg"))
    .one(db)
    .await?
    .ok_or("organization 'coolorg' not found")?;

  let default_roles = list_org_roles_impl(db, default_org.id)
    .await
    .map_err(|e| e.to_string())?;
  let coolorg_roles = list_org_roles_impl(db, coolorg_org.id)
    .await
    .map_err(|e| e.to_string())?;
  let editor_role_id = default_roles
    .get("editor")
    .ok_or("Default org missing editor role")?;
  let viewer_role_id = coolorg_roles
    .get("viewer")
    .ok_or("CoolOrg missing viewer role")?;

  let (user_id, _) = add_org_user_impl(
    db,
    default_org.id,
    &AddOrgUserBody {
      user_id: None,
      email: Some(MULTI_EMAIL.to_string()),
      password: Some(SEED_PASSWORD.to_string()),
    },
  )
  .await
  .map_err(|e| e.to_string())?;

  add_org_user_roles_impl(db, default_org.id, user_id, &[*editor_role_id])
    .await
    .map_err(|e| e.to_string())?;

  add_org_user_impl(
    db,
    coolorg_org.id,
    &AddOrgUserBody {
      user_id: Some(user_id),
      email: None,
      password: None,
    },
  )
  .await
  .map_err(|e| e.to_string())?;

  add_org_user_roles_impl(db, coolorg_org.id, user_id, &[*viewer_role_id])
    .await
    .map_err(|e| e.to_string())?;

  Ok(())
}
