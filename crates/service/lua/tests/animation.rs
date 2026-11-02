//! Reuse the standalone parameter animation scenario as an integration regression.

#[path = "../examples/lua_animation_smoke.rs"]
mod scenario;

#[test]
fn animation_tracks_playback_and_callback_isolation() {
  scenario::main();
}
