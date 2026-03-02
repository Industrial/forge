//! In-memory task list for the dashboard tasks page. Updated by a background loop and broadcast over WebSocket.
//! Also holds the audit-log broadcast for live audit entry updates.

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Status of a task for the dashboard.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
  Planned,
  Running,
  Ran,
}

/// A single task entry (scheduled job or run).
#[derive(Clone, Debug, Serialize)]
pub struct Task {
  pub id: String,
  pub name: String,
  pub status: TaskStatus,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub scheduled_at: Option<DateTime<Utc>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub started_at: Option<DateTime<Utc>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub finished_at: Option<DateTime<Utc>>,
}

/// Shared task state: list + broadcast sender for live updates; audit-log broadcast for new entries.
pub struct TaskState {
  pub store: Arc<RwLock<Vec<Task>>>,
  pub broadcast: broadcast::Sender<Vec<Task>>,
  /// Broadcasts a single audit log entry (JSON object) when new entries are written.
  pub audit_broadcast: broadcast::Sender<serde_json::Value>,
}

impl TaskState {
  pub fn new() -> Self {
    let (broadcast, _) = broadcast::channel(16);
    let (audit_broadcast, _) = broadcast::channel(64);
    Self {
      store: Arc::new(RwLock::new(Self::default_tasks())),
      broadcast,
      audit_broadcast,
    }
  }

  fn default_tasks() -> Vec<Task> {
    let now = Utc::now();
    vec![
      Task {
        id: "heartbeat-1".into(),
        name: "heartbeat".into(),
        status: TaskStatus::Running,
        scheduled_at: Some(now),
        started_at: Some(now),
        finished_at: None,
      },
      Task {
        id: "heartbeat-0".into(),
        name: "heartbeat".into(),
        status: TaskStatus::Ran,
        scheduled_at: Some(now - chrono::Duration::minutes(1)),
        started_at: Some(now - chrono::Duration::minutes(1)),
        finished_at: Some(now - chrono::Duration::seconds(30)),
      },
      Task {
        id: "heartbeat-2".into(),
        name: "heartbeat".into(),
        status: TaskStatus::Planned,
        scheduled_at: Some(now + chrono::Duration::minutes(1)),
        started_at: None,
        finished_at: None,
      },
    ]
  }

  /// Advance demo state and broadcast (called by background loop).
  pub async fn tick(&self) {
    let mut tasks = self.store.read().await.clone();
    let now = Utc::now();
    // Rotate: first planned -> running, first running -> ran, add new planned.
    if let Some(i) = tasks.iter().position(|t| t.status == TaskStatus::Planned) {
      tasks[i].status = TaskStatus::Running;
      tasks[i].started_at = Some(now);
    }
    if let Some(i) = tasks.iter().position(|t| t.status == TaskStatus::Running) {
      tasks[i].status = TaskStatus::Ran;
      tasks[i].finished_at = Some(now);
    }
    if let Some(i) = tasks.iter().position(|t| t.status == TaskStatus::Ran) {
      let name = tasks[i].name.clone();
      tasks[i] = Task {
        id: format!("{}-{}", name, now.timestamp_millis()),
        name: name.clone(),
        status: TaskStatus::Planned,
        scheduled_at: Some(now + chrono::Duration::minutes(1)),
        started_at: None,
        finished_at: None,
      };
    }
    *self.store.write().await = tasks.clone();
    let _ = self.broadcast.send(tasks);
  }
}
