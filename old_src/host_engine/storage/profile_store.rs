//! data/profiles 持久化数据统一读写入口。

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use crate::host_engine::boot::environment::data_dirs;
use crate::host_engine::boot::preload::persistent_data::display_profile::DisplayProfile;
use crate::host_engine::boot::preload::persistent_data::keybind_profile;
use crate::host_engine::boot::preload::persistent_data::security_profile::SecurityProfile;
use crate::host_engine::package::package_id::PackageId;
use crate::host_engine::package::package_id_registry::PackageIdRegistry;

const DEFAULT_LANGUAGE_CODE: &str = "en_us";

type ProfileStoreResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 按顺序记录的包条目。
#[derive(Clone, Debug)]
pub struct OrderedPackageEntry {
    pub uid: String,
    pub enabled: bool,
    pub debug: bool,
}

/// data/profiles 下的持久化数据快照。
#[derive(Clone, Debug)]
pub struct ProfileStore {
    pub language: String,
    pub keybinds: Value,
    pub security: SecurityProfile,
    pub display: DisplayProfile,
    pub saves: Value,
    pub best_scores: Value,
    pub package_states: HashMap<String, Value>,
    pub screensavers: Vec<OrderedPackageEntry>,
    pub bosses: Vec<OrderedPackageEntry>,
    pub games: Vec<OrderedPackageEntry>,
}

impl ProfileStore {
    /// 读取并规范化所有 profile 文件。
    pub fn open() -> ProfileStoreResult<Self> {
        let profiles_dir = profiles_dir();
        fs::create_dir_all(&profiles_dir)?;

        let language = read_language_code(&profiles_dir.join("language.txt"))?;
        let saves = read_json_object_or_default(&profiles_dir.join("saves.json"), json!({}))?;
        let best_scores =
            read_json_object_or_default(&profiles_dir.join("best_scores.json"), json!({}))?;
        let keybinds = keybind_profile::load_keybind_profile(&profiles_dir.join("keybind.json"))?;
        let security_value = read_json_object_or_default(
            &profiles_dir.join("security_state.json"),
            SecurityProfile::default().to_value(),
        )?;
        let security = SecurityProfile::from_value(&security_value);
        write_json_pretty(
            &profiles_dir.join("security_state.json"),
            &security.to_value(),
        )?;

        let display_value = read_json_object_or_default(
            &profiles_dir.join("display_state.json"),
            DisplayProfile::default().to_value(),
        )?;
        let mut display = DisplayProfile::from_value(&display_value);
        display.normalize();
        write_json_pretty(
            &profiles_dir.join("display_state.json"),
            &display.to_value(),
        )?;

        let game_state = read_json_object_map(&profiles_dir.join("game_state.json"))?;
        let screensaver_state = read_json_object_map(&profiles_dir.join("screensaver_state"))?;
        let boss_state = read_json_object_map(&profiles_dir.join("boss_state"))?;

        let mut package_states = HashMap::new();
        extend_package_states(&mut package_states, &game_state);
        extend_package_states(&mut package_states, &screensaver_state);
        extend_package_states(&mut package_states, &boss_state);

        let games = ordered_entries_from_state(&game_state);
        let screensavers = ordered_overlay_entries(
            &display.screensaver_list.order,
            &display.screensaver_list.enabled,
            &screensaver_state,
        );
        let bosses = ordered_overlay_entries(
            &display.boss_list.order,
            &display.boss_list.enabled,
            &boss_state,
        );

        warn_profile_uid_conflicts(&games, &screensavers, &bosses);

        Ok(Self {
            language,
            keybinds,
            security,
            display,
            saves,
            best_scores,
            package_states,
            screensavers,
            bosses,
            games,
        })
    }

    /// 保存键位绑定到 `data/profiles/keybind.json`。
    pub fn save_keybinds(&self) -> ProfileStoreResult<()> {
        keybind_profile::write_keybind_profile(&profiles_dir().join("keybind.json"), &self.keybinds)
    }

    /// 保存单个包状态。目标文件由 UID 前缀决定。
    pub fn save_package_state(&self, uid: &str, state: &Value) -> ProfileStoreResult<()> {
        let path = package_state_path(uid);
        let mut states = read_json_object_map(&path)?;
        states.insert(uid.to_string(), state.clone());
        write_json_pretty(&path, &Value::Object(states))?;
        Ok(())
    }

    /// 保存安全设置。
    pub fn save_security(&self) -> ProfileStoreResult<()> {
        write_json_pretty(
            &profiles_dir().join("security_state.json"),
            &self.security.to_value(),
        )
    }

    /// 保存显示设置。
    pub fn save_display(&self) -> ProfileStoreResult<()> {
        write_json_pretty(
            &profiles_dir().join("display_state.json"),
            &self.display.to_value(),
        )
    }

    /// 保存语言选择。
    pub fn save_language(&self, lang: &str) -> ProfileStoreResult<()> {
        let path = profiles_dir().join("language.txt");
        if let Some(parent_dir) = path.parent() {
            fs::create_dir_all(parent_dir)?;
        }
        fs::write(path, lang)?;
        Ok(())
    }

    /// 以现有文件格式导出游戏包状态。
    pub fn game_state_value(&self) -> Value {
        self.state_value_for_kind(PackageStateKind::Game)
    }

    /// 以现有文件格式导出 Screensaver 包状态。
    pub fn screensaver_state_value(&self) -> Value {
        self.state_value_for_kind(PackageStateKind::Screensaver)
    }

    /// 以现有文件格式导出 Boss 包状态。
    pub fn boss_state_value(&self) -> Value {
        self.state_value_for_kind(PackageStateKind::Boss)
    }

    fn state_value_for_kind(&self, kind: PackageStateKind) -> Value {
        let mut state = Map::new();
        for (uid, value) in &self.package_states {
            if package_state_kind(uid) == kind {
                state.insert(uid.clone(), value.clone());
            }
        }
        Value::Object(state)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PackageStateKind {
    Game,
    Screensaver,
    Boss,
}

fn profiles_dir() -> PathBuf {
    data_dirs::root_dir().join("data/profiles")
}

fn read_language_code(path: &Path) -> ProfileStoreResult<String> {
    if !path.is_file() {
        write_language_code(path, DEFAULT_LANGUAGE_CODE)?;
        return Ok(DEFAULT_LANGUAGE_CODE.to_string());
    }

    let raw_text = fs::read_to_string(path)?;
    let language = raw_text.trim().trim_start_matches('\u{feff}');
    if language.is_empty() {
        write_language_code(path, DEFAULT_LANGUAGE_CODE)?;
        Ok(DEFAULT_LANGUAGE_CODE.to_string())
    } else {
        Ok(language.to_string())
    }
}

fn write_language_code(path: &Path, language: &str) -> ProfileStoreResult<()> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(path, language)?;
    Ok(())
}

fn read_json_object_or_default(path: &Path, default_value: Value) -> ProfileStoreResult<Value> {
    if !path.is_file() {
        write_json_pretty(path, &default_value)?;
        return Ok(default_value);
    }

    match fs::read_to_string(path)
        .ok()
        .and_then(|raw_json| {
            serde_json::from_str::<Value>(raw_json.trim_start_matches('\u{feff}')).ok()
        })
        .filter(Value::is_object)
    {
        Some(value) => Ok(value),
        None => {
            write_json_pretty(path, &default_value)?;
            Ok(default_value)
        }
    }
}

fn read_json_object_map(path: &Path) -> ProfileStoreResult<Map<String, Value>> {
    if !path.is_file() {
        write_json_pretty(path, &json!({}))?;
        return Ok(Map::new());
    }

    let raw_json = fs::read_to_string(path)?;
    let value = serde_json::from_str::<Value>(raw_json.trim_start_matches('\u{feff}'))?;
    Ok(value.as_object().cloned().unwrap_or_default())
}

fn write_json_pretty<T: serde::Serialize>(path: &Path, value: &T) -> ProfileStoreResult<()> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

fn extend_package_states(package_states: &mut HashMap<String, Value>, states: &Map<String, Value>) {
    for (uid, state) in states {
        package_states.insert(uid.clone(), state.clone());
    }
}

fn ordered_entries_from_state(states: &Map<String, Value>) -> Vec<OrderedPackageEntry> {
    let mut entries: Vec<_> = states
        .iter()
        .map(|(uid, state)| OrderedPackageEntry {
            uid: uid.clone(),
            enabled: state_bool(state, "enabled", true),
            debug: state_bool(state, "debug", false),
        })
        .collect();

    entries.sort_by(|left, right| compare_uid(&left.uid, &right.uid));
    entries
}

fn ordered_overlay_entries(
    order: &[String],
    display_enabled: &BTreeMap<String, bool>,
    states: &Map<String, Value>,
) -> Vec<OrderedPackageEntry> {
    let mut entries = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for uid in order {
        if let Some(state) = states.get(uid) {
            entries.push(OrderedPackageEntry {
                uid: uid.clone(),
                enabled: display_enabled
                    .get(uid)
                    .copied()
                    .unwrap_or_else(|| state_bool(state, "enabled", true)),
                debug: state_bool(state, "debug", false),
            });
            seen.insert(uid.clone());
        }
    }

    let mut remaining: Vec<_> = states
        .iter()
        .filter(|(uid, _)| !seen.contains(*uid))
        .map(|(uid, state)| OrderedPackageEntry {
            uid: uid.clone(),
            enabled: display_enabled
                .get(uid)
                .copied()
                .unwrap_or_else(|| state_bool(state, "enabled", true)),
            debug: state_bool(state, "debug", false),
        })
        .collect();
    remaining.sort_by(|left, right| compare_uid(&left.uid, &right.uid));
    entries.extend(remaining);
    entries
}

fn state_bool(state: &Value, key: &str, default_value: bool) -> bool {
    state
        .get(key)
        .and_then(Value::as_bool)
        .unwrap_or(default_value)
}

fn compare_uid(left: &str, right: &str) -> std::cmp::Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

fn package_state_path(uid: &str) -> PathBuf {
    let profiles = profiles_dir();
    if uid.starts_with("mod_screensaver_") || uid.starts_with("screensaver_") {
        profiles.join("screensaver_state")
    } else if uid.starts_with("mod_boss_") || uid.starts_with("boss_") {
        profiles.join("boss_state")
    } else {
        profiles.join("game_state.json")
    }
}

fn package_state_kind(uid: &str) -> PackageStateKind {
    if uid.starts_with("mod_screensaver_") || uid.starts_with("screensaver_") {
        PackageStateKind::Screensaver
    } else if uid.starts_with("mod_boss_") || uid.starts_with("boss_") {
        PackageStateKind::Boss
    } else {
        PackageStateKind::Game
    }
}

fn warn_profile_uid_conflicts(
    games: &[OrderedPackageEntry],
    screensavers: &[OrderedPackageEntry],
    bosses: &[OrderedPackageEntry],
) {
    let mut registry = PackageIdRegistry::default();

    for game in games {
        register_or_warn(
            &mut registry,
            PackageId::from_legacy("game", "game", &game.uid),
        );
    }
    for screensaver in screensavers {
        register_or_warn(
            &mut registry,
            PackageId::from_legacy("mod", "screensaver", &screensaver.uid),
        );
    }
    for boss in bosses {
        register_or_warn(
            &mut registry,
            PackageId::from_legacy("mod", "boss", &boss.uid),
        );
    }
}

fn register_or_warn(registry: &mut PackageIdRegistry, package_id: PackageId) {
    if let Err(error) = registry.register(&package_id) {
        eprintln!("[warning] package uid conflict: {error}");
    }
}
