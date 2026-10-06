//! Tracking of submitted writes so shutdown can stop admission and await completion.

use std::{
  collections::HashMap,
  path::PathBuf,
  sync::{Arc, Condvar, Mutex},
};

use crate::TaskId;

/// Queued, active, pending-commit, and failed writes observed at one point in time.
///
/// # Fields
///
/// * `accepting_writes` - The accepting writes.
/// * `writing` - Paths currently being written.
/// * `queued` - Paths admitted but not yet started.
/// * `pending_commits` - Temporary paths awaiting final commit.
/// * `failed` - Failed write paths paired with their diagnostic messages.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WriteBarrierSnapshot {
  /// The accepting writes.
  pub accepting_writes: bool,
  /// Paths currently being written.
  pub writing: Vec<PathBuf>,
  /// Paths admitted but not yet started.
  pub queued: Vec<PathBuf>,
  /// Temporary paths awaiting final commit.
  pub pending_commits: Vec<PathBuf>,
  /// Failed write paths paired with their diagnostic messages.
  pub failed: Vec<(PathBuf, String)>,
}

#[derive(Default)]
struct WriteBarrierState {
  accepting_writes: bool,
  writing: HashMap<TaskId, PathBuf>,
  queued: HashMap<TaskId, PathBuf>,
  pending_commits: HashMap<TaskId, PathBuf>,
  failed: Vec<(PathBuf, String)>,
}

/// A shared write tracker that blocks shutdown until registered writes reach terminal states.
#[derive(Clone)]
pub struct WriteBarrier {
  shared: Arc<(Mutex<WriteBarrierState>, Condvar)>,
}

impl WriteBarrier {
  /// Create a write barrier with its initial state.
  pub fn new() -> Self {
    let state = WriteBarrierState {
      accepting_writes: true,
      ..Default::default()
    };
    Self {
      shared: Arc::new((Mutex::new(state), Condvar::new())),
    }
  }

  /// Track a queued write and optional temporary commit path, returning false after admission
  /// closes.
  ///
  /// # Arguments
  ///
  /// * `task_id` - The identifier of the asynchronous task.
  /// * `target` - The object or resource affected by the operation.
  /// * `temporary` - The temporary.
  pub fn register(&self, task_id: TaskId, target: PathBuf, temporary: Option<PathBuf>) -> bool {
    let (lock, _) = &*self.shared;
    let mut state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
    if !state.accepting_writes {
      return false;
    }
    state.queued.insert(task_id, target);
    if let Some(temporary) = temporary {
      state.pending_commits.insert(task_id, temporary);
    }
    true
  }

  /// Move a registered write from the queued set into the active set.
  pub fn start(&self, task_id: TaskId) {
    let (lock, _) = &*self.shared;
    let mut state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
    if let Some(path) = state.queued.remove(&task_id) {
      state.writing.insert(task_id, path);
    }
  }

  /// Remove a completed write from all pending sets and wake shutdown waiters.
  pub fn finish(&self, task_id: TaskId) {
    let (lock, wake) = &*self.shared;
    let mut state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
    state.queued.remove(&task_id);
    state.writing.remove(&task_id);
    state.pending_commits.remove(&task_id);
    wake.notify_all();
  }

  /// Mark the tracked write as failed, retain its diagnostic, and wake shutdown waiters.
  pub fn fail(&self, task_id: TaskId, error: String) {
    let (lock, wake) = &*self.shared;
    let mut state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
    if let Some(path) = state
      .writing
      .remove(&task_id)
      .or_else(|| state.queued.remove(&task_id))
    {
      state.failed.push((path, error));
    }
    state.pending_commits.remove(&task_id);
    wake.notify_all();
  }

  /// Reject new tracked writes while allowing registered writes to finish.
  pub fn stop_new_writes(&self) {
    let (lock, _) = &*self.shared;
    lock
      .lock()
      .unwrap_or_else(|poison| poison.into_inner())
      .accepting_writes = false;
  }

  /// Wait until every queued or active tracked write finishes or fails.
  pub fn wait(&self) {
    let (lock, wake) = &*self.shared;
    let mut state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
    while !state.queued.is_empty() || !state.writing.is_empty() {
      state = wake
        .wait(state)
        .unwrap_or_else(|poison| poison.into_inner());
    }
  }

  /// Return the snapshot for the addressed object.
  pub fn snapshot(&self) -> WriteBarrierSnapshot {
    let (lock, _) = &*self.shared;
    let state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
    let mut snapshot = WriteBarrierSnapshot {
      accepting_writes: state.accepting_writes,
      writing: state.writing.values().cloned().collect(),
      queued: state.queued.values().cloned().collect(),
      pending_commits: state.pending_commits.values().cloned().collect(),
      failed: state.failed.clone(),
    };
    snapshot.writing.sort();
    snapshot.queued.sort();
    snapshot.pending_commits.sort();
    snapshot
  }

  /// Report whether any registered write has not reached a terminal state.
  pub fn has_pending_writes(&self) -> bool {
    let (lock, _) = &*self.shared;
    let state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
    !state.queued.is_empty() || !state.writing.is_empty()
  }
}

impl Default for WriteBarrier {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tracks_queue_write_commit_and_failure() {
    let barrier = WriteBarrier::new();
    assert!(barrier.register(
      TaskId(1),
      PathBuf::from("save.json"),
      Some(PathBuf::from("save.json.tmp"))
    ));
    assert_eq!(barrier.snapshot().queued, vec![PathBuf::from("save.json")]);
    barrier.start(TaskId(1));
    assert_eq!(barrier.snapshot().writing, vec![PathBuf::from("save.json")]);
    barrier.fail(TaskId(1), "disk".to_string());
    assert_eq!(
      barrier.snapshot().failed,
      vec![(PathBuf::from("save.json"), "disk".to_string())]
    );
  }

  #[test]
  fn rejects_new_writes_after_shutdown_begins() {
    let barrier = WriteBarrier::new();
    barrier.stop_new_writes();
    assert!(!barrier.register(TaskId(1), PathBuf::from("x"), None));
  }

  #[test]
  fn wait_does_not_finish_until_an_active_write_finishes() {
    use std::sync::mpsc;
    use std::time::Duration;

    let barrier = WriteBarrier::new();
    assert!(barrier.register(TaskId(1), PathBuf::from("save.json"), None));
    barrier.start(TaskId(1));

    let waiting_barrier = barrier.clone();
    let (started_tx, started_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = mpsc::channel();
    let waiter = std::thread::spawn(move || {
      started_tx.send(()).unwrap();
      waiting_barrier.wait();
      finished_tx.send(()).unwrap();
    });

    started_rx.recv().unwrap();
    assert!(finished_rx.recv_timeout(Duration::from_millis(20)).is_err());
    barrier.finish(TaskId(1));
    finished_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    waiter.join().unwrap();
  }
}
