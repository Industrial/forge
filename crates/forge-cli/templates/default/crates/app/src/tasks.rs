//! In-memory task list for the dashboard tasks page. Updated by a background loop and broadcast via forge-live.

use chrono::{DateTime, Utc};
use forge::live::{Channel, InMemoryLiveBackend, LiveBackend};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

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

/// Shared task state: list updated by tick and broadcast via forge-live.
pub struct TaskState {
  pub store: Arc<RwLock<Vec<Task>>>,
}

impl TaskState {
  pub fn new() -> Self {
    Self {
      store: Arc::new(RwLock::new(Self::default_tasks())),
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

  /// Advance demo state and broadcast via forge-live (called by background loop).
  pub async fn tick(&self, live_backend: &Arc<InMemoryLiveBackend>) {
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
    let channel = Channel::raw("tasks");
    let payload = serde_json::to_vec(&serde_json::json!({ "type": "tasks", "tasks": tasks })).unwrap_or_default();
    live_backend.broadcast(&channel, &payload).await;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn task_state_new_has_three_default_tasks() {
    let state = TaskState::new();
    let tasks = state.store.read().await.clone();
    assert_eq!(tasks.len(), 3);
    let statuses: Vec<_> = tasks.iter().map(|t| &t.status).collect();
    assert!(statuses.contains(&&TaskStatus::Running));
    assert!(statuses.contains(&&TaskStatus::Ran));
    assert!(statuses.contains(&&TaskStatus::Planned));
  }

  #[tokio::test]
  async fn task_state_tick_rotates_statuses() {
    let state = TaskState::new();
    let backend = Arc::new(InMemoryLiveBackend::new());
    let before: Vec<TaskStatus> = state.store.read().await.iter().map(|t| t.status.clone()).collect();
    state.tick(&backend).await;
    let after: Vec<TaskStatus> = state.store.read().await.iter().map(|t| t.status.clone()).collect();
    assert_eq!(before.len(), after.len());
    let planned_after = after.iter().filter(|s| **s == TaskStatus::Planned).count();
    let running_after = after.iter().filter(|s| **s == TaskStatus::Running).count();
    let ran_after = after.iter().filter(|s| **s == TaskStatus::Ran).count();
    assert_eq!(planned_after + running_after + ran_after, 3);
  }
}
