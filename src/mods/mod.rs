use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use image::GenericImageView;
use mlua::{Lua, Table};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as JsonValue};
use unicode_width::UnicodeWidthChar;

use crate::app::i18n;
use crate::lua_bridge::script_loader::GameMeta;
use crate::utils::path_utils;

pub const MOD_API_VERSION: u32 = 1;

const DEFAULT_PACKAGE_DESCRIPTION: &str = "No package description available.";
const DEFAULT_GAME_DESCRIPTION: &str = "No description available.";
const DEFAULT_GAME_DETAIL: &str = "";
const MATH_IMAGE_CHARS: [char; 9] = ['@', '%', '#', '*', '+', '=', '-', ':', '.'];
const NUMBER_IMAGE_CHARS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const BLOCK_IMAGE_CHARS: [char; 3] = ['█', '▓', '▒'];

const DEFAULT_THUMBNAIL_LINES: [&str; 4] = ["████████", "██ ██ ██", "   ██   ", "  ████  "];

const DEFAULT_BANNER_ASCII: [&str; 7] = [
    "`7MMM.     ,MMF' .g8\"\"8q. `7MM\"\"\"Yb.",
    "    MMMb    dPMM .dP'    `YM. MM    `Yb.",
    "    M YM   ,M MM dM'      `MM MM     `Mb",
    "    M  Mb  M' MM MM        MM MM      MM",
    "    M  YM.P'  MM MM.      ,MP MM     ,MP",
    "    M  `YM'   MM `Mb.    ,dP' MM    ,dP'",
    ".JML. `'  .JMML. `\"bmmd\"' .JMMmmmdP'",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModImage {
    pub lines: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModGameInfo {
    pub namespace: String,
    pub package_name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub thumbnail: ModImage,
    pub banner: ModImage,
    pub enabled: bool,
    pub debug_enabled: bool,
}

#[derive(Clone, Debug)]
pub struct ModGameMeta {
    pub game_id: String,
    pub script_name: String,
    pub script_path: PathBuf,
    pub name: String,
    pub description: String,
    pub detail: String,
    pub save: bool,
    pub mod_info: ModGameInfo,
}

#[derive(Clone, Debug)]
pub struct ModPackage {
    pub namespace: String,
    pub enabled: bool,
    pub debug_enabled: bool,
    pub package_name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub thumbnail: ModImage,
    pub banner: ModImage,
    pub games: Vec<ModGameMeta>,
    pub errors: Vec<ModScanError>,
}

#[derive(Clone, Debug)]
pub struct ModScanOutput {
    pub packages: Vec<ModPackage>,
    pub games: Vec<GameMeta>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModState {
    pub api_version: u32,
    #[serde(default)]
    pub mods: HashMap<String, ModStateEntry>,
    #[serde(default)]
    pub scan_errors: Vec<ModScanError>,
    #[serde(default)]
    pub latest_save_game: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModStateEntry {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub debug_enabled: bool,
    #[serde(default)]
    pub package_name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub games: HashMap<String, ModGameState>,
}

impl Default for ModStateEntry {
    fn default() -> Self {
        Self {
            enabled: true,
            debug_enabled: false,
            package_name: String::new(),
            author: String::new(),
            version: String::new(),
            games: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModGameState {
    #[serde(default)]
    pub script_name: String,
    #[serde(default)]
    pub best_score: JsonValue,
    #[serde(default)]
    pub keybindings: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModScanError {
    pub namespace: String,
    pub scope: String,
    pub target: String,
    pub severity: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModScanCache {
    #[serde(default)]
    pub packages: HashMap<String, CachedPackage>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CachedPackage {
    pub meta_mtime: u64,
    #[serde(default)]
    pub script_mtimes: BTreeMap<String, u64>,
    #[serde(default)]
    pub thumbnail_cache_key: Option<String>,
    #[serde(default)]
    pub banner_cache_key: Option<String>,
    #[serde(default)]
    pub scan_ok: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct RawMeta {
    package_name: String,
    #[serde(default)]
    description: Option<String>,
    author: String,
    version: String,
    namespace: String,
    api_version: ApiVersionField,
    #[serde(default)]
    thumbnail: Option<JsonValue>,
    #[serde(default)]
    banner: Option<JsonValue>,
    #[serde(flatten)]
    extra: Map<String, JsonValue>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum ApiVersionField {
    Single(u32),
    Range([u32; 2]),
}

#[derive(Clone, Copy, Debug)]
enum ImageKind {
    Thumbnail,
    Banner,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ImageRenderMode {
    Braille,
    Math,
    Number,
    Block,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ImageColorMode {
    Grayscale,
    White,
    Color,
}

#[derive(Clone, Debug)]
struct ImageSpec {
    namespace: String,
    path: String,
    color_mode: ImageColorMode,
    render_mode: ImageRenderMode,
}

fn default_true() -> bool {
    true
}

pub fn mod_root_dir() -> Result<PathBuf> {
    Ok(path_utils::app_data_dir()?.join("mod"))
}

pub fn mod_data_dir() -> Result<PathBuf> {
    Ok(mod_root_dir()?.join("list"))
}

pub fn mod_state_path() -> Result<PathBuf> {
    Ok(mod_root_dir()?.join("mod_state.json"))
}

pub fn mod_scan_cache_path() -> Result<PathBuf> {
    Ok(mod_root_dir()?.join("scan_cache.json"))
}

pub fn mod_cache_dir() -> Result<PathBuf> {
    Ok(mod_root_dir()?.join("cache"))
}

pub fn mod_save_dir(namespace: &str) -> Result<PathBuf> {
    Ok(mod_root_dir()?.join("save").join(namespace))
}

pub fn mod_save_path(namespace: &str, game_id: &str) -> Result<PathBuf> {
    Ok(mod_save_dir(namespace)?.join(format!("{game_id}.json")))
}

pub fn mod_log_path(namespace: &str) -> Result<PathBuf> {
    Ok(mod_root_dir()?.join("logs").join(format!("{namespace}.log")))
}
pub fn load_mod_state() -> ModState {
    let Ok(path) = mod_state_path() else {
        return ModState {
            api_version: MOD_API_VERSION,
            ..Default::default()
        };
    };

    let Ok(raw) = fs::read_to_string(path) else {
        return ModState {
            api_version: MOD_API_VERSION,
            ..Default::default()
        };
    };

    let mut state = serde_json::from_str::<ModState>(raw.trim_start_matches('\u{feff}')).unwrap_or_default();
    if state.api_version == 0 {
        state.api_version = MOD_API_VERSION;
    }
    state
}

pub fn save_mod_state(state: &ModState) -> Result<()> {
    let path = mod_state_path()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

pub fn load_scan_cache() -> ModScanCache {
    let Ok(path) = mod_scan_cache_path() else {
        return ModScanCache::default();
    };
    let Ok(raw) = fs::read_to_string(path) else {
        return ModScanCache::default();
    };
    serde_json::from_str::<ModScanCache>(raw.trim_start_matches('\u{feff}')).unwrap_or_default()
}

pub fn save_scan_cache(cache: &ModScanCache) -> Result<()> {
    let path = mod_scan_cache_path()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, serde_json::to_string_pretty(cache)?)?;
    Ok(())
}

pub fn set_mod_enabled(namespace: &str, enabled: bool) -> Result<()> {
    let mut state = load_mod_state();
    state.mods.entry(namespace.to_string()).or_default().enabled = enabled;
    save_mod_state(&state)
}

pub fn set_mod_debug_enabled(namespace: &str, enabled: bool) -> Result<()> {
    let mut state = load_mod_state();
    state.mods.entry(namespace.to_string()).or_default().debug_enabled = enabled;
    save_mod_state(&state)
}

pub fn update_mod_keybindings(
    namespace: &str,
    game_id: &str,
    script_name: &str,
    bindings: HashMap<String, Vec<String>>,
) -> Result<()> {
    let mut state = load_mod_state();
    let game = state
        .mods
        .entry(namespace.to_string())
        .or_default()
        .games
        .entry(game_id.to_string())
        .or_default();
    game.script_name = script_name.to_string();
    game.keybindings = bindings;
    save_mod_state(&state)
}

pub fn read_mod_keybindings(namespace: &str, game_id: &str) -> HashMap<String, Vec<String>> {
    load_mod_state()
        .mods
        .get(namespace)
        .and_then(|entry| entry.games.get(game_id))
        .map(|game| game.keybindings.clone())
        .unwrap_or_default()
}

pub fn update_mod_best_score(namespace: &str, game_id: &str, script_name: &str, score: JsonValue) -> Result<()> {
    let mut state = load_mod_state();
    let game = state
        .mods
        .entry(namespace.to_string())
        .or_default()
        .games
        .entry(game_id.to_string())
        .or_default();
    game.script_name = script_name.to_string();
    game.best_score = score;
    save_mod_state(&state)
}

pub fn read_mod_best_score(namespace: &str, game_id: &str) -> Option<JsonValue> {
    load_mod_state()
        .mods
        .get(namespace)
        .and_then(|entry| entry.games.get(game_id))
        .map(|game| game.best_score.clone())
}

pub fn set_latest_mod_save_game(game_id: &str) -> Result<()> {
    let mut state = load_mod_state();
    state.latest_save_game = Some(game_id.to_string());
    save_mod_state(&state)
}

pub fn latest_mod_save_game_id() -> Option<String> {
    load_mod_state().latest_save_game
}

pub fn clear_latest_mod_save_game() -> Result<()> {
    let mut state = load_mod_state();
    state.latest_save_game = None;
    save_mod_state(&state)
}

pub fn mod_log(namespace: &str, level: &str, message: &str) -> Result<()> {
    let state = load_mod_state();
    let debug_enabled = state
        .mods
        .get(namespace)
        .map(|entry| entry.debug_enabled)
        .unwrap_or(false);

    if !debug_enabled {
        let level_lower = level.to_ascii_lowercase();
        if level_lower != "warn" && level_lower != "error" {
            return Ok(());
        }
    }

    let path = mod_log_path(namespace)?;
    path_utils::ensure_parent_dir(&path)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let line = format!("[{now}] [{}] {message}\n", level.to_ascii_uppercase());
    let mut existing = fs::read_to_string(&path).unwrap_or_default();
    existing.push_str(&line);
    fs::write(path, existing)?;
    Ok(())
}

pub fn scan_mods() -> Result<ModScanOutput> {
    let root = mod_data_dir()?;
    let mod_root = mod_root_dir()?;
    fs::create_dir_all(&root)?;
    fs::create_dir_all(mod_cache_dir()?)?;
    fs::create_dir_all(mod_root.join("save"))?;
    fs::create_dir_all(mod_root.join("logs"))?;

    let mut state = load_mod_state();
    let mut cache = load_scan_cache();
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
                    "meta.json",
                    "error",
                    format!("mod package scan failed: {err}"),
                ));
            }
        }
    }

    state.api_version = MOD_API_VERSION;
    state.scan_errors = global_errors;
    save_mod_state(&state)?;
    save_scan_cache(&cache)?;

    let games = packages
        .iter()
        .filter(|package| package.enabled)
        .flat_map(|package| package.games.iter().cloned())
        .map(mod_game_to_game_meta)
        .collect();

    Ok(ModScanOutput { packages, games })
}

pub fn load_mod_game_from_path(script_path: &Path) -> Result<Option<ModGameMeta>> {
    let script_path = fs::canonicalize(script_path).unwrap_or_else(|_| script_path.to_path_buf());
    let scripts_dir = script_path.parent().ok_or_else(|| anyhow!("invalid mod script path"))?;
    let package_dir = scripts_dir.parent().ok_or_else(|| anyhow!("invalid mod package path"))?;
    if scripts_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        != "scripts"
    {
        return Ok(None);
    }

    let meta_path = package_dir.join("meta.json");
    if !meta_path.exists() {
        return Ok(None);
    }

    let raw_meta = load_meta(&meta_path)?;
    validate_meta(package_dir, &raw_meta)?;
    let state = load_mod_state();
    let state_entry = state.mods.get(&raw_meta.namespace).cloned().unwrap_or_default();

    let description = resolve_mod_text(
        &raw_meta.namespace,
        raw_meta
            .description
            .as_deref()
            .unwrap_or(DEFAULT_PACKAGE_DESCRIPTION),
    );
    let base_info = ModGameInfo {
        namespace: raw_meta.namespace.clone(),
        package_name: resolve_mod_text(&raw_meta.namespace, &raw_meta.package_name),
        author: raw_meta.author.clone(),
        version: raw_meta.version.clone(),
        description,
        thumbnail: image_from_meta(&raw_meta.namespace, raw_meta.thumbnail.as_ref(), ImageKind::Thumbnail)?,
        banner: image_from_meta(&raw_meta.namespace, raw_meta.banner.as_ref(), ImageKind::Banner)?,
        enabled: state_entry.enabled,
        debug_enabled: state_entry.debug_enabled,
    };

    Ok(Some(scan_game_script(
        package_dir,
        &raw_meta,
        &base_info,
        &script_path,
    )?))
}

pub fn load_mod_helper_scripts(lua: &Lua, package_dir: &Path) -> mlua::Result<()> {
    load_helper_scripts(lua, &package_dir.join("scripts").join("function"))
}

fn scan_package(dir: &Path, state: &mut ModState, cache: &mut ModScanCache) -> Result<Option<ModPackage>> {
    let meta_path = dir.join("meta.json");
    if !meta_path.exists() {
        return Ok(None);
    }

    let raw_meta = load_meta(&meta_path)?;
    validate_meta(dir, &raw_meta)?;

    let namespace = raw_meta.namespace.clone();
    let state_entry = state.mods.entry(namespace.clone()).or_default();
    state_entry.package_name = raw_meta.package_name.clone();
    state_entry.author = raw_meta.author.clone();
    state_entry.version = raw_meta.version.clone();

    let description = resolve_mod_text(&namespace, raw_meta.description.as_deref().unwrap_or(DEFAULT_PACKAGE_DESCRIPTION));
    let thumbnail = image_from_meta(&namespace, raw_meta.thumbnail.as_ref(), ImageKind::Thumbnail)?;
    let banner = image_from_meta(&namespace, raw_meta.banner.as_ref(), ImageKind::Banner)?;

    let base_info = ModGameInfo {
        namespace: namespace.clone(),
        package_name: resolve_mod_text(&namespace, &raw_meta.package_name),
        author: raw_meta.author.clone(),
        version: raw_meta.version.clone(),
        description: description.clone(),
        thumbnail: thumbnail.clone(),
        banner: banner.clone(),
        enabled: state_entry.enabled,
        debug_enabled: state_entry.debug_enabled,
    };

    let mut errors = Vec::new();
    for key in raw_meta.extra.keys() {
        errors.push(scan_error(
            &namespace,
            "package",
            "meta.json",
            "warning",
            format!("unknown meta field ignored: {key}"),
        ));
    }

    let scripts_dir = dir.join("scripts");
    let mut scripts: Vec<PathBuf> = if scripts_dir.is_dir() {
        fs::read_dir(&scripts_dir)?
            .filter_map(|entry| entry.ok().map(|item| item.path()))
            .filter(|path| {
                path.is_file()
                    && path
                        .extension()
                        .and_then(|value| value.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("lua"))
                        .unwrap_or(false)
            })
            .collect()
    } else {
        Vec::new()
    };
    scripts.sort();

    if scripts.is_empty() {
        errors.push(scan_error(
            &namespace,
            "package",
            "scripts",
            "warning",
            "no main lua scripts found in scripts/ root".to_string(),
        ));
        cache.packages.insert(
            namespace,
            CachedPackage {
                meta_mtime: mtime_secs(&meta_path),
                scan_ok: false,
                ..Default::default()
            },
        );
        return Ok(None);
    }

    let mut games = Vec::new();
    let mut script_mtimes = BTreeMap::new();
    for script_path in scripts {
        let script_name = script_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("game")
            .to_string();
        script_mtimes.insert(script_name.clone(), mtime_secs(&script_path));
        match scan_game_script(dir, &raw_meta, &base_info, &script_path) {
            Ok(game) => {
                state_entry.games.entry(game.game_id.clone()).or_insert_with(|| ModGameState {
                    script_name: game.script_name.clone(),
                    ..Default::default()
                });
                games.push(game);
            }
            Err(err) => {
                errors.push(scan_error(
                    &namespace,
                    "game",
                    script_path.file_name().and_then(|value| value.to_str()).unwrap_or("unknown.lua"),
                    "error",
                    err.to_string(),
                ));
            }
        }
    }

    cache.packages.insert(
        namespace.clone(),
        CachedPackage {
            meta_mtime: mtime_secs(&meta_path),
            script_mtimes,
            thumbnail_cache_key: None,
            banner_cache_key: None,
            scan_ok: !games.is_empty(),
        },
    );

    if games.is_empty() {
        return Ok(None);
    }

    Ok(Some(ModPackage {
        namespace,
        enabled: base_info.enabled,
        debug_enabled: base_info.debug_enabled,
        package_name: base_info.package_name,
        author: base_info.author,
        version: base_info.version,
        description,
        thumbnail,
        banner,
        games,
        errors,
    }))
}
fn scan_game_script(
    package_dir: &Path,
    meta: &RawMeta,
    base_info: &ModGameInfo,
    script_path: &Path,
) -> Result<ModGameMeta> {
    let source = fs::read_to_string(script_path)
        .with_context(|| format!("failed to read mod script: {}", script_path.display()))?;
    let source = source.trim_start_matches('\u{feff}');

    let lua = Lua::new();
    install_scan_stubs(&lua).map_err(|err| anyhow!("failed to install mod scan stubs: {err}"))?;
    load_helper_scripts(&lua, &package_dir.join("scripts").join("function"))
        .map_err(|err| anyhow!("failed to load mod helper scripts: {err}"))?;
    lua.load(source)
        .set_name(script_path.to_string_lossy().to_string())
        .exec()
        .map_err(|err| anyhow!("lua runtime error: {err}"))?;

    let globals = lua.globals();
    let meta_table = globals
        .get::<Table>("GAME_META")
        .map_err(|_| anyhow!("GAME_META table is missing"))?;

    let script_name = script_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("game")
        .to_string();

    let raw_name = meta_table
        .get::<String>("name")
        .map_err(|_| anyhow!("GAME_META.name is missing"))?;
    let name = resolve_mod_text(&meta.namespace, &raw_name);
    if name.trim().is_empty() {
        return Err(anyhow!("GAME_META.name cannot be blank"));
    }

    let description = meta_table
        .get::<String>("description")
        .ok()
        .map(|value| resolve_mod_text(&meta.namespace, &value))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_GAME_DESCRIPTION.to_string());

    let detail = meta_table
        .get::<String>("detail")
        .ok()
        .map(|value| resolve_mod_text(&meta.namespace, &value))
        .unwrap_or_else(|| DEFAULT_GAME_DETAIL.to_string());

    let save = meta_table.get::<bool>("save").unwrap_or(false);
    let game_id = build_game_id(meta, &script_name);

    Ok(ModGameMeta {
        game_id,
        script_name,
        script_path: script_path.to_path_buf(),
        name,
        description,
        detail,
        save,
        mod_info: base_info.clone(),
    })
}

fn install_scan_stubs(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();
    globals.set("draw_text", lua.create_function(|_, ()| Ok(()))?)?;
    globals.set("draw_text_ex", lua.create_function(|_, ()| Ok(()))?)?;
    globals.set("clear", lua.create_function(|_, ()| Ok(()))?)?;
    globals.set("sleep", lua.create_function(|_, ()| Ok(()))?)?;
    globals.set("clear_input_buffer", lua.create_function(|_, ()| Ok(true))?)?;
    globals.set("get_key", lua.create_function(|_, ()| Ok(String::new()))?)?;
    globals.set("get_raw_key", lua.create_function(|_, ()| Ok(String::new()))?)?;
    globals.set("get_action_blocking", lua.create_function(|_, ()| Ok(String::new()))?)?;
    globals.set("poll_action", lua.create_function(|_, ()| Ok(String::new()))?)?;
    globals.set("is_action_pressed", lua.create_function(|_, ()| Ok(false))?)?;
    globals.set("register_action", lua.create_function(|_, ()| Ok(true))?)?;
    globals.set("save_data", lua.create_function(|_, ()| Ok(true))?)?;
    globals.set("load_data", lua.create_function(|_, ()| Ok(mlua::Value::Nil))?)?;
    globals.set("save_game_slot", lua.create_function(|_, ()| Ok(true))?)?;
    globals.set("load_game_slot", lua.create_function(|_, ()| Ok(mlua::Value::Nil))?)?;
    globals.set("update_game_stats", lua.create_function(|_, ()| Ok(true))?)?;
    globals.set("translate", lua.create_function(|_, key: String| Ok(key))?)?;
    globals.set("random", lua.create_function(|_, max: i64| Ok(max.max(0)))?)?;
    globals.set("exit_game", lua.create_function(|_, ()| Ok(()))?)?;
    globals.set("get_terminal_size", lua.create_function(|_, ()| Ok((80_u16, 24_u16)))?)?;
    globals.set("get_text_width", lua.create_function(|_, text: String| Ok(text.chars().count() as i64))?)?;
    globals.set("get_launch_mode", lua.create_function(|_, ()| Ok(String::from("new")))?)?;
    globals.set("mod_log", lua.create_function(|_, ()| Ok(true))?)?;

    globals.set("io", mlua::Value::Nil)?;
    globals.set("debug", mlua::Value::Nil)?;
    if let Ok(os_table) = globals.get::<Table>("os") {
        let _ = os_table.set("execute", mlua::Value::Nil);
        let _ = os_table.set("remove", mlua::Value::Nil);
        let _ = os_table.set("rename", mlua::Value::Nil);
        let _ = os_table.set("exit", mlua::Value::Nil);
    }
    if let Ok(package_table) = globals.get::<Table>("package") {
        let _ = package_table.set("loadlib", mlua::Value::Nil);
    }

    Ok(())
}

fn load_helper_scripts(lua: &Lua, dir: &Path) -> mlua::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(mlua::Error::external)?
        .filter_map(|entry| entry.ok().map(|item| item.path()))
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|value| value.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("lua"))
                    .unwrap_or(false)
        })
        .collect();
    entries.sort();

    for entry in entries {
        let source = fs::read_to_string(&entry).map_err(mlua::Error::external)?;
        let source = source.trim_start_matches('\u{feff}');
        lua.load(source)
            .set_name(entry.to_string_lossy().to_string())
            .exec()
            .map_err(|err| mlua::Error::external(anyhow!("helper script load failed: {err}")))?;
    }

    Ok(())
}

fn load_meta(path: &Path) -> Result<RawMeta> {
    let raw = fs::read_to_string(path)?;
    let raw = raw.trim_start_matches('\u{feff}');
    Ok(serde_json::from_str(raw)?)
}

fn validate_meta(dir: &Path, meta: &RawMeta) -> Result<()> {
    let folder_name = dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow!("invalid mod directory name"))?;

    if meta.namespace != folder_name {
        return Err(anyhow!("namespace must match directory name"));
    }
    if !meta.namespace.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return Err(anyhow!("namespace only allows letters and numbers"));
    }
    if meta.package_name.trim().is_empty() {
        return Err(anyhow!("package_name cannot be blank"));
    }
    if meta.author.trim().is_empty() {
        return Err(anyhow!("author cannot be blank"));
    }
    if meta.version.trim().is_empty() {
        return Err(anyhow!("version cannot be blank"));
    }
    if !meta.api_version.supports(MOD_API_VERSION) {
        return Err(anyhow!("api_version does not support host version {MOD_API_VERSION}"));
    }
    Ok(())
}

fn image_from_meta(namespace: &str, raw: Option<&JsonValue>, kind: ImageKind) -> Result<ModImage> {
    let image = match raw {
        Some(JsonValue::String(value)) => load_image_from_string(namespace, value, kind)?,
        Some(JsonValue::Array(value)) => parse_ascii_image_array(value, kind).unwrap_or_else(|| default_image(kind)),
        _ => default_image(kind),
    };
    Ok(normalize_image(image, kind))
}

fn load_image_from_string(namespace: &str, value: &str, kind: ImageKind) -> Result<ModImage> {
    let spec = match parse_image_spec(namespace, value) {
        Ok(spec) => spec,
        Err(_) => return Ok(default_image(kind)),
    };
    let asset_path = resolve_asset_path(&spec.namespace, &spec.path)?;
    if !asset_path.exists() {
        return Ok(default_image(kind));
    }

    match asset_path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") | Some("jpg") | Some("jpeg") | Some("webp") => {
            render_cached_raster_image(&asset_path, &spec, kind)
        }
        _ => {
            let content = fs::read_to_string(asset_path)?;
            let lines = content
                .trim_start_matches('\u{feff}')
                .lines()
                .map(|line| line.to_string())
                .collect::<Vec<_>>();
            if lines.is_empty() {
                Ok(default_image(kind))
            } else {
                Ok(ModImage { lines })
            }
        }
    }
}

fn parse_image_spec(namespace: &str, value: &str) -> Result<ImageSpec> {
    let mut color_mode = ImageColorMode::Grayscale;
    let mut render_mode = ImageRenderMode::Braille;
    let mut parts = value.split(':').collect::<Vec<_>>();

    while let Some(head) = parts.first().copied() {
        match head {
            "color" => {
                color_mode = ImageColorMode::Color;
                parts.remove(0);
            }
            "white" => {
                color_mode = ImageColorMode::White;
                parts.remove(0);
            }
            "math" => {
                render_mode = ImageRenderMode::Math;
                parts.remove(0);
            }
            "number" => {
                render_mode = ImageRenderMode::Number;
                parts.remove(0);
            }
            "block" => {
                render_mode = ImageRenderMode::Block;
                parts.remove(0);
            }
            _ => break,
        }
    }

    if parts.len() < 2 {
        return Err(anyhow!("invalid image spec"));
    }

    let image_namespace = parts.remove(0).to_string();
    if image_namespace != namespace {
        return Err(anyhow!("resource namespace mismatch"));
    }

    let path = parts.join(":");
    if path.trim().is_empty() {
        return Err(anyhow!("empty image path"));
    }

    Ok(ImageSpec {
        namespace: image_namespace,
        path,
        color_mode,
        render_mode,
    })
}

fn render_cached_raster_image(path: &Path, spec: &ImageSpec, kind: ImageKind) -> Result<ModImage> {
    fs::create_dir_all(mod_cache_dir()?)?;
    let cache_key = build_image_cache_key(path, spec, kind);
    let cache_path = mod_cache_dir()?.join(format!("{cache_key}.json"));

    if let Ok(raw) = fs::read_to_string(&cache_path) {
        if let Ok(image) =
            serde_json::from_str::<ModImage>(raw.trim_start_matches('\u{feff}'))
        {
            return Ok(image);
        }
    }

    let rendered = render_raster_image(path, spec, kind)?;
    fs::write(&cache_path, serde_json::to_string(&rendered)?)?;
    Ok(rendered)
}

fn build_image_cache_key(path: &Path, spec: &ImageSpec, kind: ImageKind) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    mtime_secs(path).hash(&mut hasher);
    spec.namespace.hash(&mut hasher);
    spec.path.hash(&mut hasher);
    match spec.color_mode {
        ImageColorMode::Grayscale => 11_u8.hash(&mut hasher),
        ImageColorMode::White => 12_u8.hash(&mut hasher),
        ImageColorMode::Color => 13_u8.hash(&mut hasher),
    }
    match spec.render_mode {
        ImageRenderMode::Braille => 1_u8.hash(&mut hasher),
        ImageRenderMode::Math => 2_u8.hash(&mut hasher),
        ImageRenderMode::Number => 5_u8.hash(&mut hasher),
        ImageRenderMode::Block => 6_u8.hash(&mut hasher),
    }
    match kind {
        ImageKind::Thumbnail => 3_u8.hash(&mut hasher),
        ImageKind::Banner => 4_u8.hash(&mut hasher),
    }
    format!("{:016x}", hasher.finish())
}

fn render_raster_image(path: &Path, spec: &ImageSpec, kind: ImageKind) -> Result<ModImage> {
    let dynamic = image::open(path)
        .with_context(|| format!("failed to open raster image: {}", path.display()))?;
    let image = match spec.render_mode {
        ImageRenderMode::Braille => render_braille_image(&dynamic, spec.color_mode, kind),
        ImageRenderMode::Math => render_charset_image(&dynamic, spec.color_mode, kind, &MATH_IMAGE_CHARS),
        ImageRenderMode::Number => render_charset_image(&dynamic, spec.color_mode, kind, &NUMBER_IMAGE_CHARS),
        ImageRenderMode::Block => render_charset_image(&dynamic, spec.color_mode, kind, &BLOCK_IMAGE_CHARS),
    };
    Ok(image)
}

fn render_braille_image(
    image: &image::DynamicImage,
    color_mode: ImageColorMode,
    kind: ImageKind,
) -> ModImage {
    let (target_h, target_w) = image_target_size(kind);
    let visual_w = image_visual_width(target_w);
    let pixel_w = (visual_w * 2) as u32;
    let pixel_h = (target_h * 4) as u32;
    let resized = resize_and_crop_image(image, pixel_w, pixel_h);

    let mut lines = Vec::with_capacity(target_h);
    for cell_y in 0..target_h {
        let mut line = if matches!(color_mode, ImageColorMode::Grayscale | ImageColorMode::Color) {
            String::from("f%")
        } else {
            String::new()
        };
        let mut current_color: Option<String> = None;

        for cell_x in 0..visual_w {
            let mut bits = 0u8;
            let mut rgb_sum = [0u32; 3];
            let mut samples = 0u32;

            for py in 0..4 {
                for px in 0..2 {
                    let x = (cell_x * 2 + px) as u32;
                    let y = (cell_y * 4 + py) as u32;
                    let pixel = resized.get_pixel(x, y).0;
                    let alpha = pixel[3] as f32 / 255.0;
                    let luminance = ((0.299 * pixel[0] as f32)
                        + (0.587 * pixel[1] as f32)
                        + (0.114 * pixel[2] as f32))
                        * alpha;

                    if alpha > 0.05 && luminance < 196.0 {
                        bits |= braille_bit(px, py);
                    }

                    rgb_sum[0] += pixel[0] as u32;
                    rgb_sum[1] += pixel[1] as u32;
                    rgb_sum[2] += pixel[2] as u32;
                    samples += 1;
                }
            }

            let ch = if bits == 0 {
                ' '
            } else {
                char::from_u32(0x2800 + bits as u32).unwrap_or(' ')
            };

            if ch != ' ' {
                if let Some(color) = image_output_color(
                    color_mode,
                    [
                        (rgb_sum[0] / samples) as u8,
                        (rgb_sum[1] / samples) as u8,
                        (rgb_sum[2] / samples) as u8,
                    ],
                ) {
                    if current_color.as_deref() != Some(color.as_str()) {
                        line.push_str(&format!("{{tc:{color}}}"));
                        current_color = Some(color);
                    }
                }
            }

            line.push(ch);
            line.push(ch);
        }

        if matches!(color_mode, ImageColorMode::Grayscale | ImageColorMode::Color) && current_color.is_some() {
            line.push_str("{tc:clear}");
        }
        lines.push(line);
    }

    ModImage { lines }
}

fn render_charset_image(
    image: &image::DynamicImage,
    color_mode: ImageColorMode,
    kind: ImageKind,
    chars: &[char],
) -> ModImage {
    let (target_h, target_w) = image_target_size(kind);
    let visual_w = image_visual_width(target_w);
    let resized = resize_and_crop_image(image, visual_w as u32, target_h as u32);
    let mut lines = Vec::with_capacity(target_h);

    for y in 0..target_h {
        let mut line = if matches!(color_mode, ImageColorMode::Grayscale | ImageColorMode::Color) {
            String::from("f%")
        } else {
            String::new()
        };
        let mut current_color: Option<String> = None;

        for x in 0..visual_w {
            let pixel = resized.get_pixel(x as u32, y as u32).0;
            let alpha = pixel[3] as f32 / 255.0;
            let luminance = ((0.299 * pixel[0] as f32)
                + (0.587 * pixel[1] as f32)
                + (0.114 * pixel[2] as f32))
                * alpha;
            let index = ((luminance / 255.0) * (chars.len() - 1) as f32).round()
                as usize;
            let ch = if alpha <= 0.05 {
                ' '
            } else {
                chars[index.min(chars.len() - 1)]
            };

            if ch != ' ' {
                if let Some(color) = image_output_color(color_mode, [pixel[0], pixel[1], pixel[2]]) {
                    if current_color.as_deref() != Some(color.as_str()) {
                        line.push_str(&format!("{{tc:{color}}}"));
                        current_color = Some(color);
                    }
                }
            }

            line.push(ch);
            line.push(ch);
        }

        if matches!(color_mode, ImageColorMode::Grayscale | ImageColorMode::Color) && current_color.is_some() {
            line.push_str("{tc:clear}");
        }
        lines.push(line);
    }

    ModImage { lines }
}

fn image_output_color(color_mode: ImageColorMode, rgb: [u8; 3]) -> Option<String> {
    match color_mode {
        ImageColorMode::White => None,
        ImageColorMode::Color => Some(format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])),
        ImageColorMode::Grayscale => {
            let gray = ((0.299 * rgb[0] as f32) + (0.587 * rgb[1] as f32) + (0.114 * rgb[2] as f32))
                .round() as u8;
            Some(format!("#{:02x}{:02x}{:02x}", gray, gray, gray))
        }
    }
}

fn resize_and_crop_image(
    image: &image::DynamicImage,
    target_w: u32,
    target_h: u32,
) -> image::RgbaImage {
    use image::imageops::FilterType;

    let (src_w, src_h) = image.dimensions();
    let src_ratio = src_w as f32 / src_h as f32;
    let dst_ratio = target_w as f32 / target_h as f32;

    let cropped = if src_ratio > dst_ratio {
        let new_w = (src_h as f32 * dst_ratio).round().max(1.0) as u32;
        let start_x = (src_w.saturating_sub(new_w)) / 2;
        image.crop_imm(start_x, 0, new_w, src_h)
    } else {
        let new_h = (src_w as f32 / dst_ratio).round().max(1.0) as u32;
        let start_y = (src_h.saturating_sub(new_h)) / 2;
        image.crop_imm(0, start_y, src_w, new_h)
    };

    cropped
        .resize_exact(target_w, target_h, FilterType::Triangle)
        .to_rgba8()
}

fn braille_bit(px: usize, py: usize) -> u8 {
    match (px, py) {
        (0, 0) => 0x01,
        (0, 1) => 0x02,
        (0, 2) => 0x04,
        (1, 0) => 0x08,
        (1, 1) => 0x10,
        (1, 2) => 0x20,
        (0, 3) => 0x40,
        (1, 3) => 0x80,
        _ => 0,
    }
}

fn parse_ascii_image_array(raw: &[JsonValue], _kind: ImageKind) -> Option<ModImage> {
    let mut lines = Vec::new();
    for row in raw {
        let mut line = String::new();
        flatten_ascii_row(row, &mut line)?;
        lines.push(line);
    }
    if lines.is_empty() {
        None
    } else {
        Some(ModImage { lines })
    }
}

fn flatten_ascii_row(value: &JsonValue, out: &mut String) -> Option<()> {
    match value {
        JsonValue::String(value) => {
            out.push_str(value);
            Some(())
        }
        JsonValue::Number(value) => {
            out.push_str(&value.to_string());
            Some(())
        }
        JsonValue::Array(items) => {
            for item in items {
                flatten_ascii_row(item, out)?;
            }
            Some(())
        }
        _ => None,
    }
}

fn resolve_mod_text(namespace: &str, raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Some((prefix, key)) = trimmed.split_once(':') {
        if prefix == namespace && !key.contains('/') && !key.contains('\\') {
            return resolve_mod_lang_key(namespace, key);
        }
    }

    if is_probable_lang_key(trimmed) {
        let resolved = resolve_mod_lang_key(namespace, trimmed);
        if !resolved.starts_with("[missing-i18n-key:") {
            return resolved;
        }
    }

    trimmed.to_string()
}

fn resolve_mod_lang_key(namespace: &str, key: &str) -> String {
    let current_code = i18n::current_language_code().replace('-', "_").to_lowercase();
    if let Some(value) = load_mod_lang_value(namespace, &current_code, key) {
        return value;
    }
    if let Some(value) = load_mod_lang_value(namespace, "en_us", key) {
        return value;
    }
    format!("[missing-i18n-key:{namespace}:{key}]")
}

fn load_mod_lang_value(namespace: &str, code: &str, key: &str) -> Option<String> {
    let lang_path = mod_data_dir().ok()?.join(namespace).join("assets").join("lang").join(format!("{code}.json"));
    let raw = fs::read_to_string(lang_path).ok()?;
    let json = serde_json::from_str::<JsonValue>(raw.trim_start_matches('\u{feff}')).ok()?;
    json.as_object()?.get(key)?.as_str().map(|value| value.to_string())
}

fn resolve_asset_path(namespace: &str, path_str: &str) -> Result<PathBuf> {
    if path_str.starts_with('/') || path_str.starts_with('\\') {
        return Err(anyhow!("asset path must be relative"));
    }
    let asset_path = mod_data_dir()?.join(namespace).join("assets").join(path_str);
    let asset_root = mod_data_dir()?.join(namespace).join("assets");
    let normalized = asset_path.components().collect::<PathBuf>();
    if path_str.split(['/', '\\']).any(|segment| segment == "." || segment == "..") {
        return Err(anyhow!("asset path cannot escape assets directory"));
    }
    if !normalized.starts_with(&asset_root) {
        return Err(anyhow!("asset path cannot escape assets directory"));
    }
    Ok(normalized)
}

fn default_image(kind: ImageKind) -> ModImage {
    let lines = match kind {
        ImageKind::Thumbnail => DEFAULT_THUMBNAIL_LINES.iter().map(|line| (*line).to_string()).collect(),
        ImageKind::Banner => DEFAULT_BANNER_ASCII.iter().map(|line| (*line).to_string()).collect(),
    };
    ModImage { lines }
}

fn normalize_image(image: ModImage, kind: ImageKind) -> ModImage {
    let (target_h, target_w) = image_target_size(kind);

    let mut lines = image.lines;
    if lines.is_empty() {
        lines = default_image(kind).lines;
    }

    lines = center_crop_or_pad_vertical(lines, target_h);
    lines = lines
        .into_iter()
        .map(|line| center_crop_or_pad_horizontal(&line, target_w))
        .collect();

    ModImage { lines }
}

fn center_crop_or_pad_vertical(mut lines: Vec<String>, target_h: usize) -> Vec<String> {
    if lines.len() > target_h {
        let start = (lines.len() - target_h) / 2;
        lines = lines.into_iter().skip(start).take(target_h).collect();
    }
    while lines.len() < target_h {
        if lines.len() % 2 == 0 {
            lines.insert(0, String::new());
        } else {
            lines.push(String::new());
        }
    }
    lines
}

fn center_crop_or_pad_horizontal(line: &str, target_w: usize) -> String {
    let current_w = visible_text_width(line);
    if current_w > target_w && !line.starts_with("f%") && !line.contains('{') {
        let chars: Vec<char> = line.chars().collect();
        let start = (chars.len().saturating_sub(target_w)) / 2;
        return chars
            .into_iter()
            .skip(start)
            .take(target_w)
            .collect::<String>();
    }
    if current_w >= target_w {
        return line.to_string();
    }
    pad_line_balanced(line, target_w - current_w)
}

fn image_target_size(kind: ImageKind) -> (usize, usize) {
    match kind {
        ImageKind::Thumbnail => (4, 8),
        ImageKind::Banner => (13, 86),
    }
}

fn image_visual_width(char_width: usize) -> usize {
    (char_width / 2).max(1)
}

fn pad_line_balanced(line: &str, pad: usize) -> String {
    let mut left = 0usize;
    let mut right = 0usize;
    let mut add_left = true;
    for _ in 0..pad {
        if add_left {
            left += 1;
        } else {
            right += 1;
        }
        add_left = !add_left;
    }
    format!("{}{}{}", " ".repeat(left), line, " ".repeat(right))
}

fn visible_text_width(text: &str) -> usize {
    let text = text.strip_prefix("f%").unwrap_or(text);
    let chars = text.chars().collect::<Vec<_>>();
    let mut i = 0usize;
    let mut width = 0usize;

    while i < chars.len() {
        match chars[i] {
            '\\' => {
                if i + 1 < chars.len() {
                    if chars[i + 1] != 'n' {
                        width += chars[i + 1].width().unwrap_or(0);
                    }
                    i += 2;
                } else {
                    width += 1;
                    i += 1;
                }
            }
            '{' => {
                if let Some(end) = chars[i + 1..].iter().position(|ch| *ch == '}') {
                    i += end + 2;
                } else {
                    width += 1;
                    i += 1;
                }
            }
            '\n' => {
                i += 1;
            }
            ch => {
                width += ch.width().unwrap_or(0);
                i += 1;
            }
        }
    }

    width
}

fn is_probable_lang_key(value: &str) -> bool {
    value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-')) && value.contains('.')
}

fn mtime_secs(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

fn build_game_id(meta: &RawMeta, script_name: &str) -> String {
    let seed = format!("{}|{}|{}", meta.package_name, meta.namespace, meta.author);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    seed.hash(&mut hasher);
    let hash = format!("{:08x}", hasher.finish() as u32);
    format!("{}:{}:{}", meta.namespace, hash, script_name)
}

fn mod_game_to_game_meta(game: ModGameMeta) -> GameMeta {
    GameMeta {
        id: game.game_id,
        name: game.name,
        description: game.description,
        detail: game.detail,
        save: game.save,
        script_path: game.script_path,
        mod_info: Some(game.mod_info),
    }
}

fn scan_error(namespace: &str, scope: &str, target: impl Into<String>, severity: &str, message: impl Into<String>) -> ModScanError {
    ModScanError {
        namespace: namespace.to_string(),
        scope: scope.to_string(),
        target: target.into(),
        severity: severity.to_string(),
        message: message.into(),
    }
}

impl ApiVersionField {
    fn supports(&self, version: u32) -> bool {
        match self {
            Self::Single(value) => *value == version,
            Self::Range([min, max]) => version >= *min && version <= *max,
        }
    }
}

