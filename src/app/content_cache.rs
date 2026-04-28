// 应用程序内容缓存管理，负责在启动时扫描所有游戏和 Mod 包，填充全局缓存，并在后续使用时提供数据查询。同时提供 Mod 文件树指纹检测，供热重载判断

use std::sync::RwLock; // 读写锁，保护全局缓存的并发访问

use once_cell::sync::Lazy; // 惰性静态初始化
use std::collections::hash_map::DefaultHasher; // 构建 Mod 文件树哈希
use std::fs; // 文件系统元数据（文件修改时间、大小）
use std::hash::{Hash, Hasher}; // 哈希计算
use std::path::Path; // 路径操作

use crate::app::i18n; // 国际化文本（加载进度提示）
use crate::game::registry::{GameDescriptor, GameRegistry, PackageDescriptor}; // 游戏注册表相关类型
use crate::game::resources; // 包级语言缓存构建
use crate::mods::{self, ModPackage}; // Mod 包扫描与类型
use crate::utils::host_log; // 错误日志

// 加载进度描述
#[derive(Clone, Debug)]
pub struct LoadingProgress {
    pub percent: u16,
    pub message: String,
}

// 全局应用缓存（私有）
#[derive(Clone, Debug, Default)]
struct AppContentCache {
    games: Vec<GameDescriptor>,
    mods: Vec<ModPackage>,
}

// 无进度的重载，内部调用 reload_with_progress
static CONTENT_CACHE: Lazy<RwLock<AppContentCache>> =
    Lazy::new(|| RwLock::new(AppContentCache::default()));

pub fn reload() {
    reload_with_progress(|_| {});
}

// 执行分阶段加载：扫描游戏 → 扫描 Mod → 收集包元数据 → 重建语言缓存 → 填充游戏显示字段 → 发布缓存。每个阶段回调进度
pub fn reload_with_progress(mut on_progress: impl FnMut(LoadingProgress)) {
    on_progress(LoadingProgress {
        percent: 5,
        message: i18n::t_or("loading.startup.prepare_cache", "Preparing content cache..."),
    });

    on_progress(LoadingProgress {
        percent: 12,
        message: i18n::t_or("loading.startup.scan_games", "Scanning games..."),
    });
    let mut games = match GameRegistry::scan_all() {
        Ok(registry) => registry.into_games(),
        Err(err) => {
            host_log::append_host_error("host.error.raw", &[("err", &err.to_string())]);
            Vec::new()
        }
    };
    on_progress(LoadingProgress {
        percent: 32,
        message: i18n::t_or("loading.startup.scan_games_done", "Game scan complete"),
    });

    on_progress(LoadingProgress {
        percent: 42,
        message: i18n::t_or("loading.startup.scan_mods", "Scanning mod packages..."),
    });
    let mods = match mods::scan_mods() {
        Ok(output) => output.packages,
        Err(err) => {
            host_log::append_host_error("host.error.raw", &[("err", &err.to_string())]);
            Vec::new()
        }
    };
    on_progress(LoadingProgress {
        percent: 62,
        message: i18n::t_or("loading.startup.scan_mods_done", "Mod package scan complete"),
    });

    on_progress(LoadingProgress {
        percent: 70,
        message: i18n::t_or(
            "loading.startup.collect_packages",
            "Collecting package metadata...",
        ),
    });
    let mut packages = Vec::<PackageDescriptor>::new();
    for game in &games {
        if let Some(package) = game.package_info()
            && !packages
                .iter()
                .any(|existing| existing.root_dir == package.root_dir)
        {
            packages.push(package.clone());
        }
    }

    on_progress(LoadingProgress {
        percent: 78,
        message: i18n::t_or(
            "loading.startup.load_languages",
            "Loading language resources...",
        ),
    });
    resources::rebuild_package_language_cache(&packages);
    on_progress(LoadingProgress {
        percent: 82,
        message: i18n::t_or(
            "loading.startup.prepare_display",
            "Preparing display data...",
        ),
    });

    let total_games = games.len().max(1);
    for (index, game) in games.iter_mut().enumerate() {
        hydrate_game_display_fields(game);
        let percent = 82 + (((index + 1) * 14) / total_games) as u16;
        on_progress(LoadingProgress {
            percent: percent.min(96),
            message: format!(
                "{} ({}/{})",
                i18n::t_or("loading.startup.prepare_display", "Preparing display data..."),
                index + 1,
                total_games
            ),
        });
    }

    on_progress(LoadingProgress {
        percent: 98,
        message: i18n::t_or(
            "loading.startup.publish_cache",
            "Publishing preloaded cache...",
        ),
    });
    if let Ok(mut cache) = CONTENT_CACHE.write() {
        *cache = AppContentCache { games, mods };
    }

    on_progress(LoadingProgress {
        percent: 100,
        message: i18n::t_or("loading.startup.ready", "Ready"),
    });
}

// 获取游戏描述符列表的克隆
pub fn games() -> Vec<GameDescriptor> {
    CONTENT_CACHE
        .read()
        .map(|cache| cache.games.clone())
        .unwrap_or_default()
}

// 获取 Mod 包列表的克隆
pub fn mods() -> Vec<ModPackage> {
    CONTENT_CACHE
        .read()
        .map(|cache| cache.mods.clone())
        .unwrap_or_default()
}

// 按 ID 查找单个游戏，返回 Option<GameDescriptor>
pub fn find_game(id: &str) -> Option<GameDescriptor> {
    CONTENT_CACHE
        .read()
        .ok()
        .and_then(|cache| cache.games.iter().find(|game| game.id == id).cloned())
}

// 计算 Mod 目录的文件树哈希（跳过 save/cache/logs 目录），用于热重载检测
pub fn current_mod_tree_fingerprint() -> Option<u64> {
    let root = mods::mod_data_dir().ok()?;
    let mut hasher = DefaultHasher::new();
    hash_mod_tree(&root, &root, &mut hasher);
    Some(hasher.finish())
}

// 为游戏描述符填充本地化显示字段：名称、描述、详情、作者、包名、版本、最佳成绩。有包信息时通过 resources::resolve_package_text 解析，否则使用原始值
fn hydrate_game_display_fields(game: &mut GameDescriptor) {
    if let Some(package) = game.package_info().cloned() {
        game.display_name = resources::resolve_package_text(&package, &game.name);
        game.display_description = resources::resolve_package_text(&package, &game.description);
        game.display_detail = resources::resolve_package_text(&package, &game.detail);
        game.display_author = resources::resolve_package_text(&package, &game.author);
        game.display_package_name = if let Some(mod_name) = package
            .mod_name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            Some(resources::resolve_package_text(&package, mod_name))
        } else {
            Some(package.package_name.clone())
        };
        game.display_package_name_allows_rich = package
            .mod_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_some();
        game.display_package_author =
            Some(resources::resolve_package_text(&package, package.author.as_str()));
        game.display_package_version =
            Some(resources::resolve_package_text(&package, package.version.as_str()));
        game.display_best_none = game
            .best_none
            .as_ref()
            .map(|raw| resources::resolve_package_text(&package, raw))
            .filter(|value| !value.trim().is_empty());
    } else {
        game.display_name = game.name.clone();
        game.display_description = game.description.clone();
        game.display_detail = game.detail.clone();
        game.display_author = game.author.clone();
        game.display_package_name = None;
        game.display_package_name_allows_rich = false;
        game.display_package_author = None;
        game.display_package_version = None;
        game.display_best_none = game.best_none.clone().filter(|value| !value.trim().is_empty());
    }
}

// 递归遍历 Mod 目录，将文件相对路径、大小、修改时间哈希化。跳过 save/cache/logs 子目录
fn hash_mod_tree(root: &Path, path: &Path, hasher: &mut DefaultHasher) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };

    if path != root {
        if let Ok(relative) = path.strip_prefix(root) {
            relative.to_string_lossy().hash(hasher);
        }
        metadata.len().hash(hasher);
        if let Ok(modified) = metadata.modified()
            && let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH)
        {
            duration.as_secs().hash(hasher);
            duration.subsec_nanos().hash(hasher);
        }
    }

    if !metadata.is_dir() {
        return;
    }

    let Ok(read_dir) = fs::read_dir(path) else {
        return;
    };
    let mut children = read_dir
        .filter_map(|entry| entry.ok().map(|item| item.path()))
        .collect::<Vec<_>>();
    children.sort();
    for child in children {
        if child.is_dir()
            && child
                .file_name()
                .and_then(|value| value.to_str())
                .map(|name| name == "save" || name == "cache" || name == "logs")
                .unwrap_or(false)
        {
            continue;
        }
        hash_mod_tree(root, &child, hasher);
    }
}
