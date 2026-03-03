//! Integration tests: dashboard requires X-Organization-Id and X-Role-Name (scope headers).
//! Without scope → 403 profile_required; with scope → 200. Scenarios expressed via headers, not session/set-profile.

use axum::http::StatusCode;

#[tokio::test]
async fn dashboard_users_without_scope_headers_403() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, _org_id, _role_name) = app::auth_with_profile(&router, "multi@email.com", app::SEED_PASSWORD)
    .await
    .expect("auth with profile");
  // Token but no X-Organization-Id / X-Role-Name → profile_required
  let (status, body) = app::test_request(&router, "GET", "/api/dashboard/users", Some(&token), None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert_eq!(json["code"].as_str(), Some("profile_required"));
}

#[tokio::test]
async fn dashboard_users_with_scope_headers_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, org_id, role_name) = app::auth_with_profile(&router, "multi@email.com", app::SEED_PASSWORD)
    .await
    .expect("auth with profile");
  let scope = [
    ("X-Organization-Id", org_id.as_str()),
    ("X-Role-Name", role_name.as_str()),
  ];
  let (status, _) = app::test_request(
    &router,
    "GET",
    "/api/dashboard/users",
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn dashboard_users_with_other_org_scope_returns_other_org_users() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, _first_org_id, _first_role) = app::auth_with_profile(&router, "multi@email.com", app::SEED_PASSWORD)
    .await
    .expect("auth with profile");
  let (_, profiles_body) = app::test_request(&router, "GET", "/api/auth/profiles", Some(&token), None, None)
    .await
    .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&profiles_body).unwrap();
  let profiles = json["profiles"].as_array().unwrap();
  let other_profile = profiles
    .iter()
    .find(|p| p["org_name"].as_str() == Some("Other"))
    .expect("multi has Other org profile");
  let org_id = other_profile["org_id"].as_str().unwrap();
  let role_name = other_profile["role"].as_str().unwrap();
  let scope = [
    ("X-Organization-Id", org_id),
    ("X-Role-Name", role_name),
  ];
  let (status_users, body_users) = app::test_request(
    &router,
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
  assert!(!users.is_empty(), "should have users in Other org");
  for u in users {
    let memberships = u["memberships"].as_array().expect("memberships");
    let org_names: Vec<&str> = memberships
      .iter()
      .filter_map(|m| m["org_name"].as_str())
      .collect();
    assert!(
      org_names.iter().any(|n| *n == "Other"),
      "each user in list should have Other org membership when scope is Other"
    );
  }
}

#[tokio::test]
async fn register_then_login_with_scope_headers_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let email = format!("profiletest_{}@example.com", uuid::Uuid::new_v4());
  let (status_reg, _) = app::test_request(
    &router,
    "POST",
    "/api/auth/register",
    None,
    Some(&format!(r#"{{"email":"{}","password":"password123"}}"#, email)),
    None,
  )
  .await
  .unwrap();
  assert_eq!(status_reg, StatusCode::CREATED);
  let (token, org_id, role_name) = app::auth_with_profile(&router, &email, "password123")
    .await
    .expect("auth with profile after register");
  let scope = [
    ("X-Organization-Id", org_id.as_str()),
    ("X-Role-Name", role_name.as_str()),
  ];
  let (status, _) = app::test_request(
    &router,
    "GET",
    "/api/dashboard/users",
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
}
