use axum::extract::FromRef;
use axum_inertia::InertiaConfig;
use forge::DbConnection;

/// Combined state for Inertia routes so both DbConnection and InertiaConfig can be extracted.
#[derive(Clone)]
pub struct AppState {
  pub db: DbConnection,
  pub inertia: InertiaConfig,
}

impl FromRef<AppState> for DbConnection {
  fn from_ref(input: &AppState) -> DbConnection {
    input.db.clone()
  }
}

impl FromRef<AppState> for InertiaConfig {
  fn from_ref(input: &AppState) -> InertiaConfig {
    input.inertia.clone()
  }
}
