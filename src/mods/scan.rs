// Mod 包扫描与验证模块，遍历 mod/ 目录下的所有子目录，加载 package.json 和 game.json，验证包结构和必填字段，生成 ModPackage 结果列表

use std::collections::BTreeMap; // 有序存储脚本文件修改时间
use std::fs; // 文件系统操作
use std::path::{Path, PathBuf}; // 路径处理
use crate::mods::image_from_meta; // 加载 Mod 图标和 Banner

use anyhow::{Result, anyhow}; // 错误处理

use crate::game::package::{GamePackageSource, load_package}; // 加载包清单
use crate::mods::types::*; // 导入所有 Mod 类型
use crate::mods::state; // Mod 状态管理（load_mod_state、save_mod_state、ensure_mod_state_entry 等）
use crate::mods::{mod_cache_dir, mod_data_dir, resolve_mod_text, DEFAULT_PACKAGE_DESCRIPTION, DEFAULT_GAME_DESCRIPTION, DEFAULT_GAME_DETAIL, mtime_secs}; // 路径工具、文本解析、默认值、文件时间戳
use crate::utils::path_utils; // mod_save_dir 路径

// 主扫描函数：遍历 mod/ 目录，对每个子目录调用 scan_package，收集结果和错误，更新状态并保存缓存
pub fn scan_mods() -> Result<ModScanOutput> {
    let root = mod_data_dir()?;
    fs::create_dir_all(&root)?;
    fs::create_dir_all(mod_cache_dir()?)?;
    fs::create_dir_all(path_utils::mod_save_dir()?)?;

    let mut state = state::load_mod_state();
    let mut cache = state::load_scan_cache();
    let mut packages = Vec::new();
    let mut global_errors = Vec::new();

    let mut dirs: Vec<PathBuf> = fs::read_dir(&root)?
        .filter_map(|entry| entry.ok().map(|item| item.path()))
        .filter(|path| path.is_dir())
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .map(|name| name != "save" && name != "cache" && name != "logs")
                .unwrap_or(false)
        })
        .collect();
    dirs.sort();

    for dir in dirs {
        match scan_package(&dir, &mut state, &mut cache) {
            Ok(Some(package)) => {
                global_errors.extend(package.errors.clone());
                packages.push(package);
            }
            Ok(None) => {}
            Err(err) => {
                let namespace = dir
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                global_errors.push(scan_error(
                    &namespace,
                    "package",
                    "package.json",
                    "error",
                    format!("mod package scan failed: {err}"),
                ));
            }
        }
    }

    state.api_version = MOD_API_VERSION;
    state.scan_errors = global_errors;
    state::save_mod_state(&state)?;
    state::save_scan_cache(&cache)?;

    Ok(ModScanOutput { packages })
}

// 扫描单个 Mod 包：加载 package.json → 验证根清单 → 解析文本 → 加载图像 → 验证目录结构 → 遍历游戏清单生成 ModGameMeta 列表
fn scan_package(
    dir: &Path,
    state: &mut ModState,
    cache: &mut ModScanCache,
) -> Result<Option<ModPackage>> {
    let package_path = dir.join("package.json");
    if !package_path.exists() {
        return Ok(None);
    }

    let package = load_package(dir, GamePackageSource::Mod)?;
    validate_mod_package_root(dir, &package.package)?;

    let namespace = package.package.namespace.clone();
    let state_entry = state::ensure_mod_state_entry(state, &namespace);
    state_entry.package_name = package.package.package_name.clone();
    state_entry.author = package.package.author.clone();
    state_entry.version = package.package.version.clone();

    let description = resolve_mod_text(
        &namespace,
        if package.package.description.trim().is_empty() {
            DEFAULT_PACKAGE_DESCRIPTION
        } else {
            package.package.description.as_str()
        },
    );
    let introduction = resolve_mod_text(
        &namespace,
        package
            .package
            .introduction
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(DEFAULT_PACKAGE_DESCRIPTION),
    );
    let thumbnail = image_from_meta(
        &namespace,
        package.package.icon.as_ref(),
        ImageKind::Thumbnail,
    )?;
    let banner = image_from_meta(&namespace, package.package.banner.as_ref(), ImageKind::Banner)?;

    let package_name_source = package
        .package
        .mod_name
        .as_deref()
        .filter(|value| !value.trim().is_empty());
    let package_name_allows_rich = package_name_source.is_some();
    let package_name = if let Some(raw) = package_name_source {
        resolve_mod_text(&namespace, raw)
    } else {
        package.package.package_name.clone()
    };
    let author = resolve_mod_text(&namespace, &package.package.author);
    let version = resolve_mod_text(&namespace, &package.package.version);
    let enabled = state_entry.enabled;
    let debug_enabled = state_entry.debug_enabled;
    let safe_mode_state = if let Some(false) = state_entry.session_safe_mode_enabled {
        ModSafeModeState::DisabledSession
    } else if !state_entry.safe_mode_enabled {
        ModSafeModeState::DisabledTrusted
    } else {
        ModSafeModeState::Enabled
    };
    let safe_mode_enabled = state_entry
        .session_safe_mode_enabled
        .unwrap_or(state_entry.safe_mode_enabled);

    let mut errors = Vec::new();
    validate_mod_structure(dir)?;

    if package.games.is_empty() {
        errors.push(scan_error(
            &namespace,
            "package",
            "game.json",
            "warning",
            "no game manifests found".to_string(),
        ));
        cache.packages.insert(
            namespace,
            CachedPackage {
                meta_mtime: mtime_secs(&package_path),
                scan_ok: false,
                ..Default::default()
            },
        );
        return Ok(None);
    }

    let mut games = Vec::new();
    let mut script_mtimes = BTreeMap::new();
    for game_manifest in &package.games {
        let script_path = resolve_mod_entry_path(dir, &game_manifest.entry);
        let script_name = Path::new(&game_manifest.entry)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("game")
            .to_string();
        script_mtimes.insert(script_name.clone(), mtime_secs(&script_path));
        match scan_game_manifest(&namespace, dir, &package.package, game_manifest) {
            Ok(game) => {
                state_entry
                    .games
                    .entry(game.game_id.clone())
                    .or_insert_with(|| ModGameState {
                        script_name: game.script_name.clone(),
                        ..Default::default()
                    });
                games.push(game);
            }
            Err(err) => {
                errors.push(scan_error(
                    &namespace,
                    "game",
                    &game_manifest.entry,
                    "error",
                    err.to_string(),
                ));
            }
        }
    }

    cache.packages.insert(
        namespace.clone(),
        CachedPackage {
            meta_mtime: mtime_secs(&package_path),
            script_mtimes,
            thumbnail_cache_key: None,
            banner_cache_key: None,
            scan_ok: !games.is_empty(),
        },
    );

    if games.is_empty() {
        return Ok(None);
    }

    let has_best_score_storage = games.iter().any(|game| game.best_none.is_some());
    let has_save_storage = games.iter().any(|game| game.save);
    let has_write_request = package.games.iter().any(|game| game.write);

    Ok(Some(ModPackage {
        namespace,
        enabled,
        debug_enabled,
        safe_mode_enabled,
        safe_mode_state,
        package_name,
        package_name_allows_rich,
        author,
        version,
        introduction,
        description,
        has_best_score_storage,
        has_save_storage,
        has_write_request,
        thumbnail,
        banner,
        games,
        errors,
    }))
}

// 解析单个游戏清单：验证入口脚本存在、解析名称/描述/详情/介绍/最佳成绩文本
fn scan_game_manifest(
    namespace: &str,
    package_dir: &Path,
    package_manifest: &crate::game::manifest::PackageManifest,
    game_manifest: &crate::game::manifest::GameManifest,
) -> Result<ModGameMeta> {
    let script_path = resolve_mod_entry_path(package_dir, &game_manifest.entry);
    if !script_path.exists() || !script_path.is_file() {
        return Err(anyhow!(
            "game entry does not exist: {}",
            script_path.display()
        ));
    }

    let script_name = Path::new(&game_manifest.entry)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("game")
        .to_string();

    let raw_name = package_manifest
        .game_name
        .as_deref()
        .unwrap_or(&game_manifest.name);
    let name = resolve_mod_text(namespace, raw_name);
    if name.trim().is_empty() {
        return Err(anyhow!("game manifest name cannot be blank"));
    }

    let raw_description = if package_manifest.description.trim().is_empty() {
        game_manifest.description.as_str()
    } else {
        package_manifest.description.as_str()
    };
    let description = if raw_description.trim().is_empty() {
        DEFAULT_GAME_DESCRIPTION.to_string()
    } else {
        resolve_mod_text(namespace, raw_description)
    };

    let raw_detail = package_manifest.detail.as_deref().unwrap_or(&game_manifest.detail);
    let detail = if raw_detail.trim().is_empty() {
        DEFAULT_GAME_DETAIL.to_string()
    } else {
        resolve_mod_text(namespace, raw_detail)
    };
    let raw_introduction = game_manifest
        .introduction
        .as_deref()
        .or(package_manifest.introduction.as_deref())
        .unwrap_or(DEFAULT_PACKAGE_DESCRIPTION);
    let introduction = resolve_mod_text(namespace, raw_introduction);

    let best_none = game_manifest
        .best_none
        .as_deref()
        .map(|value| resolve_mod_text(namespace, value))
        .filter(|value| !value.trim().is_empty());

    Ok(ModGameMeta {
        game_id: game_manifest.id.clone(),
        script_name,
        script_path,
        name,
        description,
        detail,
        introduction,
        best_none,
        save: game_manifest.save,
        write: game_manifest.write,
        min_width: game_manifest.min_width.filter(|value| *value > 0),
        min_height: game_manifest.min_height.filter(|value| *value > 0),
        max_width: game_manifest.max_width.filter(|value| *value > 0),
        max_height: game_manifest.max_height.filter(|value| *value > 0),
    })
}

// 验证 package.json 的必填字段：命名空间与目录名一致、仅允许 ASCII 字母数字和下划线、包名/作者/介绍/游戏名/描述/详情非空
pub fn validate_mod_package_root(dir: &Path, package: &crate::game::manifest::PackageManifest) -> Result<()> {
    let folder_name = dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow!("invalid mod directory name"))?;

    if package.namespace != folder_name {
        return Err(anyhow!("namespace must match directory name"));
    }
    if !package
        .namespace
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err(anyhow!(
            "namespace only allows letters, numbers, and underscore"
        ));
    }
    if package.package_name.trim().is_empty() {
        return Err(anyhow!("package_name cannot be blank"));
    }
    if package.author.trim().is_empty() {
        return Err(anyhow!("author cannot be blank"));
    }
    if package
        .introduction
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        return Err(anyhow!("introduction cannot be blank"));
    }
    if package
        .game_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        return Err(anyhow!("name cannot be blank"));
    }
    if package.description.trim().is_empty() {
        return Err(anyhow!("description cannot be blank"));
    }
    if package
        .detail
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        return Err(anyhow!("detail cannot be blank"));
    }
    Ok(())
}

// 验证 Mod 包的目录结构：scripts/ 目录及 main.lua 存在、assets/ 目录存在、assets/lang/ 目录及 en_us.json 存在
pub fn validate_mod_structure(dir: &Path) -> Result<()> {
    let scripts_dir = dir.join("scripts");
    let main_script = scripts_dir.join("main.lua");
    let assets_dir = dir.join("assets");
    let lang_dir = assets_dir.join("lang");
    let en_us = lang_dir.join("en_us.json");

    if !scripts_dir.is_dir() {
        return Err(anyhow!("scripts directory is missing"));
    }
    if !main_script.is_file() {
        return Err(anyhow!("scripts/main.lua is missing"));
    }
    if !assets_dir.is_dir() {
        return Err(anyhow!("assets directory is missing"));
    }
    if !lang_dir.is_dir() {
        return Err(anyhow!("assets/lang directory is missing"));
    }
    if !en_us.is_file() {
        return Err(anyhow!("assets/lang/en_us.json is missing"));
    }
    Ok(())
}

// 解析游戏入口脚本路径：自动补全 scripts/ 前缀
pub fn resolve_mod_entry_path(package_dir: &Path, entry: &str) -> PathBuf {
    if entry.starts_with("scripts/") || entry.starts_with("scripts\\") {
        package_dir.join(entry)
    } else {
        package_dir.join("scripts").join(entry)
    }
}

// 创建一个 ModScanError 实例
pub fn scan_error(
    namespace: &str,
    scope: &str,
    target: impl Into<String>,
    severity: &str,
    message: impl Into<String>,
) -> ModScanError {
    ModScanError {
        namespace: namespace.to_string(),
        scope: scope.to_string(),
        target: target.into(),
        severity: severity.to_string(),
        message: message.into(),
    }
}