//! Run the independently executable native link contract scenario.
#[path = "../examples/link_smoke.rs"]
mod scenario;
#[test]
fn geometry_styles_targets_and_cleanup() {
  scenario::main();
}
