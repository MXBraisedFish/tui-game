use std::{fmt, str::FromStr, sync::Arc, time::Duration};

use crate::host_engine::services::TextColor;
use unicode_width::UnicodeWidthStr;

const MAX_CHARACTER_FRAMES: usize = 4_096;
const MAX_CHARACTER_FRAME_WIDTH: usize = 512;
const MAX_CHARACTER_FRAME_HEIGHT: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimationId {
  index: u32,
  generation: u32,
}

impl AnimationId {
  pub(crate) fn new(index: u32, generation: u32) -> Self {
    Self { index, generation }
  }

  pub fn index(self) -> u32 {
    self.index
  }

  pub fn generation(self) -> u32 {
    self.generation
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimationHandle {
  id: AnimationId,
}

impl AnimationHandle {
  pub(crate) fn new(id: AnimationId) -> Self {
    Self { id }
  }

  pub fn id(self) -> AnimationId {
    self.id
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimationValueId {
  index: u32,
  generation: u32,
}

impl AnimationValueId {
  pub(crate) fn new(index: u32, generation: u32) -> Self {
    Self { index, generation }
  }

  pub fn index(self) -> u32 {
    self.index
  }

  pub fn generation(self) -> u32 {
    self.generation
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellEffectId {
  index: u32,
  generation: u32,
}

impl CellEffectId {
  pub(crate) fn new(index: u32, generation: u32) -> Self {
    Self { index, generation }
  }

  pub fn index(self) -> u32 {
    self.index
  }

  pub fn generation(self) -> u32 {
    self.generation
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EffectParameterId(pub u32);

impl EffectParameterId {
  pub const PHASE: Self = Self(0);
  pub const STRENGTH: Self = Self(1);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UiPoolId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameInstanceId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameObjectRef(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UiObjectKind {
  Slice,
  ScrollBox,
  ProgressBar,
  TextInput,
  Markdown,
  Table,
  Hyperlink,
  Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UiObjectRef {
  pub pool: UiPoolId,
  pub kind: UiObjectKind,
  pub id: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimationTarget {
  Ui(UiObjectRef),
  Game(GameObjectRef),
  Effect(CellEffectId),
  Value(AnimationValueId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimationOwner {
  Host,
  UiPool(UiPoolId),
  Game(GameInstanceId),
  Object(AnimationTarget),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimationProperty {
  OffsetX,
  OffsetY,
  WidthOffset,
  HeightOffset,
  Foreground,
  Background,
  BorderForeground,
  Visible,
  VisibleGraphemes,
  VisibleLines,
  GlyphFrame,
  Progress,
  ScrollX,
  ScrollY,
  EffectPhase,
  EffectStrength,
  EffectParameter(EffectParameterId),
  Value,
}

impl FromStr for AnimationProperty {
  type Err = AnimationError;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    match value {
      "offset_x" | "offset.x" => Ok(Self::OffsetX),
      "offset_y" | "offset.y" => Ok(Self::OffsetY),
      "width_offset" | "size.width_offset" => Ok(Self::WidthOffset),
      "height_offset" | "size.height_offset" => Ok(Self::HeightOffset),
      "foreground" | "style.foreground" => Ok(Self::Foreground),
      "background" | "style.background" => Ok(Self::Background),
      "border_foreground" | "style.border_foreground" => Ok(Self::BorderForeground),
      "visible" => Ok(Self::Visible),
      "visible_graphemes" | "text.visible_graphemes" => Ok(Self::VisibleGraphemes),
      "visible_lines" | "text.visible_lines" => Ok(Self::VisibleLines),
      "glyph_frame" | "text.glyph_frame" => Ok(Self::GlyphFrame),
      "progress" => Ok(Self::Progress),
      "scroll_x" | "scroll.x" => Ok(Self::ScrollX),
      "scroll_y" | "scroll.y" => Ok(Self::ScrollY),
      "effect_phase" | "effect.phase" => Ok(Self::EffectPhase),
      "effect_strength" | "effect.strength" => Ok(Self::EffectStrength),
      "value" => Ok(Self::Value),
      _ => Err(AnimationError::UnsupportedProperty(value.to_string())),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationValueKind {
  Float,
  Integer,
  Unsigned,
  Bool,
  Color,
  Text,
  CharacterFrame,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationColor {
  pub r: u8,
  pub g: u8,
  pub b: u8,
}

impl AnimationColor {
  pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
    Self { r, g, b }
  }
}

impl TryFrom<&TextColor> for AnimationColor {
  type Error = AnimationError;

  fn try_from(value: &TextColor) -> Result<Self, Self::Error> {
    match value {
      TextColor::Rgb { r, g, b } | TextColor::ForceRgb { r, g, b } => Ok(Self::rgb(*r, *g, *b)),
      _ => Err(AnimationError::RgbColorRequired),
    }
  }
}

impl From<AnimationColor> for TextColor {
  fn from(value: AnimationColor) -> Self {
    Self::Rgb {
      r: value.r,
      g: value.g,
      b: value.b,
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationValue {
  Float(f64),
  Integer(i64),
  Unsigned(u64),
  Bool(bool),
  Color(AnimationColor),
  Text(String),
  CharacterFrame(Arc<[String]>),
}

impl AnimationValue {
  pub fn kind(&self) -> AnimationValueKind {
    match self {
      Self::Float(_) => AnimationValueKind::Float,
      Self::Integer(_) => AnimationValueKind::Integer,
      Self::Unsigned(_) => AnimationValueKind::Unsigned,
      Self::Bool(_) => AnimationValueKind::Bool,
      Self::Color(_) => AnimationValueKind::Color,
      Self::Text(_) => AnimationValueKind::Text,
      Self::CharacterFrame(_) => AnimationValueKind::CharacterFrame,
    }
  }

  pub(crate) fn interpolate(
    &self,
    other: &Self,
    progress: f64,
    interpolation: AnimationInterpolation,
  ) -> Result<Self, AnimationError> {
    if self.kind() != other.kind() {
      return Err(AnimationError::ValueTypeMismatch {
        expected: self.kind(),
        actual: other.kind(),
      });
    }
    if interpolation == AnimationInterpolation::Step {
      return Ok(if progress.clamp(0.0, 1.0) < 1.0 {
        self.clone()
      } else {
        other.clone()
      });
    }
    Ok(match (self, other) {
      (Self::Float(from), Self::Float(to)) => Self::Float(from + (to - from) * progress),
      (Self::Integer(from), Self::Integer(to)) => {
        Self::Integer((*from as f64 + (*to as f64 - *from as f64) * progress).round() as i64)
      }
      (Self::Unsigned(from), Self::Unsigned(to)) => Self::Unsigned(
        (*from as f64 + (*to as f64 - *from as f64) * progress)
          .round()
          .max(0.0) as u64,
      ),
      (Self::Color(from), Self::Color(to)) => Self::Color(AnimationColor {
        r: lerp_channel(from.r, to.r, progress),
        g: lerp_channel(from.g, to.g, progress),
        b: lerp_channel(from.b, to.b, progress),
      }),
      (Self::Bool(_), Self::Bool(_))
      | (Self::Text(_), Self::Text(_))
      | (Self::CharacterFrame(_), Self::CharacterFrame(_)) => {
        return Err(AnimationError::DiscreteInterpolationRequired(self.kind()));
      }
      _ => unreachable!("value kinds were checked above"),
    })
  }
}

fn lerp_channel(from: u8, to: u8, progress: f64) -> u8 {
  (from as f64 + (to as f64 - from as f64) * progress)
    .round()
    .clamp(0.0, 255.0) as u8
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationInterpolation {
  Linear,
  Step,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationEasing {
  Linear,
  InQuad,
  OutQuad,
  InOutQuad,
  InCubic,
  OutCubic,
  InOutCubic,
  InSine,
  OutSine,
  InOutSine,
  InBack,
  OutBack,
  InOutBack,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TweenDefinition {
  pub from: AnimationValue,
  pub to: AnimationValue,
  pub duration: Duration,
  pub easing: AnimationEasing,
  pub interpolation: AnimationInterpolation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationKeyframe {
  pub at: Duration,
  pub value: AnimationValue,
  pub easing: AnimationEasing,
  pub interpolation: AnimationInterpolation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationTrack {
  pub property: AnimationProperty,
  pub keyframes: Vec<AnimationKeyframe>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationMarker {
  pub at: Duration,
  pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationClip {
  pub duration: Duration,
  pub tracks: Vec<AnimationTrack>,
  pub markers: Vec<AnimationMarker>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterFrame {
  pub duration: Duration,
  pub lines: Vec<String>,
}

impl CharacterFrame {
  pub fn from_text(duration: Duration, text: impl AsRef<str>) -> Self {
    Self {
      duration,
      lines: normalized_text_lines(text.as_ref()),
    }
  }

  pub fn display_size(&self) -> (usize, usize) {
    (
      self
        .lines
        .iter()
        .map(|line| UnicodeWidthStr::width(line.as_str()))
        .max()
        .unwrap_or(0),
      self.lines.len(),
    )
  }

  fn normalize(mut self) -> Result<Self, AnimationError> {
    if self.duration.is_zero() {
      return Err(AnimationError::InvalidCharacterFrames);
    }
    self.lines = normalized_text_lines(&self.lines.join("\n"));
    let (width, height) = self.display_size();
    if width > MAX_CHARACTER_FRAME_WIDTH || height > MAX_CHARACTER_FRAME_HEIGHT {
      return Err(AnimationError::CharacterFrameTooLarge { width, height });
    }
    Ok(self)
  }
}

fn normalized_text_lines(text: &str) -> Vec<String> {
  text
    .replace("\r\n", "\n")
    .replace('\r', "\n")
    .split('\n')
    .map(str::to_string)
    .collect()
}

impl AnimationClip {
  pub fn character_frames(frames: Vec<CharacterFrame>) -> Result<Self, AnimationError> {
    if frames.is_empty() {
      return Err(AnimationError::InvalidCharacterFrames);
    }
    if frames.len() > MAX_CHARACTER_FRAMES {
      return Err(AnimationError::TooManyCharacterFrames(frames.len()));
    }
    let mut at = Duration::ZERO;
    let mut keyframes = Vec::with_capacity(frames.len());
    for frame in frames {
      let frame = frame.normalize()?;
      keyframes.push(AnimationKeyframe {
        at,
        value: AnimationValue::CharacterFrame(frame.lines.into()),
        easing: AnimationEasing::Linear,
        interpolation: AnimationInterpolation::Step,
      });
      at = at.saturating_add(frame.duration);
    }
    Ok(Self {
      duration: at,
      tracks: vec![AnimationTrack {
        property: AnimationProperty::GlyphFrame,
        keyframes,
      }],
      markers: Vec::new(),
    })
  }

  pub fn character_frames_from_text(
    frames: Vec<(Duration, String)>,
  ) -> Result<Self, AnimationError> {
    Self::character_frames(
      frames
        .into_iter()
        .map(|(duration, text)| CharacterFrame::from_text(duration, text))
        .collect(),
    )
  }
}

#[derive(Clone, Debug)]
pub enum AnimationSource {
  Tween(Arc<TweenDefinition>),
  Clip(Arc<AnimationClip>),
}

impl AnimationSource {
  pub fn duration(&self) -> Duration {
    match self {
      Self::Tween(definition) => definition.duration,
      Self::Clip(clip) => clip.duration,
    }
  }

  pub fn track_count(&self) -> usize {
    match self {
      Self::Tween(_) => 1,
      Self::Clip(clip) => clip.tracks.len(),
    }
  }

  pub fn track_property(&self, track: usize) -> Option<AnimationProperty> {
    match self {
      Self::Tween(_) => None,
      Self::Clip(clip) => Some(clip.tracks.get(track)?.property),
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationBinding {
  pub track: usize,
  pub target: AnimationTarget,
  pub property: AnimationProperty,
  pub initial_value: AnimationValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationClock {
  Ui,
  Game,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationRepeatMode {
  Restart,
  PingPong,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationRepeatCount {
  Finite(u32),
  Infinite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationRepeatOptions {
  pub mode: AnimationRepeatMode,
  /// 总播放轮数；`Finite(1)` 表示只播放一次。
  pub count: AnimationRepeatCount,
}

impl Default for AnimationRepeatOptions {
  fn default() -> Self {
    Self {
      mode: AnimationRepeatMode::Restart,
      count: AnimationRepeatCount::Finite(1),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationEndMode {
  Commit,
  Restore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackState {
  Idle,
  Playing,
  Paused,
  Finished,
  Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackDirection {
  Forward,
  Reverse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimationCallbackId(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationPlaybackOptions {
  pub delay: Duration,
  pub speed: f64,
  pub clock: AnimationClock,
  pub repeat: AnimationRepeatOptions,
  pub end_mode: AnimationEndMode,
  pub auto_play: bool,
  pub emit_events: bool,
  pub callback: Option<AnimationCallbackId>,
}

impl Default for AnimationPlaybackOptions {
  fn default() -> Self {
    Self {
      delay: Duration::ZERO,
      speed: 1.0,
      clock: AnimationClock::Ui,
      repeat: AnimationRepeatOptions::default(),
      end_mode: AnimationEndMode::Commit,
      auto_play: true,
      emit_events: false,
      callback: None,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationWriteOperation {
  Override,
  Commit,
  ClearOverride,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationWrite {
  pub animation_id: AnimationId,
  pub target: AnimationTarget,
  pub property: AnimationProperty,
  pub value: Option<AnimationValue>,
  pub operation: AnimationWriteOperation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnimationEventKind {
  Started,
  Marker { name: String },
  Loop { completed: u32 },
  Finished,
  Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationEvent {
  pub id: AnimationId,
  pub kind: AnimationEventKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationCallbackRequest {
  pub callback: AnimationCallbackId,
  pub event: AnimationEvent,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationUpdate {
  pub writes: Vec<AnimationWrite>,
  pub events: Vec<AnimationEvent>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationError {
  InvalidDuration,
  InvalidSpeed,
  InvalidRepeatCount,
  InvalidTrack(usize),
  InvalidKeyframes(usize),
  InvalidBinding(usize),
  InvalidCharacterFrames,
  TooManyCharacterFrames(usize),
  CharacterFrameTooLarge {
    width: usize,
    height: usize,
  },
  InvalidPlaybackState {
    expected: PlaybackState,
    actual: PlaybackState,
  },
  StaleAnimation,
  StaleValue,
  StaleEffect,
  TargetNotFound(AnimationTarget),
  MissingEffectParameter(EffectParameterId),
  UnsupportedProperty(String),
  PropertyNotSupported {
    target: AnimationTarget,
    property: AnimationProperty,
  },
  ValueTypeMismatch {
    expected: AnimationValueKind,
    actual: AnimationValueKind,
  },
  PropertyTypeMismatch {
    property: AnimationProperty,
    actual: AnimationValueKind,
  },
  DiscreteInterpolationRequired(AnimationValueKind),
  RgbColorRequired,
}

impl fmt::Display for AnimationError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(formatter, "{self:?}")
  }
}

impl std::error::Error for AnimationError {}
