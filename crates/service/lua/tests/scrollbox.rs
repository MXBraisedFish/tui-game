//! Run independent scrollbox scenarios as integration regressions.
#[path = "../examples/lua_scrollbox_smoke.rs"]
mod scenario;
#[test]
fn scrollbox_independent_scenarios() {
  scenario::main();
}
