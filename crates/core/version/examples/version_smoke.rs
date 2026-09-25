//! Minimal entry: prints the host and API versions.

fn main() {
  assert!(!tg_core_version::HOST_VERSION.is_empty());
  println!(
    "version ok: host {} api {}",
    tg_core_version::HOST_VERSION,
    tg_core_version::HOST_API_VERSION
  );
}
