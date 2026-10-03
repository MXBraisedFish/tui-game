//! Background task execution, cancellation, completion events, and shutdown write tracking.
//!
//! # Examples
//!
//! ```rust
//! use std::time::{Duration, Instant};
//!
//! use crossbeam_channel::Sender;
//! use tg_service_async::{
//!   AsyncJob, AsyncRuntime, TaskCancellation, TaskId, TaskState, TaskStatusEvent,
//! };
//!
//! #[derive(Debug, PartialEq)]
//! enum Event {
//!   Answer(u32),
//!   Status(TaskStatusEvent),
//! }
//!
//! impl From<TaskStatusEvent> for Event {
//!   fn from(event: TaskStatusEvent) -> Self {
//!     Self::Status(event)
//!   }
//! }
//!
//! struct Answer;
//!
//! impl AsyncJob<Event> for Answer {
//!   fn run(
//!     self: Box<Self>,
//!     _id: TaskId,
//!     events: &Sender<Event>,
//!     _cancellation: &TaskCancellation,
//!   ) -> Result<(), String> {
//!     let _ = events.send(Event::Answer(42));
//!     Ok(())
//!   }
//! }
//!
//! fn main() {
//!   let runtime = AsyncRuntime::<Event>::with_worker_count(1);
//!   let id = runtime.submit(Answer);
//!   let deadline = Instant::now() + Duration::from_secs(5);
//!   let mut events = Vec::new();
//!   while events.len() < 2 && Instant::now() < deadline {
//!     events.extend(runtime.poll_events());
//!     std::thread::sleep(Duration::from_millis(5));
//!   }
//!   assert_eq!(
//!     events,
//!     [
//!       Event::Answer(42),
//!       Event::Status(TaskStatusEvent::Finished { id })
//!     ]
//!   );
//!   assert_eq!(runtime.task_state(id), Some(TaskState::Finished));
//!   println!("async ok: {events:?}");
//! }
//! ```

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

mod event_sink;
mod write_barrier;

pub use event_sink::EventSink;
pub use write_barrier::{WriteBarrier, WriteBarrierSnapshot};

/// The identity of task within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskId(
  /// The wrapped u64 value.
  pub u64,
);

/// The identity of managed thread within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ManagedThreadId(
  /// The wrapped u64 value.
  pub u64,
);

/// A clonable cancellation signal checked by background work.
#[derive(Clone)]
pub struct TaskCancellation {
  task_id: TaskId,
  cancelled: Arc<Mutex<HashSet<TaskId>>>,
}

impl TaskCancellation {
  /// Create a task cancellation initialized from `task_id`.
  #[cfg(any(test, feature = "test-support"))]
  pub fn new(task_id: TaskId) -> Self {
    Self {
      task_id,
      cancelled: Arc::new(Mutex::new(HashSet::new())),
    }
  }

  /// Report whether cancellation has been requested for this operation.
  pub fn is_cancelled(&self) -> bool {
    is_cancelled(&self.cancelled, self.task_id)
  }

  /// Cancel the task cancellation state addressed by this operation.
  #[cfg(any(test, feature = "test-support"))]
  pub fn cancel(&self) {
    self
      .cancelled
      .lock()
      .unwrap_or_else(|poison| poison.into_inner())
      .insert(self.task_id);
  }
}

/// The queued, running, completed, failed, or cancelled state of an asynchronous task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
  /// The operation is pending.
  Pending,
  /// The operation is running.
  Running,
  /// The operation is finished.
  Finished,
  /// The operation is failed.
  Failed,
  /// The operation is cancelled.
  Cancelled,
}

/// A task status event payload queued for its owning consumer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskStatusEvent {
  /// The operation is finished.
  Finished {
    /// The identifier of the owned object.
    id: TaskId,
  },
  /// A failed notification delivered to the owning consumer.
  Failed {
    /// The identifier of the owned object.
    id: TaskId,
    /// The error.
    error: String,
  },
}

/// A background operation receiving task identity, cancellation, and completion delivery.
pub trait AsyncJob<E>: Send + 'static {
  /// Execute this job with its task identity, cancellation signal, and completion sender.
  ///
  /// # Arguments
  ///
  /// * `self` - The self.
  /// * `id` - The identifier of the owned object.
  /// * `events` - The events in delivery order.
  /// * `cancellation` - The cancellation token for the operation.
  ///
  /// # Errors
  ///
  /// Return a job-specific diagnostic when execution fails. The implementation must check
  /// cancellation and report its service-specific completion events.
  fn run(
    self: Box<Self>,
    id: TaskId,
    events: &Sender<E>,
    cancellation: &TaskCancellation,
  ) -> Result<(), String>;

  /// Declare the final and temporary paths tracked by shutdown, or `None` for a job without a
  /// write.
  fn write_target(&self, _id: TaskId) -> Option<(PathBuf, PathBuf)> {
    None
  }

  /// Report service-specific cancellation for a queued job that never began execution.
  fn cancelled_before_start(&self, _id: TaskId, _events: &Sender<E>) {}

  /// Report whether this job emits its own service-specific cancellation event.
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

/// The async runtime representation used by this module.
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
  /// Create the asynchronous executor with four worker threads.
  ///
  /// # Panics
  ///
  /// Panic if the operating system cannot create a worker thread.
  pub fn new() -> Self {
    Self::with_worker_count(4)
  }

  /// Create the asynchronous executor with the requested worker count.
  ///
  /// A zero count still starts one worker.
  ///
  /// # Panics
  ///
  /// Panic if the operating system cannot create a worker thread.
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

  /// Queue the background operation and return its task identifier for state queries and
  /// completion routing.
  pub fn submit(&self, task: impl AsyncJob<E>) -> TaskId {
    let id = TaskId(self.next_task_id.fetch_add(1, Ordering::SeqCst));
    if let Some((target, temporary)) = task.write_target(id)
      && !self.write_barrier.register(id, target, Some(temporary))
    {
      set_task_state(&self.task_states, id, TaskState::Cancelled);
      return id;
    }
    set_task_state(&self.task_states, id, TaskState::Pending);
    if self
      .task_tx
      .send(WorkerMessage::Run(id, Box::new(task)))
      .is_err()
    {
      let error = "asynchronous worker queue is closed".to_string();
      set_task_state(&self.task_states, id, TaskState::Failed);
      self.write_barrier.fail(id, error.clone());
      let _ = self
        .event_tx
        .send(TaskStatusEvent::Failed { id, error }.into());
    }
    id
  }

  /// Return the shared tracker used to await outstanding writes during shutdown.
  pub fn write_barrier(&self) -> WriteBarrier {
    self.write_barrier.clone()
  }

  /// Return the last recorded lifecycle state of the asynchronous task.
  pub fn task_state(&self, id: TaskId) -> Option<TaskState> {
    let states = self.task_states.lock().unwrap_or_else(|poison| {
      // Recover the queue after a task panic so unrelated tasks can still be received.

      poison.into_inner()
    });
    states.get(&id).copied()
  }

  /// Request cancellation of the identified asynchronous task.
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

  /// Request cancellation for each supplied asynchronous task identifier.
  pub fn cancel_tasks(&self, ids: impl IntoIterator<Item = TaskId>) {
    for id in ids {
      self.cancel_task(id);
    }
  }

  /// Drain the completion events currently available from background workers.
  pub fn poll_events(&self) -> Vec<E> {
    self.event_rx.try_iter().collect()
  }

  /// Return a sender connected to the executor's completion-event queue.
  pub fn event_sender(&self) -> Sender<E> {
    self.event_tx.clone()
  }

  /// Start a listener thread whose stop signal and join handle are owned by the executor.
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
  /// Signal one managed listener to stop and join its thread.
  pub fn stop_managed_thread(&mut self, id: ManagedThreadId) -> bool {
    let Some(mut thread) = self.managed_threads.remove(&id) else {
      return false;
    };

    thread.stop.store(true, Ordering::SeqCst);
    if thread.joinable
      && let Some(handle) = thread.handle.take()
    {
      let _ = handle.join();
    }
    true
  }
  /// Signal and join every managed listener thread.
  pub fn stop_all_managed_threads(&mut self) {
    let ids = self.managed_threads.keys().copied().collect::<Vec<_>>();
    for id in ids {
      let _ = self.stop_managed_thread(id);
    }
  }

  /// Stop async work and release its owned runtime resources.
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
    // A poisoned queue must not prevent shutdown from collecting its remaining work.

    poison.into_inner()
  });
  states.insert(id, state);
}
