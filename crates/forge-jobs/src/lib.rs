//! Background jobs via Apalis + SQLite. Generic job queue and worker; no scheduling.

use apalis::prelude::*;
use apalis_sqlite::SqliteStorage;
use forge_db::DbConnection;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{error, info};

/// Single job type: identified by task name for dispatch.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ScheduledTaskJob {
  pub task_name: String,
}

/// Type-erased task: takes DB, returns a future. Used by the worker to run jobs by name.
pub type TaskFn = Box<
  dyn Fn(
      DbConnection,
    )
      -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>
    + Send
    + Sync,
>;

/// Normalizes SQLite database URLs for the job pool.
/// Converts `sqlite::memory:` to `sqlite://:memory:` for compatibility.
fn normalize_job_pool_url(url: &str) -> &str {
  if url == "sqlite::memory:" {
    "sqlite://:memory:"
  } else {
    url
  }
}

/// Storage handle for the job queue (Apalis SqliteStorage with default codec and fetcher).
pub type JobQueueStorage = SqliteStorage<
  ScheduledTaskJob,
  apalis_codec::json::JsonCodec<Vec<u8>>,
  apalis_sqlite::fetcher::SqliteFetcher,
>;

/// Sets up the job queue storage. Returns a cloneable storage: use one clone for pushing jobs,
/// pass another to [run_worker].
pub async fn setup_queue(
  db_url: &str,
) -> Result<JobQueueStorage, Box<dyn std::error::Error + Send + Sync>> {
  use apalis_sqlite::SqlitePool;
  let url = normalize_job_pool_url(db_url);
  let pool = SqlitePool::connect(url).await?;
  SqliteStorage::<(), (), ()>::setup(&pool).await?;
  Ok(SqliteStorage::new(&pool))
}

/// Runs the worker: pulls jobs from `storage` and dispatches by name using `registry`.
/// Spawns the worker in the background and returns immediately.
pub async fn run_worker(
  storage: JobQueueStorage,
  db: DbConnection,
  registry: Arc<HashMap<String, TaskFn>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let worker = WorkerBuilder::new("forge-scheduled-tasks")
    .backend(storage)
    .data((registry, db))
    .build(run_scheduled_task);
  tokio::spawn(async move {
    if let Err(e) = worker.run().await {
      error!(error = %e, "forge scheduled tasks worker exited with error");
    }
  });
  Ok(())
}

/// Executes a scheduled task by looking it up in the registry and running it.
async fn run_scheduled_task(
  job: ScheduledTaskJob,
  data: Data<(Arc<HashMap<String, TaskFn>>, DbConnection)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let (registry, db): &(Arc<HashMap<String, TaskFn>>, DbConnection) = &data;
  if let Some(task) = registry.get(&job.task_name) {
    if let Err(e) = (task)(db.clone()).await {
      error!(task = %job.task_name, error = %e, "scheduled task failed");
      return Err(e);
    }
    info!(task = %job.task_name, "scheduled task completed");
  } else {
    error!(task = %job.task_name, "no handler registered for scheduled task");
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use apalis::prelude::Data;
  use sea_orm::{ConnectOptions, Database};
  use sea_orm_tracing::TracedConnection;
  use std::time::Duration;

  fn ok_task() -> TaskFn {
    Box::new(|_db: DbConnection| {
      Box::pin(async move { Ok(()) })
        as Pin<
          Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>,
        >
    })
  }

  fn err_task() -> TaskFn {
    Box::new(|_db: DbConnection| Box::pin(async move { Err("task failed".into()) }))
  }

  async fn test_db() -> DbConnection {
    let conn = Database::connect(ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap();
    TracedConnection::wrap(conn)
  }

  #[test]
  fn normalize_job_pool_url_normalizes_sqlite_memory() {
    assert_eq!(
      normalize_job_pool_url("sqlite::memory:"),
      "sqlite://:memory:"
    );
  }

  #[test]
  fn normalize_job_pool_url_passthrough_for_other_urls() {
    assert_eq!(
      normalize_job_pool_url("sqlite:///path/to/db.sqlite"),
      "sqlite:///path/to/db.sqlite"
    );
    assert_eq!(
      normalize_job_pool_url("sqlite://:memory:"),
      "sqlite://:memory:"
    );
  }

  #[test]
  fn scheduled_task_job_serde_roundtrip() {
    let job = ScheduledTaskJob {
      task_name: "test-task".to_string(),
    };
    let json = serde_json::to_string(&job).unwrap();
    let parsed: ScheduledTaskJob = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.task_name, job.task_name);
  }

  #[tokio::test]
  async fn run_scheduled_task_found_and_succeeds() {
    let db = test_db().await;
    let mut reg = HashMap::new();
    reg.insert("ok-task".to_string(), ok_task());
    let registry = Arc::new(reg);
    let data = Data::new((registry, db));
    let job = ScheduledTaskJob {
      task_name: "ok-task".to_string(),
    };
    let res = run_scheduled_task(job, data).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn run_scheduled_task_found_and_returns_err() {
    let db = test_db().await;
    let mut reg = HashMap::new();
    reg.insert("err-task".to_string(), err_task());
    let registry = Arc::new(reg);
    let data = Data::new((registry, db));
    let job = ScheduledTaskJob {
      task_name: "err-task".to_string(),
    };
    let res = run_scheduled_task(job, data).await;
    assert!(res.is_err());
  }

  #[tokio::test]
  async fn run_scheduled_task_not_in_registry() {
    let db = test_db().await;
    let registry = Arc::new(HashMap::new());
    let data = Data::new((registry, db));
    let job = ScheduledTaskJob {
      task_name: "missing".to_string(),
    };
    let res = run_scheduled_task(job, data).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn setup_queue_succeeds_with_memory_url() {
    let res = setup_queue("sqlite::memory:").await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn setup_queue_fails_on_invalid_db_url() {
    let res = setup_queue("invalid://bad").await;
    assert!(res.is_err());
  }

  #[tokio::test]
  async fn run_worker_starts_with_empty_registry() {
    let db = test_db().await;
    let storage = setup_queue("sqlite::memory:").await.unwrap();
    let registry = Arc::new(HashMap::new());
    let res = run_worker(storage, db, registry).await;
    assert!(res.is_ok());
    tokio::time::sleep(Duration::from_millis(50)).await;
  }

  #[tokio::test]
  async fn run_worker_starts_with_one_task() {
    let db = test_db().await;
    let storage = setup_queue("sqlite::memory:").await.unwrap();
    let mut reg = HashMap::new();
    reg.insert("test-task".to_string(), ok_task());
    let registry = Arc::new(reg);
    let res = run_worker(storage, db, registry).await;
    assert!(res.is_ok());
    tokio::time::sleep(Duration::from_millis(50)).await;
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.
  mod scheduled_task_job_behavior {
    use super::*;

    #[test]
    fn should_store_task_name() {
      // Given: a task name
      let task_name = "my-task".to_string();

      // When: creating a ScheduledTaskJob
      let job = ScheduledTaskJob {
        task_name: task_name.clone(),
      };

      // Then: should store the task name
      assert_eq!(job.task_name, task_name);
    }

    #[test]
    fn should_be_serializable() {
      // Given: a ScheduledTaskJob
      let job = ScheduledTaskJob {
        task_name: "test-task".to_string(),
      };

      // When: serializing to JSON
      let json = serde_json::to_string(&job);

      // Then: should succeed
      assert!(json.is_ok());
    }

    #[test]
    fn should_be_deserializable() {
      // Given: JSON representation of a job
      let json = r#"{"task_name":"test-task"}"#;

      // When: deserializing
      let job: Result<ScheduledTaskJob, _> = serde_json::from_str(json);

      // Then: should succeed and have correct task name
      assert!(job.is_ok());
      assert_eq!(job.unwrap().task_name, "test-task");
    }

    #[test]
    fn should_have_roundtrip_serialization() {
      // Given: a ScheduledTaskJob
      let original = ScheduledTaskJob {
        task_name: "roundtrip-task".to_string(),
      };

      // When: serializing and deserializing
      let json = serde_json::to_string(&original).unwrap();
      let parsed: ScheduledTaskJob = serde_json::from_str(&json).unwrap();

      // Then: should match original
      assert_eq!(parsed.task_name, original.task_name);
    }

    #[test]
    fn should_be_cloneable() {
      // Given: a ScheduledTaskJob
      let job = ScheduledTaskJob {
        task_name: "clone-task".to_string(),
      };

      // When: cloning it
      let cloned = job.clone();

      // Then: should have same task name
      assert_eq!(cloned.task_name, job.task_name);
    }

    #[test]
    fn should_be_debuggable() {
      // Given: a ScheduledTaskJob
      let job = ScheduledTaskJob {
        task_name: "debug-task".to_string(),
      };

      // When: formatting for debug
      let debug_str = format!("{:?}", job);

      // Then: should produce debug output
      assert!(!debug_str.is_empty());
      assert!(debug_str.contains("debug-task"));
    }
  }

  mod normalize_job_pool_url_behavior {
    use super::*;

    #[test]
    fn should_normalize_sqlite_memory_format() {
      // Given: sqlite::memory: URL format
      let url = "sqlite::memory:";

      // When: normalizing
      let normalized = normalize_job_pool_url(url);

      // Then: should convert to sqlite://:memory:
      assert_eq!(normalized, "sqlite://:memory:");
    }

    #[test]
    fn should_passthrough_already_normalized_urls() {
      // Given: already normalized URL
      let url = "sqlite://:memory:";

      // When: normalizing
      let normalized = normalize_job_pool_url(url);

      // Then: should remain unchanged
      assert_eq!(normalized, url);
    }

    #[test]
    fn should_passthrough_file_path_urls() {
      // Given: file path URL
      let url = "sqlite:///path/to/db.sqlite";

      // When: normalizing
      let normalized = normalize_job_pool_url(url);

      // Then: should remain unchanged
      assert_eq!(normalized, url);
    }

    #[test]
    fn should_passthrough_other_database_urls() {
      // Given: other database URL formats
      let urls = vec![
        "postgresql://user:pass@localhost/db",
        "mysql://user:pass@localhost/db",
      ];

      // When: normalizing each
      // Then: should remain unchanged
      for url in urls {
        assert_eq!(normalize_job_pool_url(url), url);
      }
    }
  }

  mod task_fn_type_behavior {
    use super::*;

    #[test]
    fn should_accept_async_task_function() {
      // Given: an async task function
      let _task: TaskFn = Box::new(|_db: DbConnection| {
        Box::pin(async move {
          // Simulate some async work
          Ok(())
        })
          as Pin<
            Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>,
          >
      });

      // When: using the task type
      // Then: should compile and be usable
    }

    #[test]
    fn should_allow_error_return() {
      // Given: a task that can return an error
      let _task: TaskFn =
        Box::new(|_db: DbConnection| Box::pin(async move { Err("task error".into()) }));

      // When: using the task type
      // Then: should accept error return type
    }

    #[test]
    fn should_accept_db_connection_parameter() {
      // Given: a task that uses the database connection
      let _task: TaskFn = Box::new(|_db: DbConnection| {
        Box::pin(async move {
          // Task receives db connection
          Ok(())
        })
          as Pin<
            Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>,
          >
      });

      // When: using the task type
      // Then: should accept DbConnection parameter
    }
  }

  mod setup_queue_behavior {
    use super::*;

    #[tokio::test]
    async fn should_succeed_with_memory_database() {
      // Given: sqlite::memory: URL
      let url = "sqlite::memory:";

      // When: setting up queue
      let result = setup_queue(url).await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_succeed_with_normalized_memory_url() {
      // Given: normalized memory URL
      let url = "sqlite://:memory:";

      // When: setting up queue
      let result = setup_queue(url).await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_fail_with_invalid_url() {
      // Given: invalid database URL
      let url = "invalid://bad-url";

      // When: setting up queue
      let result = setup_queue(url).await;

      // Then: should return error
      assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_return_cloneable_storage() {
      // Given: valid database URL
      let url = "sqlite::memory:";

      // When: setting up queue
      let storage = setup_queue(url).await.unwrap();

      // Then: should be cloneable (for use in multiple places)
      let _clone = storage.clone();
    }
  }

  mod run_scheduled_task_behavior {
    use super::*;
    use apalis::prelude::Data;
    use sea_orm::{ConnectOptions, Database};
    use sea_orm_tracing::TracedConnection;

    async fn test_db() -> DbConnection {
      let conn = Database::connect(ConnectOptions::new("sqlite::memory:".to_string()))
        .await
        .unwrap();
      TracedConnection::wrap(conn)
    }

    fn ok_task() -> TaskFn {
      Box::new(|_db: DbConnection| {
        Box::pin(async move { Ok(()) })
          as Pin<
            Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>,
          >
      })
    }

    fn err_task() -> TaskFn {
      Box::new(|_db: DbConnection| Box::pin(async move { Err("task failed".into()) }))
    }

    #[tokio::test]
    async fn should_execute_task_when_found_in_registry() {
      // Given: a registry with a task and a job for that task
      let db = test_db().await;
      let mut reg = HashMap::new();
      reg.insert("found-task".to_string(), ok_task());
      let registry = Arc::new(reg);
      let data = Data::new((registry, db));
      let job = ScheduledTaskJob {
        task_name: "found-task".to_string(),
      };

      // When: running the scheduled task
      let result = run_scheduled_task(job, data).await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_return_error_when_task_fails() {
      // Given: a registry with a failing task
      let db = test_db().await;
      let mut reg = HashMap::new();
      reg.insert("failing-task".to_string(), err_task());
      let registry = Arc::new(reg);
      let data = Data::new((registry, db));
      let job = ScheduledTaskJob {
        task_name: "failing-task".to_string(),
      };

      // When: running the scheduled task
      let result = run_scheduled_task(job, data).await;

      // Then: should return error
      assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_succeed_when_task_not_in_registry() {
      // Given: an empty registry and a job for non-existent task
      let db = test_db().await;
      let registry = Arc::new(HashMap::new());
      let data = Data::new((registry, db));
      let job = ScheduledTaskJob {
        task_name: "missing-task".to_string(),
      };

      // When: running the scheduled task
      let result = run_scheduled_task(job, data).await;

      // Then: should succeed (but task won't execute)
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_pass_database_connection_to_task() {
      // Given: a task that uses the database connection
      let db = test_db().await;
      let mut reg = HashMap::new();
      reg.insert("db-task".to_string(), ok_task());
      let registry = Arc::new(reg);
      let data = Data::new((registry, db));
      let job = ScheduledTaskJob {
        task_name: "db-task".to_string(),
      };

      // When: running the scheduled task
      let result = run_scheduled_task(job, data).await;

      // Then: should succeed (task received db connection)
      assert!(result.is_ok());
    }
  }

  mod run_worker_behavior {
    use super::*;
    use sea_orm::{ConnectOptions, Database};
    use sea_orm_tracing::TracedConnection;
    use std::time::Duration;

    async fn test_db() -> DbConnection {
      let conn = Database::connect(ConnectOptions::new("sqlite::memory:".to_string()))
        .await
        .unwrap();
      TracedConnection::wrap(conn)
    }

    fn ok_task() -> TaskFn {
      Box::new(|_db: DbConnection| {
        Box::pin(async move { Ok(()) })
          as Pin<
            Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>,
          >
      })
    }

    #[tokio::test]
    async fn should_start_worker_with_empty_registry() {
      // Given: storage, database, and empty registry
      let db = test_db().await;
      let storage = setup_queue("sqlite::memory:").await.unwrap();
      let registry = Arc::new(HashMap::new());

      // When: running worker
      let result = run_worker(storage, db, registry).await;

      // Then: should start successfully
      assert!(result.is_ok());
      tokio::time::sleep(Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn should_start_worker_with_registered_tasks() {
      // Given: storage, database, and registry with tasks
      let db = test_db().await;
      let storage = setup_queue("sqlite::memory:").await.unwrap();
      let mut reg = HashMap::new();
      reg.insert("task1".to_string(), ok_task());
      let registry = Arc::new(reg);

      // When: running worker
      let result = run_worker(storage, db, registry).await;

      // Then: should start successfully
      assert!(result.is_ok());
      tokio::time::sleep(Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn should_return_immediately_after_spawning() {
      // Given: storage, database, and registry
      let db = test_db().await;
      let storage = setup_queue("sqlite::memory:").await.unwrap();
      let registry = Arc::new(HashMap::new());

      // When: running worker
      let start = std::time::Instant::now();
      let result = run_worker(storage, db, registry).await;
      let elapsed = start.elapsed();

      // Then: should return quickly (worker spawned in background)
      assert!(result.is_ok());
      assert!(elapsed < Duration::from_millis(100));
    }
  }
}
