//! Auth: logout, profile, profiles, switch-profile, session, tokens, admin — 401 anon, 200 auth; admin 403 non-admin, 200 global.

use axum::http::StatusCode;

async fn cookie_for(router: &axum::Router, email: &str) -> String {
  app::login_as_seed_user(router, email, app::SEED_PASSWORD)
    .await
    .expect("login")
}

#[tokio::test]
async fn get_logout_anon_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(&router, "GET", "/api/auth/logout", None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_logout_auth_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = cookie_for(&router, "viewer@default.org").await;
  let (status, _) = app::test_request(&router, "GET", "/api/auth/logout", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_profile_anon_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(&router, "GET", "/api/auth/profile", None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_profile_auth_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = cookie_for(&router, "viewer@default.org").await;
  let (status, _) = app::test_request(&router, "GET", "/api/auth/profile", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_profiles_anon_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(&router, "GET", "/api/auth/profiles", None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_profiles_auth_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = cookie_for(&router, "viewer@default.org").await;
  let (status, body) = app::test_request(&router, "GET", "/api/auth/profiles", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json.get("profiles").is_some());
}

#[tokio::test]
async fn post_switch_profile_auth_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = cookie_for(&router, "multi@email.com").await;
  let (status, body) = app::test_request(&router, "GET", "/api/auth/profiles", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let profiles = json["profiles"].as_array().unwrap();
  let org_id = profiles[0]["org_id"].as_str().unwrap();
  let body_switch = format!(r#"{{"org_id":"{}"}}"#, org_id);
  let (status2, _) = app::test_request(
    &router,
    "POST",
    "/api/auth/switch-profile",
    Some(&cookie),
    Some(&body_switch),
  )
  .await
  .unwrap();
  assert_eq!(status2, StatusCode::OK);
}

#[tokio::test]
async fn post_switch_profile_anon_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(
    &router,
    "POST",
    "/api/auth/switch-profile",
    None,
    Some(r#"{"org_id":"00000000-0000-0000-0000-000000000000"}"#),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_session_anon_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(&router, "GET", "/api/auth/session", None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_session_auth_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = cookie_for(&router, "viewer@default.org").await;
  let (status, _) = app::test_request(&router, "GET", "/api/auth/session", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn post_tokens_anon_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(&router, "POST", "/api/auth/tokens", None, Some("{}"))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn post_tokens_auth_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = cookie_for(&router, "viewer@default.org").await;
  let (status, _) = app::test_request(
    &router,
    "POST",
    "/api/auth/tokens",
    Some(&cookie),
    Some("{}"),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_admin_anon_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(&router, "GET", "/api/auth/admin", None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_admin_non_admin_403() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = cookie_for(&router, "viewer@default.org").await;
  let (status, _) = app::test_request(&router, "GET", "/api/auth/admin", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_admin_global_admin_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = cookie_for(&router, "admin@admin.com").await;
  let (status, _) = app::test_request(&router, "GET", "/api/auth/admin", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}
