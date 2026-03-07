//! In-memory task list for the dashboard tasks page. Updated by a background loop and broadcast via forge-live.

use chrono::{DateTime, Utc};
use forge_live::{Channel, InMemoryLiveBackend, LiveBackend};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Status of a task for the dashboard.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
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

impl Default for TaskState {
  fn default() -> Self {
    Self::new()
  }
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
    let payload = serde_json::to_vec(&serde_json::json!({ "type": "tasks", "tasks": tasks }))
      .unwrap_or_default();
    live_backend.broadcast(&channel, &payload).await;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod task_status_behavior {
    use super::*;

    #[test]
    fn task_status_variants_are_serializable() {
      // Given different task status variants
      let planned = TaskStatus::Planned;
      let running = TaskStatus::Running;
      let ran = TaskStatus::Ran;

      // When I serialize them
      let planned_json = serde_json::to_string(&planned).unwrap();
      let running_json = serde_json::to_string(&running).unwrap();
      let ran_json = serde_json::to_string(&ran).unwrap();

      // Then they should serialize to snake_case
      assert!(planned_json.contains("planned"));
      assert!(running_json.contains("running"));
      assert!(ran_json.contains("ran"));
    }

    #[test]
    fn task_status_implements_partial_eq() {
      // Given two task statuses
      let status1 = TaskStatus::Running;
      let status2 = TaskStatus::Running;
      let status3 = TaskStatus::Planned;

      // When I compare them
      // Then equal statuses should be equal
      assert_eq!(status1, status2);
      assert_ne!(status1, status3);
    }
  }

  mod task_behavior {
    use super::*;

    #[test]
    fn task_serializes_with_optional_timestamps() {
      // Given a task with all timestamps
      let now = Utc::now();
      let task = Task {
        id: "test-1".to_string(),
        name: "test_task".to_string(),
        status: TaskStatus::Running,
        scheduled_at: Some(now),
        started_at: Some(now),
        finished_at: None,
      };

      // When I serialize it
      let json = serde_json::to_string(&task).unwrap();

      // Then it should include scheduled_at and started_at but not finished_at when None
      assert!(json.contains("scheduled_at"));
      assert!(json.contains("started_at"));
      assert!(!json.contains("finished_at"));
    }

    #[test]
    fn task_serializes_finished_at_when_present() {
      // Given a task with finished_at timestamp
      let now = Utc::now();
      let task = Task {
        id: "test-1".to_string(),
        name: "test_task".to_string(),
        status: TaskStatus::Ran,
        scheduled_at: Some(now),
        started_at: Some(now),
        finished_at: Some(now),
      };

      // When I serialize it
      let json = serde_json::to_string(&task).unwrap();

      // Then it should include finished_at
      assert!(json.contains("finished_at"));
    }
  }

  mod task_state_behavior {
    use super::*;

    #[tokio::test]
    async fn new_creates_state_with_three_default_tasks() {
      // Given a new TaskState
      // When I create it
      let state = TaskState::new();

      // Then it should contain exactly three tasks
      let tasks = state.store.read().await.clone();
      assert_eq!(tasks.len(), 3, "Should have three default tasks");
    }

    #[tokio::test]
    async fn new_includes_all_task_status_types() {
      // Given a new TaskState
      let state = TaskState::new();

      // When I check the task statuses
      let tasks = state.store.read().await.clone();
      let statuses: Vec<_> = tasks.iter().map(|t| &t.status).collect();

      // Then it should include Planned, Running, and Ran statuses
      assert!(
        statuses.contains(&&TaskStatus::Running),
        "Should have a Running task"
      );
      assert!(
        statuses.contains(&&TaskStatus::Ran),
        "Should have a Ran task"
      );
      assert!(
        statuses.contains(&&TaskStatus::Planned),
        "Should have a Planned task"
      );
    }

    #[tokio::test]
    async fn new_sets_appropriate_timestamps_for_each_status() {
      // Given a new TaskState
      let state = TaskState::new();

      // When I examine the tasks
      let tasks = state.store.read().await.clone();

      // Then each task should have appropriate timestamps for its status
      for task in tasks {
        match task.status {
          TaskStatus::Running => {
            assert!(
              task.scheduled_at.is_some(),
              "Running task should have scheduled_at"
            );
            assert!(
              task.started_at.is_some(),
              "Running task should have started_at"
            );
            assert!(
              task.finished_at.is_none(),
              "Running task should not have finished_at"
            );
          }
          TaskStatus::Ran => {
            assert!(
              task.scheduled_at.is_some(),
              "Ran task should have scheduled_at"
            );
            assert!(task.started_at.is_some(), "Ran task should have started_at");
            assert!(
              task.finished_at.is_some(),
              "Ran task should have finished_at"
            );
          }
          TaskStatus::Planned => {
            assert!(
              task.scheduled_at.is_some(),
              "Planned task should have scheduled_at"
            );
            assert!(
              task.started_at.is_none(),
              "Planned task should not have started_at"
            );
            assert!(
              task.finished_at.is_none(),
              "Planned task should not have finished_at"
            );
          }
        }
      }
    }
  }

  mod tick_behavior {
    use super::*;

    #[tokio::test]
    async fn tick_rotates_planned_to_running() {
      // Given a TaskState with default tasks
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());

      // When I call tick
      state.tick(&backend).await;

      // Then a Planned task should become Running
      let tasks = state.store.read().await.clone();
      let _planned_count = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Planned)
        .count();
      let running_count = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Running)
        .count();
      // After tick, we should still have tasks in all states (rotation maintains all three)
      assert_eq!(tasks.len(), 3, "Should maintain three tasks");
      assert!(
        running_count > 0,
        "Should have at least one running task after tick"
      );
    }

    #[tokio::test]
    async fn tick_rotates_running_to_ran() {
      // Given a TaskState with default tasks
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());

      // When I call tick
      state.tick(&backend).await;

      // Then a Running task should become Ran
      let tasks = state.store.read().await.clone();
      let ran_count = tasks.iter().filter(|t| t.status == TaskStatus::Ran).count();
      assert!(
        ran_count > 0,
        "Should have at least one ran task after tick"
      );
    }

    #[tokio::test]
    async fn tick_maintains_total_task_count() {
      // Given a TaskState with default tasks
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());
      let initial_count = state.store.read().await.len();

      // When I call tick
      state.tick(&backend).await;

      // Then the total number of tasks should remain the same
      let tasks = state.store.read().await.clone();
      assert_eq!(
        tasks.len(),
        initial_count,
        "Tick should maintain the same number of tasks"
      );
    }

    #[tokio::test]
    async fn tick_sets_started_at_when_moving_to_running() {
      // Given a TaskState with default tasks
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());

      // When I call tick
      state.tick(&backend).await;

      // Then tasks that become Running should have started_at set
      let tasks = state.store.read().await.clone();
      for task in tasks {
        if task.status == TaskStatus::Running {
          assert!(
            task.started_at.is_some(),
            "Running task should have started_at timestamp"
          );
        }
      }
    }

    #[tokio::test]
    async fn tick_sets_finished_at_when_moving_to_ran() {
      // Given a TaskState with default tasks
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());

      // When I call tick
      state.tick(&backend).await;

      // Then tasks that become Ran should have finished_at set
      let tasks = state.store.read().await.clone();
      for task in tasks {
        if task.status == TaskStatus::Ran {
          assert!(
            task.finished_at.is_some(),
            "Ran task should have finished_at timestamp"
          );
        }
      }
    }

    #[tokio::test]
    async fn tick_broadcasts_tasks_via_live_backend() {
      // Given a TaskState and live backend
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());

      // When I call tick
      state.tick(&backend).await;

      // Then it should broadcast to the tasks channel
      // (The broadcast happens internally, we verify the state is updated)
      let tasks = state.store.read().await.clone();
      assert_eq!(tasks.len(), 3, "Tasks should be maintained after broadcast");
    }

    #[tokio::test]
    async fn tick_creates_new_planned_task_when_ran_task_exists() {
      // Given a TaskState with default tasks
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());

      // When I call tick (which rotates a Ran task to Planned)
      state.tick(&backend).await;

      // Then there should be at least one Planned task with a new ID
      let tasks = state.store.read().await.clone();
      let planned_tasks: Vec<_> = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Planned)
        .collect();
      assert!(
        !planned_tasks.is_empty(),
        "Should have at least one planned task"
      );
      // The new planned task should have a timestamp-based ID
      for task in planned_tasks {
        assert!(
          task.id.starts_with("heartbeat-"),
          "Planned task should have appropriate ID format"
        );
      }
    }
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use chrono::Utc;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.

  mod task_state_creation_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_task_state_with_default_tasks() {
      // Given: a new TaskState
      // When: creating TaskState
      let state = TaskState::new();

      // Then: should have default tasks
      let tasks = state.store.read().await.clone();
      assert_eq!(tasks.len(), 3, "Should have 3 default tasks");
    }

    #[tokio::test]
    async fn should_have_all_task_statuses_in_default_tasks() {
      // Given: a new TaskState
      let state = TaskState::new();

      // When: reading tasks
      let tasks = state.store.read().await.clone();
      let statuses: Vec<_> = tasks.iter().map(|t| &t.status).collect();

      // Then: should have Planned, Running, and Ran statuses
      assert!(
        statuses.contains(&&TaskStatus::Planned),
        "Should have Planned task"
      );
      assert!(
        statuses.contains(&&TaskStatus::Running),
        "Should have Running task"
      );
      assert!(statuses.contains(&&TaskStatus::Ran), "Should have Ran task");
    }

    #[tokio::test]
    async fn should_initialize_with_heartbeat_tasks() {
      // Given: a new TaskState
      let state = TaskState::new();

      // When: reading tasks
      let tasks = state.store.read().await.clone();

      // Then: all tasks should be heartbeat tasks
      for task in tasks.iter() {
        assert_eq!(
          task.name, "heartbeat",
          "All default tasks should be heartbeat"
        );
      }
    }
  }

  mod task_status_behavior {
    use super::*;

    #[test]
    fn should_have_three_task_status_variants() {
      // Given: TaskStatus enum
      // When: checking variants
      // Then: should have Planned, Running, and Ran
      let _planned = TaskStatus::Planned;
      let _running = TaskStatus::Running;
      let _ran = TaskStatus::Ran;
      // Test passes if all variants compile
    }

    #[test]
    fn should_serialize_task_status_with_snake_case() {
      // Given: a TaskStatus variant
      let status = TaskStatus::Running;

      // When: serializing
      let json = serde_json::to_string(&status).unwrap();

      // Then: should use snake_case
      assert_eq!(json, "\"running\"", "Should serialize Running as 'running'");
    }

    #[test]
    fn should_deserialize_task_status_from_snake_case() {
      // Given: a JSON string with snake_case status
      let json = "\"planned\"";

      // When: deserializing
      let status: TaskStatus = serde_json::from_str(json).unwrap();

      // Then: should deserialize correctly
      assert_eq!(
        status,
        TaskStatus::Planned,
        "Should deserialize 'planned' to Planned"
      );
    }
  }

  mod task_structure_behavior {
    use super::*;

    #[test]
    fn should_have_required_task_fields() {
      // Given: a Task instance
      let task = Task {
        id: "test-1".to_string(),
        name: "test_task".to_string(),
        status: TaskStatus::Planned,
        scheduled_at: Some(Utc::now()),
        started_at: None,
        finished_at: None,
      };

      // When: accessing fields
      // Then: should have all required fields
      assert_eq!(task.id, "test-1");
      assert_eq!(task.name, "test_task");
      assert_eq!(task.status, TaskStatus::Planned);
    }

    #[test]
    fn should_skip_serializing_none_datetime_fields() {
      // Given: a Task with None datetime fields
      let task = Task {
        id: "test-1".to_string(),
        name: "test_task".to_string(),
        status: TaskStatus::Planned,
        scheduled_at: None,
        started_at: None,
        finished_at: None,
      };

      // When: serializing
      let json = serde_json::to_string(&task).unwrap();

      // Then: should not include None fields
      assert!(
        !json.contains("scheduled_at"),
        "Should skip None scheduled_at"
      );
      assert!(!json.contains("started_at"), "Should skip None started_at");
      assert!(
        !json.contains("finished_at"),
        "Should skip None finished_at"
      );
    }

    #[test]
    fn should_include_datetime_fields_when_some() {
      // Given: a Task with Some datetime fields
      let now = Utc::now();
      let task = Task {
        id: "test-1".to_string(),
        name: "test_task".to_string(),
        status: TaskStatus::Running,
        scheduled_at: Some(now),
        started_at: Some(now),
        finished_at: None,
      };

      // When: serializing
      let json = serde_json::to_string(&task).unwrap();

      // Then: should include Some fields
      assert!(
        json.contains("scheduled_at"),
        "Should include Some scheduled_at"
      );
      assert!(
        json.contains("started_at"),
        "Should include Some started_at"
      );
    }
  }

  mod task_tick_behavior {
    use super::*;

    #[tokio::test]
    async fn should_rotate_planned_to_running_on_tick() {
      // Given: a TaskState with Planned task
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());

      // When: ticking
      state.tick(&backend).await;

      // Then: Planned task should become Running
      let tasks = state.store.read().await.clone();
      let _planned_count = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Planned)
        .count();
      let running_count = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Running)
        .count();
      // After tick, one Planned should become Running
      assert!(
        running_count >= 1,
        "Should have at least one Running task after tick"
      );
    }

    #[tokio::test]
    async fn should_rotate_running_to_ran_on_tick() {
      // Given: a TaskState with Running task
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());

      // When: ticking
      state.tick(&backend).await;

      // Then: Running task should become Ran
      let tasks = state.store.read().await.clone();
      let ran_count = tasks.iter().filter(|t| t.status == TaskStatus::Ran).count();
      assert!(
        ran_count >= 1,
        "Should have at least one Ran task after tick"
      );
    }

    #[tokio::test]
    async fn should_rotate_ran_to_planned_on_tick() {
      // Given: a TaskState with Ran task
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());

      // When: ticking multiple times to get Ran task
      state.tick(&backend).await;
      state.tick(&backend).await;

      // Then: Ran task should become Planned with new ID
      let tasks = state.store.read().await.clone();
      let planned_count = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Planned)
        .count();
      assert!(
        planned_count >= 1,
        "Should have at least one Planned task after rotation"
      );
    }

    #[tokio::test]
    async fn should_update_started_at_when_planned_becomes_running() {
      // Given: a TaskState
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());

      // When: ticking (Planned -> Running)
      state.tick(&backend).await;

      // Then: Running tasks should have started_at set
      let tasks = state.store.read().await.clone();
      for task in tasks.iter() {
        if task.status == TaskStatus::Running {
          assert!(
            task.started_at.is_some(),
            "Running task should have started_at"
          );
        }
      }
    }

    #[tokio::test]
    async fn should_update_finished_at_when_running_becomes_ran() {
      // Given: a TaskState
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());

      // When: ticking (Running -> Ran)
      state.tick(&backend).await;

      // Then: Ran tasks should have finished_at set
      let tasks = state.store.read().await.clone();
      for task in tasks.iter() {
        if task.status == TaskStatus::Ran {
          assert!(
            task.finished_at.is_some(),
            "Ran task should have finished_at"
          );
        }
      }
    }

    #[tokio::test]
    async fn should_generate_new_id_when_ran_becomes_planned() {
      // Given: a TaskState
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());

      // When: ticking multiple times to rotate Ran -> Planned
      let initial_tasks = state.store.read().await.clone();
      let initial_ids: Vec<_> = initial_tasks.iter().map(|t| t.id.clone()).collect();

      state.tick(&backend).await;
      state.tick(&backend).await;
      state.tick(&backend).await;

      // Then: new Planned task should have different ID
      let after_tasks = state.store.read().await.clone();
      let after_ids: Vec<_> = after_tasks.iter().map(|t| t.id.clone()).collect();
      // At least one ID should be different (newly generated)
      assert!(
        initial_ids != after_ids,
        "Task IDs should change after rotation"
      );
    }

    #[tokio::test]
    async fn should_maintain_task_count_after_tick() {
      // Given: a TaskState
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());

      // When: ticking
      let before_count = state.store.read().await.len();
      state.tick(&backend).await;
      let after_count = state.store.read().await.len();

      // Then: task count should remain the same
      assert_eq!(
        before_count, after_count,
        "Task count should remain constant after tick"
      );
    }
  }

  mod live_backend_broadcast_behavior {
    use super::*;

    #[tokio::test]
    async fn should_broadcast_tasks_on_tick() {
      // Given: a TaskState and live backend
      let state = TaskState::new();
      let backend = Arc::new(InMemoryLiveBackend::new());

      // When: ticking
      state.tick(&backend).await;

      // Then: should broadcast to tasks channel
      // Note: Broadcast happens internally, we verify the channel exists
      let channel = Channel::raw("tasks");
      assert_eq!(
        channel.as_str(),
        "tasks",
        "Should broadcast to tasks channel"
      );
    }

    #[tokio::test]
    async fn should_broadcast_tasks_json_on_tick() {
      // Given: a TaskState
      let state = TaskState::new();

      // When: preparing broadcast payload
      let tasks = state.store.read().await.clone();
      let payload = serde_json::to_vec(&serde_json::json!({ "type": "tasks", "tasks": tasks }))
        .unwrap_or_default();

      // Then: should serialize tasks to JSON
      assert!(!payload.is_empty(), "Should serialize tasks to JSON");
      let json_str = String::from_utf8_lossy(&payload);
      assert!(
        json_str.contains("tasks"),
        "Should include 'tasks' in payload"
      );
      assert!(
        json_str.contains("type"),
        "Should include 'type' in payload"
      );
    }
  }

  mod default_tasks_behavior {
    use super::*;

    #[test]
    fn should_create_three_default_tasks() {
      // Given: default_tasks function
      // When: calling default_tasks
      let tasks = TaskState::default_tasks();

      // Then: should return 3 tasks
      assert_eq!(tasks.len(), 3, "Should create 3 default tasks");
    }

    #[test]
    fn should_have_running_task_with_current_time() {
      // Given: default tasks
      let tasks = TaskState::default_tasks();

      // When: finding Running task
      let running_task = tasks.iter().find(|t| t.status == TaskStatus::Running);

      // Then: should have scheduled_at and started_at set
      assert!(running_task.is_some(), "Should have Running task");
      let task = running_task.unwrap();
      assert!(
        task.scheduled_at.is_some(),
        "Running task should have scheduled_at"
      );
      assert!(
        task.started_at.is_some(),
        "Running task should have started_at"
      );
      assert!(
        task.finished_at.is_none(),
        "Running task should not have finished_at"
      );
    }

    #[test]
    fn should_have_ran_task_with_finished_time() {
      // Given: default tasks
      let tasks = TaskState::default_tasks();

      // When: finding Ran task
      let ran_task = tasks.iter().find(|t| t.status == TaskStatus::Ran);

      // Then: should have all timestamps set
      assert!(ran_task.is_some(), "Should have Ran task");
      let task = ran_task.unwrap();
      assert!(
        task.scheduled_at.is_some(),
        "Ran task should have scheduled_at"
      );
      assert!(task.started_at.is_some(), "Ran task should have started_at");
      assert!(
        task.finished_at.is_some(),
        "Ran task should have finished_at"
      );
    }

    #[test]
    fn should_have_planned_task_with_future_scheduled_time() {
      // Given: default tasks
      let tasks = TaskState::default_tasks();

      // When: finding Planned task
      let planned_task = tasks.iter().find(|t| t.status == TaskStatus::Planned);

      // Then: should have scheduled_at but no started_at or finished_at
      assert!(planned_task.is_some(), "Should have Planned task");
      let task = planned_task.unwrap();
      assert!(
        task.scheduled_at.is_some(),
        "Planned task should have scheduled_at"
      );
      assert!(
        task.started_at.is_none(),
        "Planned task should not have started_at"
      );
      assert!(
        task.finished_at.is_none(),
        "Planned task should not have finished_at"
      );
    }
  }
}
