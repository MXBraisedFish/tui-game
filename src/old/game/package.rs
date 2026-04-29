// 游戏包的发现、加载与验证。核心功能包括：扫描 official_games/ 和 mods_data/ 目录下的包，解析 package.json 和 game.json，验证字段完整性、API 版本、动作键绑定唯一性等，并为每个游戏生成稳定的唯一 ID

use std::collections::HashMap; // 存储重复键检测用的桶
use std::fs; // 目录遍历、文件读取
use std::path::{Path, PathBuf}; // 路径操作

use anyhow::{Context, Result, anyhow}; // 错误处理
use serde_json::Value; // JSON 值类型

use crate::app::i18n; // 国际化错误消息
use crate::core::key::is_explicit_semantic_key; // 验证键名是否为显式语义键
use crate::game::action::ActionKeys; // 动作键枚举
use crate::game::manifest::{GameManifest, PackageManifest}; // 清单结构
use crate::utils::host_log; // 日志记录

pub const HOST_GAME_API_VERSION: u32 = 7; // 宿主当前支持的 Lua API 版本号
const MAX_ACTION_KEYS_PER_BINDING: usize = 5; // 单个动作最多可绑定的键数量

// 区分包的来源，影响验证规则和存储位置
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GamePackageSource {
    Official, // 官方游戏（内置或用户安装）
    Mod, // Mod 游戏
}

// 代表一个发现并成功加载的包，可能包含多个游戏
#[derive(Clone, Debug)]
pub struct GamePackage {
    pub root_dir: PathBuf,
    pub source: GamePackageSource,
    pub package: PackageManifest,
    pub games: Vec<GameManifest>,
}

// 扫描目录下的所有有效包（每个子目录需包含 package.json）
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

// 加载单个包（读取 package.json 和 game.json/games/*.json）
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

// 生成稳定的游戏 ID（格式 tui_game_{16位哈希}）
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

// 读取并解析 package.json，Mod 包自动补全空命名空间为目录名
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

// 读取包根下的 game.json 和 games/*.json
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

// 读取单个游戏清单，进行形状验证、动作键截断、绑定去重检查
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

// 若动作键超过 5 个，截断并发出警告
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

// 检查同一游戏内是否有多个动作绑定到相同的键（大小写敏感由 case_sensitive 决定）
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

// 综合验证：入口脚本路径、尺寸约束、Mod 特有要求（作者、介绍等）
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

// 检查游戏的 api 字段是否与宿主版本兼容
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

// 将 API 值格式化为可读字符串用于错误消息
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

// 使用 FNV-1a + SplitMix64 生成 16 位 Base62 字符串，用于稳定的游戏 ID
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

// FNV-1a 哈希算法
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

// SplitMix64 伪随机数生成器，用于扩散哈希
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}