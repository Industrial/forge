//! CoolOrg: organization, roles, permissions, users. Uses app handler impls.

use forge_db::DbConnection;

use app::handlers::rest::{
  CreateOrgRoleBody, CreateOrganizationBody, add_org_user_roles_impl, ensure_org_role_impl,
  ensure_org_user_impl, ensure_organization_impl, list_org_roles_impl,
};

const SEED_PASSWORD: &str = "password";

pub async fn seed(db: &DbConnection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let org_id = ensure_organization_impl(
    db,
    &CreateOrganizationBody {
      name: "CoolOrg".to_string(),
      slug: Some("coolorg".to_string()),
    },
  )
  .await
  .map_err(|e| e.to_string())?;

  for name in ["owner", "admin", "editor", "viewer"] {
    ensure_org_role_impl(
      db,
      org_id,
      &CreateOrgRoleBody {
        name: name.to_string(),
        display_name: None,
      },
    )
    .await
    .map_err(|e| e.to_string())?;
  }

  crate::seed_role_permissions_for_org(db, org_id).await?;

  let role_ids = list_org_roles_impl(db, org_id)
    .await
    .map_err(|e| e.to_string())?;

  let users = [
    ("owner@coolorg.org", "owner"),
    ("orgadmin@coolorg.org", "admin"),
    ("editor@coolorg.org", "editor"),
    ("viewer@coolorg.org", "viewer"),
  ];
  for (email, role_name) in users {
    let (user_id, _) = ensure_org_user_impl(db, org_id, email, SEED_PASSWORD)
      .await
      .map_err(|e| e.to_string())?;
    let role_id = role_ids
      .get(role_name)
      .ok_or_else(|| format!("role not found: {}", role_name))?;
    add_org_user_roles_impl(db, org_id, user_id, &[*role_id])
      .await
      .map_err(|e| e.to_string())?;
  }
  Ok(())
}
