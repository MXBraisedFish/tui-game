//! Network service: validated HTTP(S) requests executed as async jobs, with SSRF-safe address checks and bounded status history.

mod executor;
mod security;

use std::{
  collections::{BTreeMap, HashMap, VecDeque},
  fmt,
};

use crossbeam_channel::Sender;
use reqwest::{
  Url,
  header::{HeaderName, HeaderValue},
};

use tg_service_async::{AsyncJob, AsyncRuntime, TaskCancellation, TaskId, TaskStatusEvent};

pub(crate) use executor::run_network_task;

const TERMINAL_STATUS_LIMIT: usize = 256;
pub(crate) const MAX_REQUEST_BODY_BYTES: usize = 1024 * 1024;
pub(crate) const MAX_RESPONSE_BODY_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const MAX_URL_BYTES: usize = 8192;
pub(crate) const MAX_REQUEST_HEADERS: usize = 64;
pub(crate) const MAX_REQUEST_HEADER_BYTES: usize = 32 * 1024;
pub(crate) const MAX_REDIRECTS: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkMethod {
  Get,
  Post,
}

impl NetworkMethod {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Get => "get",
      Self::Post => "post",
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkResponseMode {
  Text,
  Bytes,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkRequestBody {
  Empty,
  Text(String),
  Bytes(Vec<u8>),
  Json(serde_json::Value),
}

impl NetworkRequestBody {
  fn is_empty(&self) -> bool {
    matches!(self, Self::Empty)
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkHeader {
  pub name: String,
  pub value: String,
}

impl NetworkHeader {
  pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      value: value.into(),
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkRequest {
  pub method: NetworkMethod,
  pub url: String,
  pub headers: Vec<NetworkHeader>,
  pub body: NetworkRequestBody,
  pub response_mode: NetworkResponseMode,
}

impl NetworkRequest {
  pub fn get(url: impl Into<String>, response_mode: NetworkResponseMode) -> Self {
    Self {
      method: NetworkMethod::Get,
      url: url.into(),
      headers: Vec::new(),
      body: NetworkRequestBody::Empty,
      response_mode,
    }
  }

  pub fn post(
    url: impl Into<String>,
    body: NetworkRequestBody,
    response_mode: NetworkResponseMode,
  ) -> Self {
    Self {
      method: NetworkMethod::Post,
      url: url.into(),
      headers: Vec::new(),
      body,
      response_mode,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkResponseBody {
  Text(String),
  Bytes(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkResponse {
  pub original_url: String,
  pub final_url: String,
  pub status: u16,
  pub headers: BTreeMap<String, String>,
  pub body: NetworkResponseBody,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkErrorCode {
  InvalidRequest,
  PermissionDenied,
  TooLarge,
  InvalidUtf8,
  Cancelled,
  Timeout,
  Network,
  Unsupported,
  Internal,
}

impl NetworkErrorCode {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::InvalidRequest => "invalid_request",
      Self::PermissionDenied => "permission_denied",
      Self::TooLarge => "too_large",
      Self::InvalidUtf8 => "invalid_utf8",
      Self::Cancelled => "cancelled",
      Self::Timeout => "timeout",
      Self::Network => "network",
      Self::Unsupported => "unsupported",
      Self::Internal => "internal",
    }
  }

  fn message(self) -> &'static str {
    match self {
      Self::InvalidRequest => "invalid network request",
      Self::PermissionDenied => "network destination is not permitted",
      Self::TooLarge => "network request or response exceeds its size limit",
      Self::InvalidUtf8 => "network response is not valid UTF-8",
      Self::Cancelled => "network request was cancelled",
      Self::Timeout => "network request timed out",
      Self::Network => "network request failed",
      Self::Unsupported => "network operation is not supported",
      Self::Internal => "internal network operation failed",
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkError {
  pub code: NetworkErrorCode,
  pub message: String,
  stage: &'static str,
}

impl NetworkError {
  pub fn at(code: NetworkErrorCode, stage: &'static str) -> Self {
    Self {
      code,
      message: code.message().to_string(),
      stage,
    }
  }

  pub(crate) fn stage(&self) -> &'static str {
    self.stage
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkSubmitError {
  pub code: NetworkErrorCode,
  pub message: String,
}

impl NetworkSubmitError {
  fn new(code: NetworkErrorCode) -> Self {
    Self {
      code,
      message: code.message().to_string(),
    }
  }
}

impl fmt::Display for NetworkSubmitError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(&self.message)
  }
}

impl std::error::Error for NetworkSubmitError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkEvent {
  Started {
    task_id: TaskId,
    method: NetworkMethod,
    url: String,
  },
  Finished {
    task_id: TaskId,
    method: NetworkMethod,
    response: NetworkResponse,
  },
  Failed {
    task_id: TaskId,
    method: NetworkMethod,
    url: String,
    error: NetworkError,
  },
  Cancelled {
    task_id: TaskId,
    method: NetworkMethod,
    url: String,
  },
}

impl NetworkEvent {
  pub fn task_id(&self) -> TaskId {
    match self {
      Self::Started { task_id, .. }
      | Self::Finished { task_id, .. }
      | Self::Failed { task_id, .. }
      | Self::Cancelled { task_id, .. } => *task_id,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkRequestStatus {
  Queued,
  Running,
  Completed { status: u16 },
  Failed { code: NetworkErrorCode },
  Cancelled,
}

#[derive(Clone)]
pub struct NetworkTask {
  request: NormalizedNetworkRequest,
}

impl fmt::Debug for NetworkTask {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("NetworkTask")
      .field("method", &self.request.method)
      .field("url", &security::redacted_url(&self.request.url))
      .field("response_mode", &self.request.response_mode)
      .finish_non_exhaustive()
  }
}

impl<E: From<NetworkEvent> + Send + 'static> AsyncJob<E> for NetworkTask {
  fn run(
    self: Box<Self>,
    id: TaskId,
    events: &Sender<E>,
    cancellation: &TaskCancellation,
  ) -> Result<(), String> {
    run_network_task(id, *self, events, cancellation)
  }

  fn cancelled_before_start(&self, id: TaskId, events: &Sender<E>) {
    emit_cancelled(id, self, events);
  }

  fn reports_own_cancellation(&self) -> bool {
    true
  }
}

#[derive(Clone)]
struct NormalizedNetworkRequest {
  method: NetworkMethod,
  url: Url,
  headers: Vec<(HeaderName, HeaderValue)>,
  body: Vec<u8>,
  response_mode: NetworkResponseMode,
  address_policy: security::AddressPolicy,
}

pub struct NetworkService {
  active: HashMap<TaskId, NetworkRequestStatus>,
  terminal: HashMap<TaskId, NetworkRequestStatus>,
  terminal_order: VecDeque<TaskId>,
}

impl NetworkService {
  pub fn new() -> Self {
    Self {
      active: HashMap::new(),
      terminal: HashMap::new(),
      terminal_order: VecDeque::new(),
    }
  }

  pub fn submit<E>(
    &mut self,
    async_runtime: &AsyncRuntime<E>,
    request: NetworkRequest,
  ) -> Result<TaskId, NetworkSubmitError>
  where
    E: From<NetworkEvent> + From<TaskStatusEvent> + Send + 'static,
  {
    let request = normalize_request(request, security::AddressPolicy::PublicOnly)?;
    let task_id = async_runtime.submit(NetworkTask { request });
    self.active.insert(task_id, NetworkRequestStatus::Queued);
    Ok(task_id)
  }

  pub fn cancel<E>(&mut self, async_runtime: &AsyncRuntime<E>, task_id: TaskId) -> bool
  where
    E: From<TaskStatusEvent> + Send + 'static,
  {
    if !self.active.contains_key(&task_id) {
      return false;
    }
    async_runtime.cancel_task(task_id);
    true
  }

  pub fn status(&self, task_id: TaskId) -> Option<&NetworkRequestStatus> {
    self
      .active
      .get(&task_id)
      .or_else(|| self.terminal.get(&task_id))
  }

  pub fn active_count(&self) -> usize {
    self.active.len()
  }

  pub fn handle_engine_event(&mut self, event: &NetworkEvent) {
    let task_id = event.task_id();
    if !self.active.contains_key(&task_id) {
      return;
    }
    match event {
      NetworkEvent::Started { .. } => {
        self.active.insert(task_id, NetworkRequestStatus::Running);
      }
      NetworkEvent::Finished { response, .. } => {
        self.finish(
          task_id,
          NetworkRequestStatus::Completed {
            status: response.status,
          },
        );
      }
      NetworkEvent::Failed { error, .. } => {
        self.finish(task_id, NetworkRequestStatus::Failed { code: error.code });
      }
      NetworkEvent::Cancelled { .. } => {
        self.finish(task_id, NetworkRequestStatus::Cancelled);
      }
    }
  }

  fn finish(&mut self, task_id: TaskId, status: NetworkRequestStatus) {
    self.active.remove(&task_id);
    self.terminal.insert(task_id, status);
    self.terminal_order.push_back(task_id);
    while self.terminal_order.len() > TERMINAL_STATUS_LIMIT {
      if let Some(expired) = self.terminal_order.pop_front() {
        self.terminal.remove(&expired);
      }
    }
  }
}

impl Default for NetworkService {
  fn default() -> Self {
    Self::new()
  }
}

pub(crate) fn emit_cancelled<E: From<NetworkEvent>>(
  task_id: TaskId,
  task: &NetworkTask,
  event_tx: &Sender<E>,
) {
  let _ = event_tx.send(
    NetworkEvent::Cancelled {
      task_id,
      method: task.request.method,
      url: task.request.url.to_string(),
    }
    .into(),
  );
}

fn normalize_request(
  request: NetworkRequest,
  address_policy: security::AddressPolicy,
) -> Result<NormalizedNetworkRequest, NetworkSubmitError> {
  if request.url.len() > MAX_URL_BYTES {
    return Err(NetworkSubmitError::new(NetworkErrorCode::TooLarge));
  }
  let mut url = Url::parse(&request.url)
    .map_err(|_| NetworkSubmitError::new(NetworkErrorCode::InvalidRequest))?;
  if !matches!(url.scheme(), "http" | "https") {
    return Err(NetworkSubmitError::new(NetworkErrorCode::Unsupported));
  }
  if !url.username().is_empty() || url.password().is_some() || url.host_str().is_none() {
    return Err(NetworkSubmitError::new(NetworkErrorCode::InvalidRequest));
  }
  url.set_fragment(None);
  if request.method == NetworkMethod::Get && !request.body.is_empty() {
    return Err(NetworkSubmitError::new(NetworkErrorCode::InvalidRequest));
  }

  let default_content_type = match &request.body {
    NetworkRequestBody::Text(_) => Some("text/plain; charset=utf-8"),
    NetworkRequestBody::Bytes(_) => Some("application/octet-stream"),
    NetworkRequestBody::Json(_) => Some("application/json"),
    NetworkRequestBody::Empty => None,
  };
  let mut body = match request.body {
    NetworkRequestBody::Empty => Vec::new(),
    NetworkRequestBody::Text(text) => text.into_bytes(),
    NetworkRequestBody::Bytes(bytes) => bytes,
    NetworkRequestBody::Json(value) => serde_json::to_vec(&value)
      .map_err(|_| NetworkSubmitError::new(NetworkErrorCode::InvalidRequest))?,
  };
  if body.len() > MAX_REQUEST_BODY_BYTES {
    body.clear();
    return Err(NetworkSubmitError::new(NetworkErrorCode::TooLarge));
  }

  if request.headers.len() > MAX_REQUEST_HEADERS {
    return Err(NetworkSubmitError::new(NetworkErrorCode::TooLarge));
  }
  let mut headers = Vec::with_capacity(request.headers.len() + 1);
  let mut total_header_bytes = 0usize;
  let mut has_content_type = false;
  for header in request.headers {
    total_header_bytes = total_header_bytes
      .saturating_add(header.name.len())
      .saturating_add(header.value.len());
    if total_header_bytes > MAX_REQUEST_HEADER_BYTES {
      return Err(NetworkSubmitError::new(NetworkErrorCode::TooLarge));
    }
    let name = HeaderName::from_bytes(header.name.as_bytes())
      .map_err(|_| NetworkSubmitError::new(NetworkErrorCode::InvalidRequest))?;
    if security::is_forbidden_request_header(name.as_str()) {
      return Err(NetworkSubmitError::new(NetworkErrorCode::PermissionDenied));
    }
    let value = HeaderValue::from_str(&header.value)
      .map_err(|_| NetworkSubmitError::new(NetworkErrorCode::InvalidRequest))?;
    has_content_type |= name == reqwest::header::CONTENT_TYPE;
    headers.push((name, value));
  }

  if request.method == NetworkMethod::Post
    && !has_content_type
    && let Some(value) = default_content_type
  {
    headers.push((
      reqwest::header::CONTENT_TYPE,
      HeaderValue::from_static(value),
    ));
  }

  Ok(NormalizedNetworkRequest {
    method: request.method,
    url,
    headers,
    body,
    response_mode: request.response_mode,
    address_policy,
  })
}

#[cfg(test)]
fn normalize_request_for_test(
  request: NetworkRequest,
) -> Result<NormalizedNetworkRequest, NetworkSubmitError> {
  normalize_request(request, security::AddressPolicy::AllowLoopbackForTests)
}

#[cfg(test)]
mod tests {
  use std::{thread, time::Duration};

  use super::*;
  use tg_service_async::TaskState;

  #[derive(Debug)]
  enum TestEvent {
    Network(NetworkEvent),
    Status,
  }

  impl From<NetworkEvent> for TestEvent {
    fn from(event: NetworkEvent) -> Self {
      Self::Network(event)
    }
  }

  impl From<TaskStatusEvent> for TestEvent {
    fn from(_: TaskStatusEvent) -> Self {
      Self::Status
    }
  }

  /// Keeps the only worker busy so the next task stays queued.
  struct OccupyWorker(Duration);

  impl AsyncJob<TestEvent> for OccupyWorker {
    fn run(
      self: Box<Self>,
      _id: TaskId,
      _events: &Sender<TestEvent>,
      _cancellation: &TaskCancellation,
    ) -> Result<(), String> {
      thread::sleep(self.0);
      Ok(())
    }
  }

  #[test]
  fn request_validation_rejects_unsupported_or_credentialed_urls() {
    for url in [
      "file:///secret",
      "ftp://example.com/file",
      "https://user:password@example.com/",
    ] {
      assert!(
        NetworkService::new()
          .submit(
            &AsyncRuntime::<TestEvent>::with_worker_count(1),
            NetworkRequest::get(url, NetworkResponseMode::Text),
          )
          .is_err()
      );
    }
  }

  #[test]
  fn get_rejects_body_and_forbidden_headers() {
    let request = NetworkRequest {
      method: NetworkMethod::Get,
      url: "https://example.com".to_string(),
      headers: Vec::new(),
      body: NetworkRequestBody::Text("no".to_string()),
      response_mode: NetworkResponseMode::Text,
    };
    assert!(normalize_request(request, security::AddressPolicy::PublicOnly).is_err());

    let mut request = NetworkRequest::get("https://example.com", NetworkResponseMode::Text);
    request
      .headers
      .push(NetworkHeader::new("Host", "private.example"));
    assert!(normalize_request(request, security::AddressPolicy::PublicOnly).is_err());
  }

  #[test]
  fn body_variants_receive_their_default_content_types() {
    let cases = [
      (
        NetworkRequestBody::Text("text".to_string()),
        "text/plain; charset=utf-8",
      ),
      (
        NetworkRequestBody::Bytes(vec![1, 2, 3]),
        "application/octet-stream",
      ),
      (
        NetworkRequestBody::Json(serde_json::json!({"ok": true})),
        "application/json",
      ),
    ];
    for (body, expected) in cases {
      let request = normalize_request_for_test(NetworkRequest::post(
        "http://127.0.0.1/",
        body,
        NetworkResponseMode::Bytes,
      ))
      .unwrap();
      let content_type = request
        .headers
        .iter()
        .find(|(name, _)| *name == reqwest::header::CONTENT_TYPE)
        .unwrap()
        .1
        .to_str()
        .unwrap();
      assert_eq!(content_type, expected);
    }

    let mut request = NetworkRequest::post(
      "http://127.0.0.1/",
      NetworkRequestBody::Json(serde_json::json!({})),
      NetworkResponseMode::Bytes,
    );
    request.headers.push(NetworkHeader::new(
      "Content-Type",
      "application/problem+json",
    ));
    let request = normalize_request_for_test(request).unwrap();
    assert_eq!(
      request
        .headers
        .iter()
        .find(|(name, _)| *name == reqwest::header::CONTENT_TYPE)
        .unwrap()
        .1
        .to_str()
        .unwrap(),
      "application/problem+json"
    );
  }

  #[test]
  fn normalization_removes_fragments_and_enforces_request_limits() {
    let request = normalize_request_for_test(NetworkRequest::get(
      "https://example.com/path?visible=yes#ignored",
      NetworkResponseMode::Text,
    ))
    .unwrap();
    assert_eq!(request.url.as_str(), "https://example.com/path?visible=yes");

    let oversized = NetworkRequest::post(
      "https://example.com/",
      NetworkRequestBody::Bytes(vec![0; MAX_REQUEST_BODY_BYTES + 1]),
      NetworkResponseMode::Bytes,
    );
    assert!(matches!(
      normalize_request_for_test(oversized),
      Err(NetworkSubmitError {
        code: NetworkErrorCode::TooLarge,
        ..
      })
    ));
  }

  #[test]
  fn cancelling_a_queued_network_task_emits_cancelled_and_clears_state() {
    let runtime = AsyncRuntime::<TestEvent>::with_worker_count(1);
    runtime.submit(OccupyWorker(Duration::from_millis(50)));
    let request = normalize_request_for_test(NetworkRequest::get(
      "http://127.0.0.1/",
      NetworkResponseMode::Text,
    ))
    .unwrap();
    let task_id = runtime.submit(NetworkTask { request });
    runtime.cancel_task(task_id);

    let mut cancelled = false;
    for _ in 0..50 {
      for event in runtime.poll_events() {
        if matches!(
          event,
          TestEvent::Network(NetworkEvent::Cancelled {
            task_id: actual,
            ..
          }) if actual == task_id
        ) {
          cancelled = true;
        }
      }
      if cancelled {
        break;
      }
      thread::sleep(Duration::from_millis(10));
    }
    assert!(cancelled);
    assert_eq!(runtime.task_state(task_id), Some(TaskState::Cancelled));
  }

  #[test]
  fn status_history_is_bounded_and_active_tasks_are_not_evicted() {
    let mut service = NetworkService::new();
    for raw in 0..=TERMINAL_STATUS_LIMIT {
      let task_id = TaskId(raw as u64);
      service.active.insert(task_id, NetworkRequestStatus::Queued);
      service.handle_engine_event(&NetworkEvent::Cancelled {
        task_id,
        method: NetworkMethod::Get,
        url: "https://example.com/".to_string(),
      });
    }
    assert_eq!(service.terminal.len(), TERMINAL_STATUS_LIMIT);
    assert!(service.status(TaskId(0)).is_none());
    assert_eq!(
      service.status(TaskId(TERMINAL_STATUS_LIMIT as u64)),
      Some(&NetworkRequestStatus::Cancelled)
    );
  }
}
