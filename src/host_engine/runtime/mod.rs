// 引入标准线程库
use std::thread;
use std::time::Duration;

// 引用结构体和枚举
use crate::host_engine::core::{ExitState, FrameScheduler, RuntimeWorld};
use crate::host_engine::services::{
  EngineServices,
  Key,
  KeyBinding,
  KeyPattern,
  LogSource,
};

// 运行函数
pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  // 启用终端模式
  services.terminal.enter(&mut services.log);

  // 启动全局键盘监听器
  services.input.start_key_listener();

  // 构建一个帧循环
  let mut scheduler = FrameScheduler::new();

  // 开始循环
  // while world.session.is_running() {
    // 1. 帧起始
    // - 画布起始帧
    // - 时间起始帧

    // 2. 输入处理 -> 路由到状态机

    // 3. 更新 -> 路由到状态机

    // 4. 绘制 -> 路由到状态机

    // 5. 呈现

    // 6. 帧等待（结束）
  // }

  // 返回退出信息块
  ExitState::new()
}
