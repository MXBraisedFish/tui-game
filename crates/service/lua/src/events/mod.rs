//! Script-visible event payloads and ownership-aware delivery.

mod broker;
#[cfg(test)]
mod translate;

use mlua::{Lua, Table};

use tg_core_input::{MouseButton, MouseEvent, MouseEventKind, ScrollDirection};
use tg_service_network::{NetworkError, NetworkErrorCode, NetworkMethod};

pub use broker::{
  LuaEnqueueError, LuaEventBroker, LuaEventCallbackId, LuaEventDelivery, LuaEventRoute,
  LuaRoutableEvent, LuaSessionToken, LuaTaskOperation, MAX_LUA_EVENTS_PER_FRAME,
  MAX_LUA_FILE_TASKS_PER_SESSION, MAX_LUA_IMAGE_TASKS_PER_SESSION,
  MAX_LUA_NETWORK_TASKS_PER_SESSION, MAX_LUA_PENDING_EVENTS,
};
/// The retained state of Lua action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaActionState {
  /// A transition into the pressed state.
  Pressed,
  /// The operation is held.
  Held,
  /// A transition out of the pressed state.
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
  /// Return the stable string key for this Lua action state.
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Pressed => "pressed",
      Self::Held => "held",
      Self::Released => "released",
    }
  }
}

/// The Lua timer kind representation used by this module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaTimerKind {
  /// The timer setting for Lua timer kind.
  Timer,
  /// The delay setting for Lua timer kind.
  Delay,
  /// The repeat setting for Lua timer kind.
  Repeat,
  /// The sleep setting for Lua timer kind.
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

/// The Lua timer event kind representation used by this module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaTimerEventKind {
  /// A tick notification delivered to the owning consumer.
  Tick,
  /// The operation is finished.
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

/// A Lua timer event payload queued for its owning consumer.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `timer_kind` - The timer kind.
/// * `kind` - The Lua timer event kind carried by this Lua timer event.
/// * `executed_count` - The executed count.
/// * `object_id` - The script timer handle, when available.
/// * `tip` - The custom text attached to the event.
/// * `revision` - The internal revision rejecting stale deliveries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaTimerEvent {
  /// The identifier of the owned object.
  pub id: u64,
  /// The timer kind.
  pub timer_kind: LuaTimerKind,
  /// The Lua timer event kind carried by this Lua timer event.
  pub kind: LuaTimerEventKind,
  /// The executed count.
  pub executed_count: Option<u32>,
  /// The script object handle, when the event comes from the timer library.
  pub object_id: Option<String>,
  /// The custom text supplied when creating or configuring the timer.
  pub tip: Option<String>,
  /// The internal schedule revision used to discard invalidated deliveries.
  pub revision: Option<u64>,
}

/// The Lua animation event kind representation used by this module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaAnimationEventKind {
  /// A started notification delivered to the owning consumer.
  Started,
  /// A marker notification delivered to the owning consumer.
  Marker {
    /// The name used to identify the object or field.
    name: String,
  },
  /// A loop notification delivered to the owning consumer.
  Loop {
    /// The completed.
    completed: u32,
  },
  /// The operation is finished.
  Finished,
  /// A cancelled notification delivered to the owning consumer.
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

/// A Lua animation event payload queued for its owning consumer.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `kind` - The Lua animation event kind carried by this Lua animation event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaAnimationEvent {
  /// The identifier of the owned object.
  pub id: u64,
  /// The Lua animation event kind carried by this Lua animation event.
  pub kind: LuaAnimationEventKind,
}

/// The Lua file operation representation used by this module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaFileOperation {
  /// The read text setting for Lua file operation.
  ReadText,
  /// The read bytes setting for Lua file operation.
  ReadBytes,
  /// The write text setting for Lua file operation.
  WriteText,
  /// The write bytes setting for Lua file operation.
  WriteBytes,
  /// The list dir setting for Lua file operation.
  ListDir,
  /// The create dir setting for Lua file operation.
  CreateDir,
  /// The remove setting for Lua file operation.
  Remove,
}

impl LuaFileOperation {
  /// Return the stable string key for this Lua file operation.
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

  /// Report whether this Lua file operation is write.
  pub fn is_write(self) -> bool {
    matches!(
      self,
      Self::WriteText | Self::WriteBytes | Self::ListDir | Self::CreateDir | Self::Remove
    )
  }
}

/// Failures reported by Lua event operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaEventErrorCode {
  /// The invalid request failure condition.
  InvalidRequest,
  /// The permission denied failure condition.
  PermissionDenied,
  /// The not found failure condition.
  NotFound,
  /// The too large failure condition.
  TooLarge,
  /// The invalid UTF-8 failure condition.
  InvalidUtf8,
  /// The cancelled failure condition.
  Cancelled,
  /// The timeout failure condition.
  Timeout,
  /// The io failure condition.
  Io,
  /// The network failure condition.
  Network,
  /// The unsupported failure condition.
  Unsupported,
  /// The decode failure condition.
  Decode,
  /// The backend unavailable failure condition.
  BackendUnavailable,
  /// The internal failure condition.
  Internal,
}

impl LuaEventErrorCode {
  /// Return the stable string key for this Lua event error code.
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

/// Failures reported by Lua event operations.
///
/// # Fields
///
/// * `code` - The stable error or language code.
/// * `message` - The diagnostic or display message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaEventError {
  /// The stable error or language code.
  pub code: LuaEventErrorCode,
  /// The diagnostic or display message.
  pub message: String,
}

impl LuaEventError {
  /// Create a stable public error message from its code without exposing internal diagnostic
  /// detail.
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

/// The Lua file outcome representation used by this module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaFileOutcome {
  /// The text setting for Lua file outcome.
  Text(String),
  /// The bytes setting for Lua file outcome.
  Bytes(Vec<u8>),
  /// The written setting for Lua file outcome.
  Written,
  /// The directory created setting for Lua file outcome.
  DirectoryCreated,
  /// The removed setting for Lua file outcome.
  Removed,
  /// The entries setting for Lua file outcome.
  Entries(Vec<LuaFileEntry>),
  /// The failed setting for Lua file outcome.
  Failed(LuaEventError),
}

/// The Lua file entry representation used by this module.
///
/// # Fields
///
/// * `path` - The filesystem path to read, write, or resolve.
/// * `file_type` - The file type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaFileEntry {
  /// The filesystem path to read, write, or resolve.
  pub path: String,
  /// The file type.
  pub file_type: String,
}

/// A Lua file event payload queued for its owning consumer.
///
/// # Fields
///
/// * `request_id` - The identifier of the request.
/// * `kind` - The Lua file operation carried by this Lua file event.
/// * `path` - The filesystem path to read, write, or resolve.
/// * `tip` - The tip.
/// * `outcome` - The outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaFileEvent {
  /// The identifier of the request.
  pub request_id: u64,
  /// The Lua file operation carried by this Lua file event.
  pub kind: LuaFileOperation,
  /// The filesystem path to read, write, or resolve.
  pub path: String,
  /// The tip.
  pub tip: Option<String>,
  /// The outcome.
  pub outcome: LuaFileOutcome,
}

/// The Lua i18n event kind representation used by this module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaI18nEventKind {
  /// A created notification delivered to the owning consumer.
  Created,
  /// A reloaded notification delivered to the owning consumer.
  Reloaded,
}

impl LuaI18nEventKind {
  /// Return the stable string key for this Lua i18n event kind.
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Created => "created",
      Self::Reloaded => "reloaded",
    }
  }
}

/// A Lua i18n event payload queued for its owning consumer.
///
/// # Fields
///
/// * `request_id` - The identifier of the request.
/// * `kind` - The Lua i18n event kind carried by this Lua i18n event.
/// * `ok` - The ok.
/// * `message` - The diagnostic or display message.
/// * `language_code` - The registered language code.
/// * `callback_language_code` - The callback language code.
/// * `warning` - Missing-language warnings accompanying successful loading.
/// * `namespaces` - The namespaces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaI18nEvent {
  /// The identifier of the request.
  pub request_id: u64,
  /// The Lua i18n event kind carried by this Lua i18n event.
  pub kind: LuaI18nEventKind,
  /// The ok.
  pub ok: bool,
  /// The diagnostic or display message.
  pub message: String,
  /// Missing primary or fallback resources, or `None` when no warning was reported.
  pub warning: Option<String>,
  /// The registered language code.
  pub language_code: String,
  /// The callback language code.
  pub callback_language_code: String,
  /// The namespaces.
  pub(crate) namespaces:
    Option<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
}

/// The Lua image outcome representation used by this module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaImageOutcome {
  /// The converted setting for Lua image outcome.
  Converted(String),
  /// The failed setting for Lua image outcome.
  Failed(LuaEventError),
}

/// A Lua image event payload queued for its owning consumer.
///
/// # Fields
///
/// * `request_id` - The identifier of the request.
/// * `outcome` - The outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaImageEvent {
  /// The identifier of the request.
  pub request_id: u64,
  /// The outcome.
  pub outcome: LuaImageOutcome,
}

/// The Lua network outcome representation used by this module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaNetworkOutcome {
  /// The response setting for Lua network outcome.
  Response {
    /// The final url.
    final_url: String,
    /// The status.
    status: u16,
    /// The headers.
    headers: std::collections::BTreeMap<String, String>,
    /// The body.
    body: LuaNetworkBody,
  },
  /// The failed setting for Lua network outcome.
  Failed(LuaEventError),
}

/// The Lua network body representation used by this module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaNetworkBody {
  /// The text setting for Lua network body.
  Text(String),
  /// The bytes setting for Lua network body.
  Bytes(Vec<u8>),
}

/// A Lua network event payload queued for its owning consumer.
///
/// # Fields
///
/// * `request_id` - The identifier of the request.
/// * `method` - The method.
/// * `url` - The url.
/// * `outcome` - The outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaNetworkEvent {
  /// The identifier of the request.
  pub request_id: u64,
  /// The method.
  pub method: NetworkMethod,
  /// The url.
  pub url: String,
  /// The outcome.
  pub outcome: LuaNetworkOutcome,
}

/// A Lua hit area event payload queued for its owning consumer.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `kind` - The &'static str carried by this Lua hit area event.
/// * `x` - The horizontal coordinate in terminal cells.
/// * `y` - The vertical coordinate in terminal cells.
/// * `button` - The mouse button to query.
/// * `dx` - The dx.
/// * `dy` - The dy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaHitAreaEvent {
  /// The identifier of the owned object.
  pub id: u64,
  /// The &'static str carried by this Lua hit area event.
  pub kind: &'static str,
  /// The horizontal coordinate in terminal cells.
  pub x: u16,
  /// The vertical coordinate in terminal cells.
  pub y: u16,
  /// The mouse button to query.
  pub button: Option<&'static str>,
  /// The dx.
  pub dx: Option<i32>,
  /// The dy.
  pub dy: Option<i32>,
}

/// A Lua hyperlink event payload queued for its owning consumer.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `link` - The link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaHyperlinkEvent {
  /// The identifier of the owned object.
  pub id: u64,
  /// The link.
  pub link: String,
}

/// A Lua markdown event payload queued for its owning consumer.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `href` - The href.
/// * `text` - The text to process or display.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaMarkdownEvent {
  /// The identifier of the owned object.
  pub id: u64,
  /// The href.
  pub href: String,
  /// The text to process or display.
  pub text: String,
}

/// A Lua text input event payload queued for its owning consumer.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `kind` - The &'static str carried by this Lua text input event.
/// * `value` - The value to store or convert.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaTextInputEvent {
  /// The identifier of the owned object.
  pub id: u64,
  /// The &'static str carried by this Lua text input event.
  pub kind: &'static str,
  /// The value to store or convert.
  pub value: Option<String>,
}

/// A Lua scroll box event payload queued for its owning consumer.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `x` - The horizontal coordinate in terminal cells.
/// * `y` - The vertical coordinate in terminal cells.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaScrollBoxEvent {
  /// The identifier of the owned object.
  pub id: u64,
  /// The horizontal coordinate in terminal cells.
  pub x: u16,
  /// The vertical coordinate in terminal cells.
  pub y: u16,
}

/// The Lua audio event kind representation used by this module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaAudioEventKind {
  /// A ready notification delivered to the owning consumer.
  Ready,
  /// A started notification delivered to the owning consumer.
  Started,
  /// The operation is paused.
  Paused,
  /// A resumed notification delivered to the owning consumer.
  Resumed,
  /// The operation is stopped.
  Stopped,
  /// The operation is finished.
  Finished,
  /// A failed notification delivered to the owning consumer.
  Failed,
}

impl LuaAudioEventKind {
  /// Return the stable string key for this Lua audio event kind.
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

/// A Lua audio event payload queued for its owning consumer.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `kind` - The Lua audio event kind carried by this Lua audio event.
/// * `duration_ms` - The duration measured in milliseconds.
/// * `position_ms` - The position measured in milliseconds.
/// * `error` - The error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaAudioEvent {
  /// The identifier of the owned object.
  pub id: u64,
  /// The Lua audio event kind carried by this Lua audio event.
  pub kind: LuaAudioEventKind,
  /// The duration measured in milliseconds.
  pub duration_ms: Option<u64>,
  /// The position measured in milliseconds.
  pub position_ms: Option<u64>,
  /// The error.
  pub error: Option<LuaEventError>,
}

/// The Lua event data representation used by this module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuaEventData {
  /// The action setting for Lua event data.
  Action {
    /// The action.
    action: String,
    /// The Lua action state carried by this Lua event data.
    state: LuaActionState,
  },
  /// A physical keyboard state transition using the canonical key token.
  Key {
    /// The portable key name, including left/right modifier distinctions.
    key: String,
    /// The transition into pressed, held, or released.
    state: LuaActionState,
  },
  /// The mouse setting for Lua event data.
  Mouse {
    /// The &'static str carried by this Lua event data.
    kind: &'static str,
    /// The mouse button to query.
    button: Option<&'static str>,
    /// The scroll.
    scroll: Option<&'static str>,
    /// The horizontal coordinate in terminal cells.
    x: u16,
    /// The vertical coordinate in terminal cells.
    y: u16,
  },
  /// The resize setting for Lua event data.
  Resize {
    /// The width in terminal columns.
    width: u16,
    /// The height in terminal rows.
    height: u16,
  },
  /// The focus setting for Lua event data.
  Focus {
    /// Whether terminal focus was gained rather than lost.
    gained: bool,
  },
  /// The overlay started setting for Lua event data.
  OverlayStarted,
  /// The overlay stopped setting for Lua event data.
  OverlayStopped,
  /// The timer setting for Lua event data.
  Timer(LuaTimerEvent),
  /// The animation setting for Lua event data.
  Animation(LuaAnimationEvent),
  /// The file setting for Lua event data.
  File(LuaFileEvent),
  /// The i18n setting for Lua event data.
  I18n(LuaI18nEvent),
  /// The image setting for Lua event data.
  Image(LuaImageEvent),
  /// The network setting for Lua event data.
  Network(LuaNetworkEvent),
  /// The audio setting for Lua event data.
  Audio(LuaAudioEvent),
  /// The hit area setting for Lua event data.
  HitArea(LuaHitAreaEvent),
  /// The hyperlink setting for Lua event data.
  Hyperlink(LuaHyperlinkEvent),
  /// The markdown setting for Lua event data.
  Markdown(LuaMarkdownEvent),
  /// The text input setting for Lua event data.
  TextInput(LuaTextInputEvent),
  /// The scroll box setting for Lua event data.
  ScrollBox(LuaScrollBoxEvent),
}

impl LuaEventData {
  /// Return the current event type.
  pub fn event_type(&self) -> &'static str {
    match self {
      Self::Action { .. } => "action",
      Self::Key { .. } => "key",
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

  /// Convert a terminal pointer event into a script-visible mouse event payload.
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

  /// Report whether this event completes its one-shot operation or terminates its timer/animation callback.
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
        | Self::Key { .. }
        | Self::Mouse { .. }
        | Self::HitArea(_)
        | Self::Hyperlink(_)
        | Self::Markdown(_)
        | Self::TextInput(_)
        | Self::ScrollBox(_)
    )
  }

  /// Report whether the addressed object is coalescible with.
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

  /// Report whether this event category may be delivered to the specified session kind.
  pub(super) fn allowed_for(&self, kind: super::LuaSessionKind) -> bool {
    match kind {
      super::LuaSessionKind::Game => true,
      super::LuaSessionKind::Screensaver => match self {
        Self::Action { .. }
        | Self::Key { .. }
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

  /// Build the script-visible event table without exposing host task identifiers or session
  /// generations.
  ///
  /// # Errors
  ///
  /// Propagate Lua allocation or field-conversion errors while constructing the event payload.
  pub(super) fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
    let data = lua.create_table()?;
    match self {
      Self::Action { action, state } => {
        data.set("action", action.as_str())?;
        data.set("state", state.as_str())?;
      }
      Self::Key { key, state } => {
        data.set("key", key.as_str())?;
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
        if let Some(id) = &event.object_id {
          data.set("id", id.as_str())?;
        } else {
          data.set("id", event.id.to_string())?;
        }
        data.set("timer_kind", event.timer_kind.as_str())?;
        data.set("kind", event.kind.as_str())?;
        data.set("executed_count", event.executed_count)?;
        data.set("tip", event.tip.as_deref())?;
      }
      Self::Animation(event) => {
        data.set("id", event.id.to_string())?;
        data.set("kind", event.kind.as_str())?;
        match &event.kind {
          LuaAnimationEventKind::Marker { name } => data.set("name", name.as_str())?,
          LuaAnimationEventKind::Loop { completed } => data.set("completed", *completed)?,
          _ => {}
        }
      }
      Self::File(event) => {
        data.set("request_id", event.request_id.to_string())?;
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
        data.set("request_id", event.request_id.to_string())?;
        data.set("kind", event.kind.as_str())?;
        data.set("ok", event.ok)?;
        data.set("message", event.message.as_str())?;
        data.set("warning", event.warning.as_deref())?;
        data.set("language_code", event.language_code.as_str())?;
        data.set(
          "callback_language_code",
          event.callback_language_code.as_str(),
        )?;
      }
      Self::Image(event) => {
        data.set("request_id", event.request_id.to_string())?;
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
        data.set("request_id", event.request_id.to_string())?;
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
        data.set("id", event.id.to_string())?;
        data.set("kind", event.kind.as_str())?;
        data.set("duration_ms", event.duration_ms)?;
        data.set("position_ms", event.position_ms)?;
        if let Some(error) = &event.error {
          data.set("error", error_table(lua, error)?)?;
        }
      }
      Self::HitArea(event) => {
        data.set("id", event.id.to_string())?;
        data.set("kind", event.kind)?;
        data.set("x", event.x)?;
        data.set("y", event.y)?;
        data.set("button", event.button)?;
        data.set("dx", event.dx)?;
        data.set("dy", event.dy)?;
      }
      Self::Hyperlink(event) => {
        data.set("id", event.id.to_string())?;
        data.set("kind", "clicked")?;
        data.set("link", event.link.as_str())?;
      }
      Self::Markdown(event) => {
        data.set("id", event.id.to_string())?;
        data.set("kind", "link_clicked")?;
        data.set("href", event.href.as_str())?;
        data.set("text", event.text.as_str())?;
      }
      Self::TextInput(event) => {
        data.set("id", event.id.to_string())?;
        data.set("kind", event.kind)?;
        data.set("value", event.value.as_deref())?;
      }
      Self::ScrollBox(event) => {
        data.set("id", event.id.to_string())?;
        data.set("kind", "scrolled")?;
        data.set("x", event.x)?;
        data.set("y", event.y)?;
      }
    }
    Ok(data)
  }
}

/// A Lua runtime event payload queued for its owning consumer.
///
/// # Fields
///
/// * `sequence` - The sequence.
/// * `frame` - The composed terminal-cell frame.
/// * `data` - The data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaRuntimeEvent {
  /// The sequence.
  pub sequence: u64,
  /// The composed terminal-cell frame.
  pub frame: u64,
  /// The data.
  pub data: LuaEventData,
}

fn error_table(lua: &Lua, error: &LuaEventError) -> mlua::Result<Table> {
  let table = lua.create_table()?;
  table.set("code", error.code.as_str())?;
  table.set("message", error.message.as_str())?;
  Ok(table)
}

/// Map filesystem failures to script-visible errors without revealing host paths.
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

/// Map network failures to script-visible errors without leaking destination details.
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
          object_id: None,
          tip: None,
          revision: None,
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
      let data = event.to_lua_table(&lua).unwrap();
      for field in ["id", "request_id"] {
        let value = data.get::<Value>(field).unwrap();
        assert!(
          matches!(value, Value::Nil | Value::String(_)),
          "{expected_type}.{field}"
        );
      }
    }
  }

  #[test]
  fn request_ids_keep_the_full_unsigned_range_as_strings() {
    let lua = Lua::new();
    for request_id in [1, i64::MAX as u64 + 1, u64::MAX] {
      let error = LuaEventError::sanitized(LuaEventErrorCode::Io);
      let cases = [
        LuaEventData::File(LuaFileEvent {
          request_id,
          kind: LuaFileOperation::ReadText,
          path: "missing.txt".into(),
          tip: None,
          outcome: LuaFileOutcome::Failed(error.clone()),
        }),
        LuaEventData::Image(LuaImageEvent {
          request_id,
          outcome: LuaImageOutcome::Failed(error.clone()),
        }),
        LuaEventData::I18n(LuaI18nEvent {
          request_id,
          kind: LuaI18nEventKind::Created,
          ok: false,
          message: "load failed".into(),
          language_code: "zh_cn".into(),
          callback_language_code: "en_us".into(),
          warning: None,
          namespaces: None,
        }),
        LuaEventData::Network(LuaNetworkEvent {
          request_id,
          method: NetworkMethod::Get,
          url: "https://example.invalid".into(),
          outcome: LuaNetworkOutcome::Failed(error),
        }),
      ];
      for event in cases {
        let data = event.to_lua_table(&lua).unwrap();
        let Value::String(id) = data.get::<Value>("request_id").unwrap() else {
          panic!("string request ID")
        };
        assert_eq!(id.to_str().unwrap().as_ref(), request_id.to_string());
      }
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
      object_id: None,
      tip: None,
      revision: None,
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
