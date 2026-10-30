//! Verify official Tetris event handling without launching the host.

#[path = "../examples/lua_official_tetris_smoke.rs"]
mod scenario;

#[test]
fn official_tetris_handles_event_descriptions_and_clears_held_inputs() {
  scenario::main();
}
