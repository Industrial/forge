//! Integration tests: scope headers (X-Organization-Id, X-Role-Id) and /api users/organizations.
//! Uses GET /api/users and GET /api/organizations/{id}/users; multi@email.com has Default and CoolOrg.

use axum::http::StatusCode;

#[tokio::test]
async fn get_users_with_token_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "multi@email.com", app::SEED_PASSWORD)
      .await
      .expect("auth with profile");
  let scope = [
    ("X-Organization-Id", org_id.as_str()),
    ("X-Role-Id", role_id.as_str()),
  ];
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
  assert!(
    json.get("users").and_then(|u| u.as_array()).is_some(),
    "response has users array"
  );
}

#[tokio::test]
async fn get_users_with_scope_headers_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "multi@email.com", app::SEED_PASSWORD)
      .await
      .expect("auth with profile");
  let scope = [
    ("X-Organization-Id", org_id.as_str()),
    ("X-Role-Id", role_id.as_str()),
  ];
  let (status, _) = app::test_request(
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
}

#[tokio::test]
async fn get_org_users_coolorg_returns_coolorg_users() {
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
  let coolorg_profile = profiles
    .iter()
    .find(|p| p["org_name"].as_str() == Some("CoolOrg"))
    .expect("multi has CoolOrg org profile");
  let org_id = coolorg_profile["org_id"].as_str().unwrap();
  let role_id = coolorg_profile["role_id"].as_str().unwrap();
  let scope = [("X-Organization-Id", org_id), ("X-Role-Id", role_id)];
  let (status_users, body_users) = app::test_request(
    &client,
    "GET",
    &format!("/api/organizations/{}/users", org_id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status_users, StatusCode::OK);
  let users_json: app::serde_json::Value = app::serde_json::from_slice(&body_users).unwrap();
  let users = users_json["users"].as_array().expect("users array");
  assert!(!users.is_empty(), "should have users in CoolOrg");
  for u in users {
    assert!(
      u.get("roles").and_then(|r| r.as_array()).is_some(),
      "each user has roles"
    );
  }
}

#[tokio::test]
async fn register_then_login_then_get_users_403_without_scope() {
  let client = app::test_client().await.expect("test_client");
  let email = format!("profiletest_{}@example.com", uuid::Uuid::new_v4());
  let (status_reg, _) = app::test_request(
    &client,
    "POST",
    "/api/auth/register",
    None,
    Some(&format!(
      r#"{{"email":"{}","password":"password123"}}"#,
      email
    )),
    None,
  )
  .await
  .unwrap();
  assert_eq!(status_reg, StatusCode::CREATED);
  let (token, _org_id, _role_id) = app::auth_with_profile(&client, &email, "password123")
    .await
    .expect("auth with profile after register");
  // New user has no global scope; GET /api/users without scope headers returns 403.
  let (status, _) = app::test_request(&client, "GET", "/api/users", Some(&token), None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}
