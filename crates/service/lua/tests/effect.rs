#[path = "../examples/lua_effect_smoke.rs"]
mod scenario;

#[test]
fn effect_lifecycle_composition_and_callback_isolation() {
  scenario::main();
}
