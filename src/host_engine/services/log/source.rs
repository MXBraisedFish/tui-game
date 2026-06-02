#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogSource {
  Engine,   // 引擎
  Boot,     // 启动阶段
  Runtime,  // 运行阶段
  Shutdown, // 关闭阶段
  Termianl, // 终端服务
  Render,   // 绘制服务
  Input,    // 输入服务
  Storage,  // 数据服务
  Pack,     // 包服务
  Lua,      // Lua服务
  Game,     // 游戏服务
  Overlay,  // 覆盖屏幕服务
  Ui,       // 引擎UI服务
  Crash,    // 异常恢复服务
  I18n,     // 国际化服务
}
