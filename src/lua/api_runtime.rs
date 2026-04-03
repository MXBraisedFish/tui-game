use mlua::Lua;

use crate::core::command::RuntimeCommand;
use crate::core::event::InputEvent;

pub fn install_runtime_api(_lua: &Lua) -> mlua::Result<()> {
    Ok(())
}

pub fn default_tick_event() -> InputEvent {
    InputEvent::Tick { dt_ms: 16 }
}

pub fn exit_command() -> RuntimeCommand {
    RuntimeCommand::ExitGame
}
