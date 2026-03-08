//! Integration tests for the generic entity handler: GET/POST /api/entities/{entity_id}
//! and GET/PATCH/DELETE /api/entities/{entity_id}/{id}.
//!
//! Covers: all entities (organization, user, role, permission, audit), authentication,
//! authorization, filter (all operators), sort, pagination, and error cases.
//! Create/update/delete are only supported for organization; other entities return 4xx.

use axum::http::StatusCode;

const ENTITY_ORGANIZATION: &str = "organization";
const ENTITY_USER: &str = "user";
const ENTITY_ROLE: &str = "role";
const ENTITY_PERMISSION: &str = "permission";
const ENTITY_AUDIT: &str = "audit";

fn scope_headers<'a>(org_id: &'a str, role_id: &'a str) -> [(&'static str, &'a str); 2] {
  [("X-Organization-Id", org_id), ("X-Role-Id", role_id)]
}

/// Percent-encode a query parameter value for use in URI (filter JSON etc.).
fn encode_query_value(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  for b in s.bytes() {
    match b {
      b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
      b' ' => out.push_str("%20"),
      b'[' => out.push_str("%5B"),
      b']' => out.push_str("%5D"),
      b'{' => out.push_str("%7B"),
      b'}' => out.push_str("%7D"),
      b'"' => out.push_str("%22"),
      b':' => out.push_str("%3A"),
      b',' => out.push_str("%2C"),
      _ => out.push_str(&format!("%{:02X}", b)),
    }
  }
  out
}

fn list_path(entity_id: &str) -> String {
  format!("/api/entities/{}", entity_id)
}

fn list_path_with_query(entity_id: &str, query: &str) -> String {
  format!("/api/entities/{}?{}", entity_id, query)
}

fn get_path(entity_id: &str, id: &str) -> String {
  format!("/api/entities/{}/{}", entity_id, id)
}

async fn test_client_with_migrations() -> app::TestClient {
  let client: app::TestClient = app::test_client_with_migrations()
    .await
    .expect("test_client_with_migrations");
  client
}

// ---------- Authentication: list requires auth for all entities ----------

#[tokio::test]
async fn list_organization_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(
    &client,
    "GET",
    &list_path(ENTITY_ORGANIZATION),
    None,
    None,
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_user_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", &list_path(ENTITY_USER), None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_role_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", &list_path(ENTITY_ROLE), None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_permission_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(
    &client,
    "GET",
    &list_path(ENTITY_PERMISSION),
    None,
    None,
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_audit_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", &list_path(ENTITY_AUDIT), None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ---------- List 200 with scope for each entity ----------

#[tokio::test]
async fn list_organization_200_with_scope() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (status, body) = app::test_request(
    &client,
    "GET",
    &list_path(ENTITY_ORGANIZATION),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json.get("data").and_then(|d| d.as_array()).is_some());
}

#[tokio::test]
async fn list_user_200_with_scope() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (status, body) = app::test_request(
    &client,
    "GET",
    &list_path(ENTITY_USER),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json.get("data").and_then(|d| d.as_array()).is_some());
}

#[tokio::test]
async fn list_role_200_with_scope() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (status, body) = app::test_request(
    &client,
    "GET",
    &list_path(ENTITY_ROLE),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json.get("data").and_then(|d| d.as_array()).is_some());
}

#[tokio::test]
async fn list_permission_200_with_scope() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (status, body) = app::test_request(
    &client,
    "GET",
    &list_path(ENTITY_PERMISSION),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json.get("data").and_then(|d| d.as_array()).is_some());
}

#[tokio::test]
async fn list_audit_200_with_scope() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (status, body) = app::test_request(
    &client,
    "GET",
    &list_path(ENTITY_AUDIT),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json.get("data").and_then(|d| d.as_array()).is_some());
}

// ---------- List with filter (one valid filter per entity) ----------

#[tokio::test]
async fn list_organization_with_filter_eq() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let filter = r#"[{"field":"name","operator":"eq","value":"Default"}]"#;
  let path = list_path_with_query(
    ENTITY_ORGANIZATION,
    &format!("filter={}", encode_query_value(filter)),
  );
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let data = json["data"].as_array().unwrap();
  assert!(!data.is_empty());
  assert_eq!(data[0]["name"], "Default");
}

#[tokio::test]
async fn list_user_with_filter_eq() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let filter = r#"[{"field":"email","operator":"eq","value":"viewer@default.org"}]"#;
  let path = list_path_with_query(
    ENTITY_USER,
    &format!("filter={}", encode_query_value(filter)),
  );
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let data = json["data"].as_array().unwrap();
  assert!(!data.is_empty());
  assert_eq!(data[0]["email"], "viewer@default.org");
}

#[tokio::test]
async fn list_role_with_filter_eq() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let filter = r#"[{"field":"name","operator":"eq","value":"viewer"}]"#;
  let path = list_path_with_query(
    ENTITY_ROLE,
    &format!("filter={}", encode_query_value(filter)),
  );
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json.get("data").and_then(|d| d.as_array()).is_some());
}

#[tokio::test]
async fn list_permission_with_filter_eq() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let filter = r#"[{"field":"permission_key","operator":"eq","value":"dashboard.users.read"}]"#;
  let path = list_path_with_query(
    ENTITY_PERMISSION,
    &format!("filter={}", encode_query_value(filter)),
  );
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json.get("data").and_then(|d| d.as_array()).is_some());
}

#[tokio::test]
async fn list_audit_with_filter_eq() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let filter = r#"[{"field":"event_kind","operator":"eq","value":"auth"}]"#;
  let path = list_path_with_query(
    ENTITY_AUDIT,
    &format!("filter={}", encode_query_value(filter)),
  );
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json.get("data").and_then(|d| d.as_array()).is_some());
}

// ---------- List with other filter operators (ne, in) ----------

#[tokio::test]
async fn list_organization_filter_ne() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let filter = r#"[{"field":"name","operator":"ne","value":"NonexistentOrg"}]"#;
  let path = list_path_with_query(
    ENTITY_ORGANIZATION,
    &format!("filter={}", encode_query_value(filter)),
  );
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json.get("data").and_then(|d| d.as_array()).is_some());
}

#[tokio::test]
async fn list_organization_filter_in() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let filter = r#"[{"field":"name","operator":"in","value":["Default","Other"]}]"#;
  let path = list_path_with_query(
    ENTITY_ORGANIZATION,
    &format!("filter={}", encode_query_value(filter)),
  );
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let data = json["data"].as_array().unwrap();
  assert!(!data.is_empty());
}

// ---------- List with sort (valid sort field asc/desc) ----------

#[tokio::test]
async fn list_organization_sort_name_asc() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_ORGANIZATION, "sort=name&order=asc");
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let data = json["data"].as_array().unwrap();
  if data.len() >= 2 {
    let a = data[0]["name"].as_str().unwrap_or("");
    let b = data[1]["name"].as_str().unwrap_or("");
    assert!(a <= b, "expected sort asc: {} <= {}", a, b);
  }
}

#[tokio::test]
async fn list_organization_sort_name_desc() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_ORGANIZATION, "sort=name&order=desc");
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let data = json["data"].as_array().unwrap();
  if data.len() >= 2 {
    let a = data[0]["name"].as_str().unwrap_or("");
    let b = data[1]["name"].as_str().unwrap_or("");
    assert!(a >= b, "expected sort desc: {} >= {}", a, b);
  }
}

#[tokio::test]
async fn list_user_sort_email_asc() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_USER, "sort=email&order=asc");
  let (status, _) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn list_audit_sort_occurred_at_desc() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_AUDIT, "sort=occurred_at&order=desc");
  let (status, _) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

// ---------- List with pagination ----------

#[tokio::test]
async fn list_organization_pagination_offset_limit() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_ORGANIZATION, "offset=0&limit=5");
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let data = json["data"].as_array().unwrap();
  assert!(data.len() <= 5);
}

#[tokio::test]
async fn list_organization_pagination_second_page() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_ORGANIZATION, "offset=1&limit=1");
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let data = json["data"].as_array().unwrap();
  assert!(data.len() <= 1);
}

#[tokio::test]
async fn list_organization_limit_100_ok() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_ORGANIZATION, "offset=0&limit=100");
  let (status, _) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn list_organization_limit_over_100_400() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_ORGANIZATION, "offset=0&limit=101");
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::BAD_REQUEST);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let msg = json["message"].as_str().unwrap_or("");
  assert!(
    msg.contains("limit") && msg.contains("100"),
    "expected limit error: {}",
    msg
  );
}

#[tokio::test]
async fn list_organization_limit_zero_400() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_ORGANIZATION, "offset=0&limit=0");
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::BAD_REQUEST);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let msg = json["message"].as_str().unwrap_or("");
  assert!(msg.contains("limit"), "expected limit error: {}", msg);
}

// ---------- List with cursor pagination ----------

#[tokio::test]
async fn list_organization_cursor_first_page() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_ORGANIZATION, "cursor=&limit=2");
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let data = json["data"].as_array().unwrap();
  assert!(
    data.len() <= 2,
    "cursor first page should return at most limit=2"
  );
  if json.get("next_cursor").is_some() {
    assert_eq!(data.len(), 2, "next_cursor implies full page");
  }
}

#[tokio::test]
async fn list_organization_cursor_second_page() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path_first = list_path_with_query(ENTITY_ORGANIZATION, "cursor=&limit=2");
  let (status1, body1) = app::test_request(
    &client,
    "GET",
    &path_first,
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status1, StatusCode::OK);
  let json1: app::serde_json::Value = app::serde_json::from_slice(&body1).unwrap();
  let next_cursor = match json1.get("next_cursor").and_then(|c| c.as_str()) {
    Some(c) => c.to_string(),
    None => return,
  };
  let path_second = list_path_with_query(
    ENTITY_ORGANIZATION,
    &format!("cursor={}&limit=2", encode_query_value(&next_cursor)),
  );
  let (status2, body2) = app::test_request(
    &client,
    "GET",
    &path_second,
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status2, StatusCode::OK);
  let json2: app::serde_json::Value = app::serde_json::from_slice(&body2).unwrap();
  let data2 = json2["data"].as_array().unwrap();
  let data1 = json1["data"].as_array().unwrap();
  if !data2.is_empty() && !data1.is_empty() {
    let id1 = data1[0]["id"].as_str().unwrap_or("");
    let id2_first = data2[0]["id"].as_str().unwrap_or("");
    assert_ne!(
      id1, id2_first,
      "second page should not repeat first page items"
    );
  }
}

#[tokio::test]
async fn list_organization_cursor_invalid_400() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_ORGANIZATION, "cursor=not-a-number&limit=2");
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::BAD_REQUEST);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let msg = json["message"].as_str().unwrap_or("");
  assert!(msg.contains("cursor"), "expected cursor error: {}", msg);
}

#[tokio::test]
async fn list_organization_cursor_and_offset_400() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_ORGANIZATION, "cursor=0&offset=1&limit=2");
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::BAD_REQUEST);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let msg = json["message"].as_str().unwrap_or("");
  assert!(
    msg.contains("offset") && msg.contains("cursor"),
    "expected cannot use both: {}",
    msg
  );
}

// ---------- List error cases: unknown entity, invalid filter/sort, expand/include ----------

#[tokio::test]
async fn list_unknown_entity_404() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (status, body) = app::test_request(
    &client,
    "GET",
    "/api/entities/nonexistent",
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::NOT_FOUND);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert_eq!(json["message"].as_str(), Some("Unknown entity"));
}

#[tokio::test]
async fn list_invalid_filter_json_400() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_ORGANIZATION, "filter=not-json");
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::BAD_REQUEST);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let msg = json["message"].as_str().unwrap_or("");
  assert!(msg.contains("filter") || msg.contains("JSON"), "{}", msg);
}

#[tokio::test]
async fn list_invalid_filter_operator_400() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let filter = r#"[{"field":"name","operator":"invalid_op","value":"x"}]"#;
  let path = list_path_with_query(
    ENTITY_ORGANIZATION,
    &format!("filter={}", encode_query_value(filter)),
  );
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::BAD_REQUEST);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let msg = json["message"].as_str().unwrap_or("");
  assert!(
    msg.contains("operator") || msg.contains("invalid"),
    "{}",
    msg
  );
}

#[tokio::test]
async fn list_invalid_filter_field_400() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let filter = r#"[{"field":"invalid_field","operator":"eq","value":"x"}]"#;
  let path = list_path_with_query(
    ENTITY_ORGANIZATION,
    &format!("filter={}", encode_query_value(filter)),
  );
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::BAD_REQUEST);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let msg = json["message"].as_str().unwrap_or("");
  assert!(
    msg.contains("field") || msg.contains("invalid") || msg.contains("allowed"),
    "{}",
    msg
  );
}

#[tokio::test]
async fn list_invalid_sort_field_400() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_ORGANIZATION, "sort=invalid_field&order=asc");
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::BAD_REQUEST);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let msg = json["message"].as_str().unwrap_or("");
  assert!(
    msg.contains("sort")
      || msg.contains("field")
      || msg.contains("invalid")
      || msg.contains("allowed"),
    "{}",
    msg
  );
}

#[tokio::test]
async fn list_expand_rejected_400() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let path = list_path_with_query(ENTITY_ORGANIZATION, "expand=users");
  let (status, body) = app::test_request(&client, "GET", &path, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::BAD_REQUEST);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let msg = json["message"].as_str().unwrap_or("");
  assert!(msg.contains("expand") || msg.contains("include"), "{}", msg);
}

// ---------- Get by id: 401, 403, 404, 400 invalid UUID, 200 ----------

#[tokio::test]
async fn get_organization_by_id_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(
    &client,
    "GET",
    &get_path(ENTITY_ORGANIZATION, "00000000-0000-0000-0000-000000000001"),
    None,
    None,
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_organization_by_id_invalid_uuid_400() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (status, body) = app::test_request(
    &client,
    "GET",
    &get_path(ENTITY_ORGANIZATION, "not-a-uuid"),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::BAD_REQUEST);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert_eq!(json["message"].as_str(), Some("Invalid id"));
}

#[tokio::test]
async fn get_organization_by_id_200_when_exists() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (_, list_body) = app::test_request(
    &client,
    "GET",
    &list_path(ENTITY_ORGANIZATION),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  let list_json: app::serde_json::Value = app::serde_json::from_slice(&list_body).unwrap();
  let data = list_json["data"].as_array().unwrap();
  let id = data.first().and_then(|e| e["id"].as_str()).unwrap();
  let (status, body) = app::test_request(
    &client,
    "GET",
    &get_path(ENTITY_ORGANIZATION, id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert_eq!(json["id"], id);
}

#[tokio::test]
async fn get_unknown_entity_404() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (status, body) = app::test_request(
    &client,
    "GET",
    "/api/entities/nonexistent/00000000-0000-0000-0000-000000000001",
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::NOT_FOUND);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert_eq!(json["message"].as_str(), Some("Unknown entity"));
}

// ---------- Create: only organization supported ----------

#[tokio::test]
async fn create_organization_201() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let slug = format!("test-org-{}", uuid::Uuid::new_v4());
  let body = format!(r#"{{"name":"Test Org","slug":"{}"}}"#, slug);
  let (status, res_body) = app::test_request(
    &client,
    "POST",
    &list_path(ENTITY_ORGANIZATION),
    Some(&token),
    Some(&body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::CREATED);
  let json: app::serde_json::Value = app::serde_json::from_slice(&res_body).unwrap();
  assert!(json.get("id").is_some());
  assert_eq!(json["name"], "Test Org");
  assert_eq!(json["slug"], slug);
}

#[tokio::test]
async fn create_organization_non_object_body_400() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (status, body) = app::test_request(
    &client,
    "POST",
    &list_path(ENTITY_ORGANIZATION),
    Some(&token),
    Some("[]"),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::BAD_REQUEST);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(
    json["message"]
      .as_str()
      .unwrap_or("")
      .contains("JSON object")
  );
}

#[tokio::test]
async fn create_unknown_entity_404() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (status, _) = app::test_request(
    &client,
    "POST",
    "/api/entities/nonexistent",
    Some(&token),
    Some(r#"{"name":"x"}"#),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---------- Update: only organization supported ----------

#[tokio::test]
async fn update_organization_200() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let slug_created = format!("update-test-{}", uuid::Uuid::new_v4());
  let create_body = format!(r#"{{"name":"To Update","slug":"{}"}}"#, slug_created);
  let (_, create_res) = app::test_request(
    &client,
    "POST",
    &list_path(ENTITY_ORGANIZATION),
    Some(&token),
    Some(&create_body),
    Some(&scope),
  )
  .await
  .unwrap();
  let create_json: app::serde_json::Value = app::serde_json::from_slice(&create_res).unwrap();
  let id = create_json["id"].as_str().unwrap();
  let patch = r#"{"name":"Updated Name","slug":"updated-slug"}"#;
  let (status, res_body) = app::test_request(
    &client,
    "PATCH",
    &get_path(ENTITY_ORGANIZATION, id),
    Some(&token),
    Some(patch),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&res_body).unwrap();
  assert_eq!(json["name"], "Updated Name");
  assert_eq!(json["slug"], "updated-slug");
  // Clean up: delete the org we created
  let _ = app::test_request(
    &client,
    "DELETE",
    &get_path(ENTITY_ORGANIZATION, id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await;
}

#[tokio::test]
async fn update_organization_non_object_body_400() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let slug_created = format!("patch-400-{}", uuid::Uuid::new_v4());
  let create_body = format!(r#"{{"name":"Patch400","slug":"{}"}}"#, slug_created);
  let (_, create_res) = app::test_request(
    &client,
    "POST",
    &list_path(ENTITY_ORGANIZATION),
    Some(&token),
    Some(&create_body),
    Some(&scope),
  )
  .await
  .unwrap();
  let create_json: app::serde_json::Value = app::serde_json::from_slice(&create_res).unwrap();
  let id = create_json["id"].as_str().unwrap();
  let (status, _) = app::test_request(
    &client,
    "PATCH",
    &get_path(ENTITY_ORGANIZATION, id),
    Some(&token),
    Some("null"),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::BAD_REQUEST);
  let _ = app::test_request(
    &client,
    "DELETE",
    &get_path(ENTITY_ORGANIZATION, id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await;
}

// ---------- Delete: only organization supported ----------

#[tokio::test]
async fn delete_organization_200() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let create_body = r#"{"name":"To Delete","slug":"to-delete"}"#;
  let (_, create_res) = app::test_request(
    &client,
    "POST",
    &list_path(ENTITY_ORGANIZATION),
    Some(&token),
    Some(create_body),
    Some(&scope),
  )
  .await
  .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&create_res).unwrap();
  let id = json["id"].as_str().unwrap();
  let (status, _) = app::test_request(
    &client,
    "DELETE",
    &get_path(ENTITY_ORGANIZATION, id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn delete_unknown_entity_404() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (status, _) = app::test_request(
    &client,
    "DELETE",
    "/api/entities/nonexistent/00000000-0000-0000-0000-000000000001",
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---------- CUD not supported for user, role, permission, audit ----------

#[tokio::test]
async fn create_user_not_supported_4xx_or_5xx() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (status, body) = app::test_request(
    &client,
    "POST",
    &list_path(ENTITY_USER),
    Some(&token),
    Some(r#"{"email":"x@x.com","password":"x"}"#),
    Some(&scope),
  )
  .await
  .unwrap();
  assert!(
    status.is_client_error() || status.is_server_error(),
    "POST user should not succeed with 2xx: {}",
    status
  );
  let msg = app::serde_json::from_slice::<app::serde_json::Value>(&body)
    .ok()
    .and_then(|j| j["message"].as_str().map(String::from))
    .unwrap_or_else(|| String::from_utf8_lossy(&body).into_owned());
  assert!(
    msg.contains("not supported") || msg.contains("Unknown") || msg.is_empty(),
    "expected not supported or unknown: {}",
    msg
  );
}

#[tokio::test]
async fn update_audit_not_supported_4xx_or_5xx() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (status, body) = app::test_request(
    &client,
    "PATCH",
    &get_path(ENTITY_AUDIT, "00000000-0000-0000-0000-000000000001"),
    Some(&token),
    Some(r#"{"outcome":"x"}"#),
    Some(&scope),
  )
  .await
  .unwrap();
  assert!(
    status.is_client_error() || status.is_server_error(),
    "PATCH audit should not succeed with 2xx: {}",
    status
  );
  let msg = String::from_utf8_lossy(&body);
  assert!(
    msg.contains("not supported") || msg.contains("read-only") || status == StatusCode::NOT_FOUND,
    "expected not supported or 404: status={} body={}",
    status,
    msg
  );
}

#[tokio::test]
async fn delete_permission_not_supported_4xx_or_5xx() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(&org_id, &role_id);
  let (_, list_body) = app::test_request(
    &client,
    "GET",
    &list_path(ENTITY_PERMISSION),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  let list_json: app::serde_json::Value = app::serde_json::from_slice(&list_body).unwrap();
  let id = list_json["data"]
    .as_array()
    .and_then(|a| a.first())
    .and_then(|e| e["id"].as_str())
    .unwrap_or("00000000-0000-0000-0000-000000000001");
  let (status, _) = app::test_request(
    &client,
    "DELETE",
    &get_path(ENTITY_PERMISSION, id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert!(
    status.is_client_error() || status.is_server_error(),
    "DELETE permission should not succeed with 2xx: {}",
    status
  );
}
