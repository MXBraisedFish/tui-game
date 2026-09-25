use std::{
  collections::{BTreeMap, HashMap},
  fs, io,
  path::Path,
};

use serde::{Deserialize, Serialize};

use tg_core_atomic_fs::atomic_write;
use super::layout;
use super::service::StorageService;
use tg_core_package_id::PackageId;
use tg_service_log::{HostLogMessage, LogService, LogSource};

/// 终端配置文件：存储 Unicode 支持、颜色模式和鼠标支持的用户偏好。
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// 按键映射持久化表。default 保存包或宿主的原始定义，user 保存实际生效的用户映射。
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

  /// 截屏导出时按顺序尝试的自定义字体路径或系统字体名称。
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

impl Default for AutoSplitDuration {
  fn default() -> Self {
    Self::Minutes3
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
  /// Next value in the settings page cycling order.
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
  /// Next value in the settings page cycling order.
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
  /// Next value in the settings page cycling order.
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

  /// Next value in the settings page cycling order.
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SafeModeDefault {
  On,
  OffPermanent,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackageDefaultState {
  pub enabled: bool,
  pub debug: bool,
  pub safe_mode: SafeModeDefault,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GamePackageState {
  pub enabled: bool,
  pub debug: bool,
  pub safe_mode: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScreensaverPackageState {
  /// 包管理器总开关：关闭后不进入屏保列表。
  pub enabled: bool,
  pub debug: bool,

  /// 屏保列表中的局内启用状态，与包总开关相互独立。
  pub playlist_enabled: bool,

  /// 已启用屏保的显示顺序；未启用时不参与排序。
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
        log_profile_read_error(log, "language", &self.profile_language_path(), &error);
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
    atomic_write(&path, json.as_bytes(), true).map_err(|error| {
      log.error_operation_failed(
        LogSource::Storage,
        "write_profile",
        "key_bindings",
        error.to_string(),
      );
      error
    })?;
    log_profile_change(log, "key_bindings", changed);
    Ok(())
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
    atomic_write(&path, content.as_bytes(), true).map_err(|error| {
      log.warn_operation_failed(
        LogSource::Storage,
        "write_profile",
        "display_settings",
        error.to_string(),
      );
      error
    })?;
    self.display_settings = profile.clone();
    log_profile_change(log, "display_settings", changed);
    Ok(())
  }

  /// 从文件读取终端配置。
  pub fn read_terminal_profile(&self, log: &mut LogService) -> Option<TerminalProfile> {
    let content = fs::read_to_string(self.profile_terminal_path())
      .map_err(|error| {
        log_profile_read_error(log, "terminal", &self.profile_terminal_path(), &error);
        error
      })
      .ok()?;
    serde_json::from_str(&content)
      .map_err(|error| {
        log.warn_operation_failed(
          LogSource::Storage,
          "parse_profile",
          "terminal",
          error.to_string(),
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

  /// 清空已保存的终端能力检测结果，使下次启动重新进入能力检测流程。
  pub fn reset_terminal_profile(&self, log: &mut LogService) -> std::io::Result<()> {
    self.write_terminal_profile(&TerminalProfile::default(), log)
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
        log_profile_read_error(
          log,
          "package_state",
          &self.profile_package_state_path(),
          &error,
        );
        error
      })
      .ok()?;
    serde_json::from_str(&content)
      .map_err(|error| {
        log.warn_operation_failed(
          LogSource::Storage,
          "parse_profile",
          "package_state",
          error.to_string(),
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
      safe_mode: defaults.safe_mode == SafeModeDefault::On,
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
      .map_err(|error| {
        log_profile_read_error(log, "screenshot", &self.profile_screenshot_path(), &error);
        error
      })
      .ok()?;
    serde_json::from_str(&content)
      .map_err(|error| {
        log.warn_operation_failed(
          LogSource::Storage,
          "parse_profile",
          "screenshot",
          error.to_string(),
        );
        error
      })
      .ok()
  }

  pub fn read_recording_profile(&self, log: &mut LogService) -> Option<RecordingProfile> {
    let content = fs::read_to_string(self.profile_recording_path())
      .map_err(|error| {
        log_profile_read_error(log, "recording", &self.profile_recording_path(), &error);
        error
      })
      .ok()?;
    let profile = serde_json::from_str::<RecordingProfile>(&content)
      .map_err(|error| {
        log.warn_operation_failed(
          LogSource::Storage,
          "parse_profile",
          "recording",
          error.to_string(),
        );
        error
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
    assert_eq!(DisplayOrderMode::Random.next().next(), DisplayOrderMode::Random);
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
        state.safe_mode = false;
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
        safe_mode: false,
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
    let mut profile = PackageStateProfile::default();
    profile.defaults = PackageDefaultState {
      enabled: false,
      debug: true,
      safe_mode: SafeModeDefault::OffPermanent,
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
    assert_eq!(profile.defaults.safe_mode, SafeModeDefault::OffPermanent);
    assert_eq!(profile.games[&game_id.storage_key()].enabled, false);
    assert_eq!(profile.games[&game_id.storage_key()].safe_mode, false);
    assert_eq!(
      profile.screensavers[&screensaver_id.storage_key()].enabled,
      false
    );
  }

  #[test]
  fn incomplete_package_profile_is_rejected() {
    assert!(
      serde_json::from_str::<PackageStateProfile>(r#"{"games":{},"screensavers":{}}"#).is_err()
    );
  }

  #[test]
  fn removed_safe_mode_value_is_rejected() {
    assert!(serde_json::from_str::<SafeModeDefault>(r#""off_temporary""#).is_err());
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
