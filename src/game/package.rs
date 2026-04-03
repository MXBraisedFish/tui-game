use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::game::manifest::{GameManifest, PackageManifest};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GamePackageSource {
    Official,
    Mod,
    LegacyBuiltin,
}

#[derive(Clone, Debug)]
pub struct GamePackage {
    pub root_dir: PathBuf,
    pub source: GamePackageSource,
    pub package: PackageManifest,
    pub games: Vec<GameManifest>,
}

pub fn discover_packages(base_dir: &Path, source: GamePackageSource) -> Result<Vec<GamePackage>> {
    if !base_dir.exists() {
        return Ok(Vec::new());
    }

    let mut packages = Vec::new();
    let mut entries: Vec<PathBuf> = fs::read_dir(base_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    entries.sort();

    for root_dir in entries {
        let package_manifest_path = root_dir.join("package.json");
        if !package_manifest_path.exists() {
            continue;
        }
        packages.push(load_package(&root_dir, source.clone())?);
    }

    Ok(packages)
}

pub fn load_package(root_dir: &Path, source: GamePackageSource) -> Result<GamePackage> {
    let package = read_package_manifest(root_dir)?;
    let games = read_game_manifests(root_dir)?;
    Ok(GamePackage {
        root_dir: root_dir.to_path_buf(),
        source,
        package,
        games,
    })
}

fn read_package_manifest(root_dir: &Path) -> Result<PackageManifest> {
    let path = root_dir.join("package.json");
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read package manifest: {}", path.display()))?;
    serde_json::from_str(raw.trim_start_matches('\u{feff}'))
        .with_context(|| format!("invalid package manifest json: {}", path.display()))
}

fn read_game_manifests(root_dir: &Path) -> Result<Vec<GameManifest>> {
    let mut manifests = Vec::new();

    let single = root_dir.join("game.json");
    if single.exists() {
        manifests.push(read_game_manifest(&single)?);
    }

    let games_dir = root_dir.join("games");
    if games_dir.exists() {
        let mut entries: Vec<PathBuf> = fs::read_dir(&games_dir)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("json"))
                    .unwrap_or(false)
            })
            .collect();
        entries.sort();
        for path in entries {
            manifests.push(read_game_manifest(&path)?);
        }
    }

    Ok(manifests)
}

fn read_game_manifest(path: &Path) -> Result<GameManifest> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read game manifest: {}", path.display()))?;
    serde_json::from_str(raw.trim_start_matches('\u{feff}'))
        .with_context(|| format!("invalid game manifest json: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_test_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time drift")
            .as_nanos();
        std::env::temp_dir().join(format!("tui_game_{name}_{unique}"))
    }

    #[test]
    fn load_package_reads_single_game_manifest() {
        let root = temp_test_dir("single_package");
        fs::create_dir_all(&root).expect("create package dir");
        fs::write(
            root.join("package.json"),
            r#"{
  "namespace": "demo",
  "package_name": "Demo Package",
  "author": "Tester",
  "version": "1.0.0",
  "description": "demo.desc",
  "api_version": 1
}"#,
        )
        .expect("write package");
        fs::write(
            root.join("game.json"),
            r#"{
  "id": "demo.runtime",
  "name": "demo.name",
  "description": "demo.description",
  "detail": "demo.detail",
  "entry": "scripts/demo.lua",
  "save": true,
  "best_none": "demo.best_none",
  "min_width": 40,
  "min_height": 20,
  "actions": {
    "confirm": ["enter", "space"]
  }
}"#,
        )
        .expect("write game");

        let package = load_package(&root, GamePackageSource::Official).expect("load package");
        assert_eq!(package.package.namespace, "demo");
        assert_eq!(package.games.len(), 1);
        assert_eq!(package.games[0].id, "demo.runtime");
        assert_eq!(package.games[0].actions["confirm"].keys(), vec!["enter".to_string(), "space".to_string()]);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn discover_packages_reads_games_directory_and_ignores_non_packages() {
        let base = temp_test_dir("discover_packages");
        let package_root = base.join("alpha");
        let ignored_root = base.join("ignored");
        let games_dir = package_root.join("games");
        fs::create_dir_all(&games_dir).expect("create package dirs");
        fs::create_dir_all(&ignored_root).expect("create ignored dir");

        fs::write(
            package_root.join("package.json"),
            r#"{
  "namespace": "alpha",
  "package_name": "Alpha Pack",
  "author": "Tester",
  "version": "1.0.0",
  "description": "alpha.desc",
  "api_version": [1, 2]
}"#,
        )
        .expect("write package");
        fs::write(
            games_dir.join("one.json"),
            r#"{
  "id": "alpha.one",
  "name": "alpha.one.name",
  "description": "alpha.one.description",
  "detail": "alpha.one.detail",
  "entry": "scripts/one.lua",
  "save": false,
  "actions": {}
}"#,
        )
        .expect("write game one");
        fs::write(
            games_dir.join("two.json"),
            concat!(
                "\u{feff}",
                "{\n",
                "  \"id\": \"alpha.two\",\n",
                "  \"name\": \"alpha.two.name\",\n",
                "  \"description\": \"alpha.two.description\",\n",
                "  \"detail\": \"alpha.two.detail\",\n",
                "  \"entry\": \"scripts/two.lua\",\n",
                "  \"save\": true,\n",
                "  \"actions\": {\n",
                "    \"move_left\": [\"left\", \"a\"]\n",
                "  }\n",
                "}\n"
            ),
        )
        .expect("write game two");

        let packages = discover_packages(&base, GamePackageSource::Mod).expect("discover packages");
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].package.namespace, "alpha");
        assert_eq!(packages[0].games.len(), 2);
        assert_eq!(packages[0].games[1].id, "alpha.two");
        assert_eq!(
            packages[0].games[1].actions["move_left"].keys(),
            vec!["left".to_string(), "a".to_string()]
        );

        let _ = fs::remove_dir_all(base);
    }
}
