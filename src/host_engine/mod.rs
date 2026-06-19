//! 宿主引擎 — 顶层模块
//!
//! 本模块是程序核心的入口，负责组织三个生命周期阶段：
//! 1. **Boot（启动）** — 初始化服务与游戏世界
//! 2. **Runtime（运行）** — 主事件循环
//! 3. **Shutdown（关闭）** — 清理资源并退出
//!
//! 同时注册 panic 钩子，确保崩溃时能记录阶段信息便于排查。
//!
//! ## 子模块
//! - `boot`    — 启动阶段，加载资源、初始化服务
//! - `runtime` — 运行阶段，主循环、帧更新与渲染
//! - `shutdown`— 关闭阶段，释放资源、保存状态
//! - `core`    — 核心工具，包含 panic 钩子与崩溃阶段标记
//! - `services`— 引擎服务集合（渲染、日志、Lua 等）

// --- 子模块声明 ---

// 启动阶段：负责加载资源、初始化服务与游戏世界
pub mod boot;
// 运行阶段：主游戏循环，处理输入、更新状态、渲染画面
pub mod runtime;
// 关闭阶段：安全释放资源，保存必要状态后退出
pub mod shutdown;
// 核心工具：panic 钩子安装、崩溃阶段标记
pub mod core;
// 引擎服务：渲染画布、日志记录、Lua 虚拟机等
pub mod services;
// 用户界面：UI 树节点与页面实现
pub mod ui;

// --- 内部依赖 ---

use crate::host_engine::core::{CrashPhase, install_panic_hook, set_crash_phase};

// 日志
use self::services::LogSource;

// --- 入口函数 ---

/// 程序主流程
///
/// 按照 **启动 → 运行 → 关闭** 三个阶段依次执行：
/// 1. 安装 panic 钩子，确保崩溃时记录当前阶段
/// 2. 调用 `boot::prepare()` 初始化服务与世界
/// 3. 进入 `runtime::run()` 主循环
/// 4. 主循环退出后调用 `shutdown::close()` 清理资源
pub fn run() {
  // 安装全局 panic 钩子，崩溃时输出当前阶段信息
  install_panic_hook();

  // --- 启动阶段 ---
  set_crash_phase(CrashPhase::Init);
  let boot_output = boot::prepare();

  // 分离引擎服务和游戏世界
  let mut services = boot_output.services;
  let mut world = boot_output.world;

  // 验证 Lua 虚拟机是否正常启动
  match services.lua.eval("return 'Lua VM active'") {
    Ok(result) => services
      .log
      .info(LogSource::Boot, &format!("[Boot] Lua: {result}")),
    Err(error) => services
      .log
      .error(LogSource::Boot, &format!("[Boot] Lua error: {error}")),
  }

  // --- 运行阶段 ---
  set_crash_phase(CrashPhase::Runtime);
  let exit_state = runtime::run(&mut services, &mut world);

  // --- 关闭阶段 ---
  set_crash_phase(CrashPhase::Shutdown);
  shutdown::close(&mut services, world, exit_state);
}
