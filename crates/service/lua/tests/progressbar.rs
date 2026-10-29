//! Reuse the independent progress bar scenario as an integration regression.
#[path = "../examples/lua_progressbar_smoke.rs"]
mod scenario;
#[test]
fn progressbar_lifecycle_geometry_and_containers() {
  scenario::main();
}
