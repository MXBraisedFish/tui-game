use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use serde_json::{Value, json};

use crate::host_engine::boot::loading::{LoadingHandle, LoadingStage};
use crate::host_engine::boot::preload::game_modules::manifest::GameModuleScanError;
use crate::host_engine::boot::preload::{game_modules, overlay_modules};
use crate::host_engine::keybind::action_schema::{
    ActionDefault as KeybindActionDefault, ActionSchema,
};
use crate::host_engine::package::kind::{ColorPack, GamePackage, OverlayPackage};
use crate::host_engine::package::manifest::parse_manifest;
use crate::host_engine::package::package_id::{PackageId, PackageKind};
use crate::host_engine::package::package_id_registry::PackageIdRegistry;
use crate::host_engine::package::registry::{Package, PackageRegistry};
use crate::host_engine::package::scanner::Scanner;
use crate::host_engine::package::validator::validate_package_at;
use crate::host_engine::runtime::event_dispatch::EngineEvent;
use crate::host_engine::storage::cache_store::CacheStore;
use crate::host_engine::storage::profile_store::ProfileStore;

type PackageManagerResult<T> = Result<T, Box<dyn Error>>;

#[derive(Clone, Copy, Debug)]
pub enum PackageRef<'a> {
    Game(&'a GamePackage),
    Screensaver(&'a OverlayPackage),
    Boss(&'a OverlayPackage),
    ColorPack(&'a ColorPack),
}

#[derive(Clone)]
pub struct PackageManager {
    pub games: PackageRegistry<GamePackage>,
    pub screensavers: PackageRegistry<OverlayPackage>,
    pub bosses: PackageRegistry<OverlayPackage>,
    pub color_packs: PackageRegistry<ColorPack>,

    game_scan_errors: Vec<GameModuleScanError>,
    overlay_scan_errors: Vec<overlay_modules::OverlayScanError>,
    uid_registry: PackageIdRegistry,
    profile_store: Arc<ProfileStore>,
    cache_store: Arc<CacheStore>,
}

impl PackageManager {
    pub fn new(profile_store: Arc<ProfileStore>, cache_store: Arc<CacheStore>) -> Self {
        Self {
            games: PackageRegistry::new(),
            screensavers: PackageRegistry::new(),
            bosses: PackageRegistry::new(),
            color_packs: PackageRegistry::new(),
            game_scan_errors: Vec::new(),
            overlay_scan_errors: Vec::new(),
            uid_registry: PackageIdRegistry::default(),
            profile_store,
            cache_store,
        }
    }

    pub fn refresh_all(&mut self, progress: &LoadingHandle) -> PackageManagerResult<()> {
        self.uid_registry = PackageIdRegistry::default();
        progress.update(LoadingStage::ScanGame, 25)?;
        self.refresh_games()?;
        progress.update(LoadingStage::ScanGame, 35)?;
        self.refresh_overlays()?;
        progress.update(LoadingStage::ScanGame, 45)?;
        self.reconcile_states()?;
        self.rebuild_display_orders()?;
        self.persist_scan_cache()?;
        Ok(())
    }

    pub fn scan_in_background(kind: PackageKind, sender: Sender<EngineEvent>) {
        let _ = Scanner::scan_directories(kind);
        let _ = sender.send(EngineEvent::PackagesRefreshed(kind));
    }

    pub fn apply_scan_result(&mut self, kind: PackageKind) -> PackageManagerResult<()> {
        match kind {
            PackageKind::Game => self.refresh_games()?,
            PackageKind::Screensaver | PackageKind::Boss => self.refresh_overlays()?,
            PackageKind::ColorPack | PackageKind::UiPack => {}
        }
        self.reconcile_states()?;
        self.rebuild_display_orders()?;
        self.persist_scan_cache()?;
        Ok(())
    }

    pub fn refresh_games(&mut self) -> PackageManagerResult<()> {
        self.validate_discovered_packages(PackageKind::Game)?;
        let registry = game_modules::load()?;
        self.game_scan_errors = registry.errors.clone();
        self.games = PackageRegistry::new();
        for game_module in registry.games {
            let package = GamePackage::from(game_module);
            self.register_package_id(&package.id)?;
            self.games.insert(package)?;
        }
        Ok(())
    }

    fn refresh_overlays(&mut self) -> PackageManagerResult<()> {
        self.validate_discovered_packages(PackageKind::Screensaver)?;
        self.validate_discovered_packages(PackageKind::Boss)?;
        let registry = overlay_modules::load()?;
        self.overlay_scan_errors = registry.errors.clone();
        self.screensavers = PackageRegistry::new();
        for package in registry.screensavers {
            let package = OverlayPackage::from(package);
            self.register_package_id(&package.id)?;
            self.screensavers.insert(package)?;
        }
        self.bosses = PackageRegistry::new();
        for package in registry.bosses {
            let package = OverlayPackage::from(package);
            self.register_package_id(&package.id)?;
            self.bosses.insert(package)?;
        }
        Ok(())
    }

    pub fn enable_package(&mut self, id: &PackageId) -> PackageManagerResult<()> {
        self.set_package_enabled(id, true)
    }

    pub fn disable_package(&mut self, id: &PackageId) -> PackageManagerResult<()> {
        self.set_package_enabled(id, false)
    }

    pub fn remove_package(&mut self, id: &PackageId) -> PackageManagerResult<()> {
        let removed = match id.kind {
            PackageKind::Game => self.games.remove(&id.uid).is_some(),
            PackageKind::Screensaver => self.screensavers.remove(&id.uid).is_some(),
            PackageKind::Boss => self.bosses.remove(&id.uid).is_some(),
            PackageKind::ColorPack => self.color_packs.remove(&id.uid).is_some(),
            PackageKind::UiPack => false,
        };

        if removed {
            self.uid_registry.remove(id);
            self.profile_store
                .save_package_state(&id.uid, &json!({ "enabled": false }))?;
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, format!("package not found: {id}")).into())
        }
    }

    pub fn set_sort_order(
        &mut self,
        kind: PackageKind,
        order: &[String],
    ) -> PackageManagerResult<()> {
        match kind {
            PackageKind::Game => self.games.set_display_order(order),
            PackageKind::Screensaver => self.screensavers.set_display_order(order),
            PackageKind::Boss => self.bosses.set_display_order(order),
            PackageKind::ColorPack => self.color_packs.set_display_order(order),
            PackageKind::UiPack => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "ui package sort order is not managed by PackageManager",
                )
                .into());
            }
        }
        Ok(())
    }

    pub fn action_schemas(&self) -> HashMap<PackageId, ActionSchema> {
        let mut schemas = HashMap::new();
        for game in self.games.iter() {
            let actions = game
                .actions
                .iter()
                .map(|(name, binding)| {
                    (
                        name.clone(),
                        KeybindActionDefault {
                            default_keys: action_keys_from_value(&binding.key),
                            display_name: binding.key_name.clone(),
                        },
                    )
                })
                .collect::<HashMap<_, _>>();
            schemas.insert(game.id().clone(), ActionSchema { actions });
        }
        schemas
    }

    /// 当前已注册但被禁用的包 ID 集合。
    pub fn disabled_package_ids(&self) -> std::collections::HashSet<PackageId> {
        let mut disabled = std::collections::HashSet::new();
        collect_disabled_package_ids(&self.games, &mut disabled);
        collect_disabled_package_ids(&self.screensavers, &mut disabled);
        collect_disabled_package_ids(&self.bosses, &mut disabled);
        collect_disabled_package_ids(&self.color_packs, &mut disabled);
        disabled
    }

    pub fn games(&self) -> &[GamePackage] {
        self.games.all()
    }

    pub fn screensavers(&self) -> &[OverlayPackage] {
        self.screensavers.all()
    }

    pub fn bosses(&self) -> &[OverlayPackage] {
        self.bosses.all()
    }

    pub fn enabled_games(&self) -> Vec<&GamePackage> {
        self.games.enabled_packages()
    }

    pub fn game_scan_errors(&self) -> &[GameModuleScanError] {
        &self.game_scan_errors
    }

    pub fn overlay_scan_errors(&self) -> &[overlay_modules::OverlayScanError] {
        &self.overlay_scan_errors
    }

    pub fn package_by_uid(&self, uid: &str) -> Option<PackageRef<'_>> {
        self.games
            .get(uid)
            .map(PackageRef::Game)
            .or_else(|| self.screensavers.get(uid).map(PackageRef::Screensaver))
            .or_else(|| self.bosses.get(uid).map(PackageRef::Boss))
            .or_else(|| self.color_packs.get(uid).map(PackageRef::ColorPack))
    }

    fn set_package_enabled(&mut self, id: &PackageId, enabled: bool) -> PackageManagerResult<()> {
        match id.kind {
            PackageKind::Game => self.games.set_enabled(&id.uid, enabled),
            PackageKind::Screensaver => self.screensavers.set_enabled(&id.uid, enabled),
            PackageKind::Boss => self.bosses.set_enabled(&id.uid, enabled),
            PackageKind::ColorPack => self.color_packs.set_enabled(&id.uid, enabled),
            PackageKind::UiPack => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "ui package enabled state is not managed by PackageManager",
                )
                .into());
            }
        }

        let state = self.package_state_value(&id.uid, enabled);
        self.profile_store.save_package_state(&id.uid, &state)?;
        self.reconcile_states()?;
        self.rebuild_display_orders()?;
        Ok(())
    }

    fn package_state_value(&self, uid: &str, enabled: bool) -> Value {
        let mut state = self
            .profile_store
            .package_states
            .get(uid)
            .cloned()
            .unwrap_or_else(|| json!({}));
        if !state.is_object() {
            state = json!({});
        }
        if let Some(object) = state.as_object_mut() {
            object.insert("enabled".to_string(), Value::Bool(enabled));
            object
                .entry("debug".to_string())
                .or_insert(Value::Bool(false));
        }
        state
    }

    fn reconcile_states(&mut self) -> PackageManagerResult<()> {
        reconcile_registry(&mut self.games, &self.profile_store.package_states);
        reconcile_registry(&mut self.screensavers, &self.profile_store.package_states);
        reconcile_registry(&mut self.bosses, &self.profile_store.package_states);
        reconcile_registry(&mut self.color_packs, &self.profile_store.package_states);
        Ok(())
    }

    fn rebuild_display_orders(&mut self) -> PackageManagerResult<()> {
        let game_order = self
            .profile_store
            .games
            .iter()
            .map(|entry| entry.uid.clone())
            .collect::<Vec<_>>();
        let screensaver_order = self
            .profile_store
            .screensavers
            .iter()
            .map(|entry| entry.uid.clone())
            .collect::<Vec<_>>();
        let boss_order = self
            .profile_store
            .bosses
            .iter()
            .map(|entry| entry.uid.clone())
            .collect::<Vec<_>>();

        self.games.set_display_order(&game_order);
        self.screensavers.set_display_order(&screensaver_order);
        self.bosses.set_display_order(&boss_order);
        Ok(())
    }

    fn persist_scan_cache(&self) -> PackageManagerResult<()> {
        let game_registry = game_modules::GameModuleRegistry {
            games: self.games.iter().map(GamePackage::to_legacy).collect(),
            errors: self.game_scan_errors.clone(),
        };
        self.cache_store.write_game_scan_cache(&game_registry)?;
        let screensaver_packages = self
            .screensavers
            .iter()
            .map(OverlayPackage::to_legacy)
            .collect::<Vec<_>>();
        let boss_packages = self
            .bosses
            .iter()
            .map(OverlayPackage::to_legacy)
            .collect::<Vec<_>>();
        let _ = screensaver_packages;
        let _ = boss_packages;
        self.cache_store.save_scan_cache()?;
        Ok(())
    }

    fn register_package_id(&mut self, id: &PackageId) -> PackageManagerResult<()> {
        match self.uid_registry.register(id) {
            Ok(()) => Ok(()),
            Err(error) => {
                eprintln!("[warning] package uid conflict: {error}");
                Ok(())
            }
        }
    }

    fn validate_discovered_packages(&self, kind: PackageKind) -> PackageManagerResult<()> {
        #[cfg(not(debug_assertions))]
        {
            let _ = kind;
            return Ok(());
        }

        #[cfg(debug_assertions)]
        {
            for package_path in Scanner::scan_directories(kind)? {
                let manifest_path = package_path.path.join("package.json");
                match parse_manifest(&manifest_path) {
                    Ok(manifest) => {
                        let validation = if kind == PackageKind::Game {
                            match crate::host_engine::package::manifest::parse_game_manifest(
                                &package_path.path.join("game.json"),
                            ) {
                                Ok(game_manifest) => {
                                    crate::host_engine::package::validator::validate_game_package_at(
                                        &manifest,
                                        &game_manifest,
                                        Some(&package_path.path),
                                    )
                                }
                                Err(error) => {
                                    eprintln!(
                                        "[warning] failed to parse game manifest {}: {error}",
                                        package_path.path.join("game.json").display()
                                    );
                                    continue;
                                }
                            }
                        } else {
                            validate_package_at(&manifest, kind, Some(&package_path.path))
                        };
                        for warning in validation.warnings {
                            eprintln!(
                                "[warning] {} {}: {}",
                                manifest_path.display(),
                                warning.field,
                                warning.message
                            );
                        }
                        if !validation.errors.is_empty() {
                            eprintln!(
                                "[warning] {} validation failed: {} error(s)",
                                manifest_path.display(),
                                validation.errors.len()
                            );
                        }
                    }
                    Err(error) => {
                        eprintln!(
                            "[warning] failed to parse package manifest {}: {error}",
                            manifest_path.display()
                        );
                    }
                }
            }
            Ok(())
        }
    }
}

fn action_keys_from_value(value: &Value) -> Vec<String> {
    match value {
        Value::String(key) => vec![key.clone()],
        Value::Array(keys) => keys
            .iter()
            .filter_map(Value::as_str)
            .map(ToString::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn collect_disabled_package_ids<T: Package>(
    registry: &PackageRegistry<T>,
    disabled: &mut std::collections::HashSet<PackageId>,
) {
    for package in registry.iter() {
        if !registry.is_enabled(&package.id().uid) {
            disabled.insert(package.id().clone());
        }
    }
}

fn reconcile_registry<T: Package>(
    registry: &mut PackageRegistry<T>,
    states: &HashMap<String, Value>,
) {
    let state_updates = registry
        .all()
        .iter()
        .map(|package| {
            let uid = package.id().uid.clone();
            let enabled = states
                .get(&uid)
                .and_then(|state| state.get("enabled"))
                .and_then(Value::as_bool)
                .unwrap_or(true);
            (uid, enabled)
        })
        .collect::<Vec<_>>();

    for (uid, enabled) in state_updates {
        registry.set_enabled(&uid, enabled);
        registry.unmark_missing(&uid);
    }

    for uid in states.keys() {
        if registry.get(uid).is_none() {
            registry.mark_missing(uid);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_engine::boot::preload::persistent_data::display_profile::DisplayProfile;
    use crate::host_engine::boot::preload::persistent_data::security_profile::SecurityProfile;

    #[test]
    fn manager_new_starts_empty() {
        let profile_store = Arc::new(ProfileStore {
            language: "en_us".to_string(),
            keybinds: json!({}),
            security: SecurityProfile::default(),
            display: DisplayProfile::default(),
            saves: json!({}),
            best_scores: json!({}),
            package_states: HashMap::new(),
            screensavers: Vec::new(),
            bosses: Vec::new(),
            games: Vec::new(),
        });
        let cache_store = Arc::new(CacheStore::default());
        let manager = PackageManager::new(profile_store, cache_store);

        assert_eq!(manager.games().len(), 0);
        assert_eq!(manager.screensavers().len(), 0);
        assert_eq!(manager.bosses().len(), 0);
    }

    #[test]
    fn manager_can_query_registered_color_pack() {
        let mut manager = empty_manager();
        let color_pack = ColorPack::new("theme_blue", "Blue Theme");
        let uid = color_pack.id().uid.clone();

        manager.color_packs.insert(color_pack).unwrap();

        match manager.package_by_uid(&uid) {
            Some(PackageRef::ColorPack(package)) => assert_eq!(package.name(), "Blue Theme"),
            other => panic!("unexpected package lookup result: {other:?}"),
        }
    }

    #[test]
    fn manager_set_sort_order_updates_color_pack_display_order() {
        let mut manager = empty_manager();
        manager
            .color_packs
            .insert(ColorPack::new("first", "First"))
            .unwrap();
        manager
            .color_packs
            .insert(ColorPack::new("second", "Second"))
            .unwrap();

        manager
            .set_sort_order(
                PackageKind::ColorPack,
                &["second".to_string(), "first".to_string()],
            )
            .unwrap();

        let names = manager
            .color_packs
            .displayed_packages()
            .into_iter()
            .map(Package::name)
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["Second", "First"]);
    }

    #[test]
    fn manager_reports_disabled_package_ids_from_registry_state() {
        let mut manager = empty_manager();
        let color_pack = ColorPack::new("theme_disabled", "Theme");
        let package_id = color_pack.id().clone();
        manager.color_packs.insert(color_pack).unwrap();
        manager.color_packs.set_enabled(&package_id.uid, false);

        let disabled = manager.disabled_package_ids();

        assert!(disabled.contains(&package_id));
    }

    fn empty_manager() -> PackageManager {
        let profile_store = Arc::new(ProfileStore {
            language: "en_us".to_string(),
            keybinds: json!({}),
            security: SecurityProfile::default(),
            display: DisplayProfile::default(),
            saves: json!({}),
            best_scores: json!({}),
            package_states: HashMap::new(),
            screensavers: Vec::new(),
            bosses: Vec::new(),
            games: Vec::new(),
        });
        let cache_store = Arc::new(CacheStore::default());
        PackageManager::new(profile_store, cache_store)
    }
}
