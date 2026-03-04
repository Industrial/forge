//! Integration tests: GET/POST /api/users and GET/POST/PATCH/DELETE /api/organizations/{id}/users.
//! Uses Bearer token and scope headers (X-Organization-Id, X-Role-Id).

use axum::http::StatusCode;

fn scope_headers<'a>(org_id: &'a str, role_id: &'a str) -> [(&'static str, &'a str); 2] {
  [("X-Organization-Id", org_id), ("X-Role-Id", role_id)]
}

#[tokio::test]
async fn get_api_users_anonymous_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", "/api/users", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_api_users_viewer_without_scope_403() {
  let client = app::test_client().await.expect("test_client");
  let (token, _org_id, _role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let (status, _) = app::test_request(&client, "GET", "/api/users", Some(&token), None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_api_users_viewer_with_scope_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (status, body) = app::test_request(
    &client,
    "GET",
    "/api/users",
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json.get("users").and_then(|u| u.as_array()).is_some());
}

#[tokio::test]
async fn get_api_users_viewer_scope_only_sees_default_org() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (status, body) = app::test_request(
    &client,
    "GET",
    "/api/users",
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let users = json["users"].as_array().unwrap();
  let empty: &[app::serde_json::Value] = &[];
  for u in users {
    for m in u["memberships"]
      .as_array()
      .map(|v| v.as_slice())
      .unwrap_or(empty)
    {
      let org_name = m["org_name"].as_str().unwrap_or("");
      assert!(
        !org_name.eq_ignore_ascii_case("CoolOrg"),
        "viewer with Default org scope must not see CoolOrg in memberships"
      );
    }
  }
}

#[tokio::test]
async fn get_api_users_viewer_coolorg_only_coolorg_org() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@coolorg.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (status, body) = app::test_request(
    &client,
    "GET",
    "/api/users",
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let users = json["users"].as_array().unwrap();
  let empty: &[app::serde_json::Value] = &[];
  for u in users {
    let memberships = u["memberships"]
      .as_array()
      .map(|v| v.as_slice())
      .unwrap_or(empty);
    let has_coolorg = memberships.iter().any(|m| {
      m["org_name"]
        .as_str()
        .map(|s| s == "CoolOrg")
        .unwrap_or(false)
    });
    assert!(
      has_coolorg,
      "viewer@coolorg.org with CoolOrg scope: each returned user should have at least one membership in CoolOrg"
    );
  }
}

#[tokio::test]
async fn get_api_users_admin_global_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, _org_id, _role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let (status, body) = app::test_request(&client, "GET", "/api/users", Some(&token), None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let users = json["users"].as_array().expect("response has users array");
  let empty: &[app::serde_json::Value] = &[];
  let has_coolorg = users.iter().any(|u| {
    u["memberships"]
      .as_array()
      .map(|v| v.as_slice())
      .unwrap_or(empty)
      .iter()
      .any(|m| {
        m["org_name"]
          .as_str()
          .map(|s| s.eq_ignore_ascii_case("CoolOrg"))
          .unwrap_or(false)
      })
  });
  assert!(
    has_coolorg,
    "admin GET /api/users (global) must return users that include at least one with CoolOrg (seed data)"
  );
}

#[tokio::test]
async fn post_api_org_users_viewer_403() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let body = format!(r#"{{"email":"new@example.com","password":"password123"}}"#);
  let (status, _) = app::test_request(
    &client,
    "POST",
    &format!("/api/organizations/{}/users", org_id),
    Some(&token),
    Some(&body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn post_api_org_users_editor_201() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "editor@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let unique = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis();
  let body = format!(
    r#"{{"email":"newuser-{}@example.com","password":"password123"}}"#,
    unique
  );
  let (status, _) = app::test_request(
    &client,
    "POST",
    &format!("/api/organizations/{}/users", org_id),
    Some(&token),
    Some(&body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn post_api_org_users_wrong_org_403() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let other_org_id = "00000000-0000-0000-0000-000000000001";
  let body = r#"{"email":"x@example.com","password":"password123"}"#;
  let (status, _) = app::test_request(
    &client,
    "POST",
    &format!("/api/organizations/{}/users", other_org_id),
    Some(&token),
    Some(body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_org_users_with_scope_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, _first_org_id, _first_role_id) =
    app::auth_with_profile(&client, "multi@email.com", app::SEED_PASSWORD)
      .await
      .expect("auth with profile");
  let (_, profiles_body) = app::test_request(
    &client,
    "GET",
    "/api/auth/profiles",
    Some(&token),
    None,
    None,
  )
  .await
  .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&profiles_body).unwrap();
  let profiles = json["profiles"].as_array().unwrap();
  let coolorg = profiles
    .iter()
    .find(|p| p["org_name"].as_str() == Some("CoolOrg"))
    .expect("multi has CoolOrg");
  let org_id = coolorg["org_id"].as_str().unwrap();
  let role_id = coolorg["role_id"].as_str().unwrap();
  let scope = [("X-Organization-Id", org_id), ("X-Role-Id", role_id)];
  let (status, body) = app::test_request(
    &client,
    "GET",
    &format!("/api/organizations/{}/users", org_id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let users_json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let users = users_json["users"].as_array().expect("users array");
  for u in users {
    assert!(
      u.get("roles").and_then(|r| r.as_array()).is_some(),
      "each user has roles"
    );
  }
}
