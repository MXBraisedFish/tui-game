// 引入标准线程库
use std::thread;
use std::time::Duration;

// 引入运行时输入处理
mod input;

use input::handle_runtime_keyboard;

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

  // 临时测试动作表：后续由正式动作表解析器替换
  services.input.load_key_bindings(vec![
    KeyBinding {
      pattern: KeyPattern::Single(Key::Esc),
      action: "quit".to_string(),
    },

    KeyBinding {
      pattern: KeyPattern::Single(Key::Fn(1)),
      action: "overlay.push".to_string(),
    },

    KeyBinding {
      pattern: KeyPattern::Single(Key::Fn(2)),
      action: "overlay.pop".to_string(),
    },

    KeyBinding {
      pattern: KeyPattern::Combo(Key::LeftCtrl, Key::S),
      action: "save".to_string(),
    },

    KeyBinding {
      pattern: KeyPattern::Single(Key::S),
      action: "single.s".to_string(),
    },
  ]);

  // 构建一个帧循环
  let mut scheduler = FrameScheduler::new();

  // 开始循环
  while world.session.is_running() {
    // 1. 帧起始

    // 2. 全局时间处理

    // 3. 输入处理 -> 路由到状态机

    // 4. 绘制 -> 路由到状态机

    // 5. 呈现

    // 6. 帧等待（结束）
  }

  // 返回退出信息块
  ExitState::new()
}
