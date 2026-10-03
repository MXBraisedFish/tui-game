//! Identifiers, configuration, states, and events shared by this module.

use std::{fmt, str::FromStr, sync::Arc, time::Duration};

use tg_core_style::TextColor;
use unicode_width::UnicodeWidthStr;

const MAX_CHARACTER_FRAMES: usize = 4_096;
const MAX_CHARACTER_FRAME_WIDTH: usize = 512;
const MAX_CHARACTER_FRAME_HEIGHT: usize = 512;

/// The identity of animation within its owning pool or session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimationId {
  index: u32,
  generation: u32,
}

impl AnimationId {
  /// Create an animation id initialized from `index`, `generation`.
  pub fn new(index: u32, generation: u32) -> Self {
    Self { index, generation }
  }

  /// Return the current index.
  pub fn index(self) -> u32 {
    self.index
  }

  /// Return the current generation.
  pub fn generation(self) -> u32 {
    self.generation
  }
}

/// A handle addressing one live animation instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimationHandle {
  id: AnimationId,
}

impl AnimationHandle {
  /// Create an animation handle initialized from `id`.
  pub(crate) fn new(id: AnimationId) -> Self {
    Self { id }
  }

  /// Return the current id.
  pub fn id(self) -> AnimationId {
    self.id
  }
}

/// The identity of animation value within its owning pool or session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimationValueId {
  index: u32,
  generation: u32,
}

impl AnimationValueId {
  /// Create an animation value id initialized from `index`, `generation`.
  pub(crate) fn new(index: u32, generation: u32) -> Self {
    Self { index, generation }
  }

  /// Return the current index.
  pub fn index(self) -> u32 {
    self.index
  }

  /// Return the current generation.
  pub fn generation(self) -> u32 {
    self.generation
  }
}

/// The identity of cell effect within its owning pool or session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellEffectId {
  index: u32,
  generation: u32,
}

impl CellEffectId {
  /// Create a cell effect id initialized from `index`, `generation`.
  pub(crate) fn new(index: u32, generation: u32) -> Self {
    Self { index, generation }
  }

  /// Return the current index.
  pub fn index(self) -> u32 {
    self.index
  }

  /// Return the current generation.
  pub fn generation(self) -> u32 {
    self.generation
  }
}

/// The identity of effect parameter within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u32 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EffectParameterId(
  /// The wrapped u32 value.
  pub u32,
);

impl EffectParameterId {
  /// The phase used by this module.
  pub const PHASE: Self = Self(0);
  /// The strength used by this module.
  pub const STRENGTH: Self = Self(1);
}

/// The identity of UI pool within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UiPoolId(
  /// The wrapped u64 value.
  pub u64,
);

/// The identity of game instance within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameInstanceId(
  /// The wrapped u64 value.
  pub u64,
);

/// The host-defined identity of an animatable game object.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameObjectRef(
  /// The wrapped u64 value.
  pub u64,
);

/// The UI component category used to route an animation target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UiObjectKind {
  /// A slice UI component addressed through its owning pool.
  Slice,
  /// A scroll box UI component addressed through its owning pool.
  ScrollBox,
  /// A progress bar UI component addressed through its owning pool.
  ProgressBar,
  /// A text input UI component addressed through its owning pool.
  TextInput,
  /// A markdown UI component addressed through its owning pool.
  Markdown,
  /// A table UI component addressed through its owning pool.
  Table,
  /// A hyperlink UI component addressed through its owning pool.
  Hyperlink,
  /// A other UI component addressed through its owning pool.
  Other,
}

/// A pool, component kind, and identifier locating an animatable UI object.
///
/// # Fields
///
/// * `pool` - The object pool that owns the component.
/// * `kind` - The UI object kind carried by this UI object ref.
/// * `id` - The identifier of the owned object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UiObjectRef {
  /// The object pool that owns the component.
  pub pool: UiPoolId,
  /// The UI object kind carried by this UI object ref.
  pub kind: UiObjectKind,
  /// The identifier of the owned object.
  pub id: u64,
}

/// A UI object, game object, effect, or owned value receiving animation properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimationTarget {
  /// A ui target receiving animation property writes.
  Ui(UiObjectRef),
  /// A game target receiving animation property writes.
  Game(GameObjectRef),
  /// A effect target receiving animation property writes.
  Effect(CellEffectId),
  /// A value target receiving animation property writes.
  Value(AnimationValueId),
}

/// The lifecycle owner used to clean up related animations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimationOwner {
  /// The host setting for animation owner.
  Host,
  /// The UI pool setting for animation owner.
  UiPool(UiPoolId),
  /// The game setting for animation owner.
  Game(GameInstanceId),
  /// The object setting for animation owner.
  Object(AnimationTarget),
}

/// A supported animatable object property.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimationProperty {
  /// The offset x setting for animation property.
  OffsetX,
  /// The offset y setting for animation property.
  OffsetY,
  /// The width offset setting for animation property.
  WidthOffset,
  /// The height offset setting for animation property.
  HeightOffset,
  /// The foreground setting for animation property.
  Foreground,
  /// The background setting for animation property.
  Background,
  /// The border foreground setting for animation property.
  BorderForeground,
  /// The visible setting for animation property.
  Visible,
  /// The visible graphemes setting for animation property.
  VisibleGraphemes,
  /// The visible lines setting for animation property.
  VisibleLines,
  /// The glyph frame setting for animation property.
  GlyphFrame,
  /// The progress setting for animation property.
  Progress,
  /// The scroll x setting for animation property.
  ScrollX,
  /// The scroll y setting for animation property.
  ScrollY,
  /// The effect phase setting for animation property.
  EffectPhase,
  /// The effect strength setting for animation property.
  EffectStrength,
  /// The effect parameter setting for animation property.
  EffectParameter(EffectParameterId),
  /// The value setting for animation property.
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

/// The scalar, vector, color, or discrete kind of an animation value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationValueKind {
  /// The float setting for animation value kind.
  Float,
  /// The integer setting for animation value kind.
  Integer,
  /// The unsigned setting for animation value kind.
  Unsigned,
  /// The bool setting for animation value kind.
  Bool,
  /// The color setting for animation value kind.
  Color,
  /// The text setting for animation value kind.
  Text,
  /// The character frame setting for animation value kind.
  CharacterFrame,
}

/// The animation color representation used by this module.
///
/// # Fields
///
/// * `r` - The red color component from 0 to 255.
/// * `g` - The green color component from 0 to 255.
/// * `b` - The blue color component from 0 to 255.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationColor {
  /// The red color component from 0 to 255.
  pub r: u8,
  /// The green color component from 0 to 255.
  pub g: u8,
  /// The blue color component from 0 to 255.
  pub b: u8,
}

impl AnimationColor {
  /// Create an animation color from red, green, and blue components.
  ///
  /// # Arguments
  ///
  /// * `r` - The r.
  /// * `g` - The g.
  /// * `b` - The b.
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

/// A typed value sampled or written by an animation track.
#[derive(Clone, Debug, PartialEq)]
pub enum AnimationValue {
  /// The float setting for animation value.
  Float(f64),
  /// The integer setting for animation value.
  Integer(i64),
  /// The unsigned setting for animation value.
  Unsigned(u64),
  /// The bool setting for animation value.
  Bool(bool),
  /// The color setting for animation value.
  Color(AnimationColor),
  /// The text setting for animation value.
  Text(String),
  /// The character frame setting for animation value.
  CharacterFrame(Arc<[String]>),
}

impl AnimationValue {
  /// Return the current kind.
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

  /// Interpolate compatible animation values at the supplied normalized progress.
  ///
  /// # Arguments
  ///
  /// * `other` - The other.
  /// * `progress` - The progress.
  /// * `interpolation` - The interpolation.
  ///
  /// # Errors
  ///
  /// Return `ValueTypeMismatch` for incompatible value kinds or `DiscreteInterpolationRequired`
  /// when a discrete value is requested with continuous interpolation.
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

/// The rule used to sample a track between its keyframes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationInterpolation {
  /// The linear setting for animation interpolation.
  Linear,
  /// The step setting for animation interpolation.
  Step,
}

/// The timing curve mapping normalized animation progress to interpolation weight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationEasing {
  /// The linear setting for animation easing.
  Linear,
  /// The in quad setting for animation easing.
  InQuad,
  /// The out quad setting for animation easing.
  OutQuad,
  /// The in out quad setting for animation easing.
  InOutQuad,
  /// The in cubic setting for animation easing.
  InCubic,
  /// The out cubic setting for animation easing.
  OutCubic,
  /// The in out cubic setting for animation easing.
  InOutCubic,
  /// The in sine setting for animation easing.
  InSine,
  /// The out sine setting for animation easing.
  OutSine,
  /// The in out sine setting for animation easing.
  InOutSine,
  /// The in back setting for animation easing.
  InBack,
  /// The out back setting for animation easing.
  OutBack,
  /// The in out back setting for animation easing.
  InOutBack,
}

/// Start/end values, duration, and easing for a single tween.
///
/// # Fields
///
/// * `from` - The from.
/// * `to` - The to.
/// * `duration` - The duration represented as a duration.
/// * `easing` - The easing.
/// * `interpolation` - The interpolation.
#[derive(Clone, Debug, PartialEq)]
pub struct TweenDefinition {
  /// The from.
  pub from: AnimationValue,
  /// The to.
  pub to: AnimationValue,
  /// The duration represented as a duration.
  pub duration: Duration,
  /// The easing.
  pub easing: AnimationEasing,
  /// The interpolation.
  pub interpolation: AnimationInterpolation,
}

/// A timed value within an animation track.
///
/// # Fields
///
/// * `at` - The at.
/// * `value` - The value to store or convert.
/// * `easing` - The easing.
/// * `interpolation` - The interpolation.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationKeyframe {
  /// The at.
  pub at: Duration,
  /// The value to store or convert.
  pub value: AnimationValue,
  /// The easing.
  pub easing: AnimationEasing,
  /// The interpolation.
  pub interpolation: AnimationInterpolation,
}

/// Ordered keyframes for one animated property.
///
/// # Fields
///
/// * `property` - The property.
/// * `keyframes` - The ordered keyframes retained by this owner.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationTrack {
  /// The property.
  pub property: AnimationProperty,
  /// The ordered keyframes retained by this owner.
  pub keyframes: Vec<AnimationKeyframe>,
}

/// A named timestamp that can emit an animation event.
///
/// # Fields
///
/// * `at` - The at.
/// * `name` - The name used to identify the object or field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationMarker {
  /// The at.
  pub at: Duration,
  /// The name used to identify the object or field.
  pub name: String,
}

/// Named tracks, markers, and duration describing a reusable animation clip.
///
/// # Fields
///
/// * `duration` - The duration represented as a duration.
/// * `tracks` - The ordered tracks retained by this owner.
/// * `markers` - The ordered markers retained by this owner.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationClip {
  /// The duration represented as a duration.
  pub duration: Duration,
  /// The ordered tracks retained by this owner.
  pub tracks: Vec<AnimationTrack>,
  /// The ordered markers retained by this owner.
  pub markers: Vec<AnimationMarker>,
}

/// Text and terminal-cell dimensions for one character-animation frame.
///
/// # Fields
///
/// * `duration` - The duration represented as a duration.
/// * `lines` - The ordered lines retained by this owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterFrame {
  /// The duration represented as a duration.
  pub duration: Duration,
  /// The ordered lines retained by this owner.
  pub lines: Vec<String>,
}

impl CharacterFrame {
  /// Create a character frame from its text and measured terminal-cell dimensions.
  pub fn from_text(duration: Duration, text: impl AsRef<str>) -> Self {
    Self {
      duration,
      lines: normalized_text_lines(text.as_ref()),
    }
  }

  /// Return the maximum terminal-cell dimensions occupied by the animation frames.
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
  /// Validate and store the bounded sequence of character-animation frames.
  ///
  /// # Errors
  ///
  /// Return an animation error for an empty frame set or a frame count, width, or height
  /// exceeding the supported limits.
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

  /// Split textual frames into bounded character-animation data.
  ///
  /// # Errors
  ///
  /// Return an animation error when the parsed frame sequence is empty or exceeds the supported
  /// frame and cell limits.
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

/// A tween or clip used as the source of animation samples.
#[derive(Clone, Debug)]
pub enum AnimationSource {
  /// Content originating from tween.
  Tween(Arc<TweenDefinition>),
  /// Content originating from clip.
  Clip(Arc<AnimationClip>),
}

impl AnimationSource {
  /// Return the duration for the addressed object.
  pub fn duration(&self) -> Duration {
    match self {
      Self::Tween(definition) => definition.duration,
      Self::Clip(clip) => clip.duration,
    }
  }

  /// Return the current track count.
  pub fn track_count(&self) -> usize {
    match self {
      Self::Tween(_) => 1,
      Self::Clip(clip) => clip.tracks.len(),
    }
  }

  /// Return the property addressed by a source track when that track exists.
  pub fn track_property(&self, track: usize) -> Option<AnimationProperty> {
    match self {
      Self::Tween(_) => None,
      Self::Clip(clip) => Some(clip.tracks.get(track)?.property),
    }
  }
}

/// A source track bound to a target property and its initial value.
///
/// # Fields
///
/// * `track` - The track.
/// * `target` - The object or resource affected by the operation.
/// * `property` - The property.
/// * `initial_value` - The initial value.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationBinding {
  /// The track.
  pub track: usize,
  /// The object or resource affected by the operation.
  pub target: AnimationTarget,
  /// The property.
  pub property: AnimationProperty,
  /// The initial value.
  pub initial_value: AnimationValue,
}

/// The update clock selected by an animation playback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationClock {
  /// The ui setting for animation clock.
  Ui,
  /// The game setting for animation clock.
  Game,
}

/// The animation repeat mode representation used by this module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationRepeatMode {
  /// The restart setting for animation repeat mode.
  Restart,
  /// The ping pong setting for animation repeat mode.
  PingPong,
}

/// The animation repeat count representation used by this module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationRepeatCount {
  /// The finite setting for animation repeat count.
  Finite(u32),
  /// The infinite setting for animation repeat count.
  Infinite,
}

/// Configuration values controlling animation repeat behavior.
///
/// # Fields
///
/// * `mode` - The animation repeat mode carried by this animation repeat options.
/// * `count` - The count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationRepeatOptions {
  /// The animation repeat mode carried by this animation repeat options.
  pub mode: AnimationRepeatMode,

  /// The count.
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

/// The policy controlling the property value retained after playback ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationEndMode {
  /// The commit setting for animation end mode.
  Commit,
  /// The restore setting for animation end mode.
  Restore,
}

/// The idle, playing, paused, finished, or cancelled animation playback state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackState {
  /// The operation is idle.
  Idle,
  /// The operation is playing.
  Playing,
  /// The operation is paused.
  Paused,
  /// The operation is finished.
  Finished,
  /// The operation is cancelled.
  Cancelled,
}

/// The forward or backward direction of the current animation cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackDirection {
  /// The forward setting for playback direction.
  Forward,
  /// The reverse setting for playback direction.
  Reverse,
}

/// The identity of animation callback within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimationCallbackId(
  /// The wrapped u64 value.
  pub u64,
);

/// Configuration values controlling animation playback behavior.
///
/// # Fields
///
/// * `delay` - The delay represented as a duration.
/// * `speed` - The playback or animation speed multiplier.
/// * `clock` - The clock.
/// * `repeat` - The repeat.
/// * `end_mode` - The end mode.
/// * `auto_play` - Whether the animation begins immediately after allocation.
/// * `emit_events` - Whether playback transitions enter the event queue.
/// * `callback` - The callback.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationPlaybackOptions {
  /// The delay represented as a duration.
  pub delay: Duration,
  /// The playback or animation speed multiplier.
  pub speed: f64,
  /// The clock.
  pub clock: AnimationClock,
  /// The repeat.
  pub repeat: AnimationRepeatOptions,
  /// The end mode.
  pub end_mode: AnimationEndMode,
  /// Whether the animation begins immediately after allocation.
  pub auto_play: bool,
  /// Whether playback transitions enter the event queue.
  pub emit_events: bool,
  /// The callback.
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

/// The override, commit, or clearing action applied to an animation property.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationWriteOperation {
  /// The override setting for animation write operation.
  Override,
  /// The commit setting for animation write operation.
  Commit,
  /// The clear override setting for animation write operation.
  ClearOverride,
}

/// The animation write representation used by this module.
///
/// # Fields
///
/// * `animation_id` - The identifier of the animation.
/// * `target` - The object or resource affected by the operation.
/// * `property` - The property.
/// * `value` - The value to store or convert.
/// * `operation` - The operation to execute within the boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationWrite {
  /// The identifier of the animation.
  pub animation_id: AnimationId,
  /// The object or resource affected by the operation.
  pub target: AnimationTarget,
  /// The property.
  pub property: AnimationProperty,
  /// The value to store or convert.
  pub value: Option<AnimationValue>,
  /// The operation to execute within the boundary.
  pub operation: AnimationWriteOperation,
}

/// The animation event kind representation used by this module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnimationEventKind {
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

/// A animation event payload queued for its owning consumer.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `kind` - The animation event kind carried by this animation event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationEvent {
  /// The identifier of the owned object.
  pub id: AnimationId,
  /// The animation event kind carried by this animation event.
  pub kind: AnimationEventKind,
}

/// The animation callback request representation used by this module.
///
/// # Fields
///
/// * `callback` - The callback.
/// * `event` - The event to apply or route.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationCallbackRequest {
  /// The callback.
  pub callback: AnimationCallbackId,
  /// The event to apply or route.
  pub event: AnimationEvent,
}

/// The animation update representation used by this module.
///
/// # Fields
///
/// * `writes` - The ordered writes retained by this owner.
/// * `events` - Pending events retained in delivery order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationUpdate {
  /// The ordered writes retained by this owner.
  pub writes: Vec<AnimationWrite>,
  /// Pending events retained in delivery order.
  pub events: Vec<AnimationEvent>,
}

/// Failures reported by animation operations.
#[derive(Clone, Debug, PartialEq)]
pub enum AnimationError {
  /// The invalid duration failure condition.
  InvalidDuration,
  /// The invalid speed failure condition.
  InvalidSpeed,
  /// The invalid repeat count failure condition.
  InvalidRepeatCount,
  /// The invalid track failure condition.
  InvalidTrack(usize),
  /// The invalid keyframes failure condition.
  InvalidKeyframes(usize),
  /// The invalid binding failure condition.
  InvalidBinding(usize),
  /// The invalid character frames failure condition.
  InvalidCharacterFrames,
  /// The too many character frames failure condition.
  TooManyCharacterFrames(usize),
  /// The character frame too large failure condition.
  CharacterFrameTooLarge {
    /// The width in terminal columns.
    width: usize,
    /// The height in terminal rows.
    height: usize,
  },
  /// The invalid playback state failure condition.
  InvalidPlaybackState {
    /// The expected.
    expected: PlaybackState,
    /// The actual.
    actual: PlaybackState,
  },
  /// The stale animation failure condition.
  StaleAnimation,
  /// The stale value failure condition.
  StaleValue,
  /// The stale effect failure condition.
  StaleEffect,
  /// The target not found failure condition.
  TargetNotFound(AnimationTarget),
  /// The missing effect parameter failure condition.
  MissingEffectParameter(EffectParameterId),
  /// The unsupported property failure condition.
  UnsupportedProperty(String),
  /// The property not supported failure condition.
  PropertyNotSupported {
    /// The object or resource affected by the operation.
    target: AnimationTarget,
    /// The property.
    property: AnimationProperty,
  },
  /// The value type mismatch failure condition.
  ValueTypeMismatch {
    /// The expected.
    expected: AnimationValueKind,
    /// The actual.
    actual: AnimationValueKind,
  },
  /// The property type mismatch failure condition.
  PropertyTypeMismatch {
    /// The property.
    property: AnimationProperty,
    /// The actual.
    actual: AnimationValueKind,
  },
  /// The discrete interpolation required failure condition.
  DiscreteInterpolationRequired(AnimationValueKind),
  /// The RGB color required failure condition.
  RgbColorRequired,
}

impl fmt::Display for AnimationError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(formatter, "{self:?}")
  }
}

impl std::error::Error for AnimationError {}
