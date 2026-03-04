//! Integration test: login as seed user and GET profile (template auth). Uses prebuilt server when E2E_API_URL is set.

use axum::http::StatusCode;

#[tokio::test]
async fn post_login_then_get_profile_200() {
  let client = app::test_client().await.expect("test_client");

  let token = app::login_as_seed_user(&client, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login as viewer@default.org");

  let (status, _) = app::test_request(&client, "GET", "/api/auth/me", Some(&token), None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_profile_without_token_401() {
  let client = app::test_client().await.expect("test_client");

  let (status, _) = app::test_request(&client, "GET", "/api/auth/me", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn post_login_invalid_credentials_401() {
  let client = app::test_client().await.expect("test_client");

  let body = r#"{"email":"viewer@default.org","password":"wrongpassword"}"#;
  let (status, _) = app::test_request(&client, "POST", "/api/auth/login", None, Some(body), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}
