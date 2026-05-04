//! 启动前最终校验

use std::collections::HashSet;
use std::io;

use crate::host_engine::boot::preload::cache_data::CacheData;
use crate::host_engine::boot::preload::game_modules::GameModuleRegistry;
use crate::host_engine::boot::preload::official_ui::OfficialUiRegistry;
use crate::host_engine::boot::preload::persistent_data::PersistentData;
use crate::host_engine::boot::preload::state_machine::{HostStateMachine, TopLevelState};

use super::readiness::LaunchReadiness;

const TODO_BUILD_RUNTIME_CONTEXT: &str =
    "TODO: build merged runtime context from game modules, profiles, and cache data";
const TODO_VALIDATE_UI_ENTRY: &str =
    "TODO: validate official UI Lua entry scripts and UI API contracts";
const TODO_PREPARE_IMAGE_CACHE: &str =
    "TODO: prepare game icon/banner render cache for runtime consumption";
const TODO_PREPARE_KEYBIND_CONTEXT: &str =
    "TODO: merge original keybinds with user keybind preferences";

/// 校验启动前资源是否具备最低可运行条件。
pub fn validate_launch_readiness(
    game_module_registry: &GameModuleRegistry,
    official_ui_registry: &OfficialUiRegistry,
    persistent_data: &PersistentData,
    cache_data: &CacheData,
    host_state_machine: &HostStateMachine,
) -> Result<LaunchReadiness, Box<dyn std::error::Error>> {
    validate_cache_matches_game_modules(game_module_registry, cache_data)?;
    validate_persistent_data(persistent_data)?;
    validate_state_machine(host_state_machine)?;
    validate_cache_paths(cache_data)?;

    Ok(LaunchReadiness {
        game_count: game_module_registry.games.len(),
        game_scan_error_count: game_module_registry.errors.len(),
        official_ui_package_count: official_ui_registry.packages.len(),
        official_ui_scan_error_count: official_ui_registry.errors.len(),
        removed_game_cache_count: cache_data.removed_game_uids.len(),
        image_cache_dir: cache_data.image_cache_dir.clone(),
        todo_items: vec![
            TODO_BUILD_RUNTIME_CONTEXT,
            TODO_VALIDATE_UI_ENTRY,
            TODO_PREPARE_IMAGE_CACHE,
            TODO_PREPARE_KEYBIND_CONTEXT,
        ],
    })
}

fn validate_cache_matches_game_modules(
    game_module_registry: &GameModuleRegistry,
    cache_data: &CacheData,
) -> Result<(), Box<dyn std::error::Error>> {
    let scanned_game_uids = game_module_registry
        .games
        .iter()
        .map(|game_module| game_module.uid.as_str())
        .collect::<HashSet<_>>();
    let cached_game_uids = cache_data
        .current_game_module_registry
        .games
        .iter()
        .map(|game_module| game_module.uid.as_str())
        .collect::<HashSet<_>>();

    if scanned_game_uids == cached_game_uids {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "game module scan cache does not match current scanned modules",
        )
        .into())
    }
}

fn validate_persistent_data(
    persistent_data: &PersistentData,
) -> Result<(), Box<dyn std::error::Error>> {
    if persistent_data.language_code.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "profile language code must not be empty",
        )
        .into());
    }

    validate_json_object(&persistent_data.saves, "saves")?;
    validate_json_object(&persistent_data.best_scores, "best_scores")?;
    validate_json_object(&persistent_data.keybinds, "keybinds")?;
    validate_json_object(&persistent_data.mod_state, "mod_state")?;

    Ok(())
}

fn validate_json_object(
    value: &serde_json::Value,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if value.is_object() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("persistent profile data `{name}` must be a JSON object"),
        )
        .into())
    }
}

fn validate_state_machine(
    host_state_machine: &HostStateMachine,
) -> Result<(), Box<dyn std::error::Error>> {
    if host_state_machine.top_level_state != TopLevelState::Home {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "host state machine must start from Home",
        )
        .into());
    }

    if host_state_machine.has_dialog() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "host state machine must not start with an active dialog",
        )
        .into());
    }

    Ok(())
}

fn validate_cache_paths(cache_data: &CacheData) -> Result<(), Box<dyn std::error::Error>> {
    if cache_data.image_cache_dir.is_dir() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "image cache directory is missing: {}",
                cache_data.image_cache_dir.display()
            ),
        )
        .into())
    }
}
