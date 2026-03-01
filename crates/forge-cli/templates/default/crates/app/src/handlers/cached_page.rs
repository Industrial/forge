use axum::{extract::State, response::IntoResponse};
use forge::DbConnection;

pub async fn handler(State(_db): State<DbConnection>) -> impl IntoResponse {
  let v = format!("cached-page-{}", forge::uuid::Uuid::new_v4());
  v.into_response()
}
