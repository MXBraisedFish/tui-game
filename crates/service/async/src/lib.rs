//! Async task executor: worker threads, task ids/states, cooperative cancellation, the write
//! barrier for asynchronous file writes, and managed listener threads.
//!
//! The executor is generic over the application's event type `E`; tasks implement [`AsyncJob`]
//! and report through a `Sender<E>`. Completion/failure of a task is reported as
//! [`TaskStatusEvent`], so `E` must implement `From<TaskStatusEvent>`.

use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
  },
  thread::{self, JoinHandle},
};

use crossbeam_channel::{Receiver, Sender, unbounded};

mod write_barrier;

pub use write_barrier::{WriteBarrier, WriteBarrierSnapshot};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ManagedThreadId(pub u64);

#[derive(Clone)]
pub struct TaskCancellation {
  task_id: TaskId,
  cancelled: Arc<Mutex<HashSet<TaskId>>>,
}

impl TaskCancellation {
  #[cfg(any(test, feature = "test-support"))]
  pub fn new(task_id: TaskId) -> Self {
    Self {
      task_id,
      cancelled: Arc::new(Mutex::new(HashSet::new())),
    }
  }

  pub fn is_cancelled(&self) -> bool {
    is_cancelled(&self.cancelled, self.task_id)
  }

  #[cfg(any(test, feature = "test-support"))]
  pub fn cancel(&self) {
    self
      .cancelled
      .lock()
      .unwrap_or_else(|poison| poison.into_inner())
      .insert(self.task_id);
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
  Pending,
  Running,
  Finished,
  Failed,
  Cancelled,
}

/// Completion or failure of a task, reported by the executor itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskStatusEvent {
  Finished { id: TaskId },
  Failed { id: TaskId, error: String },
}

/// A unit of work the executor can run on a worker thread.
pub trait AsyncJob<E>: Send + 'static {
  /// Runs the job; `Err` marks the task failed (unless it was cancelled meanwhile).
  fn run(self: Box<Self>, id: TaskId, events: &Sender<E>, cancellation: &TaskCancellation)
  -> Result<(), String>;

  /// File this job writes, registered with the write barrier before the job is queued.
  /// Returns `(target, temporary)`.
  fn write_target(&self, _id: TaskId) -> Option<(PathBuf, PathBuf)> {
    None
  }

  /// Called instead of `run` when the task was cancelled before it started.
  fn cancelled_before_start(&self, _id: TaskId, _events: &Sender<E>) {}

  /// Whether the job reports its own cancellation (then a cancelled `Ok` still counts as finished).
  fn reports_own_cancellation(&self) -> bool {
    false
  }
}

enum WorkerMessage<E> {
  Run(TaskId, Box<dyn AsyncJob<E>>),
  Shutdown,
}

struct ManagedThread {
  stop: Arc<AtomicBool>,
  joinable: bool,
  handle: Option<JoinHandle<()>>,
}

pub struct AsyncRuntime<E> {
  task_tx: Sender<WorkerMessage<E>>,
  event_tx: Sender<E>,
  event_rx: Receiver<E>,
  workers: Vec<JoinHandle<()>>,
  task_states: Arc<Mutex<HashMap<TaskId, TaskState>>>,
  cancelled_tasks: Arc<Mutex<HashSet<TaskId>>>,
  write_barrier: WriteBarrier,
  managed_threads: HashMap<ManagedThreadId, ManagedThread>,
  next_task_id: AtomicU64,
  next_thread_id: u64,
}

impl<E: From<TaskStatusEvent> + Send + 'static> AsyncRuntime<E> {
  pub fn new() -> Self {
    Self::with_worker_count(4)
  }

  pub fn with_worker_count(worker_count: usize) -> Self {
    let (task_tx, task_rx) = unbounded();
    let (event_tx, event_rx) = unbounded();
    let task_states = Arc::new(Mutex::new(HashMap::new()));
    let cancelled_tasks = Arc::new(Mutex::new(HashSet::new()));
    let write_barrier = WriteBarrier::new();
    let mut workers = Vec::new();

    for _ in 0..worker_count.max(1) {
      let task_rx = task_rx.clone();
      let event_tx = event_tx.clone();
      let task_states = task_states.clone();
      let cancelled_tasks = cancelled_tasks.clone();
      let write_barrier = write_barrier.clone();
      // Note: thread::spawn failure (e.g. OOM) is a process-level abort in std;
      // no recoverable error to log here.
      workers.push(thread::spawn(move || {
        worker_loop(
          task_rx,
          event_tx,
          task_states,
          cancelled_tasks,
          write_barrier,
        );
      }));
    }

    Self {
      task_tx,
      event_tx,
      event_rx,
      workers,
      task_states,
      cancelled_tasks,
      write_barrier,
      managed_threads: HashMap::new(),
      next_task_id: AtomicU64::new(1),
      next_thread_id: 1,
    }
  }

  pub fn submit(&self, task: impl AsyncJob<E>) -> TaskId {
    let id = TaskId(self.next_task_id.fetch_add(1, Ordering::SeqCst));
    if let Some((target, temporary)) = task.write_target(id) {
      if !self.write_barrier.register(id, target, Some(temporary)) {
        set_task_state(&self.task_states, id, TaskState::Cancelled);
        return id;
      }
    }
    set_task_state(&self.task_states, id, TaskState::Pending);
    if self.task_tx.send(WorkerMessage::Run(id, Box::new(task))).is_err() {
      let error = "asynchronous worker queue is closed".to_string();
      set_task_state(&self.task_states, id, TaskState::Failed);
      self.write_barrier.fail(id, error.clone());
      let _ = self.event_tx.send(TaskStatusEvent::Failed { id, error }.into());
    }
    id
  }

  pub fn write_barrier(&self) -> WriteBarrier {
    self.write_barrier.clone()
  }

  pub fn task_state(&self, id: TaskId) -> Option<TaskState> {
    let states = self.task_states.lock().unwrap_or_else(|poison| {
      // Mutex poisoned — a previous task panicked. Recover the guard.
      poison.into_inner()
    });
    states.get(&id).copied()
  }

  /// 请求取消任务。尚未开始的任务不会执行；运行中的任务会在任务边界停止提交结果。
  pub fn cancel_task(&self, id: TaskId) {
    self
      .cancelled_tasks
      .lock()
      .unwrap_or_else(|poison| poison.into_inner())
      .insert(id);
    if self.task_state(id) == Some(TaskState::Pending) {
      set_task_state(&self.task_states, id, TaskState::Cancelled);
    }
  }

  pub fn cancel_tasks(&self, ids: impl IntoIterator<Item = TaskId>) {
    for id in ids {
      self.cancel_task(id);
    }
  }

  pub fn poll_events(&self) -> Vec<E> {
    self.event_rx.try_iter().collect()
  }

  pub fn event_sender(&self) -> Sender<E> {
    self.event_tx.clone()
  }

  pub fn spawn_managed_listener<F>(&mut self, joinable: bool, start: F) -> ManagedThreadId
  where
    F: FnOnce(Sender<E>, Arc<AtomicBool>) -> JoinHandle<()> + Send + 'static,
  {
    let id = ManagedThreadId(self.next_thread_id);
    self.next_thread_id += 1;

    let stop = Arc::new(AtomicBool::new(false));
    let handle = start(self.event_tx.clone(), stop.clone());
    let handle = joinable.then_some(handle);

    self.managed_threads.insert(
      id,
      ManagedThread {
        stop,
        joinable,
        handle,
      },
    );

    id
  }

}

impl<E> AsyncRuntime<E> {
  pub fn stop_managed_thread(&mut self, id: ManagedThreadId) -> bool {
    let Some(mut thread) = self.managed_threads.remove(&id) else {
      return false;
    };

    thread.stop.store(true, Ordering::SeqCst);
    if thread.joinable {
      if let Some(handle) = thread.handle.take() {
        let _ = handle.join();
      }
    }
    true
  }
  pub fn stop_all_managed_threads(&mut self) {
    let ids = self.managed_threads.keys().copied().collect::<Vec<_>>();
    for id in ids {
      let _ = self.stop_managed_thread(id);
    }
  }
  /// 停止任务执行器并等待所有工作线程结束。
  ///
  /// Shutdown 在销毁 Lua 与其它宿主服务前显式调用，Drop 仅作为异常路径兜底。
  pub fn shutdown(&mut self) {
    self.stop_all_managed_threads();
    for _ in &self.workers {
      if self.task_tx.send(WorkerMessage::Shutdown).is_err() {
        break;
      }
    }
    while let Some(worker) = self.workers.pop() {
      let _ = worker.join();
    }
  }
}

impl<E: From<TaskStatusEvent> + Send + 'static> Default for AsyncRuntime<E> {
  fn default() -> Self {
    Self::new()
  }
}

impl<E> Drop for AsyncRuntime<E> {
  fn drop(&mut self) {
    self.shutdown();
  }
}

fn worker_loop<E: From<TaskStatusEvent> + 'static>(
  task_rx: Receiver<WorkerMessage<E>>,
  event_tx: Sender<E>,
  task_states: Arc<Mutex<HashMap<TaskId, TaskState>>>,
  cancelled_tasks: Arc<Mutex<HashSet<TaskId>>>,
  write_barrier: WriteBarrier,
) {
  while let Ok(message) = task_rx.recv() {
    match message {
      WorkerMessage::Run(id, task) => {
        if is_cancelled(&cancelled_tasks, id) {
          task.cancelled_before_start(id, &event_tx);
          set_task_state(&task_states, id, TaskState::Cancelled);
          clear_cancelled(&cancelled_tasks, id);
          write_barrier.finish(id);
          continue;
        }
        set_task_state(&task_states, id, TaskState::Running);
        write_barrier.start(id);
        let cancellation = TaskCancellation {
          task_id: id,
          cancelled: cancelled_tasks.clone(),
        };
        let reports_own_cancellation = task.reports_own_cancellation();
        let result = task.run(id, &event_tx, &cancellation);
        let was_cancelled = is_cancelled(&cancelled_tasks, id);
        clear_cancelled(&cancelled_tasks, id);
        match result {
          Ok(()) => {
            if !reports_own_cancellation && was_cancelled {
              set_task_state(&task_states, id, TaskState::Cancelled);
            } else {
              set_task_state(&task_states, id, TaskState::Finished);
              let _ = event_tx.send(TaskStatusEvent::Finished { id }.into());
            }
            write_barrier.finish(id);
          }
          Err(error) => {
            if was_cancelled {
              set_task_state(&task_states, id, TaskState::Cancelled);
              write_barrier.finish(id);
            } else {
              set_task_state(&task_states, id, TaskState::Failed);
              write_barrier.fail(id, error.clone());
              let _ = event_tx.send(TaskStatusEvent::Failed { id, error }.into());
            }
          }
        }
      }
      WorkerMessage::Shutdown => break,
    }
  }
}

fn is_cancelled(cancelled_tasks: &Arc<Mutex<HashSet<TaskId>>>, id: TaskId) -> bool {
  cancelled_tasks
    .lock()
    .unwrap_or_else(|poison| poison.into_inner())
    .contains(&id)
}

fn clear_cancelled(cancelled_tasks: &Arc<Mutex<HashSet<TaskId>>>, id: TaskId) {
  cancelled_tasks
    .lock()
    .unwrap_or_else(|poison| poison.into_inner())
    .remove(&id);
}

fn set_task_state(
  task_states: &Arc<Mutex<HashMap<TaskId, TaskState>>>,
  id: TaskId,
  state: TaskState,
) {
  let mut states = task_states.lock().unwrap_or_else(|poison| {
    // Mutex poisoned — a previous task panicked. Recover the guard.
    poison.into_inner()
  });
  states.insert(id, state);
}
