//! Auth: logout, me, profiles, tokens, admin — 401 anon, 200 auth; admin 403 non-admin, 200 global.
//! (Session and set-profile/switch-profile endpoints removed; scope is via request headers.)

use axum::http::StatusCode;

async fn token_for(client: &app::TestClient, email: &str) -> String {
  app::login_as_seed_user(client, email, app::SEED_PASSWORD)
    .await
    .expect("login")
}

#[tokio::test]
async fn get_logout_anon_200() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", "/api/auth/logout", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_logout_auth_200() {
  let client = app::test_client().await.expect("test_client");
  let token = token_for(&client, "viewer@default.org").await;
  let (status, _) = app::test_request(&client, "GET", "/api/auth/logout", Some(&token), None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_me_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", "/api/auth/me", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_me_auth_200() {
  let client = app::test_client().await.expect("test_client");
  let token = token_for(&client, "viewer@default.org").await;
  let (status, _) = app::test_request(&client, "GET", "/api/auth/me", Some(&token), None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_profiles_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", "/api/auth/profiles", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_profiles_auth_200() {
  let client = app::test_client().await.expect("test_client");
  let token = token_for(&client, "viewer@default.org").await;
  let (status, body) = app::test_request(
    &client,
    "GET",
    "/api/auth/profiles",
    Some(&token),
    None,
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json.get("profiles").is_some());
}

#[tokio::test]
async fn post_tokens_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "POST", "/api/auth/tokens", None, Some("{}"), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn post_tokens_auth_200() {
  let client = app::test_client().await.expect("test_client");
  let token = token_for(&client, "viewer@default.org").await;
  let (status, _) = app::test_request(
    &client,
    "POST",
    "/api/auth/tokens",
    Some(&token),
    Some("{}"),
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_admin_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", "/api/auth/admin", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_admin_non_admin_403() {
  let client = app::test_client().await.expect("test_client");
  let token = token_for(&client, "viewer@default.org").await;
  let (status, _) = app::test_request(&client, "GET", "/api/auth/admin", Some(&token), None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_admin_global_admin_200() {
  let client = app::test_client().await.expect("test_client");
  let token = token_for(&client, "admin@admin.com").await;
  let (status, _) = app::test_request(&client, "GET", "/api/auth/admin", Some(&token), None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}
