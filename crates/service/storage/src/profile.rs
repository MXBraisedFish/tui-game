use std::{
  collections::{BTreeMap, HashMap},
  fs, io,
  path::Path,
};

use serde::{Deserialize, Serialize, de::IgnoredAny};

use super::layout;
use super::service::StorageService;
use tg_core_atomic_fs::atomic_write;
use tg_core_package_id::PackageId;
use tg_service_log::{HostLogMessage, LogService, LogSource};

/// Terminal profile: the user's preferences for Unicode support, color mode and mouse support.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TerminalProfile {
  pub unicode: Option<bool>,

  pub color: Option<String>,

  pub mouse: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackageStateProfile {
  pub defaults: PackageDefaultState,
  pub games: HashMap<String, GamePackageState>,
  pub screensavers: HashMap<String, ScreensaverPackageState>,
}

impl PackageStateProfile {
  pub fn game(&self, id: &PackageId) -> Option<&GamePackageState> {
    self.games.get(&id.storage_key())
  }

  pub fn screensaver(&self, id: &PackageId) -> Option<&ScreensaverPackageState> {
    self.screensavers.get(&id.storage_key())
  }
}

pub type ActionKeyMap = BTreeMap<String, Vec<Vec<String>>>;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct KeyBindingMapGroup {
  pub global: ActionKeyMap,
  pub games: BTreeMap<String, ActionKeyMap>,
}

/// Persisted key binding table: `default` keeps the original definitions of the packages or the
/// host, `user` keeps the user mappings that are actually in effect.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct KeyBindingsProfile {
  pub default: KeyBindingMapGroup,
  pub user: KeyBindingMapGroup,
}

impl KeyBindingsProfile {
  pub fn synchronize(
    &mut self,
    global: ActionKeyMap,
    games: BTreeMap<String, ActionKeyMap>,
  ) -> bool {
    let previous = self.clone();
    synchronize_action_map(&mut self.default.global, &mut self.user.global, global);

    for (game, defaults) in games {
      synchronize_action_map(
        self.default.games.entry(game.clone()).or_default(),
        self.user.games.entry(game).or_default(),
        defaults,
      );
    }
    *self != previous
  }
}

fn synchronize_action_map(
  stored_default: &mut ActionKeyMap,
  user: &mut ActionKeyMap,
  current_default: ActionKeyMap,
) {
  user.retain(|action, _| current_default.contains_key(action));
  for (action, keys) in &current_default {
    if !user.contains_key(action) {
      user.insert(action.clone(), keys.clone());
    }
  }
  *stored_default = current_default;
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScreenshotDoubleAction {
  Copy,
  CopyRichText,
  #[default]
  SavePng,
  All,
}

impl ScreenshotDoubleAction {
  pub fn next(self) -> Self {
    match self {
      Self::Copy => Self::CopyRichText,
      Self::CopyRichText => Self::SavePng,
      Self::SavePng => Self::All,
      Self::All => Self::Copy,
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScreenshotProfile {
  pub guide_seen: bool,
  pub double_action: ScreenshotDoubleAction,
  pub auto_exit: bool,

  /// Custom font paths or system font names tried in order when screenshots are exported.
  pub fonts: Vec<String>,
}

impl Default for ScreenshotProfile {
  fn default() -> Self {
    Self {
      guide_seen: false,
      double_action: ScreenshotDoubleAction::SavePng,
      auto_exit: false,
      fonts: Vec::new(),
    }
  }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecordingFrameRate {
  Fps30,
  #[default]
  Fps60,
  Fps120,
}

impl RecordingFrameRate {
  pub fn value(self) -> u16 {
    match self {
      Self::Fps30 => 30,
      Self::Fps60 => 60,
      Self::Fps120 => 120,
    }
  }

  pub fn next(self) -> Self {
    match self {
      Self::Fps30 => Self::Fps60,
      Self::Fps60 => Self::Fps120,
      Self::Fps120 => Self::Fps30,
    }
  }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecordingPopupMode {
  Off,
  #[default]
  All,
  SplitOnly,
  StateOnly,
  StartStopOnly,
}

impl RecordingPopupMode {
  pub fn next(self) -> Self {
    match self {
      Self::Off => Self::All,
      Self::All => Self::SplitOnly,
      Self::SplitOnly => Self::StateOnly,
      Self::StateOnly => Self::StartStopOnly,
      Self::StartStopOnly => Self::Off,
    }
  }

  pub fn shows_split(self) -> bool {
    matches!(self, Self::All | Self::SplitOnly)
  }

  pub fn shows_pause_resume(self) -> bool {
    matches!(self, Self::All | Self::StateOnly)
  }

  pub fn shows_start_stop(self) -> bool {
    matches!(self, Self::All | Self::StartStopOnly)
  }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutoRecordingMode {
  #[default]
  Off,
  Host,
  Game,
}

impl AutoRecordingMode {
  pub fn next(self) -> Self {
    match self {
      Self::Off => Self::Host,
      Self::Host => Self::Game,
      Self::Game => Self::Off,
    }
  }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutoSplitDuration {
  Off,
  #[default]
  Minutes3,
  Minutes5,
  Minutes10,
}

impl AutoSplitDuration {
  pub fn next(self) -> Self {
    match self {
      Self::Off => Self::Minutes3,
      Self::Minutes3 => Self::Minutes5,
      Self::Minutes5 => Self::Minutes10,
      Self::Minutes10 => Self::Off,
    }
  }

  pub fn duration(self) -> Option<std::time::Duration> {
    match self {
      Self::Off => None,
      Self::Minutes3 => Some(std::time::Duration::from_secs(3 * 60)),
      Self::Minutes5 => Some(std::time::Duration::from_secs(5 * 60)),
      Self::Minutes10 => Some(std::time::Duration::from_secs(10 * 60)),
    }
  }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecordingExportFrameRate {
  #[default]
  Recorded,
  Fps30,
  Fps60,
  Fps120,
}

impl RecordingExportFrameRate {
  pub fn resolve(self, recorded: u16) -> u16 {
    match self {
      Self::Recorded => recorded,
      Self::Fps30 => 30,
      Self::Fps60 => 60,
      Self::Fps120 => 120,
    }
  }

  pub fn next(self) -> Self {
    match self {
      Self::Recorded => Self::Fps30,
      Self::Fps30 => Self::Fps60,
      Self::Fps60 => Self::Fps120,
      Self::Fps120 => Self::Recorded,
    }
  }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecordingExportQuality {
  Compact,
  #[default]
  Balanced,
  High,
}

impl RecordingExportQuality {
  pub fn next(self) -> Self {
    match self {
      Self::Compact => Self::Balanced,
      Self::Balanced => Self::High,
      Self::High => Self::Compact,
    }
  }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecordingPixelScale {
  Half,
  #[default]
  Original,
  Double,
}

impl RecordingPixelScale {
  pub fn multiplier(self) -> (u32, u32) {
    match self {
      Self::Half => (1, 2),
      Self::Original => (1, 1),
      Self::Double => (2, 1),
    }
  }

  pub fn next(self) -> Self {
    match self {
      Self::Half => Self::Original,
      Self::Original => Self::Double,
      Self::Double => Self::Half,
    }
  }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecordingGpuAcceleration {
  Off,
  #[default]
  Auto,
  Nvidia,
  Amd,
  Intel,
  Apple,
}

impl RecordingGpuAcceleration {
  pub fn next(self) -> Self {
    match self {
      Self::Off => Self::Auto,
      Self::Auto => Self::Nvidia,
      Self::Nvidia => Self::Amd,
      Self::Amd => Self::Intel,
      Self::Intel => Self::Apple,
      Self::Apple => Self::Off,
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RecordingProfile {
  pub popup: RecordingPopupMode,
  pub auto_recording: AutoRecordingMode,
  pub auto_split: AutoSplitDuration,
  pub capture_frame_rate: RecordingFrameRate,
  pub export_frame_rate: RecordingExportFrameRate,
  pub quality: RecordingExportQuality,
  pub keyframe_interval_seconds: u16,
  pub pixel_scale: RecordingPixelScale,
  pub gpu_acceleration: RecordingGpuAcceleration,
}

fn default_keyframe_interval() -> u16 {
  2
}

impl RecordingProfile {
  pub fn is_valid(&self) -> bool {
    (1..=10).contains(&self.keyframe_interval_seconds)
  }
}

impl Default for RecordingProfile {
  fn default() -> Self {
    Self {
      popup: RecordingPopupMode::default(),
      auto_recording: AutoRecordingMode::default(),
      auto_split: AutoSplitDuration::default(),
      capture_frame_rate: RecordingFrameRate::default(),
      export_frame_rate: RecordingExportFrameRate::default(),
      quality: RecordingExportQuality::default(),
      keyframe_interval_seconds: default_keyframe_interval(),
      pixel_scale: RecordingPixelScale::default(),
      gpu_acceleration: RecordingGpuAcceleration::default(),
    }
  }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisplayLogoMode {
  Order,
  Random,
  Classic,
  Neon,
  Wave,
  Error,
  Glitch,
  Select,
  Char,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisplaySourceMode {
  All,
  Mod,
  Official,
  No,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisplayOrderMode {
  Random,
  Order,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisplayFpsLimit {
  Fps30,
  Fps60,
  Fps120,
  Unlimited,
}

impl DisplayLogoMode {
  /// Returns the next value in the settings page cycling order.
  pub fn next(self) -> Self {
    match self {
      Self::Order => Self::Random,
      Self::Random => Self::Classic,
      Self::Classic => Self::Neon,
      Self::Neon => Self::Wave,
      Self::Wave => Self::Error,
      Self::Error => Self::Glitch,
      Self::Glitch => Self::Select,
      Self::Select => Self::Char,
      Self::Char => Self::Order,
    }
  }
}

impl DisplaySourceMode {
  /// Returns the next value in the settings page cycling order.
  pub fn next(self) -> Self {
    match self {
      Self::All => Self::Mod,
      Self::Mod => Self::Official,
      Self::Official => Self::No,
      Self::No => Self::All,
    }
  }
}

impl DisplayOrderMode {
  /// Returns the next value in the settings page cycling order.
  pub fn next(self) -> Self {
    match self {
      Self::Random => Self::Order,
      Self::Order => Self::Random,
    }
  }
}

impl DisplayFpsLimit {
  pub fn target_fps(self) -> Option<u16> {
    match self {
      Self::Fps30 => Some(30),
      Self::Fps60 => Some(60),
      Self::Fps120 => Some(120),
      Self::Unlimited => None,
    }
  }

  /// Returns the next value in the settings page cycling order.
  pub fn next(self) -> Self {
    match self {
      Self::Fps30 => Self::Fps60,
      Self::Fps60 => Self::Fps120,
      Self::Fps120 => Self::Unlimited,
      Self::Unlimited => Self::Fps30,
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DisplaySettingsProfile {
  pub logo_mode: DisplayLogoMode,
  pub logo_sequence_cursor: u64,
  pub top_toolbar: bool,
  pub top_toolbar_custom_text: String,
  pub screensaver_source: DisplaySourceMode,
  pub screensaver_order: DisplayOrderMode,
  pub screensaver_sequence_cursor: u64,
  pub game_list_source: DisplaySourceMode,
  pub game_list_warnings: bool,
  pub game_list_fps: DisplayFpsLimit,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(from = "PackageDefaultStateProfile", deny_unknown_fields)]
pub struct PackageDefaultState {
  pub enabled: bool,
  pub debug: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackageDefaultStateProfile {
  enabled: bool,
  debug: bool,
  #[serde(default, rename = "safe_mode")]
  _safe_mode: Option<IgnoredAny>,
}

impl From<PackageDefaultStateProfile> for PackageDefaultState {
  fn from(profile: PackageDefaultStateProfile) -> Self {
    Self {
      enabled: profile.enabled,
      debug: profile.debug,
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(from = "GamePackageStateProfile", deny_unknown_fields)]
pub struct GamePackageState {
  pub enabled: bool,
  pub debug: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GamePackageStateProfile {
  enabled: bool,
  debug: bool,
  #[serde(default, rename = "safe_mode")]
  _safe_mode: Option<IgnoredAny>,
}

impl From<GamePackageStateProfile> for GamePackageState {
  fn from(profile: GamePackageStateProfile) -> Self {
    Self {
      enabled: profile.enabled,
      debug: profile.debug,
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScreensaverPackageState {
  /// Master switch of the package manager; a disabled package is left out of the screensaver
  /// list.
  pub enabled: bool,
  pub debug: bool,

  /// Enabled state inside the screensaver list, independent of the package master switch.
  pub playlist_enabled: bool,

  /// Display order of an enabled screensaver; disabled screensavers take no part in ordering.
  pub order: Option<u32>,
}

impl Default for GamePackageState {
  fn default() -> Self {
    Self {
      enabled: true,
      debug: false,
    }
  }
}

impl Default for ScreensaverPackageState {
  fn default() -> Self {
    Self {
      enabled: true,
      debug: false,
      playlist_enabled: true,
      order: None,
    }
  }
}

impl Default for DisplaySettingsProfile {
  fn default() -> Self {
    Self {
      logo_mode: DisplayLogoMode::Order,
      logo_sequence_cursor: 0,
      top_toolbar: true,
      top_toolbar_custom_text: String::new(),
      screensaver_source: DisplaySourceMode::All,
      screensaver_order: DisplayOrderMode::Random,
      screensaver_sequence_cursor: 0,
      game_list_source: DisplaySourceMode::All,
      game_list_warnings: true,
      game_list_fps: DisplayFpsLimit::Fps60,
    }
  }
}

impl Default for PackageDefaultState {
  fn default() -> Self {
    Self {
      enabled: true,
      debug: false,
    }
  }
}

impl TerminalProfile {
  /// Returns whether all three settings are filled in (the color mode must be `truecolor` or
  /// `256`).
  pub fn is_complete(&self) -> bool {
    self.unicode.is_some()
      && self
        .color
        .as_deref()
        .is_some_and(|c| c == "truecolor" || c == "256")
      && self.mouse.is_some()
  }
}

impl StorageService {
  /// Reads the saved language code.
  pub fn read_language_code(&self, log: &mut LogService) -> Option<String> {
    let content = fs::read_to_string(self.profile_language_path())
      .inspect_err(|error| {
        log_profile_read_error(log, "language", &self.profile_language_path(), error);
      })
      .ok()?;
    let code = content.trim();
    if code.is_empty() {
      None
    } else {
      Some(code.to_string())
    }
  }

  /// Writes the language code to its profile file.
  ///
  /// # Errors
  ///
  /// Returns the I/O error when the file cannot be written.
  pub fn write_language_code(&self, language_code: &str) -> std::io::Result<()> {
    atomic_write(
      &self.profile_language_path(),
      language_code.trim().as_bytes(),
      true,
    )
  }

  pub fn read_key_bindings_profile(&self, log: &mut LogService) -> KeyBindingsProfile {
    let path = self.profile_key_bindings_path();
    let content = match fs::read_to_string(&path) {
      Ok(content) => content,
      Err(error) => {
        log_profile_read_error(log, "key_bindings", &path, &error);
        return KeyBindingsProfile::default();
      }
    };
    serde_json::from_str(&content).unwrap_or_else(|error| {
      log.warn_operation_failed(
        LogSource::Storage,
        "parse_profile",
        "key_bindings",
        error.to_string(),
      );
      KeyBindingsProfile::default()
    })
  }

  pub fn write_key_bindings_profile(
    &self,
    profile: &KeyBindingsProfile,
    log: &mut LogService,
  ) -> io::Result<()> {
    let json = serde_json::to_string_pretty(profile).map_err(io::Error::other)?;
    let path = self.profile_key_bindings_path();
    let changed = changed_profile_fields(&path, &json);
    atomic_write(&path, json.as_bytes(), true).inspect_err(|error| {
      log.error_operation_failed(
        LogSource::Storage,
        "write_profile",
        "key_bindings",
        error.to_string(),
      );
    })?;
    log_profile_change(log, "key_bindings", changed);
    Ok(())
  }

  /// Returns the default language code.
  pub fn default_language_code(&self) -> &'static str {
    layout::DEFAULT_LANGUAGE_CODE
  }

  pub fn display_settings_profile(&self) -> &DisplaySettingsProfile {
    &self.display_settings
  }

  pub fn reload_display_settings_profile(
    &mut self,
    log: &mut LogService,
  ) -> DisplaySettingsProfile {
    let path = self.profile_display_settings_path();
    let profile = fs::read_to_string(&path)
      .and_then(|content| serde_json::from_str(&content).map_err(io::Error::other))
      .unwrap_or_else(|error| {
        if error.kind() != io::ErrorKind::NotFound {
          log.warn_operation_failed(
            LogSource::Storage,
            "load_profile",
            "display_settings",
            error.to_string(),
          );
        }
        DisplaySettingsProfile::default()
      });
    self.display_settings = profile.clone();
    profile
  }

  pub fn write_display_settings_profile(
    &mut self,
    profile: &DisplaySettingsProfile,
    log: &mut LogService,
  ) -> io::Result<()> {
    let path = self.profile_display_settings_path();
    let content = serde_json::to_string_pretty(profile).map_err(io::Error::other)?;
    let changed = changed_profile_fields(&path, &content);
    atomic_write(&path, content.as_bytes(), true).inspect_err(|error| {
      log.warn_operation_failed(
        LogSource::Storage,
        "write_profile",
        "display_settings",
        error.to_string(),
      );
    })?;
    self.display_settings = profile.clone();
    log_profile_change(log, "display_settings", changed);
    Ok(())
  }

  /// Reads the terminal profile from its file.
  pub fn read_terminal_profile(&self, log: &mut LogService) -> Option<TerminalProfile> {
    let content = fs::read_to_string(self.profile_terminal_path())
      .inspect_err(|error| {
        log_profile_read_error(log, "terminal", &self.profile_terminal_path(), error);
      })
      .ok()?;
    serde_json::from_str(&content)
      .inspect_err(|error| {
        log.warn_operation_failed(
          LogSource::Storage,
          "parse_profile",
          "terminal",
          error.to_string(),
        );
      })
      .ok()
  }

  /// Reads the terminal profile, falling back to the default when it is missing or invalid.
  pub fn read_terminal_profile_or_default(&self, log: &mut LogService) -> TerminalProfile {
    self.read_terminal_profile(log).unwrap_or_default()
  }

  /// Reads the terminal profile, lets `f` modify it and writes it back.
  ///
  /// # Errors
  ///
  /// Returns an error when the modified profile cannot be serialized or written.
  pub fn update_terminal_profile(
    &self,
    log: &mut LogService,
    f: impl FnOnce(&mut TerminalProfile),
  ) -> std::io::Result<()> {
    let mut profile = self.read_terminal_profile_or_default(log);
    f(&mut profile);
    self.write_terminal_profile(&profile, log)
  }

  /// Serializes the terminal profile and writes it to its file.
  ///
  /// # Errors
  ///
  /// Returns an [`io::ErrorKind::InvalidData`] error when serialization fails and the I/O error
  /// when the file cannot be written.
  pub fn write_terminal_profile(
    &self,
    profile: &TerminalProfile,
    log: &mut LogService,
  ) -> std::io::Result<()> {
    let json = match serde_json::to_string_pretty(profile) {
      Ok(json) => json,
      Err(error) => {
        log.error_operation_failed(
          LogSource::Storage,
          "serialize_profile",
          "terminal",
          error.to_string(),
        );
        return Err(io::Error::new(
          io::ErrorKind::InvalidData,
          format!("Serialization failed: {error}"),
        ));
      }
    };
    let path = self.profile_terminal_path();
    let changed = changed_profile_fields(&path, &json);
    atomic_write(&path, json.as_bytes(), true)?;
    log_profile_change(log, "terminal", changed);
    Ok(())
  }

  /// Clears the saved terminal capability results so the next start runs capability detection
  /// again.
  ///
  /// # Errors
  ///
  /// Returns an error when the default profile cannot be written.
  pub fn reset_terminal_profile(&self, log: &mut LogService) -> std::io::Result<()> {
    self.write_terminal_profile(&TerminalProfile::default(), log)
  }

  /// Returns whether the terminal profile file is completely filled in.
  pub fn is_terminal_profile_complete(&self, log: &mut LogService) -> bool {
    self
      .read_terminal_profile(log)
      .is_some_and(|p| p.is_complete())
  }

  pub fn read_package_state(&self, log: &mut LogService) -> Option<PackageStateProfile> {
    let content = fs::read_to_string(self.profile_package_state_path())
      .inspect_err(|error| {
        log_profile_read_error(
          log,
          "package_state",
          &self.profile_package_state_path(),
          error,
        );
      })
      .ok()?;
    serde_json::from_str(&content)
      .inspect_err(|error| {
        log.warn_operation_failed(
          LogSource::Storage,
          "parse_profile",
          "package_state",
          error.to_string(),
        );
      })
      .ok()
  }

  pub fn read_package_state_or_default(&self, log: &mut LogService) -> PackageStateProfile {
    self.read_package_state(log).unwrap_or_default()
  }

  pub fn write_package_state(
    &self,
    profile: &PackageStateProfile,
    log: &mut LogService,
  ) -> std::io::Result<()> {
    let json = match serde_json::to_string_pretty(profile) {
      Ok(json) => json,
      Err(error) => {
        log.error_operation_failed(
          LogSource::Storage,
          "serialize_profile",
          "package_state",
          error.to_string(),
        );
        return Err(io::Error::new(
          io::ErrorKind::InvalidData,
          format!("Serialization failed: {error}"),
        ));
      }
    };
    let path = self.profile_package_state_path();
    let changed = changed_profile_fields(&path, &json);
    atomic_write(&path, json.as_bytes(), true)?;
    log_profile_change(log, "package_state", changed);
    Ok(())
  }

  pub fn update_game_package_state(
    &self,
    package_id: &PackageId,
    log: &mut LogService,
    f: impl FnOnce(&mut GamePackageState),
  ) -> std::io::Result<()> {
    let mut profile = self.read_package_state_or_default(log);
    let defaults = &profile.defaults;
    let initial = GamePackageState {
      enabled: defaults.enabled,
      debug: defaults.debug,
    };
    f(profile
      .games
      .entry(package_id.storage_key())
      .or_insert(initial));
    self.write_package_state(&profile, log)
  }

  pub fn update_screensaver_package_state(
    &self,
    package_id: &PackageId,
    log: &mut LogService,
    f: impl FnOnce(&mut ScreensaverPackageState),
  ) -> std::io::Result<()> {
    let mut profile = self.read_package_state_or_default(log);
    let initial = ScreensaverPackageState {
      enabled: profile.defaults.enabled,
      debug: profile.defaults.debug,
      playlist_enabled: true,
      order: None,
    };
    f(profile
      .screensavers
      .entry(package_id.storage_key())
      .or_insert(initial));
    self.write_package_state(&profile, log)
  }

  pub fn read_screenshot_profile(&self, log: &mut LogService) -> Option<ScreenshotProfile> {
    let content = fs::read_to_string(self.profile_screenshot_path())
      .inspect_err(|error| {
        log_profile_read_error(log, "screenshot", &self.profile_screenshot_path(), error);
      })
      .ok()?;
    serde_json::from_str(&content)
      .inspect_err(|error| {
        log.warn_operation_failed(
          LogSource::Storage,
          "parse_profile",
          "screenshot",
          error.to_string(),
        );
      })
      .ok()
  }

  pub fn read_recording_profile(&self, log: &mut LogService) -> Option<RecordingProfile> {
    let content = fs::read_to_string(self.profile_recording_path())
      .inspect_err(|error| {
        log_profile_read_error(log, "recording", &self.profile_recording_path(), error);
      })
      .ok()?;
    let profile = serde_json::from_str::<RecordingProfile>(&content)
      .inspect_err(|error| {
        log.warn_operation_failed(
          LogSource::Storage,
          "parse_profile",
          "recording",
          error.to_string(),
        );
      })
      .ok()?;
    if !profile.is_valid() {
      log.warn_operation_failed(
        LogSource::Storage,
        "validate_profile",
        "recording",
        "profile contains values outside the current valid range",
      );
      return None;
    }
    Some(profile)
  }

  pub fn read_recording_profile_or_default(&self, log: &mut LogService) -> RecordingProfile {
    self.read_recording_profile(log).unwrap_or_default()
  }

  pub fn recording_profile_revision(&self) -> u64 {
    self.recording_profile_revision.get()
  }

  pub fn write_recording_profile(
    &self,
    profile: &RecordingProfile,
    log: &mut LogService,
  ) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(profile).map_err(|error| {
      log.error_operation_failed(
        LogSource::Storage,
        "serialize_profile",
        "recording",
        error.to_string(),
      );
      io::Error::new(io::ErrorKind::InvalidData, error)
    })?;
    let path = self.profile_recording_path();
    let changed = changed_profile_fields(&path, &json);
    let result = atomic_write(&path, json.as_bytes(), true);
    if result.is_ok() {
      self
        .recording_profile_revision
        .set(self.recording_profile_revision.get().wrapping_add(1));
      log_profile_change(log, "recording", changed);
    }
    result
  }

  pub fn read_screenshot_profile_or_default(&self, log: &mut LogService) -> ScreenshotProfile {
    self.read_screenshot_profile(log).unwrap_or_default()
  }

  pub fn write_screenshot_profile(
    &self,
    profile: &ScreenshotProfile,
    log: &mut LogService,
  ) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(profile).map_err(|error| {
      log.error_operation_failed(
        LogSource::Storage,
        "serialize_profile",
        "screenshot",
        error.to_string(),
      );
      io::Error::new(
        io::ErrorKind::InvalidData,
        format!("Serialization failed: {error}"),
      )
    })?;
    let path = self.profile_screenshot_path();
    let changed = changed_profile_fields(&path, &json);
    atomic_write(&path, json.as_bytes(), true)?;
    log_profile_change(log, "screenshot", changed);
    Ok(())
  }

  pub fn mark_screenshot_guide_seen(&self, log: &mut LogService) {
    let mut profile = self.read_screenshot_profile_or_default(log);
    if profile.guide_seen {
      return;
    }
    profile.guide_seen = true;
    if let Err(error) = self.write_screenshot_profile(&profile, log) {
      log.warn_operation_failed(
        LogSource::Storage,
        "write_profile",
        "screenshot",
        error.to_string(),
      );
    }
  }
}

fn changed_profile_fields(path: &Path, next_json: &str) -> Option<String> {
  let next: serde_json::Value = serde_json::from_str(next_json).ok()?;
  let previous = fs::read_to_string(path)
    .ok()
    .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok());
  if previous.as_ref() == Some(&next) {
    return None;
  }

  let Some(next_fields) = next.as_object() else {
    return Some("value".to_string());
  };
  let previous_fields = previous.as_ref().and_then(serde_json::Value::as_object);
  let mut changed = next_fields
    .iter()
    .filter_map(|(field, value)| {
      (previous_fields.and_then(|fields| fields.get(field)) != Some(value)).then_some(field.clone())
    })
    .collect::<Vec<_>>();
  if let Some(previous_fields) = previous_fields {
    changed.extend(
      previous_fields
        .keys()
        .filter(|field| !next_fields.contains_key(*field))
        .cloned(),
    );
  }
  changed.sort_unstable();
  changed.dedup();
  Some(if changed.is_empty() {
    "value".to_string()
  } else {
    changed.join(",")
  })
}

fn log_profile_change(log: &mut LogService, group: &str, fields: Option<String>) {
  let Some(fields) = fields else {
    return;
  };
  log.info_message(
    LogSource::Storage,
    HostLogMessage::new(
      "log_info.setting.changed",
      "Setting group {group} changed fields: {fields}.",
    )
    .param("group", group)
    .param("fields", fields),
  );
}

fn log_profile_read_error(log: &mut LogService, profile: &str, path: &Path, error: &io::Error) {
  if error.kind() != io::ErrorKind::NotFound {
    log.warn_operation_failed(
      LogSource::Storage,
      "read_profile",
      format!("{profile}:{}", path.display()),
      error.to_string(),
    );
  }
}

#[cfg(test)]
mod tests {
  use serde_json::Value;

  use super::*;

  #[test]
  fn display_setting_cycles_return_to_start_and_fps_limits_map_to_targets() {
    let mut fps = DisplayFpsLimit::Fps30;
    let mut targets = Vec::new();
    for _ in 0..4 {
      targets.push(fps.target_fps());
      fps = fps.next();
    }
    assert_eq!(fps, DisplayFpsLimit::Fps30);
    assert_eq!(targets, [Some(30), Some(60), Some(120), None]);
    assert_eq!(
      DisplayOrderMode::Random.next().next(),
      DisplayOrderMode::Random
    );
    assert_eq!(DisplaySourceMode::No.next(), DisplaySourceMode::All);
    assert_eq!(DisplayLogoMode::Char.next(), DisplayLogoMode::Order);
  }

  fn temp_storage(name: &str) -> StorageService {
    let root = std::env::temp_dir().join(format!("tg_storage_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("data/profiles")).unwrap();
    StorageService::from_root_for_test(root)
  }

  #[test]
  fn missing_package_state_returns_default() {
    let storage = temp_storage("missing_package_state");
    let mut log = LogService::new();
    assert_eq!(
      storage.read_package_state_or_default(&mut log),
      PackageStateProfile::default()
    );
  }

  #[test]
  fn reset_terminal_profile_clears_only_capability_results() {
    let storage = temp_storage("reset_terminal_profile");
    let mut log = LogService::new();
    storage.write_language_code("zh_cn").unwrap();
    storage
      .write_terminal_profile(
        &TerminalProfile {
          unicode: Some(true),
          color: Some("truecolor".to_string()),
          mouse: Some(true),
        },
        &mut log,
      )
      .unwrap();
    assert!(storage.is_terminal_profile_complete(&mut log));

    storage.reset_terminal_profile(&mut log).unwrap();

    let profile = storage.read_terminal_profile(&mut log).unwrap();
    assert!(profile.unicode.is_none());
    assert!(profile.color.is_none());
    assert!(profile.mouse.is_none());
    assert!(!profile.is_complete());
    assert_eq!(
      storage.read_language_code(&mut log).as_deref(),
      Some("zh_cn")
    );
  }

  #[test]
  fn package_state_persists_game_and_screensaver_independently() {
    let storage = temp_storage("package_state_persists");
    let mut log = LogService::new();
    let game_id = PackageId::new(
      tg_core_package_id::PackageSource::Official,
      tg_core_package_id::PackageType::Game,
      "same_id",
    )
    .unwrap();
    let screensaver_id = PackageId::new(
      tg_core_package_id::PackageSource::Official,
      tg_core_package_id::PackageType::Screensaver,
      "same_id",
    )
    .unwrap();

    storage
      .update_game_package_state(&game_id, &mut log, |state| {
        state.enabled = false;
        state.debug = true;
      })
      .unwrap();
    storage
      .update_screensaver_package_state(&screensaver_id, &mut log, |state| {
        state.enabled = false;
        state.debug = true;
      })
      .unwrap();

    let profile = storage.read_package_state_or_default(&mut log);
    assert_eq!(
      profile.games.get(&game_id.storage_key()),
      Some(&GamePackageState {
        enabled: false,
        debug: true,
      })
    );
    assert_eq!(
      profile.screensavers.get(&screensaver_id.storage_key()),
      Some(&ScreensaverPackageState {
        enabled: false,
        debug: true,
        playlist_enabled: true,
        order: None,
      })
    );
  }

  #[test]
  fn invalid_package_state_json_falls_back_to_default() {
    let storage = temp_storage("invalid_package_state");
    let mut log = LogService::new();
    fs::write(storage.profile_package_state_path(), "{").unwrap();
    assert_eq!(
      storage.read_package_state_or_default(&mut log),
      PackageStateProfile::default()
    );
  }

  #[test]
  fn package_defaults_are_persisted_and_seed_new_package_states() {
    let storage = temp_storage("package_defaults");
    let mut log = LogService::new();
    let profile = PackageStateProfile {
      defaults: PackageDefaultState {
        enabled: false,
        debug: true,
      },
      ..Default::default()
    };
    storage.write_package_state(&profile, &mut log).unwrap();

    let game_id = PackageId::new(
      tg_core_package_id::PackageSource::Official,
      tg_core_package_id::PackageType::Game,
      "game",
    )
    .unwrap();
    let screensaver_id = PackageId::new(
      tg_core_package_id::PackageSource::Official,
      tg_core_package_id::PackageType::Screensaver,
      "screen",
    )
    .unwrap();

    storage
      .update_game_package_state(&game_id, &mut log, |state| state.debug = false)
      .unwrap();
    storage
      .update_screensaver_package_state(&screensaver_id, &mut log, |state| state.debug = false)
      .unwrap();

    let profile = storage.read_package_state_or_default(&mut log);
    assert!(!profile.games[&game_id.storage_key()].enabled);
    assert!(!profile.screensavers[&screensaver_id.storage_key()].enabled);
  }

  #[test]
  fn incomplete_package_profile_is_rejected() {
    assert!(
      serde_json::from_str::<PackageStateProfile>(r#"{"games":{},"screensavers":{}}"#).is_err()
    );
  }

  #[test]
  fn legacy_safe_mode_fields_are_ignored_without_losing_package_settings() {
    let profile: PackageStateProfile = serde_json::from_str(
      r#"
        {
          "defaults": { "enabled": false, "debug": true, "safe_mode": "off_permanent" },
          "games": {
            "game:test.game": { "enabled": false, "debug": true, "safe_mode": false }
          },
          "screensavers": {
            "screensaver:test.screen": {
              "enabled": true,
              "debug": false,
              "playlist_enabled": false,
              "order": 7
            }
          }
        }
      "#,
    )
    .unwrap();

    assert_eq!(
      profile.defaults,
      PackageDefaultState {
        enabled: false,
        debug: true,
      }
    );
    assert_eq!(
      profile.games["game:test.game"],
      GamePackageState {
        enabled: false,
        debug: true,
      }
    );
    assert_eq!(
      profile.screensavers["screensaver:test.screen"],
      ScreensaverPackageState {
        enabled: true,
        debug: false,
        playlist_enabled: false,
        order: Some(7),
      }
    );

    let serialized = serde_json::to_value(profile).unwrap();
    assert!(serialized["defaults"].get("safe_mode").is_none());
    assert!(
      serialized["games"]["game:test.game"]
        .get("safe_mode")
        .is_none()
    );
  }

  #[test]
  fn legacy_safe_mode_profile_update_preserves_other_user_data() {
    let root = std::env::temp_dir().join(format!(
      "tg_storage_legacy_safe_mode_user_data_{}",
      std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("data/profiles")).unwrap();

    let mut log = LogService::new();
    let storage = StorageService::from_root_for_test(root.clone());
    let game_id = PackageId::new(
      tg_core_package_id::PackageSource::Official,
      tg_core_package_id::PackageType::Game,
      "legacy_game",
    )
    .unwrap();
    let screen_id = PackageId::new(
      tg_core_package_id::PackageSource::Official,
      tg_core_package_id::PackageType::Screensaver,
      "legacy_screen",
    )
    .unwrap();

    storage.write_language_code("zh_cn").unwrap();

    let mut key_bindings = KeyBindingsProfile::default();
    key_bindings
      .default
      .global
      .insert("confirm".into(), vec![vec!["enter".into()]]);
    key_bindings
      .user
      .global
      .insert("confirm".into(), vec![vec!["space".into()]]);
    storage
      .write_key_bindings_profile(&key_bindings, &mut log)
      .unwrap();

    storage
      .write_continue_game_save(&game_id, serde_json::json!({"level": 8}), &mut log)
      .unwrap();
    storage
      .write_best_game_save(
        &game_id,
        crate::BestGameSave {
          best_string: "420".into(),
          data: serde_json::json!({"best_string": "420", "score": 420}),
        },
        &mut log,
      )
      .unwrap();

    fs::write(
      storage.profile_package_state_path(),
      format!(
        r#"{{
          "defaults": {{"enabled": false, "debug": true, "safe_mode": "off_permanent"}},
          "games": {{"{}": {{"enabled": false, "debug": true, "safe_mode": false}}}},
          "screensavers": {{"{}": {{"enabled": true, "debug": false, "playlist_enabled": false, "order": 7}}}}
        }}"#,
        game_id.storage_key(),
        screen_id.storage_key(),
      ),
    )
    .unwrap();
    drop(storage);

    let storage = StorageService::new(root.clone(), &mut log).unwrap();
    assert_eq!(
      storage.read_language_code(&mut log).as_deref(),
      Some("zh_cn")
    );
    assert_eq!(storage.read_key_bindings_profile(&mut log), key_bindings);
    assert_eq!(
      storage.continue_game_save().unwrap().data,
      serde_json::json!({"level": 8})
    );
    assert_eq!(
      storage.best_game_save(&game_id).unwrap().data,
      serde_json::json!({"best_string": "420", "score": 420})
    );

    storage
      .update_game_package_state(&game_id, &mut log, |state| state.debug = false)
      .unwrap();
    let package_state = storage.read_package_state_or_default(&mut log);
    assert_eq!(
      package_state.defaults,
      PackageDefaultState {
        enabled: false,
        debug: true,
      }
    );
    assert_eq!(
      package_state.games[&game_id.storage_key()],
      GamePackageState {
        enabled: false,
        debug: false,
      }
    );
    assert_eq!(
      package_state.screensavers[&screen_id.storage_key()],
      ScreensaverPackageState {
        enabled: true,
        debug: false,
        playlist_enabled: false,
        order: Some(7),
      }
    );
    let rewritten_profile: Value =
      serde_json::from_str(&fs::read_to_string(storage.profile_package_state_path()).unwrap())
        .unwrap();
    assert!(rewritten_profile["defaults"].get("safe_mode").is_none());
    assert!(
      rewritten_profile["games"][game_id.storage_key()]
        .get("safe_mode")
        .is_none()
    );

    fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn incomplete_screenshot_profile_is_rejected() {
    assert!(serde_json::from_str::<ScreenshotProfile>(r#"{"guide_seen":true}"#).is_err());
  }

  #[test]
  fn key_bindings_profile_copies_new_defaults_without_overwriting_user_changes() {
    let mut profile = KeyBindingsProfile::default();
    let mut global = ActionKeyMap::new();
    global.insert("one".into(), vec![vec!["a".into()]]);
    assert!(profile.synchronize(global.clone(), BTreeMap::new()));
    assert_eq!(profile.default.global, global);
    assert_eq!(profile.user.global, global);

    profile
      .user
      .global
      .insert("one".into(), vec![vec!["b".into()]]);
    global.insert("one".into(), vec![vec!["c".into()]]);
    global.insert("two".into(), Vec::new());
    assert!(profile.synchronize(global.clone(), BTreeMap::new()));
    assert_eq!(profile.user.global["one"], vec![vec!["b".to_string()]]);
    assert!(profile.user.global["two"].is_empty());
    assert_eq!(profile.default.global, global);
  }

  #[test]
  fn key_binding_modifier_tokens_survive_profile_reload() {
    let root =
      std::env::temp_dir().join(format!("tg_storage_modifier_tokens_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("data/profiles")).unwrap();
    let mut log = LogService::new();
    let storage = StorageService::from_root_for_test(root.clone());

    let mut profile = KeyBindingsProfile::default();
    profile.default.global.insert(
      "host.quit".into(),
      vec![vec!["left_ctrl".into(), "q".into()]],
    );
    profile.user.global.insert(
      "host.quit".into(),
      vec![
        vec!["left_ctrl".into(), "q".into()],
        vec!["right_ctrl".into(), "q".into()],
      ],
    );
    profile
      .user
      .global
      .insert("legacy.alias".into(), vec![vec!["ctrl".into(), "x".into()]]);
    storage
      .write_key_bindings_profile(&profile, &mut log)
      .unwrap();
    drop(storage);

    let storage = StorageService::new(root.clone(), &mut log).unwrap();
    let loaded = storage.read_key_bindings_profile(&mut log);
    assert_eq!(loaded, profile);
    assert_eq!(
      loaded.user.global["host.quit"],
      vec![
        vec!["left_ctrl".to_string(), "q".to_string()],
        vec!["right_ctrl".to_string(), "q".to_string()],
      ]
    );
    assert_eq!(
      loaded.user.global["legacy.alias"],
      vec![vec!["ctrl".to_string(), "x".to_string()]]
    );

    fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn key_bindings_profile_seeds_games_and_preserves_user_data_when_package_is_absent() {
    let mut profile = KeyBindingsProfile::default();
    let mut game_actions = ActionKeyMap::new();
    game_actions.insert("jump".into(), vec![vec!["space".into()]]);
    let mut games = BTreeMap::new();
    games.insert("game.one".into(), game_actions.clone());
    profile.synchronize(ActionKeyMap::new(), games);
    assert_eq!(profile.user.games["game.one"], game_actions);

    profile.synchronize(ActionKeyMap::new(), BTreeMap::new());
    assert_eq!(profile.default.games["game.one"], game_actions);
    assert_eq!(profile.user.games["game.one"], game_actions);
  }

  #[test]
  fn key_bindings_profile_removes_actions_no_longer_declared_by_an_installed_game() {
    let mut profile = KeyBindingsProfile::default();
    let mut original = ActionKeyMap::new();
    original.insert("jump".into(), vec![vec!["space".into()]]);
    original.insert("removed".into(), vec![vec!["r".into()]]);
    let mut games = BTreeMap::new();
    games.insert("game.one".into(), original);
    profile.synchronize(ActionKeyMap::new(), games);

    let mut current = ActionKeyMap::new();
    current.insert("jump".into(), vec![vec!["space".into()]]);
    let mut games = BTreeMap::new();
    games.insert("game.one".into(), current);
    profile.synchronize(ActionKeyMap::new(), games);

    assert!(!profile.user.games["game.one"].contains_key("removed"));
  }

  #[test]
  fn incomplete_recording_profile_is_rejected() {
    assert!(serde_json::from_str::<RecordingProfile>("{}").is_err());
  }

  #[test]
  fn recording_profile_rejects_invalid_numeric_values() {
    let profile = RecordingProfile {
      keyframe_interval_seconds: 0,
      ..Default::default()
    };
    assert!(!profile.is_valid());
  }

  #[test]
  fn recording_profile_rejects_invalid_persisted_options() {
    assert!(
      serde_json::from_str::<RecordingProfile>(
        r#"{
        "popup":"all",
        "auto_recording":"off",
        "auto_split":"minutes3",
        "capture_frame_rate":"fps59",
        "export_frame_rate":"fps24",
        "quality":"lossless",
        "keyframe_interval_seconds":99,
        "pixel_scale":"triple",
        "gpu_acceleration":"unknown"
      }"#,
      )
      .is_err()
    );
  }

  #[test]
  fn reading_invalid_recording_profile_uses_default_without_rewriting_disk() {
    let storage = temp_storage("recording_profile_invalid");
    let mut log = LogService::new();
    let invalid = r#"{"capture_frame_rate":"fps59"}"#;
    fs::write(storage.profile_recording_path(), invalid).unwrap();

    assert_eq!(storage.read_recording_profile(&mut log), None);
    assert_eq!(
      storage.read_recording_profile_or_default(&mut log),
      RecordingProfile::default()
    );
    assert_eq!(
      fs::read_to_string(storage.profile_recording_path()).unwrap(),
      invalid
    );
  }

  #[test]
  fn recording_export_frame_rate_prefers_recorded_value_and_supports_fixed_values() {
    assert_eq!(RecordingExportFrameRate::Recorded.resolve(120), 120);
    assert_eq!(RecordingExportFrameRate::Fps60.resolve(120), 60);
  }

  #[test]
  fn invalid_display_settings_use_defaults_without_rewriting_disk() {
    let mut storage = temp_storage("display_settings_repair");
    let mut log = LogService::new();
    let invalid = r#"{"logo_mode":"neon","top_toolbar":"invalid","game_list_source":"mod"}"#;
    fs::write(storage.profile_display_settings_path(), invalid).unwrap();

    let profile = storage.reload_display_settings_profile(&mut log);
    assert_eq!(profile, DisplaySettingsProfile::default());
    assert_eq!(
      fs::read_to_string(storage.profile_display_settings_path()).unwrap(),
      invalid
    );
  }

  #[test]
  fn display_settings_write_updates_cache_with_current_schema() {
    let mut storage = temp_storage("display_settings_write");
    let mut log = LogService::new();
    let profile = DisplaySettingsProfile {
      game_list_source: DisplaySourceMode::Official,
      game_list_warnings: false,
      top_toolbar_custom_text: "f%<fg:red>LIVE</fg>".to_string(),
      ..Default::default()
    };

    storage
      .write_display_settings_profile(&profile, &mut log)
      .unwrap();
    assert_eq!(storage.display_settings_profile(), &profile);
    let json: Value =
      serde_json::from_str(&fs::read_to_string(storage.profile_display_settings_path()).unwrap())
        .unwrap();
    assert!(json.get("custom_field").is_none());
    assert_eq!(json["game_list_source"], "official");
    assert_eq!(json["game_list_warnings"], false);
    assert_eq!(json["top_toolbar_custom_text"], "f%<fg:red>LIVE</fg>");
  }
}
