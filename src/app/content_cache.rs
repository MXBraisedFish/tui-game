use std::sync::RwLock;

use once_cell::sync::Lazy;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::app::i18n;
use crate::game::registry::{GameDescriptor, GameRegistry, PackageDescriptor};
use crate::game::resources;
use crate::mods::{self, ModPackage};
use crate::utils::host_log;

#[derive(Clone, Debug)]
pub struct LoadingProgress {
    pub percent: u16,
    pub message: String,
}

#[derive(Clone, Debug, Default)]
struct AppContentCache {
    games: Vec<GameDescriptor>,
    mods: Vec<ModPackage>,
}

static CONTENT_CACHE: Lazy<RwLock<AppContentCache>> =
    Lazy::new(|| RwLock::new(AppContentCache::default()));

pub fn reload() {
    reload_with_progress(|_| {});
}

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

pub fn games() -> Vec<GameDescriptor> {
    CONTENT_CACHE
        .read()
        .map(|cache| cache.games.clone())
        .unwrap_or_default()
}

pub fn mods() -> Vec<ModPackage> {
    CONTENT_CACHE
        .read()
        .map(|cache| cache.mods.clone())
        .unwrap_or_default()
}

pub fn find_game(id: &str) -> Option<GameDescriptor> {
    CONTENT_CACHE
        .read()
        .ok()
        .and_then(|cache| cache.games.iter().find(|game| game.id == id).cloned())
}

pub fn current_mod_tree_fingerprint() -> Option<u64> {
    let root = mods::mod_data_dir().ok()?;
    let mut hasher = DefaultHasher::new();
    hash_mod_tree(&root, &root, &mut hasher);
    Some(hasher.finish())
}

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
