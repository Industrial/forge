use forge_db::DbConnection;
use uuid::Uuid;

fn base_url() -> String {
  std::env::var("FORGE_API_BASE").unwrap_or_else(|_| "http://127.0.0.1:3000".into())
}

pub async fn create_organization(_db: &DbConnection) -> Result<Uuid, Box<dyn std::error::Error>> {
  let base = base_url();
  let url = format!("{}/api/organizations", base.trim_end_matches('/'));
  let body = serde_json::json!({ "name": "Default", "slug": "default" });
  let client = reqwest::Client::new();
  let res = client.post(&url).json(&body).send().await?;
  if !res.status().is_success() {
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    return Err(format!("POST {} failed {}: {}", url, status, text).into());
  }
  let json: serde_json::Value = res.json().await?;
  let id = json
    .get("id")
    .and_then(|v| v.as_str())
    .ok_or("response missing id")?;
  id.parse().map_err(|e: uuid::Error| e.into())
}

pub async fn create_role(organization_id: Uuid, name: &str) -> Result<(), Box<dyn std::error::Error>> {
  let base = base_url();
  let url = format!(
    "{}/api/organizations/{}/roles",
    base.trim_end_matches('/'),
    organization_id
  );
  let body = serde_json::json!({ "name": name });
  let client = reqwest::Client::new();
  let res = client.post(&url).json(&body).send().await?;
  if !res.status().is_success() {
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    return Err(format!("POST {} failed {}: {}", url, status, text).into());
  }
  Ok(())
}

/// Returns (role_name -> role_id) for the org.
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

/// Add user to org (membership only). Returns user_id.
pub async fn add_org_user(
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

/// Assign roles to a user in the org.
pub async fn add_org_user_roles(
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

/// Add one permission to a role in the org.
async fn add_role_permission(
  organization_id: Uuid,
  role_id: Uuid,
  permission_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let base = base_url();
  let url = format!(
    "{}/api/organizations/{}/roles/{}/permissions",
    base.trim_end_matches('/'),
    organization_id,
    role_id
  );
  let body = serde_json::json!({ "permission_key": permission_key });
  let client = reqwest::Client::new();
  let res = client.post(&url).json(&body).send().await?;
  if !res.status().is_success() {
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    return Err(format!("POST {} failed {}: {}", url, status, text).into());
  }
  Ok(())
}

const ORG_OWNER_ADMIN: &[&str] = &[
  "dashboard",
  "dashboard.users.read",
  "dashboard.users.write",
  "dashboard.roles.read",
  "dashboard.roles.write",
  "dashboard.permissions.read",
  "dashboard.permissions.write",
  "dashboard.audit.read",
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

const SEED_PASSWORD: &str = "password";

pub async fn seed(_db: &DbConnection) -> Result<(), Box<dyn std::error::Error>> {
  let org_id = create_organization(_db).await?;
  for name in ["owner", "admin", "editor", "viewer"] {
    create_role(org_id, name).await?;
  }
  let role_ids = list_roles(org_id).await?;
  for (role_name, keys) in [
    ("owner", ORG_OWNER_ADMIN),
    ("admin", ORG_OWNER_ADMIN),
    ("editor", ORG_EDITOR),
    ("viewer", ORG_VIEWER),
  ] {
    let role_id = role_ids.get(role_name).ok_or_else(|| format!("role not found: {}", role_name))?;
    for key in keys {
      add_role_permission(org_id, *role_id, key).await?;
    }
  }
  let users = [
    ("owner@default.org", "owner"),
    ("orgadmin@default.org", "admin"),
    ("editor@default.org", "editor"),
    ("viewer@default.org", "viewer"),
  ];
  for (email, role_name) in users {
    let user_id = add_org_user(org_id, email, SEED_PASSWORD).await?;
    let role_id = role_ids.get(role_name).ok_or_else(|| format!("role not found: {}", role_name))?;
    add_org_user_roles(org_id, user_id, &[*role_id]).await?;
  }
  Ok(())
}
