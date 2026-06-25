
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
