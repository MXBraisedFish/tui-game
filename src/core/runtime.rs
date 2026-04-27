/// 游戏运行时管理，负责启动游戏和帧生命周期

use anyhow::Result;

use crate::core::command::RuntimeCommand;
use crate::core::event::InputEvent;
use crate::core::screen::Canvas;
use crate::game::registry::GameDescriptor;
use crate::lua::engine;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaunchMode {
    New,
    Continue,
}

impl LaunchMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Continue => "continue",
        }
    }
}
/// 宿主驱动的统一游戏运行时上下文。
#[derive(Clone, Debug)]
pub struct RuntimeSession {
    pub game: GameDescriptor,
    pub canvas: Canvas,
    pub pending_commands: Vec<RuntimeCommand>,
}

impl RuntimeSession {
    pub fn new(game: GameDescriptor, width: u16, height: u16) -> Self {
        Self {
            game,
            canvas: Canvas::new(width, height),
            pending_commands: Vec::new(),
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.canvas.resize(width, height);
    }

    pub fn push_command(&mut self, command: RuntimeCommand) {
        self.pending_commands.push(command);
    }

    pub fn drain_commands(&mut self) -> Vec<RuntimeCommand> {
        std::mem::take(&mut self.pending_commands)
    }

    pub fn begin_frame(&mut self, width: u16, height: u16) {
        if self.canvas.width() != width || self.canvas.height() != height {
            self.resize(width, height);
        }
        self.canvas.clear();
    }

    pub fn synthesize_tick(dt_ms: u32) -> InputEvent {
        InputEvent::Tick { dt_ms }
    }
}

/// 统一游戏启动分发。
pub fn launch_game(game: &GameDescriptor, mode: LaunchMode) -> Result<()> {
    engine::run_game_descriptor(game, mode)
}
