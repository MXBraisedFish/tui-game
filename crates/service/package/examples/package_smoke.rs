//! Independent package smoke entry exercising the public API and checking its results.

use std::path::PathBuf;

use tg_service_log::LogService;
use tg_service_package::PackageService;

fn main() {
  let root = create_temp_dir("tg_package_smoke");
  let mut packages = PackageService::new();
  let mut log = LogService::new();

  packages.scan_all(&root, &mut log, "en_us", "missing: {key}");
  assert_eq!(packages.total_count(), 0);
  assert!(packages.game_list().is_empty());

  let game_dir = root.join("data/mod/game/i18n_smoke");
  for (relative, content) in [
    (
      "package.json",
      r#"{"mod_id":"i18n_smoke","schema_version":2,"type":"game","version":{"type":"i18n","key":"shared","callback":"1.0"},"version_code":1,"api":{"min":1,"max":1}}"#,
    ),
    (
      "display.json",
      r#"{"title":{"type":"i18n","key":"shared","callback":"Title"},"description":"Smoke game","author":"Tester"}"#,
    ),
    (
      "game.json",
      r#"{"name":{"type":"i18n","key":"shared","callback":"Game"},"detail":"Details","command":"i18n_smoke","entry":"main","language":["en_us","zh_cn"]}"#,
    ),
    (
      "actions.json",
      r#"{"move":{"description":{"type":"i18n","key":"shared","callback":"Move"},"keys":[["w"]],"priority":9},"a_shared":{"description":"Shared","keys":[["w"]]}}"#,
    ),
    ("scripts/main.lua", "function render() end"),
    (
      "assets/language/en_us/package/package.json",
      r#"{"shared":"1.1"}"#,
    ),
    (
      "assets/language/zh_cn/package/display.json",
      r#"{"shared":"Localized title"}"#,
    ),
    (
      "assets/language/zh_cn/package/game.json",
      r#"{"shared":"Localized game"}"#,
    ),
    (
      "assets/language/zh_cn/package/actions.json",
      r#"{"shared":"Localized move"}"#,
    ),
  ] {
    let path = game_dir.join(relative);
    std::fs::create_dir_all(path.parent().expect("fixture parent"))
      .expect("create fixture directories");
    std::fs::write(path, content).expect("write fixture");
  }
  packages.scan_all(&root, &mut log, "zh_cn", "missing: {key}");
  assert_eq!(packages.total_count(), 1);
  let games = packages.games();
  let package = &games[0];
  assert_eq!(package.version, "1.1");
  assert_eq!(package.display.title, "Localized title");
  let game = package.game.as_ref().expect("game config");
  assert_eq!(game.name, "Localized game");
  assert_eq!(game.action_order, ["move", "a_shared"]);
  assert_eq!(game.actions["move"].priority, 9);
  assert_eq!(game.actions["a_shared"].priority, 0);
  assert_eq!(game.actions["move"].description, "Localized move");

  let best_path = game_dir.join("assets/language/zh_cn/package/best_string.json");
  std::fs::write(
    &best_path,
    r#"{"score":"f%Localized best: {value:score}","rank":"Localized rank"}"#,
  )
  .expect("write best score language");
  let data = serde_json::json!({
    "best_string":{"type":"i18n","key":"score","callback":"f%Best: {value:score}"},
    "value":{"score":"42","rank":{"type":"i18n","key":"rank","callback":"Gold"}}
  });
  let (text, values) = packages
    .resolve_best_save(&game_dir, &data)
    .expect("resolve saved text");
  assert_eq!(text, "f%Localized best: {value:score}");
  assert_eq!(values["score"], "42");
  assert_eq!(values["rank"], "Localized rank");
  std::fs::remove_file(best_path).expect("remove language resource");
  assert_eq!(
    packages.resolve_best_save(&game_dir, &data).unwrap().0,
    "f%Best: {value:score}"
  );
  assert!(
    packages
      .resolve_best_save(&game_dir, &serde_json::json!({"best_string":42}))
      .is_err()
  );

  std::fs::remove_dir_all(&root).expect("clean up temporary directory");
  println!("package ok: empty root, per-manifest i18n and language fallback scanned");
}

fn create_temp_dir(prefix: &str) -> PathBuf {
  let nonce = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_default()
    .as_nanos();
  let root = std::env::temp_dir().join(format!("{prefix}_{}_{nonce}", std::process::id()));
  std::fs::create_dir(&root)
    .unwrap_or_else(|error| panic!("create temporary directory {}: {error}", root.display()));
  root
}
