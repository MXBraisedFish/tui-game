// use crate::host_engine::core::{RuntimeAction, RuntimeWorld};
// use crate::host_engine::services::{EngineServices, Key};

// pub fn handle_runtime_keyboard(services: &mut EngineServices, world: &mut RuntimeWorld) {
//   if services.input.was_pressed(Key::Fn(1)) {
//     world.session.handle_runtime_action(RuntimeAction::PushDebugOverlay);
//   }

//   if services.input.was_pressed(Key::Fn(2)) {
//     world.session.handle_runtime_action(RuntimeAction::PopDebugOverlay);
//   }

//   if services.input.was_pressed(Key::Esc) {
//     world.session.handle_runtime_action(RuntimeAction::Cancel);
//   }
// }
