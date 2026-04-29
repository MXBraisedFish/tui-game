// 游戏运行时管理，负责启动游戏、管理帧生命周期、持有画布和命令队列。提供 RuntimeSession 作为宿主驱动的统一上下文

use anyhow::Result; // 	统一错误返回类型

use crate::core::command::RuntimeCommand; // 命令枚举
use crate::core::event::InputEvent; // 事件枚举
use crate::core::screen::Canvas; // 虚拟画布
use crate::game::registry::GameDescriptor; // 游戏元数据描述符
use crate::lua::engine; // 	Lua 引擎，实际执行游戏脚本

// 游戏启动方式枚举
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaunchMode {
    New, // 新游戏（从头开始）
    Continue, // 继续上次的存档
}

impl LaunchMode {
    // 返回模式名字符串 "new" 或 "continue"，用于 Lua 接口
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Continue => "continue",
        }
    }
}

// 宿主驱动的统一游戏运行时上下文。
#[derive(Clone, Debug)]
pub struct RuntimeSession {
    pub game: GameDescriptor,
    pub canvas: Canvas,
    pub pending_commands: Vec<RuntimeCommand>,
}

impl RuntimeSession {
    // 创建会话，初始化画布、空命令队列
    pub fn new(game: GameDescriptor, width: u16, height: u16) -> Self {
        Self {
            game,
            canvas: Canvas::new(width, height),
            pending_commands: Vec::new(),
        }
    }

    // 调整画布大小
    pub fn resize(&mut self, width: u16, height: u16) {
        self.canvas.resize(width, height);
    }

    // 添加一个待发送命令
    pub fn push_command(&mut self, command: RuntimeCommand) {
        self.pending_commands.push(command);
    }

    // 取走所有待发送命令（清空队列）
    pub fn drain_commands(&mut self) -> Vec<RuntimeCommand> {
        std::mem::take(&mut self.pending_commands)
    }

    // 	帧开始：检查尺寸是否变化，清除画布内容
    pub fn begin_frame(&mut self, width: u16, height: u16) {
        if self.canvas.width() != width || self.canvas.height() != height {
            self.resize(width, height);
        }
        self.canvas.clear();
    }

    // 静态方法，构造一个 Tick 事件
    pub fn synthesize_tick(dt_ms: u32) -> InputEvent {
        InputEvent::Tick { dt_ms }
    }
}

// 统一入口，调用 Lua 引擎运行游戏
pub fn launch_game(game: &GameDescriptor, mode: LaunchMode) -> Result<()> {
    engine::run_game_descriptor(game, mode)
}
