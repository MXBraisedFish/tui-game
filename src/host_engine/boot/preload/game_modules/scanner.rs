//! 游戏模块扫描与清单校验

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use serde_json::{Map, Value};

use crate::host_engine::constant::{
    API_VERSION, DEFAULT_GAME_BANNER, DEFAULT_PACKAGE_ICON, MAX_ACTION_KEYS,
};

use super::manifest::{
    GameActionBinding, GameManifest, GameModule, GameModuleRegistry, GameModuleScanError,
    GameRuntimeManifest, PackageManifest,
};
use super::source::GameModuleSource;
use super::uid;

type ScannerResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 扫描指定来源的游戏模块
pub fn scan_source(source: GameModuleSource) -> ScannerResult<GameModuleRegistry> {
    let root_dir = source.root_dir();
    let mut registry = GameModuleRegistry::default();
    if !root_dir.is_dir() {
        return Ok(registry);
    }

    let mut entries = fs::read_dir(&root_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    entries.sort();

    for package_dir in entries {
        match read_game_module(&package_dir, source) {
            Ok(game_module) => registry.games.push(game_module),
            Err(error) => registry.errors.push(GameModuleScanError {
                source: source.as_str().to_string(),
                path: package_dir.display().to_string(),
                error: error.to_string(),
            }),
        }
    }

    Ok(registry)
}

fn read_game_module(package_dir: &Path, source: GameModuleSource) -> ScannerResult<GameModule> {
    let package = read_package_manifest(package_dir)?;
    let game = read_game_manifest(package_dir)?;
    let uid = generate_game_uid(source, package_dir, &package, &game);

    Ok(GameModule {
        uid,
        source,
        source_label: source.as_str().to_string(),
        root_dir: package_dir.to_path_buf(),
        package,
        game,
    })
}

fn read_package_manifest(package_dir: &Path) -> ScannerResult<PackageManifest> {
    let path = package_dir.join("package.json");
    let value = read_json_object(&path)?;

    Ok(PackageManifest {
        package: require_string(&value, "package.json", "package")?,
        package_name: require_string(&value, "package.json", "package_name")?,
        introduction: require_string(&value, "package.json", "introduction")?,
        author: require_string(&value, "package.json", "author")?,
        game_name: require_string(&value, "package.json", "game_name")?,
        description: require_string(&value, "package.json", "description")?,
        detail: require_string(&value, "package.json", "detail")?,
        version: require_string(&value, "package.json", "version")?,
        icon: image_or_default(&value, package_dir, "icon", DEFAULT_PACKAGE_ICON),
        banner: image_or_default(&value, package_dir, "banner", DEFAULT_GAME_BANNER),
    })
}

fn image_or_default(
    object: &Map<String, Value>,
    package_dir: &Path,
    field_name: &str,
    default_lines: &[&str],
) -> Value {
    object
        .get(field_name)
        .filter(|value| is_valid_image_field(package_dir, value))
        .cloned()
        .unwrap_or_else(|| default_lines_value(default_lines))
}

fn default_lines_value(lines: &[&str]) -> Value {
    Value::Array(
        lines
            .iter()
            .map(|line| Value::String((*line).to_string()))
            .collect(),
    )
}

fn is_valid_image_field(package_dir: &Path, value: &Value) -> bool {
    match value {
        Value::Array(values) => {
            !values.is_empty()
                && values
                    .iter()
                    .all(|value| value.as_str().is_some_and(|text| !text.is_empty()))
        }
        Value::String(text) => {
            let text = text.trim();
            if text.is_empty() {
                return false;
            }
            if text.starts_with("image:") || text.starts_with("color:image:") {
                return image_reference_exists(package_dir, text);
            }
            true
        }
        _ => false,
    }
}

fn image_reference_exists(package_dir: &Path, text: &str) -> bool {
    let image_path = text
        .strip_prefix("color:")
        .unwrap_or(text)
        .strip_prefix("image:")
        .unwrap_or("")
        .trim();
    let Some(clean_path) = normalize_image_path(image_path) else {
        return false;
    };
    package_dir.join("assets").join(clean_path).is_file()
}

fn normalize_image_path(path: &str) -> Option<PathBuf> {
    if path.is_empty() || Path::new(path).is_absolute() {
        return None;
    }

    let mut clean_path = PathBuf::new();
    for component in PathBuf::from(path).components() {
        match component {
            Component::Normal(part) => clean_path.push(part),
            Component::CurDir
            | Component::ParentDir
            | Component::Prefix(_)
            | Component::RootDir => {
                return None;
            }
        }
    }

    let extension = clean_path.extension()?.to_str()?.to_ascii_lowercase();
    matches!(extension.as_str(), "png" | "jpg" | "jpeg").then_some(clean_path)
}

fn read_game_manifest(package_dir: &Path) -> ScannerResult<GameManifest> {
    let path = package_dir.join("game.json");
    let value = read_json_object(&path)?;
    let runtime = require_object(&value, "game.json", "runtime")?;
    let api = require_value(&value, "game.json", "api")?.clone();
    validate_api_version(&api)?;

    Ok(GameManifest {
        api,
        entry: require_string(&value, "game.json", "entry")?,
        save: require_bool(&value, "game.json", "save")?,
        best_none: require_optional_string(&value, "game.json", "best_none")?,
        min_width: require_integer(&value, "game.json", "min_width")?,
        min_height: require_integer(&value, "game.json", "min_height")?,
        write: require_bool(&value, "game.json", "write")?,
        case_sensitive: require_bool(&value, "game.json", "case_sensitive")?,
        actions: require_actions(&value)?,
        runtime: GameRuntimeManifest {
            target_fps: require_u16(runtime, "game.json", "runtime.target_fps", "target_fps")?,
            afk_time: require_non_negative_u64(runtime, "game.json", "runtime.afk_time")?,
        },
    })
}

fn read_json_object(path: &Path) -> ScannerResult<Map<String, Value>> {
    let raw_json = fs::read_to_string(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    let value = serde_json::from_str::<Value>(raw_json.trim_start_matches('\u{feff}'))?;
    value.as_object().cloned().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} must be a JSON object", path.display()),
        )
        .into()
    })
}

fn require_value<'a>(
    object: &'a Map<String, Value>,
    file_name: &str,
    field_name: &str,
) -> ScannerResult<&'a Value> {
    object
        .get(field_name)
        .ok_or_else(|| field_missing_error(file_name, field_name))
}

fn require_string(
    object: &Map<String, Value>,
    file_name: &str,
    field_name: &str,
) -> ScannerResult<String> {
    let value = require_value(object, file_name, field_name)?;
    let text = value
        .as_str()
        .ok_or_else(|| field_type_error(file_name, field_name, "string", value))?;
    if text.trim().is_empty() {
        return Err(field_missing_error(file_name, field_name));
    }
    Ok(text.to_string())
}

fn require_optional_string(
    object: &Map<String, Value>,
    file_name: &str,
    field_name: &str,
) -> ScannerResult<Option<String>> {
    let value = require_value(object, file_name, field_name)?;
    if value.is_null() {
        return Ok(None);
    }
    let text = value
        .as_str()
        .ok_or_else(|| field_type_error(file_name, field_name, "string | null", value))?;
    Ok(Some(text.to_string()))
}

fn require_bool(
    object: &Map<String, Value>,
    file_name: &str,
    field_name: &str,
) -> ScannerResult<bool> {
    let value = require_value(object, file_name, field_name)?;
    value
        .as_bool()
        .ok_or_else(|| field_type_error(file_name, field_name, "boolean", value))
}

fn require_integer(
    object: &Map<String, Value>,
    file_name: &str,
    field_name: &str,
) -> ScannerResult<i64> {
    let value = require_value(object, file_name, field_name)?;
    value
        .as_i64()
        .ok_or_else(|| field_type_error(file_name, field_name, "integer", value))
}

fn require_non_negative_u64(
    object: &Map<String, Value>,
    file_name: &str,
    field_name: &str,
) -> ScannerResult<u64> {
    let value = require_value(object, file_name, field_name)?;
    value
        .as_u64()
        .ok_or_else(|| field_type_error(file_name, field_name, "non-negative integer", value))
}

fn require_u16(
    object: &Map<String, Value>,
    file_name: &str,
    field_name: &str,
    json_key: &str,
) -> ScannerResult<u16> {
    let value = object
        .get(json_key)
        .ok_or_else(|| field_missing_error(file_name, field_name))?;
    let number = value
        .as_u64()
        .ok_or_else(|| field_type_error(file_name, field_name, "integer", value))?;
    u16::try_from(number).map_err(|_| field_type_error(file_name, field_name, "u16", value))
}

fn require_object<'a>(
    object: &'a Map<String, Value>,
    file_name: &str,
    field_name: &str,
) -> ScannerResult<&'a Map<String, Value>> {
    let value = require_value(object, file_name, field_name)?;
    value
        .as_object()
        .ok_or_else(|| field_type_error(file_name, field_name, "object", value))
}

fn require_actions(
    object: &Map<String, Value>,
) -> ScannerResult<BTreeMap<String, GameActionBinding>> {
    let actions = require_object(object, "game.json", "actions")?;
    let mut bindings = BTreeMap::new();

    for (action_name, action_value) in actions {
        let action_object = action_value.as_object().ok_or_else(|| {
            field_type_error(
                "game.json",
                &format!("actions.{action_name}"),
                "object",
                action_value,
            )
        })?;
        let key_value = action_object.get("key").ok_or_else(|| {
            field_missing_error("game.json", &format!("actions.{action_name}.key"))
        })?;
        let key_value = normalize_action_key(key_value, action_name)?;
        let key_name_value = action_object.get("key_name").ok_or_else(|| {
            field_missing_error("game.json", &format!("actions.{action_name}.key_name"))
        })?;
        let key_name = key_name_value
            .as_str()
            .ok_or_else(|| {
                field_type_error(
                    "game.json",
                    &format!("actions.{action_name}.key_name"),
                    "string",
                    key_name_value,
                )
            })?
            .to_string();
        if key_name.trim().is_empty() {
            return Err(field_missing_error(
                "game.json",
                &format!("actions.{action_name}.key_name"),
            ));
        }
        bindings.insert(
            action_name.clone(),
            GameActionBinding {
                key: key_value,
                key_name,
            },
        );
    }

    Ok(bindings)
}

fn normalize_action_key(value: &Value, action_name: &str) -> ScannerResult<Value> {
    match value {
        Value::String(text) if !text.trim().is_empty() => Ok(value.clone()),
        Value::Array(values) if !values.is_empty() => {
            let mut normalized_keys = Vec::new();
            for (index, item) in values.iter().take(MAX_ACTION_KEYS).enumerate() {
                let Some(text) = item.as_str() else {
                    return Err(field_type_error(
                        "game.json",
                        &format!("actions.{action_name}.key[{}]", index + 1),
                        "string",
                        item,
                    ));
                };
                if text.trim().is_empty() {
                    return Err(field_missing_error(
                        "game.json",
                        &format!("actions.{action_name}.key[{}]", index + 1),
                    ));
                }
                normalized_keys.push(Value::String(text.to_string()));
            }
            Ok(Value::Array(normalized_keys))
        }
        _ => Err(field_type_error(
            "game.json",
            &format!("actions.{action_name}.key"),
            "string | array",
            value,
        )),
    }
}

fn validate_api_version(api: &Value) -> ScannerResult<()> {
    match api {
        Value::Number(number) => {
            let Some(version) = number.as_i64() else {
                return Err(api_version_type_error(api));
            };
            if version == -1 || version == i64::from(API_VERSION) {
                return Ok(());
            }
            Err(api_version_mismatch_error(api))
        }
        Value::Array(values) if values.len() == 2 => {
            let Some(min_version) = values[0].as_i64() else {
                return Err(api_version_type_error(api));
            };
            let Some(max_version) = values[1].as_i64() else {
                return Err(api_version_type_error(api));
            };
            let host_version = i64::from(API_VERSION);
            if min_version <= host_version && host_version <= max_version {
                return Ok(());
            }
            Err(api_version_mismatch_error(api))
        }
        _ => Err(api_version_type_error(api)),
    }
}

fn generate_game_uid(
    source: GameModuleSource,
    package_dir: &Path,
    package: &PackageManifest,
    game: &GameManifest,
) -> String {
    let namespace = package_dir
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    let seed = format!(
        "{}|{}|{}|{}|{}|{}",
        source.as_str(),
        namespace,
        package.package,
        package.game_name,
        package.author,
        game.entry
    );
    format!("{}{}", source.uid_prefix(), uid::hash_base62_16(&seed))
}

fn field_missing_error(file_name: &str, field_name: &str) -> Box<dyn std::error::Error> {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{file_name} missing required field: {field_name}"),
    )
    .into()
}

fn api_version_mismatch_error(actual_value: &Value) -> Box<dyn std::error::Error> {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "game.json api version mismatch: expected {}, got {}",
            API_VERSION, actual_value
        ),
    )
    .into()
}

fn api_version_type_error(actual_value: &Value) -> Box<dyn std::error::Error> {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "game.json field api type mismatch: expected -1 | integer | [min, max], got {}",
            json_type_name(actual_value)
        ),
    )
    .into()
}

fn field_type_error(
    file_name: &str,
    field_name: &str,
    expected_type: &str,
    actual_value: &Value,
) -> Box<dyn std::error::Error> {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "{file_name} field {field_name} type mismatch: expected {expected_type}, got {}",
            json_type_name(actual_value)
        ),
    )
    .into()
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
