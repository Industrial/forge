//! Shared Inertia data (auth, flash, app name) and partial-reload–aware props.
//! Merge these into every page and honor X-Inertia-Partial-Data when present.

use axum::response::IntoResponse;
use axum_inertia::{partial::Partial, props::Props, Inertia};
use serde_json::{Map, Value};
use tower_sessions::Session;

use db::models::user;

/// Session keys for one-time flash messages (read once then cleared).
pub const FLASH_MESSAGE: &str = "flash_message";
pub const FLASH_ERROR: &str = "flash_error";

const APP_NAME: &str = "App";

/// Build shared props: auth.user, flash (message + error), appName.
/// Reads flash from session and removes keys so they are one-time.
async fn shared_props(session: &Session, user: Option<user::Model>) -> Value {
  let message: Option<String> = session.get(FLASH_MESSAGE).await.ok().flatten();
  let error: Option<String> = session.get(FLASH_ERROR).await.ok().flatten();
  let _ = session.remove::<String>(FLASH_MESSAGE).await;
  let _ = session.remove::<String>(FLASH_ERROR).await;
  let auth_user = user.as_ref().map(|u| {
    serde_json::json!({
      "id": u.id.to_string(),
      "email": u.email,
    })
  });
  serde_json::json!({
    "auth": { "user": auth_user },
    "flash": {
      "message": message,
      "error": error,
    },
    "appName": APP_NAME,
  })
}

/// Clear flash keys after reading so they are one-time. Call after shared_props if you didn't flush.
pub async fn take_flash(session: &Session) -> (Option<String>, Option<String>) {
  let message: Option<String> = session.get(FLASH_MESSAGE).await.ok().flatten();
  let error: Option<String> = session.get(FLASH_ERROR).await.ok().flatten();
  session.remove::<String>(FLASH_MESSAGE).await.ok();
  session.remove::<String>(FLASH_ERROR).await.ok();
  (message, error)
}

/// Props that merge shared data with page props and support partial reloads (only requested keys).
pub struct MergedProps {
  pub shared: Value,
  pub page: Value,
}

impl Props for MergedProps {
  #[allow(refining_impl_trait)]
  fn serialize(self, partial: Option<&Partial>) -> Result<Value, std::convert::Infallible> {
    let merged = merge_json(&self.shared, &self.page);
    match partial {
      None => Ok(merged),
      Some(p) => {
        let requested: std::collections::HashSet<&str> = p.props.iter().map(|s| s.as_str()).collect();
        Ok(filter_object(&merged, &requested))
      }
    }
  }
}

fn merge_json(shared: &Value, page: &Value) -> Value {
  let mut out = match shared {
    Value::Object(m) => m.clone(),
    _ => Map::new(),
  };
  if let Value::Object(page_map) = page {
    for (k, v) in page_map {
      out.insert(k.clone(), v.clone());
    }
  }
  Value::Object(out)
}

fn filter_object(value: &Value, keys: &std::collections::HashSet<&str>) -> Value {
  match value {
    Value::Object(m) => {
      let out: Map<String, Value> = m
        .iter()
        .filter(|(k, _)| keys.contains(k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
      Value::Object(out)
    }
    _ => value.clone(),
  }
}

/// Render an Inertia page with shared data (auth, flash, appName) merged with page props.
/// Supports partial reloads when the client sends X-Inertia-Partial-Data.
/// Takes `Option<user::Model>` by value so the future does not capture a reference (Rust 2024).
pub async fn render_with_shared(
  i: Inertia,
  session: Session,
  user: Option<user::Model>,
  component: &str,
  page_props: Value,
) -> impl IntoResponse {
  let shared = shared_props(&session, user).await;
  let merged = MergedProps {
    shared,
    page: page_props,
  };
  i.render(component, merged)
}
