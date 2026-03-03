//! Integration test: login as seed user and GET profile (template auth).

use axum::http::StatusCode;

#[tokio::test]
async fn post_login_then_get_profile_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");

  let token = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login as viewer@default.org");

  let (status, _) = app::test_request(&router, "GET", "/api/auth/me", Some(&token), None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_profile_without_cookie_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");

  let (status, _) = app::test_request(&router, "GET", "/api/auth/me", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn post_login_invalid_credentials_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");

  let body = r#"{"email":"viewer@default.org","password":"wrongpassword"}"#;
  let (status, _) = app::test_request(&router, "POST", "/api/auth/login", None, Some(body), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}
