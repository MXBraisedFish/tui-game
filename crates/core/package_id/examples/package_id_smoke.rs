//! Minimal entry: builds one package identity and prints its storage key.

use tg_core_package_id::{PackageId, PackageSource, PackageType};

fn main() {
  let id = PackageId::new(PackageSource::Mod, PackageType::Game, "smoke").expect("valid mod id");
  assert_eq!(id.storage_key(), "mod/game/smoke");
  assert!(PackageId::new(PackageSource::Mod, PackageType::Game, "../escape").is_err());
  println!("package_id ok: {id}");
}
