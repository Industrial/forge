//! Adds one user (multi@email.com) to both Default and CoolOrg: editor in Default, viewer in CoolOrg.

use forge_db::DbConnection;
use uuid::Uuid;

fn base_url() -> String {
  std::env::var("FORGE_API_BASE").unwrap_or_else(|_| "http://127.0.0.1:3000".into())
}

/// GET /api/organizations; returns (slug -> org_id).
async fn list_organizations() -> Result<std::collections::HashMap<String, Uuid>, Box<dyn std::error::Error>> {
  let base = base_url();
  let url = format!("{}/api/organizations", base.trim_end_matches('/'));
  let client = reqwest::Client::new();
  let res = client.get(&url).send().await?;
  if !res.status().is_success() {
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    return Err(format!("GET {} failed {}: {}", url, status, text).into());
  }
  let json: serde_json::Value = res.json().await?;
  let orgs = json.get("organizations").and_then(|v| v.as_array()).ok_or("response missing organizations")?;
  let mut map = std::collections::HashMap::new();
  for o in orgs {
    let slug = o.get("slug").and_then(|v| v.as_str()).ok_or("org missing slug")?;
    let id = o.get("id").and_then(|v| v.as_str()).ok_or("org missing id")?;
    map.insert(slug.to_string(), id.parse()?);
  }
  Ok(map)
}

async fn list_roles(organization_id: Uuid) -> Result<std::collections::HashMap<String, Uuid>, Box<dyn std::error::Error>> {
  let base = base_url();
  let url = format!(
    "{}/api/organizations/{}/roles",
    base.trim_end_matches('/'),
    organization_id
  );
  let client = reqwest::Client::new();
  let res = client.get(&url).send().await?;
  if !res.status().is_success() {
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    return Err(format!("GET {} failed {}: {}", url, status, text).into());
  }
  let json: serde_json::Value = res.json().await?;
  let roles = json.get("roles").and_then(|v| v.as_array()).ok_or("response missing roles")?;
  let mut map = std::collections::HashMap::new();
  for r in roles {
    let name = r.get("name").and_then(|v| v.as_str()).ok_or("role missing name")?;
    let id = r.get("id").and_then(|v| v.as_str()).ok_or("role missing id")?;
    map.insert(name.to_string(), id.parse()?);
  }
  Ok(map)
}

async fn add_org_user(
  organization_id: Uuid,
  email: &str,
  password: &str,
) -> Result<Uuid, Box<dyn std::error::Error>> {
  let base = base_url();
  let url = format!(
    "{}/api/organizations/{}/users",
    base.trim_end_matches('/'),
    organization_id
  );
  let body = serde_json::json!({ "email": email, "password": password });
  let client = reqwest::Client::new();
  let res = client.post(&url).json(&body).send().await?;
  if !res.status().is_success() {
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    return Err(format!("POST {} failed {}: {}", url, status, text).into());
  }
  let json: serde_json::Value = res.json().await?;
  let id = json
    .get("user_id")
    .and_then(|v| v.as_str())
    .ok_or("response missing user_id")?;
  id.parse().map_err(|e: uuid::Error| e.into())
}

/// Add existing user to org (membership only).
async fn add_org_user_by_id(organization_id: Uuid, user_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
  let base = base_url();
  let url = format!(
    "{}/api/organizations/{}/users",
    base.trim_end_matches('/'),
    organization_id
  );
  let body = serde_json::json!({ "user_id": user_id.to_string() });
  let client = reqwest::Client::new();
  let res = client.post(&url).json(&body).send().await?;
  if !res.status().is_success() {
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    return Err(format!("POST {} failed {}: {}", url, status, text).into());
  }
  Ok(())
}

async fn add_org_user_roles(
  organization_id: Uuid,
  user_id: Uuid,
  role_ids: &[Uuid],
) -> Result<(), Box<dyn std::error::Error>> {
  if role_ids.is_empty() {
    return Ok(());
  }
  let base = base_url();
  let url = format!(
    "{}/api/organizations/{}/users/{}/roles",
    base.trim_end_matches('/'),
    organization_id,
    user_id
  );
  let ids: Vec<String> = role_ids.iter().map(Uuid::to_string).collect();
  let body = serde_json::json!({ "role_ids": ids });
  let client = reqwest::Client::new();
  let res = client.post(&url).json(&body).send().await?;
  if !res.status().is_success() {
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    return Err(format!("POST {} failed {}: {}", url, status, text).into());
  }
  Ok(())
}

const SEED_PASSWORD: &str = "password";
const MULTI_EMAIL: &str = "multi@email.com";

pub async fn seed(_db: &DbConnection) -> Result<(), Box<dyn std::error::Error>> {
  let orgs = list_organizations().await?;
  let default_org_id = orgs.get("default").ok_or("organization 'default' not found")?;
  let coolorg_org_id = orgs.get("coolorg").ok_or("organization 'coolorg' not found")?;

  let default_roles = list_roles(*default_org_id).await?;
  let coolorg_roles = list_roles(*coolorg_org_id).await?;
  let editor_role_id = default_roles.get("editor").ok_or("Default org missing editor role")?;
  let viewer_role_id = coolorg_roles.get("viewer").ok_or("CoolOrg missing viewer role")?;

  let user_id = add_org_user(*default_org_id, MULTI_EMAIL, SEED_PASSWORD).await?;
  add_org_user_roles(*default_org_id, user_id, &[*editor_role_id]).await?;

  add_org_user_by_id(*coolorg_org_id, user_id).await?;
  add_org_user_roles(*coolorg_org_id, user_id, &[*viewer_role_id]).await?;

  Ok(())
}
