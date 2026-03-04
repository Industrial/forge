//! Dashboard: GET /api/dashboard/tasks — 401 anon, 200 any auth.

use axum::http::StatusCode;

#[tokio::test]
async fn get_tasks_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", "/api/dashboard/tasks", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_tasks_auth_200() {
  let client = app::test_client().await.expect("test_client");
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = [
    ("X-Organization-Id", org_id.as_str()),
    ("X-Role-Id", role_id.as_str()),
  ];
  let (status, _) = app::test_request(
    &client,
    "GET",
    "/api/dashboard/tasks",
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
}
