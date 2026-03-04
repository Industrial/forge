//! GET /api/permissions — code-defined permission keys. Unprotected; 200 anon or auth.

use axum::http::StatusCode;

#[tokio::test]
async fn get_permissions_anon_200() {
  let client = app::test_client().await.expect("test_client");
  let (status, body) = app::test_request(&client, "GET", "/api/permissions", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json.get("permissions").and_then(|p| p.as_array()).is_some());
}

#[tokio::test]
async fn get_permissions_auth_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, _org_id, _role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let (status, body) = app::test_request(&client, "GET", "/api/permissions", Some(&token), None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json.get("permissions").and_then(|p| p.as_array()).is_some());
}
