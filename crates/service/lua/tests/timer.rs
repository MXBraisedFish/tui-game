//! Reuse the independent timer scenario as an integration regression.

#[path = "../examples/lua_timer_smoke.rs"]
mod scenario;

#[test]
fn timer_wait_states_callbacks_and_session_ownership() {
  scenario::main();
}
