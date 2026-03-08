//! Integration tests: scope headers (X-Organization-Id, X-Role-Id) and entity/dashboard APIs.
//! Uses GET /api/entities/user and GET /api/dashboard/users; multi@email.com has Default and CoolOrg.

use axum::http::StatusCode;

/// Helper function to create a test client with migrations run.
/// Uses the shared app::test_client_with_migrations() which checks E2E_API_URL first.
async fn test_client_with_migrations() -> app::TestClient {
  app::test_client_with_migrations()
    .await
    .expect("test_client_with_migrations")
}

#[tokio::test]
async fn get_users_with_token_200() {
  let client = test_client_with_migrations().await;
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
    "/api/entities/user",
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(
    json.get("data").and_then(|u| u.as_array()).is_some(),
    "response has data array"
  );
}

#[tokio::test]
async fn get_users_with_scope_headers_200() {
  let client = test_client_with_migrations().await;
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
    "/api/entities/user",
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
  let client = test_client_with_migrations().await;
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
  // GET /api/dashboard/users with CoolOrg scope returns users in that org.
  let (status_users, body_users) = app::test_request(
    &client,
    "GET",
    "/api/dashboard/users",
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
}

#[tokio::test]
async fn register_then_login_then_get_users_403_without_scope() {
  let client = test_client_with_migrations().await;
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
  // New user has no global scope; GET /api/entities/user without scope headers returns 400 Bad Request (scope headers required).
  let (status, _) = app::test_request(
    &client,
    "GET",
    "/api/entities/user",
    Some(&token),
    None,
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::BAD_REQUEST);
}
