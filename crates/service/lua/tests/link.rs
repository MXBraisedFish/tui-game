//! Reuse the independent Lua link scenario as an integration regression.
#[path = "../examples/lua_link_smoke.rs"]
mod scenario;
#[test]
fn links_events_and_input_mouse_default() {
  scenario::main();
}
