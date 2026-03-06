//! GET /api/entities/audit — 401 anon; 403 non-admin (e.g. viewer); 200 admin only.

use axum::http::StatusCode;

const AUDIT_LIST: &str = "/api/entities/audit";

#[tokio::test]
async fn get_audit_log_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", AUDIT_LIST, None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_audit_log_viewer_403() {
  let client = app::test_client().await.expect("test_client");
  let (token, _org_id, _role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let (status, _) = app::test_request(&client, "GET", AUDIT_LIST, Some(&token), None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_audit_log_admin_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, _org_id, _role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let (status, _) = app::test_request(&client, "GET", AUDIT_LIST, Some(&token), None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

/// GET /api/entities/audit/:id — same auth as list: 401 anon, 403 non-admin, 200 admin.
#[tokio::test]
async fn get_audit_log_by_id_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let id = "00000000-0000-0000-0000-000000000000";
  let (status, _) = app::test_request(
    &client,
    "GET",
    &format!("/api/entities/audit/{}", id),
    None,
    None,
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_audit_log_by_id_viewer_403() {
  let client = app::test_client().await.expect("test_client");
  let (token, _org_id, _role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let id = "00000000-0000-0000-0000-000000000000";
  let (status, _) = app::test_request(
    &client,
    "GET",
    &format!("/api/entities/audit/{}", id),
    Some(&token),
    None,
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_audit_log_by_id_admin_200_or_404() {
  let client = app::test_client().await.expect("test_client");
  let (token, _org_id, _role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let (status_list, body_list) =
    app::test_request(&client, "GET", AUDIT_LIST, Some(&token), None, None)
      .await
      .unwrap();
  assert_eq!(status_list, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body_list).unwrap();
  let data = json["data"].as_array().unwrap();
  let id = data
    .first()
    .and_then(|e| e["id"].as_str())
    .unwrap_or("00000000-0000-0000-0000-000000000000");
  let (status, _) = app::test_request(
    &client,
    "GET",
    &format!("/api/entities/audit/{}", id),
    Some(&token),
    None,
    None,
  )
  .await
  .unwrap();
  assert!(status == StatusCode::OK || status == StatusCode::NOT_FOUND);
}
