use std::{collections::HashMap, fs, io};

use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

use super::layout;
use super::service::StorageService;
use crate::host_engine::services::{LogService, LogSource};

/// 终端配置文件：存储 Unicode 支持、颜色模式和鼠标支持的用户偏好。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerminalProfile {
  pub unicode: Option<bool>,

  pub color: Option<String>,

  pub mouse: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageStateProfile {
  #[serde(default)]
  pub defaults: PackageDefaultState,

  #[serde(default)]
  pub games: HashMap<String, GamePackageState>,

  #[serde(default)]
  pub screensavers: HashMap<String, ScreensaverPackageState>,
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

fn default_screenshot_double_action() -> ScreenshotDoubleAction {
  ScreenshotDoubleAction::SavePng
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScreenshotProfile {
  #[serde(default)]
  pub guide_seen: bool,

  #[serde(default = "default_screenshot_double_action")]
  pub double_action: ScreenshotDoubleAction,

  #[serde(default)]
  pub auto_exit: bool,

  /// 截屏导出时按顺序尝试的自定义字体路径或系统字体名称。
  #[serde(default)]
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutoSplitDuration {
  Off,
  Minutes5,
  Minutes10,
}

impl AutoSplitDuration {
  pub fn next(self) -> Self {
    match self {
      Self::Off => Self::Minutes5,
      Self::Minutes5 => Self::Minutes10,
      Self::Minutes10 => Self::Off,
    }
  }

  pub fn duration(self) -> Option<std::time::Duration> {
    match self {
      Self::Off => None,
      Self::Minutes5 => Some(std::time::Duration::from_secs(5 * 60)),
      Self::Minutes10 => Some(std::time::Duration::from_secs(10 * 60)),
    }
  }
}

impl Default for AutoSplitDuration {
  fn default() -> Self {
    Self::Minutes10
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
  pub fn resolve(self, recorded: Option<u16>, legacy: u16) -> u16 {
    match self {
      Self::Recorded => recorded.unwrap_or(legacy),
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordingProfile {
  /// 录屏导出时按顺序尝试的自定义字体路径或系统字体名称。
  #[serde(default)]
  pub fonts: Vec<String>,
  #[serde(default, deserialize_with = "deserialize_or_default")]
  pub popup: RecordingPopupMode,
  #[serde(default, deserialize_with = "deserialize_or_default")]
  pub auto_recording: AutoRecordingMode,
  #[serde(default, deserialize_with = "deserialize_or_default")]
  pub auto_split: AutoSplitDuration,
  #[serde(default, deserialize_with = "deserialize_or_default")]
  pub capture_frame_rate: RecordingFrameRate,
  #[serde(default, deserialize_with = "deserialize_or_default")]
  pub export_frame_rate: RecordingExportFrameRate,
  #[serde(
    default = "default_legacy_frame_rate",
    deserialize_with = "deserialize_legacy_frame_rate"
  )]
  pub legacy_frame_rate: u16,
  #[serde(default, deserialize_with = "deserialize_or_default")]
  pub quality: RecordingExportQuality,
  #[serde(
    default = "default_keyframe_interval",
    deserialize_with = "deserialize_keyframe_interval"
  )]
  pub keyframe_interval_seconds: u16,
  #[serde(default, deserialize_with = "deserialize_or_default")]
  pub pixel_scale: RecordingPixelScale,
}

fn deserialize_or_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
  D: Deserializer<'de>,
  T: DeserializeOwned + Default,
{
  let value = Value::deserialize(deserializer)?;
  Ok(serde_json::from_value(value).unwrap_or_default())
}

fn deserialize_legacy_frame_rate<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
  D: Deserializer<'de>,
{
  let value = Value::deserialize(deserializer)?;
  Ok(
    serde_json::from_value(value)
      .ok()
      .filter(|value| matches!(value, 30 | 60 | 120))
      .unwrap_or_else(default_legacy_frame_rate),
  )
}

fn deserialize_keyframe_interval<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
  D: Deserializer<'de>,
{
  let value = Value::deserialize(deserializer)?;
  Ok(
    serde_json::from_value(value)
      .ok()
      .filter(|value| (1..=10).contains(value))
      .unwrap_or_else(default_keyframe_interval),
  )
}

fn default_legacy_frame_rate() -> u16 {
  30
}

fn default_keyframe_interval() -> u16 {
  2
}

impl RecordingProfile {
  pub fn repair(&mut self) -> bool {
    let mut repaired = false;
    if !matches!(self.legacy_frame_rate, 30 | 60 | 120) {
      self.legacy_frame_rate = default_legacy_frame_rate();
      repaired = true;
    }
    if !(1..=10).contains(&self.keyframe_interval_seconds) {
      self.keyframe_interval_seconds = default_keyframe_interval();
      repaired = true;
    }
    repaired
  }
}

impl Default for RecordingProfile {
  fn default() -> Self {
    Self {
      fonts: Vec::new(),
      popup: RecordingPopupMode::default(),
      auto_recording: AutoRecordingMode::default(),
      auto_split: AutoSplitDuration::default(),
      capture_frame_rate: RecordingFrameRate::default(),
      export_frame_rate: RecordingExportFrameRate::default(),
      legacy_frame_rate: default_legacy_frame_rate(),
      quality: RecordingExportQuality::default(),
      keyframe_interval_seconds: default_keyframe_interval(),
      pixel_scale: RecordingPixelScale::default(),
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisplaySettingsProfile {
  pub logo_mode: DisplayLogoMode,
  #[serde(default)]
  pub logo_sequence_cursor: u64,
  pub top_toolbar: bool,
  #[serde(default)]
  pub top_toolbar_custom_text: String,
  pub screensaver_source: DisplaySourceMode,
  pub screensaver_order: DisplayOrderMode,
  #[serde(default)]
  pub screensaver_sequence_cursor: u64,
  pub game_list_source: DisplaySourceMode,
  pub game_list_warnings: bool,
  pub game_list_fps: DisplayFpsLimit,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SafeModeDefault {
  On,
  #[serde(alias = "off_temporary")]
  OffPermanent,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageDefaultState {
  #[serde(default = "default_enabled")]
  pub enabled: bool,

  #[serde(default)]
  pub debug: bool,

  #[serde(default)]
  pub safe_mode: SafeModeDefault,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GamePackageState {
  #[serde(default = "default_enabled")]
  pub enabled: bool,

  #[serde(default)]
  pub debug: bool,

  #[serde(default = "default_safe_mode")]
  pub safe_mode: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScreensaverPackageState {
  /// 包管理器总开关：关闭后不进入屏保列表。
  #[serde(default = "default_enabled")]
  pub enabled: bool,

  #[serde(default)]
  pub debug: bool,

  /// 屏保列表中的局内启用状态，与包总开关相互独立。
  #[serde(default = "default_enabled")]
  pub playlist_enabled: bool,

  /// 已启用屏保的显示顺序；未启用时不参与排序。
  #[serde(default)]
  pub order: Option<u32>,
}

impl Default for GamePackageState {
  fn default() -> Self {
    Self {
      enabled: true,
      debug: false,
      safe_mode: true,
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

impl Default for SafeModeDefault {
  fn default() -> Self {
    Self::On
  }
}

impl Default for PackageDefaultState {
  fn default() -> Self {
    Self {
      enabled: true,
      debug: false,
      safe_mode: SafeModeDefault::On,
    }
  }
}

fn default_enabled() -> bool {
  true
}

fn default_safe_mode() -> bool {
  true
}

impl Default for TerminalProfile {
  fn default() -> Self {
    Self {
      unicode: None,
      color: None,
      mouse: None,
    }
  }
}

impl TerminalProfile {
  /// 检查三项配置是否已全部填写完毕。
  pub fn is_complete(&self) -> bool {
    self.unicode.is_some()
      && self
        .color
        .as_deref()
        .map_or(false, |c| c == "truecolor" || c == "256")
      && self.mouse.is_some()
  }
}

impl StorageService {
  /// 读取保存的语言代码。
  pub fn read_language_code(&self, log: &mut LogService) -> Option<String> {
    let content = fs::read_to_string(self.profile_language_path())
      .map_err(|error| {
        log.warn(
          LogSource::Storage,
          format!("Failed to read language code: {err}", err = error),
        );
        error
      })
      .ok()?;
    let code = content.trim();
    if code.is_empty() {
      None
    } else {
      Some(code.to_string())
    }
  }

  /// 写入语言代码到配置文件。
  pub fn write_language_code(&self, language_code: &str) -> std::io::Result<()> {
    fs::write(self.profile_language_path(), language_code.trim())
  }

  /// 返回默认语言代码。
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
    let mut values = read_json_object(&path);
    let mut repaired = false;
    let defaults = DisplaySettingsProfile::default();
    let profile = DisplaySettingsProfile {
      logo_mode: read_profile_field(&mut values, "logo_mode", defaults.logo_mode, &mut repaired),
      logo_sequence_cursor: read_profile_field(
        &mut values,
        "logo_sequence_cursor",
        defaults.logo_sequence_cursor,
        &mut repaired,
      ),
      top_toolbar: read_profile_field(
        &mut values,
        "top_toolbar",
        defaults.top_toolbar,
        &mut repaired,
      ),
      top_toolbar_custom_text: read_profile_field(
        &mut values,
        "top_toolbar_custom_text",
        defaults.top_toolbar_custom_text,
        &mut repaired,
      ),
      screensaver_source: read_profile_field(
        &mut values,
        "screensaver_source",
        defaults.screensaver_source,
        &mut repaired,
      ),
      screensaver_order: read_profile_field(
        &mut values,
        "screensaver_order",
        defaults.screensaver_order,
        &mut repaired,
      ),
      screensaver_sequence_cursor: read_profile_field(
        &mut values,
        "screensaver_sequence_cursor",
        defaults.screensaver_sequence_cursor,
        &mut repaired,
      ),
      game_list_source: read_profile_field(
        &mut values,
        "game_list_source",
        defaults.game_list_source,
        &mut repaired,
      ),
      game_list_warnings: read_profile_field(
        &mut values,
        "game_list_warnings",
        defaults.game_list_warnings,
        &mut repaired,
      ),
      game_list_fps: read_profile_field(
        &mut values,
        "game_list_fps",
        defaults.game_list_fps,
        &mut repaired,
      ),
    };

    if repaired {
      write_json_object(&path, &values, log, "display settings profile");
    }
    self.display_settings = profile.clone();
    profile
  }

  pub fn write_display_settings_profile(
    &mut self,
    profile: &DisplaySettingsProfile,
    log: &mut LogService,
  ) -> io::Result<()> {
    let path = self.profile_display_settings_path();
    let mut values = read_json_object(&path);
    set_profile_field(&mut values, "logo_mode", profile.logo_mode);
    set_profile_field(
      &mut values,
      "logo_sequence_cursor",
      profile.logo_sequence_cursor,
    );
    set_profile_field(&mut values, "top_toolbar", profile.top_toolbar);
    set_profile_field(
      &mut values,
      "top_toolbar_custom_text",
      &profile.top_toolbar_custom_text,
    );
    set_profile_field(
      &mut values,
      "screensaver_source",
      profile.screensaver_source,
    );
    set_profile_field(&mut values, "screensaver_order", profile.screensaver_order);
    set_profile_field(
      &mut values,
      "screensaver_sequence_cursor",
      profile.screensaver_sequence_cursor,
    );
    set_profile_field(&mut values, "game_list_source", profile.game_list_source);
    set_profile_field(
      &mut values,
      "game_list_warnings",
      profile.game_list_warnings,
    );
    set_profile_field(&mut values, "game_list_fps", profile.game_list_fps);
    let content = serde_json::to_string_pretty(&values).map_err(io::Error::other)?;
    fs::write(&path, content).map_err(|error| {
      log.warn(
        LogSource::Storage,
        format!("Failed to write display settings profile: {error}"),
      );
      error
    })?;
    self.display_settings = profile.clone();
    Ok(())
  }

  /// 从文件读取终端配置。
  pub fn read_terminal_profile(&self, log: &mut LogService) -> Option<TerminalProfile> {
    let content = fs::read_to_string(self.profile_terminal_path())
      .map_err(|error| {
        log.warn(
          LogSource::Storage,
          format!("Failed to read terminal profile: {err}", err = error),
        );
        error
      })
      .ok()?;
    serde_json::from_str(&content)
      .map_err(|error| {
        log.warn(
          LogSource::Storage,
          format!("Failed to parse terminal profile JSON: {err}", err = error),
        );
        error
      })
      .ok()
  }

  /// 读取终端配置，缺失时返回默认值。
  pub fn read_terminal_profile_or_default(&self, log: &mut LogService) -> TerminalProfile {
    self.read_terminal_profile(log).unwrap_or_default()
  }

  /// 读取并修改终端配置后写回。
  pub fn update_terminal_profile(
    &self,
    log: &mut LogService,
    f: impl FnOnce(&mut TerminalProfile),
  ) -> std::io::Result<()> {
    let mut profile = self.read_terminal_profile_or_default(log);
    f(&mut profile);
    self.write_terminal_profile(&profile, log)
  }

  /// 将终端配置序列化后写入文件。
  pub fn write_terminal_profile(
    &self,
    profile: &TerminalProfile,
    log: &mut LogService,
  ) -> std::io::Result<()> {
    let json = match serde_json::to_string_pretty(profile) {
      Ok(json) => json,
      Err(error) => {
        log.error(
          LogSource::Storage,
          format!("Failed to serialize terminal profile: {err}", err = error),
        );
        return Err(io::Error::new(
          io::ErrorKind::InvalidData,
          format!("Serialization failed: {error}"),
        ));
      }
    };
    fs::write(self.profile_terminal_path(), json)
  }

  /// 检查终端配置文件是否已填写完整。
  pub fn is_terminal_profile_complete(&self, log: &mut LogService) -> bool {
    self
      .read_terminal_profile(log)
      .map_or(false, |p| p.is_complete())
  }

  pub fn read_package_state(&self, log: &mut LogService) -> Option<PackageStateProfile> {
    let content = fs::read_to_string(self.profile_package_state_path())
      .map_err(|error| {
        log.warn(
          LogSource::Storage,
          format!("Failed to read package state: {err}", err = error),
        );
        error
      })
      .ok()?;
    serde_json::from_str(&content)
      .map_err(|error| {
        log.warn(
          LogSource::Storage,
          format!("Failed to parse package state JSON: {err}", err = error),
        );
        error
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
        log.error(
          LogSource::Storage,
          format!("Failed to serialize package state: {err}", err = error),
        );
        return Err(io::Error::new(
          io::ErrorKind::InvalidData,
          format!("Serialization failed: {error}"),
        ));
      }
    };
    fs::write(self.profile_package_state_path(), json)
  }

  pub fn update_game_package_state(
    &self,
    mod_id: &str,
    log: &mut LogService,
    f: impl FnOnce(&mut GamePackageState),
  ) -> std::io::Result<()> {
    let mut profile = self.read_package_state_or_default(log);
    let defaults = &profile.defaults;
    let initial = GamePackageState {
      enabled: defaults.enabled,
      debug: defaults.debug,
      safe_mode: defaults.safe_mode == SafeModeDefault::On,
    };
    f(profile.games.entry(mod_id.to_string()).or_insert(initial));
    self.write_package_state(&profile, log)
  }

  pub fn update_screensaver_package_state(
    &self,
    mod_id: &str,
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
      .entry(mod_id.to_string())
      .or_insert(initial));
    self.write_package_state(&profile, log)
  }

  pub fn read_screenshot_profile(&self, log: &mut LogService) -> Option<ScreenshotProfile> {
    let content = fs::read_to_string(self.profile_screenshot_path())
      .map_err(|error| {
        log.warn(
          LogSource::Storage,
          format!("Failed to read screenshot profile: {err}", err = error),
        );
        error
      })
      .ok()?;
    serde_json::from_str(&content)
      .map_err(|error| {
        log.warn(
          LogSource::Storage,
          format!(
            "Failed to parse screenshot profile JSON: {err}",
            err = error
          ),
        );
        error
      })
      .ok()
  }

  pub fn read_recording_profile(&self, log: &mut LogService) -> Option<RecordingProfile> {
    let content = fs::read_to_string(self.profile_recording_path())
      .map_err(|error| {
        log.warn(
          LogSource::Storage,
          format!("Failed to read recording profile: {error}"),
        );
        error
      })
      .ok()?;
    let original = serde_json::from_str::<Value>(&content)
      .map_err(|error| {
        log.warn(
          LogSource::Storage,
          format!("Failed to parse recording profile JSON: {error}"),
        );
        error
      })
      .ok()?;
    let mut profile = serde_json::from_value::<RecordingProfile>(original.clone())
      .map_err(|error| {
        log.warn(
          LogSource::Storage,
          format!("Failed to parse recording profile values: {error}"),
        );
        error
      })
      .ok()?;
    let mut repaired = profile.repair();
    let normalized = serde_json::to_value(&profile).ok()?;
    repaired |= [
      "fonts",
      "popup",
      "auto_recording",
      "auto_split",
      "capture_frame_rate",
      "export_frame_rate",
      "legacy_frame_rate",
      "quality",
      "keyframe_interval_seconds",
      "pixel_scale",
    ]
    .into_iter()
    .any(|field| original.get(field) != normalized.get(field));
    if repaired {
      let _ = self.write_recording_profile(&profile, log);
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
      log.error(
        LogSource::Storage,
        format!("Failed to serialize recording profile: {error}"),
      );
      io::Error::new(io::ErrorKind::InvalidData, error)
    })?;
    let result = fs::write(self.profile_recording_path(), json);
    if result.is_ok() {
      self
        .recording_profile_revision
        .set(self.recording_profile_revision.get().wrapping_add(1));
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
      log.error(
        LogSource::Storage,
        format!("Failed to serialize screenshot profile: {err}", err = error),
      );
      io::Error::new(
        io::ErrorKind::InvalidData,
        format!("Serialization failed: {error}"),
      )
    })?;
    fs::write(self.profile_screenshot_path(), json)
  }

  pub fn mark_screenshot_guide_seen(&self, log: &mut LogService) {
    let mut profile = self.read_screenshot_profile_or_default(log);
    if profile.guide_seen {
      return;
    }
    profile.guide_seen = true;
    if let Err(error) = self.write_screenshot_profile(&profile, log) {
      log.warn(
        LogSource::Storage,
        format!("Failed to write screenshot profile: {error}"),
      );
    }
  }
}

fn read_json_object(path: &std::path::Path) -> Map<String, Value> {
  fs::read_to_string(path)
    .ok()
    .and_then(|content| serde_json::from_str::<Value>(&content).ok())
    .and_then(|value| value.as_object().cloned())
    .unwrap_or_default()
}

fn read_profile_field<T>(
  values: &mut Map<String, Value>,
  key: &str,
  default: T,
  repaired: &mut bool,
) -> T
where
  T: Clone + DeserializeOwned + Serialize,
{
  if let Some(value) = values
    .get(key)
    .and_then(|value| serde_json::from_value(value.clone()).ok())
  {
    return value;
  }
  set_profile_field(values, key, default.clone());
  *repaired = true;
  default
}

fn set_profile_field<T: Serialize>(values: &mut Map<String, Value>, key: &str, value: T) {
  if let Ok(value) = serde_json::to_value(value) {
    values.insert(key.to_string(), value);
  }
}

fn write_json_object(
  path: &std::path::Path,
  values: &Map<String, Value>,
  log: &mut LogService,
  name: &str,
) {
  let result = serde_json::to_string_pretty(values)
    .map_err(io::Error::other)
    .and_then(|content| fs::write(path, content));
  if let Err(error) = result {
    log.warn(
      LogSource::Storage,
      format!("Failed to repair {name}: {error}"),
    );
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
  fn package_state_persists_game_and_screensaver_independently() {
    let storage = temp_storage("package_state_persists");
    let mut log = LogService::new();

    storage
      .update_game_package_state("same_id", &mut log, |state| {
        state.enabled = false;
        state.debug = true;
        state.safe_mode = false;
      })
      .unwrap();
    storage
      .update_screensaver_package_state("same_id", &mut log, |state| {
        state.enabled = false;
        state.debug = true;
      })
      .unwrap();

    let profile = storage.read_package_state_or_default(&mut log);
    assert_eq!(
      profile.games.get("same_id"),
      Some(&GamePackageState {
        enabled: false,
        debug: true,
        safe_mode: false,
      })
    );
    assert_eq!(
      profile.screensavers.get("same_id"),
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
    let mut profile = PackageStateProfile::default();
    profile.defaults = PackageDefaultState {
      enabled: false,
      debug: true,
      safe_mode: SafeModeDefault::OffPermanent,
    };
    storage.write_package_state(&profile, &mut log).unwrap();

    storage
      .update_game_package_state("game", &mut log, |state| state.debug = false)
      .unwrap();
    storage
      .update_screensaver_package_state("screen", &mut log, |state| state.debug = false)
      .unwrap();

    let profile = storage.read_package_state_or_default(&mut log);
    assert_eq!(profile.defaults.safe_mode, SafeModeDefault::OffPermanent);
    assert_eq!(profile.games["game"].enabled, false);
    assert_eq!(profile.games["game"].safe_mode, false);
    assert_eq!(profile.screensavers["screen"].enabled, false);
  }

  #[test]
  fn old_package_profile_uses_default_settings() {
    let profile: PackageStateProfile =
      serde_json::from_str(r#"{"games":{},"screensavers":{}}"#).unwrap();
    assert_eq!(profile.defaults, PackageDefaultState::default());
  }

  #[test]
  fn temporary_default_from_older_profile_becomes_permanent() {
    let profile: PackageStateProfile = serde_json::from_str(
      r#"{"defaults":{"enabled":true,"debug":false,"safe_mode":"off_temporary"}}"#,
    )
    .unwrap();
    assert_eq!(profile.defaults.safe_mode, SafeModeDefault::OffPermanent);
  }

  #[test]
  fn old_screenshot_profile_receives_new_defaults() {
    let profile: ScreenshotProfile = serde_json::from_str(r#"{"guide_seen":true}"#).unwrap();
    assert!(profile.guide_seen);
    assert_eq!(profile.double_action, ScreenshotDoubleAction::SavePng);
    assert!(!profile.auto_exit);
  }

  #[test]
  fn old_recording_profile_receives_export_defaults() {
    let profile: RecordingProfile = serde_json::from_str("{}").unwrap();
    assert_eq!(profile.capture_frame_rate, RecordingFrameRate::Fps60);
    assert_eq!(
      profile.export_frame_rate,
      RecordingExportFrameRate::Recorded
    );
    assert_eq!(profile.legacy_frame_rate, 30);
    assert_eq!(profile.quality, RecordingExportQuality::Balanced);
    assert_eq!(profile.keyframe_interval_seconds, 2);
    assert_eq!(profile.pixel_scale, RecordingPixelScale::Original);
  }

  #[test]
  fn recording_profile_repairs_invalid_numeric_values() {
    let mut profile = RecordingProfile {
      legacy_frame_rate: 59,
      keyframe_interval_seconds: 0,
      ..Default::default()
    };
    assert!(profile.repair());
    assert_eq!(profile.legacy_frame_rate, 30);
    assert_eq!(profile.keyframe_interval_seconds, 2);
    assert!(!profile.repair());
  }

  #[test]
  fn recording_profile_tolerates_invalid_persisted_options() {
    let profile: RecordingProfile = serde_json::from_str(
      r#"{
        "capture_frame_rate":"fps59",
        "export_frame_rate":"fps24",
        "legacy_frame_rate":"bad",
        "quality":"lossless",
        "keyframe_interval_seconds":99,
        "pixel_scale":"triple"
      }"#,
    )
    .unwrap();
    assert_eq!(profile, RecordingProfile::default());
  }

  #[test]
  fn reading_recording_profile_persists_repaired_values() {
    let storage = temp_storage("recording_profile_repair");
    let mut log = LogService::new();
    fs::write(
      storage.profile_recording_path(),
      r#"{"capture_frame_rate":"fps59","pixel_scale":"triple","keyframe_interval_seconds":99}"#,
    )
    .unwrap();

    assert_eq!(
      storage.read_recording_profile(&mut log),
      Some(RecordingProfile::default())
    );
    let value: Value =
      serde_json::from_str(&fs::read_to_string(storage.profile_recording_path()).unwrap()).unwrap();
    assert_eq!(value["capture_frame_rate"], "fps60");
    assert_eq!(value["pixel_scale"], "original");
    assert_eq!(value["keyframe_interval_seconds"], 2);
  }

  #[test]
  fn recording_export_frame_rate_prefers_recorded_value_and_supports_fixed_values() {
    assert_eq!(
      RecordingExportFrameRate::Recorded.resolve(Some(120), 30),
      120
    );
    assert_eq!(RecordingExportFrameRate::Recorded.resolve(None, 30), 30);
    assert_eq!(RecordingExportFrameRate::Fps60.resolve(Some(120), 30), 60);
  }

  #[test]
  fn display_settings_repairs_only_missing_or_invalid_fields() {
    let mut storage = temp_storage("display_settings_repair");
    let mut log = LogService::new();
    fs::write(
      storage.profile_display_settings_path(),
      r#"{
        "logo_mode": "neon",
        "top_toolbar": "invalid",
        "game_list_source": "mod",
        "custom_field": 7
      }"#,
    )
    .unwrap();

    let profile = storage.reload_display_settings_profile(&mut log);
    assert_eq!(profile.logo_mode, DisplayLogoMode::Neon);
    assert!(profile.top_toolbar);
    assert_eq!(profile.game_list_source, DisplaySourceMode::Mod);
    assert!(profile.game_list_warnings);
    assert!(profile.top_toolbar_custom_text.is_empty());

    let json: Value =
      serde_json::from_str(&fs::read_to_string(storage.profile_display_settings_path()).unwrap())
        .unwrap();
    assert_eq!(json["custom_field"], 7);
    assert_eq!(json["top_toolbar"], true);
    assert_eq!(json["top_toolbar_custom_text"], "");
    assert_eq!(json["screensaver_source"], "all");
  }

  #[test]
  fn display_settings_write_updates_cache_and_preserves_unknown_fields() {
    let mut storage = temp_storage("display_settings_write");
    let mut log = LogService::new();
    fs::write(
      storage.profile_display_settings_path(),
      r#"{"custom_field":"keep"}"#,
    )
    .unwrap();
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
    assert_eq!(json["custom_field"], "keep");
    assert_eq!(json["game_list_source"], "official");
    assert_eq!(json["game_list_warnings"], false);
    assert_eq!(json["top_toolbar_custom_text"], "f%<fg:red>LIVE</fg>");
  }
}
