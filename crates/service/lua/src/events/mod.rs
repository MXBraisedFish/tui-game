mod broker;
#[cfg(test)]
mod translate;

use mlua::{Lua, Table};

use tg_core_input::{MouseButton, MouseEvent, MouseEventKind, ScrollDirection};
use tg_service_network::{NetworkError, NetworkErrorCode, NetworkMethod};

pub use broker::{
  LuaEnqueueError, LuaEventBroker, LuaEventCallbackId, LuaEventDelivery, LuaEventRoute,
  LuaRoutableEvent, LuaSessionToken, LuaTaskOperation, MAX_LUA_EVENTS_PER_FRAME,
  MAX_LUA_FILE_TASKS_PER_SESSION, MAX_LUA_NETWORK_TASKS_PER_SESSION, MAX_LUA_PENDING_EVENTS,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaActionState {
  Pressed,
  Held,
  Released,
}

impl From<tg_core_input::KeyState> for LuaActionState {
  fn from(value: tg_core_input::KeyState) -> Self {
    match value {
      tg_core_input::KeyState::Pressed => Self::Pressed,
      tg_core_input::KeyState::Held => Self::Held,
      tg_core_input::KeyState::Released => Self::Released,
    }
  }
}

impl LuaActionState {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Pressed => "pressed",
      Self::Held => "held",
      Self::Released => "released",
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaTimerKind {
  Timer,
  Delay,
  Repeat,
  Sleep,
}

impl LuaTimerKind {
  fn as_str(self) -> &'static str {
    match self {
      Self::Timer => "timer",
      Self::Delay => "delay",
      Self::Repeat => "repeat",
      Self::Sleep => "sleep",
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaTimerEventKind {
  Tick,
  Finished,
}

impl LuaTimerEventKind {
  fn as_str(self) -> &'static str {
    match self {
      Self::Tick => "tick",
      Self::Finished => "finished",
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaTimerEvent {
  pub id: u64,
  pub timer_kind: LuaTimerKind,
  pub kind: LuaTimerEventKind,
  pub executed_count: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaAnimationEventKind {
  Started,
  Marker { name: String },
  Loop { completed: u32 },
  Finished,
  Cancelled,
}

impl LuaAnimationEventKind {
  fn as_str(&self) -> &'static str {
    match self {
      Self::Started => "started",
      Self::Marker { .. } => "marker",
      Self::Loop { .. } => "loop",
      Self::Finished => "finished",
      Self::Cancelled => "cancelled",
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaAnimationEvent {
  pub id: u64,
  pub kind: LuaAnimationEventKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaFileOperation {
  ReadText,
  ReadBytes,
  WriteText,
  WriteBytes,
  ListDir,
  CreateDir,
  Remove,
}

impl LuaFileOperation {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::ReadText => "read_text",
      Self::ReadBytes => "read_bytes",
      Self::WriteText => "write_text",
      Self::WriteBytes => "write_bytes",
      Self::ListDir => "list_dir",
      Self::CreateDir => "create_dir",
      Self::Remove => "remove",
    }
  }

  pub fn is_write(self) -> bool {
    matches!(
      self,
      Self::WriteText | Self::WriteBytes | Self::ListDir | Self::CreateDir | Self::Remove
    )
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaEventErrorCode {
  InvalidRequest,
  PermissionDenied,
  NotFound,
  TooLarge,
  InvalidUtf8,
  Cancelled,
  Timeout,
  Io,
  Network,
  Unsupported,
  Decode,
  BackendUnavailable,
  Internal,
}

impl LuaEventErrorCode {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::InvalidRequest => "invalid_request",
      Self::PermissionDenied => "permission_denied",
      Self::NotFound => "not_found",
      Self::TooLarge => "too_large",
      Self::InvalidUtf8 => "invalid_utf8",
      Self::Cancelled => "cancelled",
      Self::Timeout => "timeout",
      Self::Io => "io",
      Self::Network => "network",
      Self::Unsupported => "unsupported",
      Self::Decode => "decode",
      Self::BackendUnavailable => "backend_unavailable",
      Self::Internal => "internal",
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaEventError {
  pub code: LuaEventErrorCode,
  pub message: String,
}

impl LuaEventError {
  pub fn sanitized(code: LuaEventErrorCode) -> Self {
    Self {
      code,
      message: match code {
        LuaEventErrorCode::InvalidRequest => "invalid request",
        LuaEventErrorCode::PermissionDenied => "permission denied",
        LuaEventErrorCode::NotFound => "resource not found",
        LuaEventErrorCode::TooLarge => "resource exceeds its size limit",
        LuaEventErrorCode::InvalidUtf8 => "resource is not valid UTF-8",
        LuaEventErrorCode::Cancelled => "request was cancelled",
        LuaEventErrorCode::Timeout => "request timed out",
        LuaEventErrorCode::Io => "I/O operation failed",
        LuaEventErrorCode::Network => "network operation failed",
        LuaEventErrorCode::Unsupported => "operation is not supported",
        LuaEventErrorCode::Decode => "audio resource could not be decoded",
        LuaEventErrorCode::BackendUnavailable => "audio output is unavailable",
        LuaEventErrorCode::Internal => "internal operation failed",
      }
      .to_string(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaFileOutcome {
  Text(String),
  Bytes(Vec<u8>),
  Written,
  DirectoryCreated,
  Removed,
  Entries(Vec<LuaFileEntry>),
  Failed(LuaEventError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaFileEntry {
  pub path: String,
  pub file_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaFileEvent {
  pub request_id: u64,
  pub kind: LuaFileOperation,
  pub path: String,
  pub tip: Option<String>,
  pub outcome: LuaFileOutcome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaI18nEventKind {
  Created,
  Reloaded,
}

impl LuaI18nEventKind {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Created => "created",
      Self::Reloaded => "reloaded",
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaI18nEvent {
  pub kind: LuaI18nEventKind,
  pub ok: bool,
  pub message: String,
  pub language_code: String,
  pub callback_language_code: String,
  pub(crate) namespaces:
    Option<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaImageOutcome {
  Converted(String),
  Failed(LuaEventError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaImageEvent {
  pub request_id: u64,
  pub outcome: LuaImageOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaNetworkOutcome {
  Response {
    final_url: String,
    status: u16,
    headers: std::collections::BTreeMap<String, String>,
    body: LuaNetworkBody,
  },
  Failed(LuaEventError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaNetworkBody {
  Text(String),
  Bytes(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaNetworkEvent {
  pub request_id: u64,
  pub method: NetworkMethod,
  pub url: String,
  pub outcome: LuaNetworkOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaHitAreaEvent {
  pub id: u64,
  pub kind: &'static str,
  pub x: u16,
  pub y: u16,
  pub button: Option<&'static str>,
  pub dx: Option<i32>,
  pub dy: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaHyperlinkEvent {
  pub id: u64,
  pub link: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaMarkdownEvent {
  pub id: u64,
  pub href: String,
  pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaTextInputEvent {
  pub id: u64,
  pub kind: &'static str,
  pub value: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaScrollBoxEvent {
  pub id: u64,
  pub x: u16,
  pub y: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaAudioEventKind {
  Ready,
  Started,
  Paused,
  Resumed,
  Stopped,
  Finished,
  Failed,
}

impl LuaAudioEventKind {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Ready => "ready",
      Self::Started => "started",
      Self::Paused => "paused",
      Self::Resumed => "resumed",
      Self::Stopped => "stopped",
      Self::Finished => "finished",
      Self::Failed => "failed",
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaAudioEvent {
  pub id: u64,
  pub kind: LuaAudioEventKind,
  pub duration_ms: Option<u64>,
  pub position_ms: Option<u64>,
  pub error: Option<LuaEventError>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaEventData {
  Action {
    action: String,
    state: LuaActionState,
  },
  Mouse {
    kind: &'static str,
    button: Option<&'static str>,
    scroll: Option<&'static str>,
    x: u16,
    y: u16,
  },
  Resize {
    width: u16,
    height: u16,
  },
  Focus {
    gained: bool,
  },
  OverlayStarted,
  OverlayStopped,
  Timer(LuaTimerEvent),
  Animation(LuaAnimationEvent),
  File(LuaFileEvent),
  I18n(LuaI18nEvent),
  Image(LuaImageEvent),
  Network(LuaNetworkEvent),
  Audio(LuaAudioEvent),
  HitArea(LuaHitAreaEvent),
  Hyperlink(LuaHyperlinkEvent),
  Markdown(LuaMarkdownEvent),
  TextInput(LuaTextInputEvent),
  ScrollBox(LuaScrollBoxEvent),
}

impl LuaEventData {
  pub fn event_type(&self) -> &'static str {
    match self {
      Self::Action { .. } => "action",
      Self::Mouse { .. } => "mouse",
      Self::Resize { .. } => "resize",
      Self::Focus { .. } => "focus",
      Self::OverlayStarted => "overlay_started",
      Self::OverlayStopped => "overlay_stopped",
      Self::Timer(_) => "timer",
      Self::Animation(_) => "animation",
      Self::File(_) => "file",
      Self::I18n(_) => "i18n",
      Self::Image(_) => "image",
      Self::Network(_) => "network",
      Self::Audio(_) => "audio",
      Self::HitArea(_) => "hit_area",
      Self::Hyperlink(_) => "hyperlink",
      Self::Markdown(_) => "markdown",
      Self::TextInput(_) => "text_input",
      Self::ScrollBox(_) => "scroll_box",
    }
  }

  pub fn mouse(event: MouseEvent) -> Self {
    Self::Mouse {
      kind: match event.kind {
        MouseEventKind::Press => "pressed",
        MouseEventKind::Release => "released",
        MouseEventKind::Move => "moved",
        MouseEventKind::Drag => "dragged",
        MouseEventKind::Hold => "held",
        MouseEventKind::Scroll => "scrolled",
      },
      button: event.button.map(mouse_button),
      scroll: event.scroll.map(scroll_direction),
      x: event.x,
      y: event.y,
    }
  }

  pub fn callback_is_terminal(&self) -> bool {
    match self {
      Self::Timer(event) => event.kind == LuaTimerEventKind::Finished,
      Self::Animation(event) => matches!(
        event.kind,
        LuaAnimationEventKind::Finished | LuaAnimationEventKind::Cancelled
      ),
      Self::File(_) | Self::I18n(_) | Self::Image(_) | Self::Network(_) => true,
      _ => false,
    }
  }

  fn is_interactive(&self) -> bool {
    matches!(
      self,
      Self::Action { .. }
        | Self::Mouse { .. }
        | Self::HitArea(_)
        | Self::Hyperlink(_)
        | Self::Markdown(_)
        | Self::TextInput(_)
        | Self::ScrollBox(_)
    )
  }

  pub(super) fn is_coalescible_with(&self, newer: &Self) -> bool {
    match (self, newer) {
      (Self::Resize { .. }, Self::Resize { .. }) => true,
      (
        Self::Mouse {
          kind: left_kind,
          button: left_button,
          ..
        },
        Self::Mouse {
          kind: right_kind,
          button: right_button,
          ..
        },
      ) => {
        left_kind == right_kind
          && left_button == right_button
          && matches!(*left_kind, "moved" | "held")
      }
      (Self::HitArea(left), Self::HitArea(right)) => {
        left.id == right.id && left.kind == "hover_move" && right.kind == "hover_move"
      }
      (Self::ScrollBox(left), Self::ScrollBox(right)) => left.id == right.id,
      _ => false,
    }
  }

  pub(super) fn allowed_for(&self, kind: super::LuaSessionKind) -> bool {
    match kind {
      super::LuaSessionKind::Game => true,
      super::LuaSessionKind::Screensaver => match self {
        Self::Action { .. }
        | Self::Mouse { .. }
        | Self::OverlayStarted
        | Self::OverlayStopped
        | Self::HitArea(_)
        | Self::Hyperlink(_)
        | Self::Markdown(_)
        | Self::TextInput(_)
        | Self::ScrollBox(_) => false,
        Self::File(event) => !event.kind.is_write(),
        Self::Resize { .. }
        | Self::Focus { .. }
        | Self::Timer(_)
        | Self::Animation(_)
        | Self::I18n(_)
        | Self::Image(_)
        | Self::Network(_) => true,
        Self::Audio(_) => true,
      },
    }
  }

  pub(super) fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
    let data = lua.create_table()?;
    match self {
      Self::Action { action, state } => {
        data.set("action", action.as_str())?;
        data.set("state", state.as_str())?;
      }
      Self::Mouse {
        kind,
        button,
        scroll,
        x,
        y,
      } => {
        data.set("kind", *kind)?;
        data.set("button", *button)?;
        data.set("scroll", *scroll)?;
        data.set("x", *x)?;
        data.set("y", *y)?;
      }
      Self::Resize { width, height } => {
        data.set("width", *width)?;
        data.set("height", *height)?;
      }
      Self::Focus { gained } => data.set("gained", *gained)?,
      Self::OverlayStarted | Self::OverlayStopped => {}
      Self::Timer(event) => {
        data.set("id", event.id)?;
        data.set("timer_kind", event.timer_kind.as_str())?;
        data.set("kind", event.kind.as_str())?;
        data.set("executed_count", event.executed_count)?;
      }
      Self::Animation(event) => {
        data.set("id", event.id)?;
        data.set("kind", event.kind.as_str())?;
        match &event.kind {
          LuaAnimationEventKind::Marker { name } => data.set("name", name.as_str())?,
          LuaAnimationEventKind::Loop { completed } => data.set("completed", *completed)?,
          _ => {}
        }
      }
      Self::File(event) => {
        data.set("request_id", event.request_id)?;
        data.set("kind", event.kind.as_str())?;
        data.set("path", event.path.as_str())?;
        data.set("tip", event.tip.as_deref())?;
        match &event.outcome {
          LuaFileOutcome::Text(text) => {
            data.set("ok", true)?;
            data.set("text", text.as_str())?;
          }
          LuaFileOutcome::Bytes(bytes) => {
            data.set("ok", true)?;
            data.set("bytes", lua.create_string(bytes)?)?;
          }
          LuaFileOutcome::Written | LuaFileOutcome::DirectoryCreated | LuaFileOutcome::Removed => {
            data.set("ok", true)?
          }
          LuaFileOutcome::Entries(entries) => {
            data.set("ok", true)?;
            let values = lua.create_table()?;
            for (index, entry) in entries.iter().enumerate() {
              let value = lua.create_table()?;
              value.set("path", entry.path.as_str())?;
              value.set("file_type", entry.file_type.as_str())?;
              values.raw_set(index + 1, value)?;
            }
            data.set("entries", values)?;
          }
          LuaFileOutcome::Failed(error) => {
            data.set("ok", false)?;
            data.set("error", error_table(lua, error)?)?;
          }
        }
      }
      Self::I18n(event) => {
        data.set("kind", event.kind.as_str())?;
        data.set("ok", event.ok)?;
        data.set("message", event.message.as_str())?;
        data.set("language_code", event.language_code.as_str())?;
        data.set(
          "callback_language_code",
          event.callback_language_code.as_str(),
        )?;
      }
      Self::Image(event) => {
        data.set("request_id", event.request_id)?;
        data.set("kind", "convert")?;
        match &event.outcome {
          LuaImageOutcome::Converted(output) => {
            data.set("ok", true)?;
            data.set("output", output.as_str())?;
          }
          LuaImageOutcome::Failed(error) => {
            data.set("ok", false)?;
            data.set("error", error_table(lua, error)?)?;
          }
        }
      }
      Self::Network(event) => {
        data.set("request_id", event.request_id)?;
        data.set("kind", event.method.as_str())?;
        data.set("url", event.url.as_str())?;
        match &event.outcome {
          LuaNetworkOutcome::Response {
            final_url,
            status,
            headers,
            body,
          } => {
            data.set("ok", true)?;
            data.set("final_url", final_url.as_str())?;
            data.set("status", *status)?;
            let header_table = lua.create_table()?;
            for (name, value) in headers {
              header_table.set(name.as_str(), value.as_str())?;
            }
            data.set("headers", header_table)?;
            match body {
              LuaNetworkBody::Text(text) => data.set("text", text.as_str())?,
              LuaNetworkBody::Bytes(bytes) => {
                data.set("bytes", lua.create_string(bytes)?)?;
              }
            }
          }
          LuaNetworkOutcome::Failed(error) => {
            data.set("ok", false)?;
            data.set("error", error_table(lua, error)?)?;
          }
        }
      }
      Self::Audio(event) => {
        data.set("id", event.id)?;
        data.set("kind", event.kind.as_str())?;
        data.set("duration_ms", event.duration_ms)?;
        data.set("position_ms", event.position_ms)?;
        if let Some(error) = &event.error {
          data.set("error", error_table(lua, error)?)?;
        }
      }
      Self::HitArea(event) => {
        data.set("id", event.id)?;
        data.set("kind", event.kind)?;
        data.set("x", event.x)?;
        data.set("y", event.y)?;
        data.set("button", event.button)?;
        data.set("dx", event.dx)?;
        data.set("dy", event.dy)?;
      }
      Self::Hyperlink(event) => {
        data.set("id", event.id)?;
        data.set("kind", "clicked")?;
        data.set("link", event.link.as_str())?;
      }
      Self::Markdown(event) => {
        data.set("id", event.id)?;
        data.set("kind", "link_clicked")?;
        data.set("href", event.href.as_str())?;
        data.set("text", event.text.as_str())?;
      }
      Self::TextInput(event) => {
        data.set("id", event.id)?;
        data.set("kind", event.kind)?;
        data.set("value", event.value.as_deref())?;
      }
      Self::ScrollBox(event) => {
        data.set("id", event.id)?;
        data.set("kind", "scrolled")?;
        data.set("x", event.x)?;
        data.set("y", event.y)?;
      }
    }
    Ok(data)
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaRuntimeEvent {
  pub sequence: u64,
  pub frame: u64,
  pub data: LuaEventData,
}

fn error_table(lua: &Lua, error: &LuaEventError) -> mlua::Result<Table> {
  let table = lua.create_table()?;
  table.set("code", error.code.as_str())?;
  table.set("message", error.message.as_str())?;
  Ok(table)
}

pub(super) fn sanitize_io_error(message: &str) -> LuaEventError {
  let lower = message.to_ascii_lowercase();
  let code = if lower.contains("not found") || lower.contains("cannot find") {
    LuaEventErrorCode::NotFound
  } else if lower.contains("permission") || lower.contains("access is denied") {
    LuaEventErrorCode::PermissionDenied
  } else if lower.contains("utf-8")
    || lower.contains("utf8")
    || lower.contains("utf-16")
    || lower.contains("decode")
    || lower.contains("replacement")
    || lower.contains("binary data")
    || lower.contains("nul")
  {
    LuaEventErrorCode::InvalidUtf8
  } else if lower.contains("too large") || lower.contains("size limit") || lower.contains("exceeds")
  {
    LuaEventErrorCode::TooLarge
  } else if lower.contains("cancel") {
    LuaEventErrorCode::Cancelled
  } else if lower.contains("timeout") || lower.contains("timed out") {
    LuaEventErrorCode::Timeout
  } else {
    LuaEventErrorCode::Io
  };
  LuaEventError::sanitized(code)
}

pub(super) fn sanitize_network_error(error: &NetworkError) -> LuaEventError {
  let code = match error.code {
    NetworkErrorCode::InvalidRequest => LuaEventErrorCode::InvalidRequest,
    NetworkErrorCode::PermissionDenied => LuaEventErrorCode::PermissionDenied,
    NetworkErrorCode::TooLarge => LuaEventErrorCode::TooLarge,
    NetworkErrorCode::InvalidUtf8 => LuaEventErrorCode::InvalidUtf8,
    NetworkErrorCode::Cancelled => LuaEventErrorCode::Cancelled,
    NetworkErrorCode::Timeout => LuaEventErrorCode::Timeout,
    NetworkErrorCode::Network => LuaEventErrorCode::Network,
    NetworkErrorCode::Unsupported => LuaEventErrorCode::Unsupported,
    NetworkErrorCode::Internal => LuaEventErrorCode::Internal,
  };
  LuaEventError::sanitized(code)
}

fn mouse_button(button: MouseButton) -> &'static str {
  match button {
    MouseButton::Left => "left",
    MouseButton::Middle => "middle",
    MouseButton::Right => "right",
  }
}

fn scroll_direction(direction: ScrollDirection) -> &'static str {
  match direction {
    ScrollDirection::Up => "up",
    ScrollDirection::Down => "down",
    ScrollDirection::Left => "left",
    ScrollDirection::Right => "right",
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use mlua::Value;

  #[test]
  fn service_errors_never_echo_raw_host_details() {
    let raw = r#"Access is denied: C:\Users\secret\save.json"#;
    let error = sanitize_io_error(raw);
    assert_eq!(error.code, LuaEventErrorCode::PermissionDenied);
    assert!(!error.message.contains("C:\\"));
    assert!(!error.message.contains("secret"));
    assert_eq!(
      sanitize_io_error("file exceeds 1 MiB").code,
      LuaEventErrorCode::TooLarge
    );
    assert_eq!(
      sanitize_io_error("text cannot be decoded without replacement").code,
      LuaEventErrorCode::InvalidUtf8
    );
  }

  #[test]
  fn binary_file_payload_is_a_lua_string() {
    let lua = Lua::new();
    let data = LuaEventData::File(LuaFileEvent {
      request_id: 7,
      kind: LuaFileOperation::ReadBytes,
      path: "assets/data.bin".to_string(),
      tip: None,
      outcome: LuaFileOutcome::Bytes(vec![0, 1, 255]),
    })
    .to_lua_table(&lua)
    .unwrap();
    assert!(data.get::<bool>("ok").unwrap());
    assert_eq!(
      data
        .get::<mlua::LuaString>("bytes")
        .unwrap()
        .as_bytes()
        .as_ref(),
      &[0, 1, 255]
    );
  }

  #[test]
  fn directory_file_payloads_only_expose_success_metadata() {
    let lua = Lua::new();
    for (kind, outcome, expected_kind) in [
      (
        LuaFileOperation::CreateDir,
        LuaFileOutcome::DirectoryCreated,
        "create_dir",
      ),
      (LuaFileOperation::Remove, LuaFileOutcome::Removed, "remove"),
    ] {
      let data = LuaEventData::File(LuaFileEvent {
        request_id: 8,
        kind,
        path: "save/slot-a".to_string(),
        tip: Some("operation".to_string()),
        outcome,
      })
      .to_lua_table(&lua)
      .unwrap();
      assert_eq!(data.get::<String>("kind").unwrap(), expected_kind);
      assert_eq!(data.get::<String>("path").unwrap(), "save/slot-a");
      assert_eq!(data.get::<String>("tip").unwrap(), "operation");
      assert!(data.get::<bool>("ok").unwrap());
      for field in ["text", "bytes", "entries", "error"] {
        assert!(matches!(data.get::<Value>(field).unwrap(), Value::Nil));
      }
    }
  }

  #[test]
  fn network_payload_exposes_exactly_one_response_body_field() {
    let lua = Lua::new();
    let text = LuaEventData::Network(LuaNetworkEvent {
      request_id: 11,
      method: NetworkMethod::Get,
      url: "https://example.com/request".to_string(),
      outcome: LuaNetworkOutcome::Response {
        final_url: "https://example.com/final".to_string(),
        status: 200,
        headers: std::collections::BTreeMap::from([(
          "content-type".to_string(),
          "text/plain".to_string(),
        )]),
        body: LuaNetworkBody::Text("hello".to_string()),
      },
    })
    .to_lua_table(&lua)
    .unwrap();
    assert_eq!(text.get::<String>("kind").unwrap(), "get");
    assert_eq!(
      text.get::<String>("url").unwrap(),
      "https://example.com/request"
    );
    assert_eq!(
      text.get::<String>("final_url").unwrap(),
      "https://example.com/final"
    );
    assert_eq!(text.get::<u16>("status").unwrap(), 200);
    assert_eq!(text.get::<String>("text").unwrap(), "hello");
    assert!(matches!(text.get::<Value>("bytes").unwrap(), Value::Nil));
    let headers = text.get::<Table>("headers").unwrap();
    assert_eq!(headers.get::<String>("content-type").unwrap(), "text/plain");

    let bytes = LuaEventData::Network(LuaNetworkEvent {
      request_id: 12,
      method: NetworkMethod::Post,
      url: "https://example.com/upload".to_string(),
      outcome: LuaNetworkOutcome::Response {
        final_url: "https://example.com/upload".to_string(),
        status: 201,
        headers: std::collections::BTreeMap::new(),
        body: LuaNetworkBody::Bytes(vec![0, 1, 255]),
      },
    })
    .to_lua_table(&lua)
    .unwrap();
    assert_eq!(bytes.get::<String>("kind").unwrap(), "post");
    assert!(matches!(bytes.get::<Value>("text").unwrap(), Value::Nil));
    assert_eq!(
      bytes
        .get::<mlua::LuaString>("bytes")
        .unwrap()
        .as_bytes()
        .as_ref(),
      &[0, 1, 255]
    );
  }

  #[test]
  fn network_failure_only_exposes_sanitized_error() {
    let lua = Lua::new();
    let data = LuaEventData::Network(LuaNetworkEvent {
      request_id: 13,
      method: NetworkMethod::Get,
      url: "https://example.com/private?token=secret".to_string(),
      outcome: LuaNetworkOutcome::Failed(sanitize_network_error(&NetworkError::at(
        NetworkErrorCode::Timeout,
        "response_body",
      ))),
    })
    .to_lua_table(&lua)
    .unwrap();
    assert!(!data.get::<bool>("ok").unwrap());
    assert!(matches!(data.get::<Value>("status").unwrap(), Value::Nil));
    assert!(matches!(data.get::<Value>("text").unwrap(), Value::Nil));
    assert!(matches!(data.get::<Value>("bytes").unwrap(), Value::Nil));
    let error = data.get::<Table>("error").unwrap();
    assert_eq!(error.get::<String>("code").unwrap(), "timeout");
    let message = error.get::<String>("message").unwrap();
    assert!(!message.contains("C:\\"));
    assert!(!message.contains("private"));
  }

  #[test]
  fn every_protocol_variant_builds_its_declared_lua_payload() {
    let lua = Lua::new();
    let cases = vec![
      (
        LuaEventData::Action {
          action: "jump".to_string(),
          state: LuaActionState::Held,
        },
        "action",
      ),
      (
        LuaEventData::Mouse {
          kind: "scrolled",
          button: None,
          scroll: Some("down"),
          x: 4,
          y: 5,
        },
        "mouse",
      ),
      (
        LuaEventData::Resize {
          width: 80,
          height: 24,
        },
        "resize",
      ),
      (LuaEventData::Focus { gained: true }, "focus"),
      (LuaEventData::OverlayStarted, "overlay_started"),
      (LuaEventData::OverlayStopped, "overlay_stopped"),
      (
        LuaEventData::Timer(LuaTimerEvent {
          id: 1,
          timer_kind: LuaTimerKind::Repeat,
          kind: LuaTimerEventKind::Tick,
          executed_count: Some(2),
        }),
        "timer",
      ),
      (
        LuaEventData::Animation(LuaAnimationEvent {
          id: 2,
          kind: LuaAnimationEventKind::Marker {
            name: "middle".to_string(),
          },
        }),
        "animation",
      ),
      (
        LuaEventData::File(LuaFileEvent {
          request_id: 3,
          kind: LuaFileOperation::ReadText,
          path: "assets/file.txt".to_string(),
          tip: None,
          outcome: LuaFileOutcome::Text("text".to_string()),
        }),
        "file",
      ),
      (
        LuaEventData::Image(LuaImageEvent {
          request_id: 4,
          outcome: LuaImageOutcome::Converted("image".to_string()),
        }),
        "image",
      ),
      (
        LuaEventData::Network(LuaNetworkEvent {
          request_id: 5,
          method: NetworkMethod::Get,
          url: "https://example.invalid".to_string(),
          outcome: LuaNetworkOutcome::Response {
            final_url: "https://example.invalid/final".to_string(),
            status: 404,
            headers: std::collections::BTreeMap::from([(
              "content-type".to_string(),
              "text/plain".to_string(),
            )]),
            body: LuaNetworkBody::Text("missing".to_string()),
          },
        }),
        "network",
      ),
      (
        LuaEventData::Audio(LuaAudioEvent {
          id: 6,
          kind: LuaAudioEventKind::Ready,
          duration_ms: Some(1_200),
          position_ms: None,
          error: None,
        }),
        "audio",
      ),
      (
        LuaEventData::HitArea(LuaHitAreaEvent {
          id: 6,
          kind: "drag",
          x: 7,
          y: 8,
          button: Some("left"),
          dx: Some(-1),
          dy: Some(2),
        }),
        "hit_area",
      ),
      (
        LuaEventData::Hyperlink(LuaHyperlinkEvent {
          id: 7,
          link: "target".to_string(),
        }),
        "hyperlink",
      ),
      (
        LuaEventData::Markdown(LuaMarkdownEvent {
          id: 8,
          href: "target".to_string(),
          text: "label".to_string(),
        }),
        "markdown",
      ),
      (
        LuaEventData::TextInput(LuaTextInputEvent {
          id: 9,
          kind: "changed",
          value: Some("value".to_string()),
        }),
        "text_input",
      ),
      (
        LuaEventData::ScrollBox(LuaScrollBoxEvent { id: 10, x: 2, y: 3 }),
        "scroll_box",
      ),
    ];

    for (event, expected_type) in cases {
      assert_eq!(event.event_type(), expected_type);
      event.to_lua_table(&lua).unwrap();
    }
  }

  #[test]
  fn optional_protocol_fields_are_nil_when_not_applicable() {
    let lua = Lua::new();
    let mouse = LuaEventData::Mouse {
      kind: "moved",
      button: None,
      scroll: None,
      x: 0,
      y: 0,
    }
    .to_lua_table(&lua)
    .unwrap();
    assert_eq!(mouse.get::<Value>("button").unwrap(), Value::Nil);
    assert_eq!(mouse.get::<Value>("scroll").unwrap(), Value::Nil);

    let timer = LuaEventData::Timer(LuaTimerEvent {
      id: 1,
      timer_kind: LuaTimerKind::Timer,
      kind: LuaTimerEventKind::Finished,
      executed_count: None,
    })
    .to_lua_table(&lua)
    .unwrap();
    assert_eq!(timer.get::<Value>("executed_count").unwrap(), Value::Nil);

    let audio = LuaEventData::Audio(LuaAudioEvent {
      id: 9,
      kind: LuaAudioEventKind::Failed,
      duration_ms: None,
      position_ms: None,
      error: Some(LuaEventError::sanitized(
        LuaEventErrorCode::BackendUnavailable,
      )),
    })
    .to_lua_table(&lua)
    .unwrap();
    assert_eq!(audio.get::<String>("kind").unwrap(), "failed");
    assert_eq!(audio.get::<Value>("duration_ms").unwrap(), Value::Nil);
    assert_eq!(audio.get::<Value>("position_ms").unwrap(), Value::Nil);
    assert_eq!(
      audio
        .get::<Table>("error")
        .unwrap()
        .get::<String>("code")
        .unwrap(),
      "backend_unavailable"
    );
  }
}
