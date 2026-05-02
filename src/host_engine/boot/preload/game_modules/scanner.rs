//! 游戏模块扫描与清单校验

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use serde_json::{Map, Value};

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
        mod_name: require_string(&value, "package.json", "mod_name")?,
        introduction: require_string(&value, "package.json", "introduction")?,
        author: require_string(&value, "package.json", "author")?,
        game_name: require_string(&value, "package.json", "game_name")?,
        description: require_string(&value, "package.json", "description")?,
        detail: require_string(&value, "package.json", "detail")?,
        version: require_string(&value, "package.json", "version")?,
        icon: require_value(&value, "package.json", "icon")?.clone(),
        banner: require_value(&value, "package.json", "banner")?.clone(),
    })
}

fn read_game_manifest(package_dir: &Path) -> ScannerResult<GameManifest> {
    let path = package_dir.join("game.json");
    let value = read_json_object(&path)?;
    let runtime = require_object(&value, "game.json", "runtime")?;

    Ok(GameManifest {
        api: require_value(&value, "game.json", "api")?.clone(),
        entry: require_string(&value, "game.json", "entry")?,
        save: require_bool(&value, "game.json", "save")?,
        best_none: require_optional_string(&value, "game.json", "best_none")?,
        min_width: require_integer(&value, "game.json", "min_width")?,
        min_height: require_integer(&value, "game.json", "min_height")?,
        write: require_bool(&value, "game.json", "write")?,
        case_sensitive: require_bool(&value, "game.json", "case_sensitive")?,
        actions: require_actions(&value)?,
        runtime: GameRuntimeManifest {
            target_fps: require_u16(runtime, "game.json", "runtime.target_fps")?,
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

fn require_u16(
    object: &Map<String, Value>,
    file_name: &str,
    field_name: &str,
) -> ScannerResult<u16> {
    let value = object
        .get("target_fps")
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
        let key_value = require_value(
            action_object,
            "game.json",
            &format!("actions.{action_name}.key"),
        )?;
        validate_action_key(key_value, action_name)?;
        let key_name = require_string(
            action_object,
            "game.json",
            &format!("actions.{action_name}.key_name"),
        )?;
        bindings.insert(
            action_name.clone(),
            GameActionBinding {
                key: key_value.clone(),
                key_name,
            },
        );
    }

    Ok(bindings)
}

fn validate_action_key(value: &Value, action_name: &str) -> ScannerResult<()> {
    match value {
        Value::String(text) if !text.trim().is_empty() => Ok(()),
        Value::Array(values) if !values.is_empty() => {
            for (index, item) in values.iter().enumerate() {
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
            }
            Ok(())
        }
        _ => Err(field_type_error(
            "game.json",
            &format!("actions.{action_name}.key"),
            "string | array",
            value,
        )),
    }
}

fn generate_game_uid(
    source: GameModuleSource,
    _package_dir: &Path,
    package: &PackageManifest,
    game: &GameManifest,
) -> String {
    let seed = format!(
        "{}|{}|{}|{}|{}",
        source.as_str(),
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
