/// 游戏包的发现、加载与验证
/// 业务逻辑：
/// 包发现
/// 包加载
/// 游戏清单验证
/// 动作按键验证
/// 字段类型验证
/// 入口脚本验证
/// 尺寸约束验证
/// Mod 特有验证
/// API 版本验证
/// 游戏 ID 生成
/// 日志对象

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use serde_json::Value;

use crate::app::i18n;
use crate::core::key::is_explicit_semantic_key;
use crate::game::action::ActionKeys;
use crate::game::manifest::{GameManifest, PackageManifest};
use crate::utils::host_log;

pub const HOST_GAME_API_VERSION: u32 = 7;
const MAX_ACTION_KEYS_PER_BINDING: usize = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GamePackageSource {
    Official,
    Mod,
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
    let package = read_package_manifest(root_dir, &source)?;
    let games = read_game_manifests(root_dir, &package, &source)?;
    Ok(GamePackage {
        root_dir: root_dir.to_path_buf(),
        source,
        package,
        games,
    })
}

fn read_package_manifest(root_dir: &Path, source: &GamePackageSource) -> Result<PackageManifest> {
    let path = root_dir.join("package.json");
    let raw = fs::read_to_string(&path).with_context(|| {
        i18n::t_or("host.error.read_package_json_failed", "Failed to read package.json: {path}")
            .replace("{path}", &path.display().to_string())
    })?;
    let mut manifest: PackageManifest = serde_json::from_str(raw.trim_start_matches('\u{feff}'))
        .with_context(|| {
            i18n::t_or(
                "host.error.invalid_package_json",
                "Invalid JSON format in package.json: {path}",
            )
            .replace("{path}", &path.display().to_string())
        })?;
    if matches!(source, GamePackageSource::Mod) && manifest.namespace.trim().is_empty() {
        manifest.namespace = root_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
    }
    Ok(manifest)
}

fn read_game_manifests(
    root_dir: &Path,
    package: &PackageManifest,
    source: &GamePackageSource,
) -> Result<Vec<GameManifest>> {
    let mut manifests = Vec::new();

    let single = root_dir.join("game.json");
    if single.exists() {
        manifests.push(read_game_manifest(root_dir, package, source, &single)?);
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
            manifests.push(read_game_manifest(root_dir, package, source, &path)?);
        }
    }

    Ok(manifests)
}

fn read_game_manifest(
    root_dir: &Path,
    package: &PackageManifest,
    source: &GamePackageSource,
    path: &Path,
) -> Result<GameManifest> {
    let raw = fs::read_to_string(path).with_context(|| {
        i18n::t_or("host.error.read_game_json_failed", "Failed to read game.json: {path}")
            .replace("{path}", &path.display().to_string())
    })?;
    let raw_value: Value =
        serde_json::from_str(raw.trim_start_matches('\u{feff}')).with_context(|| {
            i18n::t_or(
                "host.error.invalid_game_json",
                "Invalid JSON format in game.json: {path}",
            )
            .replace("{path}", &path.display().to_string())
        })?;
    let log_object = game_log_object_id(package, path, &raw_value);
    let _log_object_guard = host_log::scoped_log_object(log_object);
    validate_game_actions_shape(&raw_value)?;
    validate_game_case_sensitive_shape(&raw_value)?;
    let mut manifest: GameManifest = serde_json::from_value(raw_value).with_context(|| {
        i18n::t_or(
            "host.error.invalid_game_json",
            "Invalid JSON format in game.json: {path}",
        )
        .replace("{path}", &path.display().to_string())
    })?;
    truncate_action_keys_with_warning(&mut manifest);
    validate_action_key_bindings(&manifest)?;
    manifest.id = expected_game_id(package, &manifest);
    validate_game_manifest(root_dir, package, source, &manifest, path)?;
    Ok(manifest)
}

fn truncate_action_keys_with_warning(manifest: &mut GameManifest) {
    for (action, binding) in &mut manifest.actions {
        let ActionKeys::Multiple(keys) = &mut binding.key else {
            continue;
        };
        if keys.len() <= MAX_ACTION_KEYS_PER_BINDING {
            continue;
        }
        let key_count = keys.len().to_string();
        host_log::append_host_warning(
            "host.warning.action_key_limit_exceeded",
            &[("action", action.as_str()), ("key_count", &key_count)],
        );
        keys.truncate(MAX_ACTION_KEYS_PER_BINDING);
    }
}

fn validate_action_key_bindings(manifest: &GameManifest) -> Result<()> {
    let mut buckets: HashMap<String, Vec<String>> = HashMap::new();

    for binding in manifest.actions.values() {
        for key in binding.keys() {
            let trimmed = key.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !is_explicit_semantic_key(trimmed) {
                host_log::append_host_warning(
                    "host.warning.action_key_non_explicit",
                    &[("key", trimmed)],
                );
            }
            let canonical = if manifest.case_sensitive {
                trimmed.to_string()
            } else {
                trimmed.to_lowercase()
            };
            buckets
                .entry(canonical)
                .or_default()
                .push(trimmed.to_string());
        }
    }

    let conflicts = buckets
        .into_values()
        .filter(|keys| keys.len() > 1)
        .flatten()
        .collect::<Vec<_>>();
    if !conflicts.is_empty() {
        return Err(action_key_duplicate_error(&conflicts.join(", ")));
    }

    Ok(())
}

fn validate_game_actions_shape(root: &Value) -> Result<()> {
    let Some(actions) = root.get("actions") else {
        return Ok(());
    };
    let Some(actions_obj) = actions.as_object() else {
        return Err(field_invalid_type_error("game.json", "actions", "object", actions));
    };

    for (action_name, binding) in actions_obj {
        let binding_key = format!("actions.{action_name}");
        let Some(binding_obj) = binding.as_object() else {
            return Err(field_invalid_type_error(
                "game.json",
                &binding_key,
                "object",
                binding,
            ));
        };

        let key_field = format!("{binding_key}.key");
        let Some(key_value) = binding_obj.get("key") else {
            return Err(field_blank_error("game.json", &key_field));
        };
        validate_action_key_value(&key_field, key_value)?;

        let key_name_field = format!("{binding_key}.key_name");
        let Some(key_name_value) = binding_obj.get("key_name") else {
            return Err(field_blank_error("game.json", &key_name_field));
        };
        let Some(key_name) = key_name_value.as_str() else {
            return Err(field_invalid_type_error(
                "game.json",
                &key_name_field,
                "string",
                key_name_value,
            ));
        };
        if key_name.trim().is_empty() {
            return Err(field_blank_error("game.json", &key_name_field));
        }
    }

    Ok(())
}

fn action_key_duplicate_error(keys: &str) -> anyhow::Error {
    host_log::append_host_error("host.error.action_key_duplicate", &[("keys", keys)]);
    anyhow!(
        "{}",
        i18n::t_or(
            "host.error.action_key_duplicate",
            "Key action registry has duplicate key bindings. Conflicting keys: {keys}"
        )
        .replace("{keys}", keys)
    )
}

fn validate_game_case_sensitive_shape(root: &Value) -> Result<()> {
    let Some(value) = root.get("case_sensitive") else {
        return Ok(());
    };
    if value.is_boolean() {
        Ok(())
    } else {
        Err(field_invalid_type_error(
            "game.json",
            "case_sensitive",
            "boolean",
            value,
        ))
    }
}

fn validate_action_key_value(field_key: &str, value: &Value) -> Result<()> {
    match value {
        Value::String(text) => {
            if text.trim().is_empty() {
                Err(field_blank_error("game.json", field_key))
            } else {
                Ok(())
            }
        }
        Value::Array(values) => {
            if values.is_empty() {
                return Err(field_blank_error("game.json", field_key));
            }
            for (index, item) in values.iter().enumerate() {
                let item_key = format!("{field_key}[{}]", index + 1);
                let Some(text) = item.as_str() else {
                    return Err(field_invalid_type_error(
                        "game.json",
                        &item_key,
                        "string",
                        item,
                    ));
                };
                if text.trim().is_empty() {
                    return Err(field_blank_error("game.json", &item_key));
                }
            }
            Ok(())
        }
        _ => Err(field_invalid_type_error(
            "game.json",
            field_key,
            "string | array",
            value,
        )),
    }
}

fn field_blank_error(file: &str, key: &str) -> anyhow::Error {
    anyhow!(
        "{}",
        i18n::t_or("host.error.field_blank", "Field \"{key}\" in {file} cannot be empty")
            .replace("{file}", file)
            .replace("{key}", key)
    )
}

fn field_invalid_type_error(file: &str, key: &str, expected: &str, actual: &Value) -> anyhow::Error {
    anyhow!(
        "{}",
        i18n::t_or(
            "host.error.field_invalid_type",
            "Field \"{key}\" in {file} has invalid type: expected {type}, got {actual_type}",
        )
        .replace("{file}", file)
        .replace("{key}", key)
        .replace("{type}", expected)
        .replace("{actual_type}", json_value_type_name(actual))
    )
}

fn json_value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn validate_game_manifest(
    _root_dir: &Path,
    package: &PackageManifest,
    source: &GamePackageSource,
    manifest: &GameManifest,
    path: &Path,
) -> Result<()> {
    validate_game_api_version(package, &manifest.api)?;

    if manifest.entry.trim().is_empty() {
        return Err(anyhow!("game entry cannot be blank"));
    }
    if Path::new(&manifest.entry).is_absolute()
        || manifest.entry.starts_with('/')
        || manifest.entry.starts_with('\\')
        || manifest
            .entry
            .split(['/', '\\'])
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(anyhow!("game entry must be a relative script path"));
    }
    if matches!(manifest.min_width, Some(0)) {
        return Err(anyhow!("min_width must be greater than 0 when provided"));
    }
    if matches!(manifest.min_height, Some(0)) {
        return Err(anyhow!("min_height must be greater than 0 when provided"));
    }

    if matches!(source, GamePackageSource::Mod) {
        if package.author.trim().is_empty() {
            return Err(anyhow!("mod game author cannot be blank"));
        }
        let introduction = package
            .introduction
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("mod game introduction cannot be blank"))?;
        let _ = introduction;
        if package.package_name.trim().is_empty() {
            return Err(anyhow!("mod package name cannot be blank"));
        }
        if package
            .game_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
        {
            return Err(anyhow!("mod display name cannot be blank"));
        }
    }

    let effective_name = package
        .game_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| manifest.name.trim());
    if effective_name.is_empty() {
        return Err(anyhow!("game name cannot be blank"));
    }

    if manifest.icon.is_some()
        && !matches!(
            manifest.icon,
            Some(serde_json::Value::Null)
                | Some(serde_json::Value::String(_))
                | Some(serde_json::Value::Array(_))
                | Some(serde_json::Value::Object(_))
        )
    {
        return Err(anyhow!("icon must be null, string, array, or object"));
    }
    if manifest.banner.is_some()
        && !matches!(
            manifest.banner,
            Some(serde_json::Value::Null)
                | Some(serde_json::Value::String(_))
                | Some(serde_json::Value::Array(_))
                | Some(serde_json::Value::Object(_))
        )
    {
        return Err(anyhow!("banner must be null, string, array, or object"));
    }

    if path.extension().and_then(|value| value.to_str()) != Some("json") {
        return Err(anyhow!("game manifest must be json"));
    }

    Ok(())
}

fn validate_game_api_version(
    package: &PackageManifest,
    api: &Option<serde_json::Value>,
) -> Result<()> {
    let Some(api) = api.as_ref() else {
        return Ok(());
    };

    let host_version = HOST_GAME_API_VERSION;
    let supported = match api {
        serde_json::Value::Number(value) => value
            .as_u64()
            .map(|version| version == host_version as u64)
            .unwrap_or(false),
        serde_json::Value::Array(values) if values.len() == 2 => {
            let min = values[0].as_u64();
            let max = values[1].as_u64();
            match (min, max) {
                (Some(min), Some(max)) if min <= max => {
                    (min..=max).contains(&(host_version as u64))
                }
                _ => false,
            }
        }
        _ => false,
    };

    if supported {
        return Ok(());
    }

    let actual = api_version_display(api);
    Err(anyhow!(
        "{}",
        i18n::t_or(
            "host.error.api_version_mismatch",
            "\"{mod_namespce}\" API version mismatch: expected {api_version}, got {actual_api_version}"
        )
        .replace("{mod_namespce}", package.namespace.trim())
        .replace("{api_version}", &host_version.to_string())
        .replace("{actual_api_version}", &actual)
    ))
}

fn api_version_display(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Number(number) => number.to_string(),
        serde_json::Value::Array(values) if values.len() == 2 => {
            format!(
                "{}-{}",
                values[0]
                    .as_u64()
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| values[0].to_string()),
                values[1]
                    .as_u64()
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| values[1].to_string())
            )
        }
        _ => value.to_string(),
    }
}

pub fn expected_game_id(package: &PackageManifest, manifest: &GameManifest) -> String {
    let seed = format!(
        "{}{}{}{}{}",
        package.namespace.trim(),
        package.package_name.trim(),
        package.author.trim(),
        package
            .game_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| manifest.name.trim()),
        manifest.entry.trim()
    );
    format!("tui_game_{}", stable_base62_hash16(&seed))
}

fn game_log_object_id(package: &PackageManifest, path: &Path, raw: &Value) -> String {
    let raw_name = raw
        .get("game_name")
        .or_else(|| raw.get("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let package_name = package
        .game_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let name = package_name
        .or(raw_name)
        .unwrap_or_else(|| path.file_stem().and_then(|value| value.to_str()).unwrap_or_default());
    let entry = raw
        .get("entry")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| path.file_name().and_then(|value| value.to_str()).unwrap_or_default());
    let seed = format!(
        "{}{}{}{}{}",
        package.namespace.trim(),
        package.package_name.trim(),
        package.author.trim(),
        name,
        entry
    );
    format!("tui_game_{}", stable_base62_hash16(&seed))
}

fn stable_base62_hash16(seed: &str) -> String {
    const ALPHABET: &[u8; 62] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    let mut state = fnv1a64(seed.as_bytes());
    let mut out = String::with_capacity(16);

    for _ in 0..16 {
        state = splitmix64(state);
        let index = (state % ALPHABET.len() as u64) as usize;
        out.push(ALPHABET[index] as char);
    }

    out
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::manifest::RuntimeManifest;
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
        let base = temp_test_dir("single_package");
        let root = base.join("demo");
        fs::create_dir_all(&root).expect("create package dir");
        let package_manifest = PackageManifest {
            namespace: "demo".to_string(),
            package_name: "Demo Package".to_string(),
            mod_name: Some("Demo Mod".to_string()),
            author: "Tester".to_string(),
            version: String::new(),
            introduction: Some("demo intro".to_string()),
            game_name: Some("Demo Name".to_string()),
            description: "demo.desc".to_string(),
            detail: Some("demo detail".to_string()),
            icon: None,
            banner: None,
            api_version: None,
        };
        let _game_manifest = GameManifest {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            detail: String::new(),
            author: String::new(),
            introduction: None,
            icon: None,
            banner: None,
            entry: "scripts/demo.lua".to_string(),
            save: true,
            best_none: Some("No record yet".to_string()),
            min_width: Some(40),
            min_height: Some(20),
            max_width: None,
            max_height: None,
            actions: Default::default(),
            runtime: RuntimeManifest {
                target_fps: Some(30),
            },
            api: Some(serde_json::json!(7)),
            write: false,
            case_sensitive: false,
        };
        let game_id = expected_game_id(&package_manifest, &_game_manifest);
        fs::write(
            root.join("package.json"),
            r#"{
  "package": "Demo Package",
  "introduction": "demo intro",
  "author": "Tester",
  "game_name": "Demo Name",
  "description": "demo.desc",
  "detail": "demo detail"
}"#,
        )
        .expect("write package");
        fs::write(
            root.join("game.json"),
            format!(
                r#"{{
  "entry": "scripts/demo.lua",
  "api": 7,
  "save": true,
  "best_none": "No record yet",
  "min_width": 40,
  "min_height": 20,
  "write": false,
  "runtime": {{
    "target_fps": 30
  }},
  "actions": {{
    "confirm": {{
      "key": ["enter", "space"],
      "key_name": "Confirm"
    }}
  }}
}}"#,
            ),
        )
        .expect("write game");

        let package = load_package(&root, GamePackageSource::Mod).expect("load package");
        assert_eq!(package.package.namespace, "demo");
        assert_eq!(package.games.len(), 1);
        assert_eq!(package.games[0].id, game_id);
        assert_eq!(package.games[0].runtime.target_fps, Some(30));
        assert_eq!(
            package.games[0].actions["confirm"].keys(),
            vec!["enter".to_string(), "space".to_string()]
        );

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn discover_packages_reads_games_directory_and_ignores_non_packages() {
        let base = temp_test_dir("discover_packages");
        let package_root = base.join("alpha");
        let ignored_root = base.join("ignored");
        let games_dir = package_root.join("games");
        fs::create_dir_all(&games_dir).expect("create package dirs");
        fs::create_dir_all(&ignored_root).expect("create ignored dir");
        let package_manifest = PackageManifest {
            namespace: "alpha".to_string(),
            package_name: "Alpha Pack".to_string(),
            mod_name: Some("Alpha Mod".to_string()),
            author: "Tester".to_string(),
            version: String::new(),
            introduction: Some("alpha introduction".to_string()),
            game_name: Some("Alpha Display".to_string()),
            description: "alpha.desc".to_string(),
            detail: Some("alpha detail".to_string()),
            icon: None,
            banner: None,
            api_version: None,
        };
        let _game_one = GameManifest {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            detail: String::new(),
            author: String::new(),
            introduction: None,
            icon: None,
            banner: None,
            entry: "scripts/one.lua".to_string(),
            save: false,
            best_none: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            actions: Default::default(),
            runtime: Default::default(),
            api: Some(serde_json::json!([1, 7])),
            write: false,
            case_sensitive: false,
        };
        let _game_two = GameManifest {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            detail: String::new(),
            author: String::new(),
            introduction: None,
            icon: None,
            banner: None,
            entry: "scripts/two.lua".to_string(),
            save: true,
            best_none: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            actions: Default::default(),
            runtime: Default::default(),
            api: Some(serde_json::json!([1, 7])),
            write: false,
            case_sensitive: false,
        };
        let _game_one_id = expected_game_id(&package_manifest, &_game_one);

        fs::write(
            package_root.join("package.json"),
            r#"{
  "package": "Alpha Pack",
  "introduction": "alpha introduction",
  "author": "Tester",
  "game_name": "Alpha Display",
  "description": "alpha.desc",
  "detail": "alpha detail"
}"#,
        )
        .expect("write package");
        fs::write(
            games_dir.join("one.json"),
            format!(
                r#"{{
  "api": [1, 7],
  "entry": "scripts/one.lua",
  "save": false,
  "actions": {{}}
}}"#,
            ),
        )
        .expect("write game one");
        fs::write(
            games_dir.join("two.json"),
            concat!(
                "\u{feff}",
                "{\n",
                "  \"api\": [1, 7],\n",
                "  \"entry\": \"scripts/two.lua\",\n",
                "  \"save\": true,\n",
                "  \"actions\": {\n",
                "    \"move_left\": {\n",
                "      \"key\": [\"left\", \"a\"],\n",
                "      \"key_name\": \"Move Left\"\n",
                "    }\n",
                "  }\n",
                "}\n"
            ),
        )
        .expect("write game two");

        let packages = discover_packages(&base, GamePackageSource::Mod).expect("discover packages");
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].package.namespace, "alpha");
        assert_eq!(packages[0].games.len(), 2);
        assert_eq!(
            packages[0].games[1].id,
            expected_game_id(&package_manifest, &_game_two)
        );
        assert_eq!(
            packages[0].games[1].actions["move_left"].keys(),
            vec!["left".to_string(), "a".to_string()]
        );

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn expected_game_id_is_stable() {
        let package = PackageManifest {
            namespace: "examplepack".to_string(),
            package_name: "Example Pack".to_string(),
            mod_name: Some("Example Mod".to_string()),
            author: "Tester".to_string(),
            version: String::new(),
            introduction: Some("Intro".to_string()),
            game_name: Some("Word Puzzle".to_string()),
            description: String::new(),
            detail: Some("Detail".to_string()),
            icon: None,
            banner: None,
            api_version: None,
        };
        let manifest = GameManifest {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            detail: String::new(),
            author: String::new(),
            introduction: None,
            icon: None,
            banner: None,
            entry: "scripts/word_puzzle.lua".to_string(),
            save: false,
            best_none: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            actions: Default::default(),
            runtime: Default::default(),
            api: Some(serde_json::json!(7)),
            write: false,
            case_sensitive: false,
        };
        let id = expected_game_id(&package, &manifest);
        assert_eq!(id, expected_game_id(&package, &manifest));
        assert!(id.starts_with("tui_game_"));
        assert_eq!(id.len(), "tui_game_".len() + 16);
        assert!(id["tui_game_".len()..]
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric()));
    }
}
