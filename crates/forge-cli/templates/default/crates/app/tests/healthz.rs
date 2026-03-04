//! Integration test: health endpoints (forge layer). Uses prebuilt server when E2E_API_URL is set.

#[tokio::test]
async fn get_healthz_livez_readyz_return_200() {
  let client = app::test_client().await.expect("test_client");

  for path in ["/healthz", "/livez", "/readyz"] {
    let (status, _) = app::test_request(&client, "GET", path, None, None, None)
      .await
      .unwrap();
    assert_eq!(
      status,
      axum::http::StatusCode::OK,
      "GET {} should return 200",
      path
    );
  }
}
