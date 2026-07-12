pub mod boot;

pub mod runtime;

pub mod shutdown;

pub mod core;

pub mod services;

pub mod ui;

use crate::host_engine::core::{CrashPhase, install_panic_hook, set_crash_phase};

use self::services::LogSource;

/// 启动并运行引擎主循环，依次执行引导、运行时、关闭三个阶段
pub fn run() {
  install_panic_hook();

  set_crash_phase(CrashPhase::Init);
  let boot_output = boot::prepare();

  let mut services = boot_output.services;
  let mut world = boot_output.world;

  let eval_result = services
    .lua
    .eval("return 'Lua VM active'", &mut services.log);
  match eval_result {
    Ok(result) => services
      .log
      .info(LogSource::Boot, &format!("[Boot] Lua: {result}")),
    Err(error) => services
      .log
      .error(LogSource::Boot, &format!("[Boot] Lua error: {error}")),
  }

  set_crash_phase(CrashPhase::Runtime);
  let exit_state = runtime::run(&mut services, &mut world);

  set_crash_phase(CrashPhase::Shutdown);
  shutdown::close(&mut services, world, exit_state);
}
