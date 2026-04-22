use std::sync::RwLock;

use once_cell::sync::Lazy;

use crate::game::registry::{GameDescriptor, GameRegistry, PackageDescriptor};
use crate::game::resources;
use crate::mods::{self, ModPackage};
use crate::utils::host_log;

#[derive(Clone, Debug, Default)]
struct AppContentCache {
    games: Vec<GameDescriptor>,
    mods: Vec<ModPackage>,
}

static CONTENT_CACHE: Lazy<RwLock<AppContentCache>> =
    Lazy::new(|| RwLock::new(AppContentCache::default()));

pub fn reload() {
    let mut games = match GameRegistry::scan_all() {
        Ok(registry) => registry.into_games(),
        Err(err) => {
            host_log::append_host_error("host.error.raw", &[("err", &err.to_string())]);
            Vec::new()
        }
    };

    let mods = match mods::scan_mods() {
        Ok(output) => output.packages,
        Err(err) => {
            host_log::append_host_error("host.error.raw", &[("err", &err.to_string())]);
            Vec::new()
        }
    };

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

    resources::rebuild_package_language_cache(&packages);
    for game in &mut games {
        hydrate_game_display_fields(game);
    }

    if let Ok(mut cache) = CONTENT_CACHE.write() {
        *cache = AppContentCache { games, mods };
    }
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
