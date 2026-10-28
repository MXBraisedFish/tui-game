//! Run the independent embedded scrollbox scenario as a regression.
#[path = "../examples/scrollbox_smoke.rs"]
mod scenario;
#[test]
fn embedded_geometry_and_wheel_routing() {
  scenario::main();
}
