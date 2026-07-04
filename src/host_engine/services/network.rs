use super::async_runtime::{AsyncRuntime, EngineTask, NetworkTask, TaskId};

pub struct NetworkService;

impl NetworkService {
  pub fn new() -> Self {
    Self
  }

  pub fn get(&self, async_runtime: &AsyncRuntime, url: String) -> TaskId {
    async_runtime.submit(EngineTask::Network(NetworkTask::Get { url }))
  }
}

impl Default for NetworkService {
  fn default() -> Self {
    Self::new()
  }
}
