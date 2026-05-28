use std::result;

// 统一模块导出
// 启动阶段
pub mod boot;
// 运行阶段
pub mod runtime;
// 关闭阶段
pub mod shutdown;
// 过渡
pub mod core;
// 引擎服务
pub mod services;

// 主流程运行程序
pub fn run() {
  // 启动，返回启动输出
  let boot_output = boot::prepare();

  // 分离引擎服务和运行时世界
  let mut services = boot_output.services;
  let mut world = boot_output.world;

  match services.lua.eval("return 'Lua VM active'") {
    Ok(result) => services.log.info("[Boot] Lua: {}"),
    Err(error) => services.log.error("[Boot] Lua error: {}")
  }

  // 运行，返回退出状态
  let exit_state = runtime::run(&mut services, &mut world);

  // 关闭
  shutdown::close(&mut services, world, exit_state);
}