//! Integration tests: session profile, needs_profile_select, dashboard without profile, set-profile, register flow.

use axum::http::StatusCode;

async fn cookie_for(router: &axum::Router, email: &str) -> String {
  app::login_as_seed_user(router, email, app::SEED_PASSWORD)
    .await
    .expect("login")
}

#[tokio::test]
async fn session_after_single_profile_login_has_profile_and_no_needs_select() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = cookie_for(&router, "viewer@default.org").await;
  let (status, body) = app::test_request(&router, "GET", "/api/auth/session", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert_eq!(json["needs_profile_select"], false);
  assert!(json["user"]["current_org_id"].as_str().is_some(), "session should have current_org_id");
  assert!(json["user"]["current_role_name"].as_str().is_some(), "session should have current_role_name");
}

#[tokio::test]
async fn session_after_multi_profile_login_needs_profile_select() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = cookie_for(&router, "multi@email.com").await;
  let (status, body) = app::test_request(&router, "GET", "/api/auth/session", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert_eq!(json["needs_profile_select"], true);
}

#[tokio::test]
async fn dashboard_users_without_profile_403() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = cookie_for(&router, "multi@email.com").await;
  let (status, body) = app::test_request(&router, "GET", "/api/dashboard/users", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert_eq!(json["code"].as_str(), Some("profile_required"));
}

#[tokio::test]
async fn set_profile_then_dashboard_users_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = cookie_for(&router, "multi@email.com").await;
  let (_, profiles_body) = app::test_request(&router, "GET", "/api/auth/profiles", Some(&cookie), None)
    .await
    .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&profiles_body).unwrap();
  let profiles = json["profiles"].as_array().unwrap();
  let default_profile = profiles
    .iter()
    .find(|p| p["org_name"].as_str() == Some("Default"))
    .expect("multi has Default org profile");
  let org_id = default_profile["org_id"].as_str().unwrap();
  let body_set = format!(r#"{{"org_id":"{}"}}"#, org_id);
  let (status_set, _) = app::test_request(
    &router,
    "POST",
    "/api/auth/set-profile",
    Some(&cookie),
    Some(&body_set),
  )
  .await
  .unwrap();
  assert_eq!(status_set, StatusCode::OK);
  let (status_users, _) = app::test_request(&router, "GET", "/api/dashboard/users", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status_users, StatusCode::OK);
}

#[tokio::test]
async fn set_profile_to_other_then_users_list_scoped_to_that_org() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = cookie_for(&router, "multi@email.com").await;
  let (_, profiles_body) = app::test_request(&router, "GET", "/api/auth/profiles", Some(&cookie), None)
    .await
    .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&profiles_body).unwrap();
  let profiles = json["profiles"].as_array().unwrap();
  let other_profile = profiles
    .iter()
    .find(|p| p["org_name"].as_str() == Some("Other"))
    .expect("multi has Other org profile");
  let org_id = other_profile["org_id"].as_str().unwrap();
  let body_set = format!(r#"{{"org_id":"{}"}}"#, org_id);
  let (status_set, _) = app::test_request(
    &router,
    "POST",
    "/api/auth/set-profile",
    Some(&cookie),
    Some(&body_set),
  )
  .await
  .unwrap();
  assert_eq!(status_set, StatusCode::OK);
  let (status_users, body_users) = app::test_request(&router, "GET", "/api/dashboard/users", Some(&cookie), None)
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
      "each user in list should have Other org membership when profile is Other"
    );
  }
}

#[tokio::test]
async fn register_then_login_session_has_profile() {
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
  )
  .await
  .unwrap();
  assert_eq!(status_reg, StatusCode::CREATED);
  let cookie = app::login_as_seed_user(&router, &email, "password123").await.expect("login after register");
  let (status, body) = app::test_request(&router, "GET", "/api/auth/session", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert_eq!(json["needs_profile_select"], false);
  assert!(json["user"]["current_org_id"].as_str().is_some());
  assert!(json["user"]["current_role_name"].as_str().is_some());
}
