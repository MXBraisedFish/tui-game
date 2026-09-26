use std::{
  collections::BTreeMap,
  io::Read,
  thread,
  time::{Duration, Instant},
};

use crossbeam_channel::{RecvTimeoutError, Sender, bounded};
use reqwest::{
  Method, StatusCode, Url,
  blocking::{Client, Response},
  header::{AUTHORIZATION, CONTENT_TYPE, COOKIE, HeaderMap, HeaderValue, LOCATION},
  redirect::Policy,
};

use tg_service_async::{TaskCancellation, TaskId};

use super::{
  MAX_REDIRECTS, MAX_RESPONSE_BODY_BYTES, NetworkError, NetworkErrorCode, NetworkEvent,
  NetworkMethod, NetworkResponse, NetworkResponseBody, NetworkResponseMode, NetworkTask,
  security::{self, ResolvedDestination},
};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const TOTAL_TIMEOUT: Duration = Duration::from_secs(15);
const READ_BUFFER_BYTES: usize = 64 * 1024;

pub(crate) fn run_network_task<E: From<NetworkEvent>>(
  task_id: TaskId,
  task: NetworkTask,
  event_tx: &Sender<E>,
  cancellation: &TaskCancellation,
) -> Result<(), String> {
  let method = task.request.method;
  let original_url = task.request.url.to_string();
  if cancellation.is_cancelled() {
    super::emit_cancelled(task_id, &task, event_tx);
    return Err("network request cancelled".to_string());
  }
  let _ = event_tx.send(
    NetworkEvent::Started {
      task_id,
      method,
      url: original_url.clone(),
    }
    .into(),
  );

  match execute(&task, cancellation) {
    Ok(response) => {
      let _ = event_tx.send(
        NetworkEvent::Finished {
          task_id,
          method,
          response,
        }
        .into(),
      );
      Ok(())
    }
    Err(error) if error.code == NetworkErrorCode::Cancelled => {
      super::emit_cancelled(task_id, &task, event_tx);
      Err("network request cancelled".to_string())
    }
    Err(error) => {
      let code = error.code;
      let stage = error.stage();
      let _ = event_tx.send(
        NetworkEvent::Failed {
          task_id,
          method,
          url: original_url,
          error,
        }
        .into(),
      );
      Err(format!(
        "network request failed during {stage} with {}",
        code.as_str()
      ))
    }
  }
}

fn execute(
  task: &NetworkTask,
  cancellation: &TaskCancellation,
) -> Result<NetworkResponse, NetworkError> {
  let started = Instant::now();
  let original_url = task.request.url.to_string();
  let mut current_url = task.request.url.clone();
  let mut method = task.request.method;
  let mut body = task.request.body.clone();
  let mut headers = header_map(&task.request.headers);

  for redirect_count in 0..=MAX_REDIRECTS {
    check_cancelled(cancellation)?;
    let remaining = TOTAL_TIMEOUT
      .checked_sub(started.elapsed())
      .ok_or_else(|| NetworkError::at(NetworkErrorCode::Timeout, "request_total"))?;
    if remaining.is_zero() {
      return Err(NetworkError::at(NetworkErrorCode::Timeout, "request_total"));
    }

    let destination = resolve_destination(
      &current_url,
      task.request.address_policy,
      started,
      cancellation,
    )?;
    let client = client_for_destination(&destination, remaining)?;
    let mut request = client
      .request(reqwest_method(method), current_url.clone())
      .headers(headers.clone())
      .timeout(remaining);
    if method == NetworkMethod::Post {
      request = request.body(body.clone());
    }
    let response = request
      .send()
      .map_err(|error| map_reqwest_error(error, "send"))?;
    check_cancelled(cancellation)?;

    if let Some(next) = redirect_target(&current_url, &response)? {
      if redirect_count == MAX_REDIRECTS {
        return Err(NetworkError::at(
          NetworkErrorCode::InvalidRequest,
          "redirect",
        ));
      }
      if !same_origin(&current_url, &next) {
        headers.remove(AUTHORIZATION);
        headers.remove(COOKIE);
      }
      if matches!(
        response.status(),
        StatusCode::MOVED_PERMANENTLY | StatusCode::FOUND | StatusCode::SEE_OTHER
      ) && method == NetworkMethod::Post
      {
        method = NetworkMethod::Get;
        body.clear();
        headers.remove(CONTENT_TYPE);
      }
      current_url = next;
      continue;
    }

    return finish_response(
      original_url,
      current_url,
      response,
      task.request.response_mode,
      cancellation,
    );
  }
  Err(NetworkError::at(NetworkErrorCode::Internal, "redirect"))
}

fn client_for_destination(
  destination: &ResolvedDestination,
  remaining: Duration,
) -> Result<Client, NetworkError> {
  Client::builder()
    .redirect(Policy::none())
    .no_proxy()
    .connect_timeout(CONNECT_TIMEOUT.min(remaining))
    .timeout(remaining)
    .gzip(true)
    .brotli(true)
    .deflate(true)
    .zstd(true)
    .resolve_to_addrs(&destination.host, &destination.addresses)
    .build()
    .map_err(|error| map_reqwest_error(error, "client"))
}

fn header_map(headers: &[(reqwest::header::HeaderName, HeaderValue)]) -> HeaderMap {
  let mut result = HeaderMap::new();
  for (name, value) in headers {
    result.append(name.clone(), value.clone());
  }
  result
}

fn resolve_destination(
  url: &Url,
  policy: security::AddressPolicy,
  started: Instant,
  cancellation: &TaskCancellation,
) -> Result<ResolvedDestination, NetworkError> {
  let url = url.clone();
  let (result_tx, result_rx) = bounded(1);
  thread::spawn(move || {
    let _ = result_tx.send(security::resolve_destination(&url, policy));
  });

  loop {
    check_cancelled(cancellation)?;
    let remaining = TOTAL_TIMEOUT
      .checked_sub(started.elapsed())
      .ok_or_else(|| NetworkError::at(NetworkErrorCode::Timeout, "dns"))?;
    if remaining.is_zero() {
      return Err(NetworkError::at(NetworkErrorCode::Timeout, "dns"));
    }
    match result_rx.recv_timeout(remaining.min(Duration::from_millis(25))) {
      Ok(result) => return result,
      Err(RecvTimeoutError::Timeout) => {}
      Err(RecvTimeoutError::Disconnected) => {
        return Err(NetworkError::at(NetworkErrorCode::Internal, "dns"));
      }
    }
  }
}

fn reqwest_method(method: NetworkMethod) -> Method {
  match method {
    NetworkMethod::Get => Method::GET,
    NetworkMethod::Post => Method::POST,
  }
}

fn redirect_target(current: &Url, response: &Response) -> Result<Option<Url>, NetworkError> {
  if !matches!(
    response.status(),
    StatusCode::MOVED_PERMANENTLY
      | StatusCode::FOUND
      | StatusCode::SEE_OTHER
      | StatusCode::TEMPORARY_REDIRECT
      | StatusCode::PERMANENT_REDIRECT
  ) {
    return Ok(None);
  }
  let Some(location) = response.headers().get(LOCATION) else {
    return Ok(None);
  };
  let location = location
    .to_str()
    .map_err(|_| NetworkError::at(NetworkErrorCode::InvalidRequest, "redirect"))?;
  let mut next = current
    .join(location)
    .map_err(|_| NetworkError::at(NetworkErrorCode::InvalidRequest, "redirect"))?;
  if !matches!(next.scheme(), "http" | "https")
    || !next.username().is_empty()
    || next.password().is_some()
    || next.host_str().is_none()
  {
    return Err(NetworkError::at(
      NetworkErrorCode::PermissionDenied,
      "redirect",
    ));
  }
  next.set_fragment(None);
  if next.as_str().len() > super::MAX_URL_BYTES {
    return Err(NetworkError::at(NetworkErrorCode::TooLarge, "redirect"));
  }
  Ok(Some(next))
}

fn same_origin(left: &Url, right: &Url) -> bool {
  left.scheme() == right.scheme()
    && left.host_str() == right.host_str()
    && left.port_or_known_default() == right.port_or_known_default()
}

fn finish_response(
  original_url: String,
  final_url: Url,
  mut response: Response,
  response_mode: NetworkResponseMode,
  cancellation: &TaskCancellation,
) -> Result<NetworkResponse, NetworkError> {
  if response
    .content_length()
    .is_some_and(|length| length > MAX_RESPONSE_BODY_BYTES as u64)
  {
    return Err(NetworkError::at(
      NetworkErrorCode::TooLarge,
      "response_headers",
    ));
  }
  let status = response.status().as_u16();
  let headers = filtered_response_headers(response.headers());
  let mut bytes = Vec::new();
  let mut chunk = [0u8; READ_BUFFER_BYTES];
  loop {
    check_cancelled(cancellation)?;
    let count = response.read(&mut chunk).map_err(|error| {
      if error.kind() == std::io::ErrorKind::TimedOut {
        NetworkError::at(NetworkErrorCode::Timeout, "response_body")
      } else {
        NetworkError::at(NetworkErrorCode::Network, "response_body")
      }
    })?;
    if count == 0 {
      break;
    }
    if bytes.len().saturating_add(count) > MAX_RESPONSE_BODY_BYTES {
      return Err(NetworkError::at(
        NetworkErrorCode::TooLarge,
        "response_body",
      ));
    }
    bytes.extend_from_slice(&chunk[..count]);
  }
  check_cancelled(cancellation)?;

  let body = match response_mode {
    NetworkResponseMode::Text => NetworkResponseBody::Text(
      String::from_utf8(bytes)
        .map_err(|_| NetworkError::at(NetworkErrorCode::InvalidUtf8, "response_decode"))?,
    ),
    NetworkResponseMode::Bytes => NetworkResponseBody::Bytes(bytes),
  };
  Ok(NetworkResponse {
    original_url,
    final_url: final_url.to_string(),
    status,
    headers,
    body,
  })
}

fn filtered_response_headers(headers: &HeaderMap) -> BTreeMap<String, String> {
  let mut filtered = BTreeMap::<String, String>::new();
  for (name, value) in headers {
    if !security::is_safe_response_header(name.as_str()) {
      continue;
    }
    let Ok(value) = value.to_str() else {
      continue;
    };
    filtered
      .entry(name.as_str().to_string())
      .and_modify(|current| {
        current.push_str(", ");
        current.push_str(value);
      })
      .or_insert_with(|| value.to_string());
  }
  filtered
}

fn check_cancelled(cancellation: &TaskCancellation) -> Result<(), NetworkError> {
  if cancellation.is_cancelled() {
    Err(NetworkError::at(NetworkErrorCode::Cancelled, "cancel"))
  } else {
    Ok(())
  }
}

fn map_reqwest_error(error: reqwest::Error, stage: &'static str) -> NetworkError {
  if error.is_timeout() {
    NetworkError::at(NetworkErrorCode::Timeout, stage)
  } else if error.is_builder() {
    NetworkError::at(NetworkErrorCode::InvalidRequest, stage)
  } else {
    NetworkError::at(NetworkErrorCode::Network, stage)
  }
}

#[cfg(test)]
mod tests {
  use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
  };

  use crossbeam_channel::unbounded;

  use super::*;
  use crate::host_engine::services::network::{
    NetworkHeader, NetworkRequest, NetworkRequestBody, normalize_request_for_test,
  };

  fn serve_once(response: &'static [u8]) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    thread::spawn(move || {
      let (mut stream, _) = listener.accept().unwrap();
      let mut request = [0u8; 4096];
      let _ = stream.read(&mut request);
      stream.write_all(response).unwrap();
    });
    format!("http://{address}/")
  }

  fn read_request(stream: &mut TcpStream) -> Vec<u8> {
    stream
      .set_read_timeout(Some(Duration::from_secs(1)))
      .unwrap();
    let mut request = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
      let count = stream.read(&mut buffer).unwrap();
      if count == 0 {
        break;
      }
      request.extend_from_slice(&buffer[..count]);
      let Some(header_end) = request.windows(4).position(|part| part == b"\r\n\r\n") else {
        continue;
      };
      let header_end = header_end + 4;
      let headers = String::from_utf8_lossy(&request[..header_end]);
      let content_length = headers
        .lines()
        .find_map(|line| {
          let (name, value) = line.split_once(':')?;
          name
            .eq_ignore_ascii_case("content-length")
            .then(|| value.trim().parse::<usize>().ok())
            .flatten()
        })
        .unwrap_or(0);
      if request.len() >= header_end + content_length {
        break;
      }
    }
    request
  }

  fn redirected_post(status: u16) -> Vec<u8> {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
      let (mut first, _) = listener.accept().unwrap();
      let _ = read_request(&mut first);
      first
        .write_all(
          format!("HTTP/1.1 {status} Redirect\r\nLocation: /next\r\nContent-Length: 0\r\n\r\n")
            .as_bytes(),
        )
        .unwrap();
      let (mut second, _) = listener.accept().unwrap();
      let request = read_request(&mut second);
      second
        .write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n")
        .unwrap();
      request
    });
    let response = execute_test(NetworkRequest::post(
      format!("http://{address}/start"),
      NetworkRequestBody::Text("payload".to_string()),
      NetworkResponseMode::Bytes,
    ))
    .unwrap();
    assert_eq!(response.status, 204);
    server.join().unwrap()
  }

  fn execute_test(request: NetworkRequest) -> Result<NetworkResponse, NetworkError> {
    let task = NetworkTask {
      request: normalize_request_for_test(request).unwrap(),
    };
    execute(&task, &TaskCancellation::new(TaskId(1)))
  }

  #[test]
  fn get_returns_text_and_filters_headers() {
    let url = serve_once(
      b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nETag: test\r\nX-Secret: no\r\nContent-Length: 5\r\n\r\nhello",
    );
    let response = execute_test(NetworkRequest::get(url, NetworkResponseMode::Text)).unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(
      response.body,
      NetworkResponseBody::Text("hello".to_string())
    );
    assert_eq!(response.headers.get("etag"), Some(&"test".to_string()));
    assert!(!response.headers.contains_key("x-secret"));
  }

  #[test]
  fn post_sends_body_and_custom_header() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
      let (mut stream, _) = listener.accept().unwrap();
      let request = read_request(&mut stream);
      let request = String::from_utf8_lossy(&request);
      assert!(request.starts_with("POST / HTTP/1.1"));
      assert!(request.to_ascii_lowercase().contains("x-game-token: yes"));
      assert!(request.ends_with("payload"));
      stream
        .write_all(b"HTTP/1.1 201 Created\r\nContent-Length: 0\r\n\r\n")
        .unwrap();
    });
    let mut request = NetworkRequest::post(
      format!("http://{address}/"),
      NetworkRequestBody::Text("payload".to_string()),
      NetworkResponseMode::Bytes,
    );
    request
      .headers
      .push(NetworkHeader::new("X-Game-Token", "yes"));
    let response = execute_test(request).unwrap();
    assert_eq!(response.status, 201);
    assert_eq!(response.body, NetworkResponseBody::Bytes(Vec::new()));
    server.join().unwrap();
  }

  #[test]
  fn http_error_and_no_content_statuses_are_normal_responses() {
    let error_url = serve_once(b"HTTP/1.1 503 Unavailable\r\nContent-Length: 5\r\n\r\nlater");
    let response = execute_test(NetworkRequest::get(error_url, NetworkResponseMode::Text)).unwrap();
    assert_eq!(response.status, 503);
    assert_eq!(
      response.body,
      NetworkResponseBody::Text("later".to_string())
    );

    let empty_url = serve_once(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n");
    let response =
      execute_test(NetworkRequest::get(empty_url, NetworkResponseMode::Bytes)).unwrap();
    assert_eq!(response.status, 204);
    assert_eq!(response.body, NetworkResponseBody::Bytes(Vec::new()));
  }

  #[test]
  fn post_redirect_method_rules_match_http_semantics() {
    for status in [301, 302, 303] {
      let request = redirected_post(status);
      assert!(
        request.starts_with(b"GET /next HTTP/1.1"),
        "{status} must change POST to GET"
      );
      assert!(
        !request.ends_with(b"payload"),
        "{status} must remove the POST body"
      );
    }
    for status in [307, 308] {
      let request = redirected_post(status);
      assert!(
        request.starts_with(b"POST /next HTTP/1.1"),
        "{status} must preserve POST"
      );
      assert!(
        request.ends_with(b"payload"),
        "{status} must preserve the POST body"
      );
    }
  }

  #[test]
  fn cross_origin_redirect_removes_credentials() {
    let target = TcpListener::bind("127.0.0.1:0").unwrap();
    let target_address = target.local_addr().unwrap();
    let target_server = thread::spawn(move || {
      let (mut stream, _) = target.accept().unwrap();
      let request = read_request(&mut stream);
      stream
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
        .unwrap();
      String::from_utf8_lossy(&request).to_ascii_lowercase()
    });

    let source = TcpListener::bind("127.0.0.1:0").unwrap();
    let source_address = source.local_addr().unwrap();
    let source_server = thread::spawn(move || {
      let (mut stream, _) = source.accept().unwrap();
      let _ = read_request(&mut stream);
      stream
        .write_all(
          format!(
            "HTTP/1.1 302 Found\r\nLocation: http://{target_address}/final\r\nContent-Length: 0\r\n\r\n"
          )
          .as_bytes(),
        )
        .unwrap();
    });

    let mut request = NetworkRequest::get(
      format!("http://{source_address}/"),
      NetworkResponseMode::Bytes,
    );
    request
      .headers
      .push(NetworkHeader::new("Authorization", "Bearer secret"));
    request
      .headers
      .push(NetworkHeader::new("Cookie", "session=secret"));
    request
      .headers
      .push(NetworkHeader::new("X-Game-Token", "kept"));
    execute_test(request).unwrap();
    source_server.join().unwrap();
    let target_request = target_server.join().unwrap();
    assert!(!target_request.contains("authorization:"));
    assert!(!target_request.contains("cookie:"));
    assert!(target_request.contains("x-game-token: kept"));
  }

  #[test]
  fn response_size_is_rejected_before_reading_an_oversized_body() {
    let url =
      serve_once(b"HTTP/1.1 200 OK\r\nContent-Length: 4194305\r\nConnection: close\r\n\r\n");
    assert_eq!(
      execute_test(NetworkRequest::get(url, NetworkResponseMode::Bytes))
        .unwrap_err()
        .code,
      NetworkErrorCode::TooLarge
    );
  }

  #[test]
  fn response_mode_rejects_invalid_utf8_but_bytes_accept_it() {
    let invalid = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n\xff\xfe";
    let text_url = serve_once(invalid);
    assert_eq!(
      execute_test(NetworkRequest::get(text_url, NetworkResponseMode::Text))
        .unwrap_err()
        .code,
      NetworkErrorCode::InvalidUtf8
    );
    let bytes_url = serve_once(invalid);
    assert_eq!(
      execute_test(NetworkRequest::get(bytes_url, NetworkResponseMode::Bytes))
        .unwrap()
        .body,
      NetworkResponseBody::Bytes(vec![0xff, 0xfe])
    );
  }

  #[test]
  fn redirects_are_followed_and_revalidated() {
    let target = serve_once(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok");
    let redirect = format!("HTTP/1.1 302 Found\r\nLocation: {target}\r\nContent-Length: 0\r\n\r\n");
    let redirect: &'static [u8] = Box::leak(redirect.into_bytes().into_boxed_slice());
    let url = serve_once(redirect);
    let response = execute_test(NetworkRequest::get(url, NetworkResponseMode::Text)).unwrap();
    assert_eq!(response.body, NetworkResponseBody::Text("ok".to_string()));
    assert_eq!(response.final_url, target);
  }

  #[test]
  fn public_policy_rejects_loopback() {
    let request = NetworkRequest::get("http://127.0.0.1:80/", NetworkResponseMode::Text);
    let task = NetworkTask {
      request: super::super::normalize_request(request, security::AddressPolicy::PublicOnly)
        .unwrap(),
    };
    assert_eq!(
      execute(&task, &TaskCancellation::new(TaskId(1)))
        .unwrap_err()
        .code,
      NetworkErrorCode::PermissionDenied
    );
  }

  #[test]
  fn cancelled_request_returns_cancelled_error() {
    let url = serve_once(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok");
    let task = NetworkTask {
      request: normalize_request_for_test(NetworkRequest::get(url, NetworkResponseMode::Text))
        .unwrap(),
    };
    let cancellation = TaskCancellation::new(TaskId(5));
    cancellation.cancel();
    assert_eq!(
      execute(&task, &cancellation).unwrap_err().code,
      NetworkErrorCode::Cancelled
    );
  }

  #[test]
  fn task_runner_emits_one_terminal_event() {
    let url = serve_once(b"HTTP/1.1 404 Not Found\r\nContent-Length: 3\r\n\r\n404");
    let task = NetworkTask {
      request: normalize_request_for_test(NetworkRequest::get(url, NetworkResponseMode::Text))
        .unwrap(),
    };
    let (sender, receiver) = unbounded::<NetworkEvent>();
    run_network_task(TaskId(7), task, &sender, &TaskCancellation::new(TaskId(7))).unwrap();
    let events = receiver.try_iter().collect::<Vec<_>>();
    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], NetworkEvent::Started { .. }));
    assert!(matches!(
      events[1],
      NetworkEvent::Finished {
        response: NetworkResponse { status: 404, .. },
        ..
      }
    ));
  }
}
