use super::LogPhase;

/// 日志来源分类，标识产生日志的子系统。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogSource {
  Engine,
  Boot,
  Runtime,
  Shutdown,
  Termianl,
  Render,
  Input,
  Storage,
  Pack,
  Lua,
  Game,
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
      Self::Termianl => "log.service.terminal",
      Self::Render => "log.service.render",
      Self::Input => "log.service.input",
      Self::Storage => "log.service.storage",
      Self::Pack => "log.service.package",
      Self::Lua => "log.service.lua",
      Self::Game => "log.service.game",
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
      Self::Termianl => "Terminal",
      Self::Render => "Render",
      Self::Input => "Input",
      Self::Storage => "Storage",
      Self::Pack => "Package",
      Self::Lua => "Lua",
      Self::Game => "Game",
      Self::Overlay => "Overlay",
      Self::Ui => "Ui",
      Self::Crash => "Crash",
      Self::I18n => "I18n",
    }
  }
}
