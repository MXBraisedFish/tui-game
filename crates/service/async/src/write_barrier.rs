use std::{
  collections::HashMap,
  path::PathBuf,
  sync::{Arc, Condvar, Mutex},
};

use crate::TaskId;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WriteBarrierSnapshot {
  pub accepting_writes: bool,
  pub writing: Vec<PathBuf>,
  pub queued: Vec<PathBuf>,
  pub pending_commits: Vec<PathBuf>,
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

/// 统一追踪异步文件写入，供 Shutdown 建立停止提交与等待完成屏障。
#[derive(Clone)]
pub struct WriteBarrier {
  shared: Arc<(Mutex<WriteBarrierState>, Condvar)>,
}

impl WriteBarrier {
  pub fn new() -> Self {
    let state = WriteBarrierState {
      accepting_writes: true,
      ..Default::default()
    };
    Self {
      shared: Arc::new((Mutex::new(state), Condvar::new())),
    }
  }

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

  pub fn start(&self, task_id: TaskId) {
    let (lock, _) = &*self.shared;
    let mut state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
    if let Some(path) = state.queued.remove(&task_id) {
      state.writing.insert(task_id, path);
    }
  }

  pub fn finish(&self, task_id: TaskId) {
    let (lock, wake) = &*self.shared;
    let mut state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
    state.queued.remove(&task_id);
    state.writing.remove(&task_id);
    state.pending_commits.remove(&task_id);
    wake.notify_all();
  }

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

  pub fn stop_new_writes(&self) {
    let (lock, _) = &*self.shared;
    lock
      .lock()
      .unwrap_or_else(|poison| poison.into_inner())
      .accepting_writes = false;
  }

  pub fn wait(&self) {
    let (lock, wake) = &*self.shared;
    let mut state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
    while !state.queued.is_empty() || !state.writing.is_empty() {
      state = wake
        .wait(state)
        .unwrap_or_else(|poison| poison.into_inner());
    }
  }

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
}
