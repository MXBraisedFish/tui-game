use super::LogPhase;

/// 日志来源分类，标识产生日志的子系统。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogSource {
  Engine,
  Boot,
  Runtime,
  Shutdown,
  Terminal,
  Render,
  Input,
  Storage,
  Audio,
  Pack,
  Lua,
  Game,
  Screensaver,
  Overlay,
  Ui,
  Crash,
  I18n,
}

impl LogSource {
  pub fn phase(self) -> LogPhase {
    match self {
      Self::Boot => LogPhase::Boot,
      Self::Shutdown => LogPhase::Shutdown,
      Self::Crash => LogPhase::Crash,
      _ => LogPhase::Runtime,
    }
  }

  pub fn key(self) -> &'static str {
    match self {
      Self::Engine => "log.service.engine",
      Self::Boot => "log.service.boot",
      Self::Runtime => "log.service.runtime",
      Self::Shutdown => "log.service.shutdown",
      Self::Terminal => "log.service.terminal",
      Self::Render => "log.service.render",
      Self::Input => "log.service.input",
      Self::Storage => "log.service.storage",
      Self::Audio => "log.service.audio",
      Self::Pack => "log.service.package",
      Self::Lua => "log.service.lua",
      Self::Game => "log.service.game",
      Self::Screensaver => "log.service.screensaver",
      Self::Overlay => "log.service.overlay",
      Self::Ui => "log.service.ui",
      Self::Crash => "log.service.crash",
      Self::I18n => "log.service.i18n",
    }
  }

  pub fn default_label(self) -> &'static str {
    match self {
      Self::Engine => "Engine",
      Self::Boot => "Boot",
      Self::Runtime => "Runtime",
      Self::Shutdown => "Shutdown",
      Self::Terminal => "Terminal",
      Self::Render => "Render",
      Self::Input => "Input",
      Self::Storage => "Storage",
      Self::Audio => "Audio",
      Self::Pack => "Package",
      Self::Lua => "Lua",
      Self::Game => "Game",
      Self::Screensaver => "Screensaver",
      Self::Overlay => "Overlay",
      Self::Ui => "Ui",
      Self::Crash => "Crash",
      Self::I18n => "I18n",
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{LogLevel, format_log_level};

  #[test]
  fn lifecycle_sources_map_to_their_phase_and_others_to_runtime() {
    assert_eq!(LogSource::Boot.phase(), LogPhase::Boot);
    assert_eq!(LogSource::Shutdown.phase(), LogPhase::Shutdown);
    assert_eq!(LogSource::Crash.phase(), LogPhase::Crash);
    assert_eq!(LogSource::Lua.phase(), LogPhase::Runtime);
    assert_eq!(LogSource::Pack.key(), "log.service.package");
    assert_eq!(LogSource::Pack.default_label(), "Package");
    assert_eq!(format_log_level(LogLevel::Warn), "WARN");
    assert_eq!(LogLevel::Fatal.key(), "log.level.fatal");
  }
}
