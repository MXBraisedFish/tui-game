use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use unicode_width::UnicodeWidthChar;

use crate::app::i18n;
use crate::game::package::{GamePackageSource, load_package};
use crate::utils::path_utils;

pub const MOD_API_VERSION: u32 = 1;

const DEFAULT_PACKAGE_DESCRIPTION: &str = "No package description available.";
const DEFAULT_GAME_DESCRIPTION: &str = "No description available.";
const DEFAULT_GAME_DETAIL: &str = "";
const MATH_IMAGE_CHARS: [char; 9] = ['@', '%', '#', '*', '+', '=', '-', ':', '.'];
const NUMBER_IMAGE_CHARS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const BLOCK_IMAGE_CHARS: [char; 3] = ['\u{2588}', '\u{2593}', '\u{2591}'];

const DEFAULT_THUMBNAIL_LINES: [&str; 4] = [
    "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}",
    "\u{2588}\u{2588} \u{2588}\u{2588} \u{2588}\u{2588}",
    "   \u{2588}\u{2588}   ",
    "  \u{2588}\u{2588}\u{2588}\u{2588}  ",
];

const DEFAULT_BANNER_ASCII: [&str; 7] = [
    "`7MMM.     ,MMF' .g8\"\"8q. `7MM\"\"\"Yb.   ",
    "  MMMb    dPMM .dP'    `YM. MM    `Yb. ",
    "  M YM   ,M MM dM'      `MM MM     `Mb ",
    "  M  Mb  M' MM MM        MM MM      MM ",
    "  M  YM.P'  MM MM.      ,MP MM     ,MP ",
    "  M  `YM'   MM `Mb.    ,dP' MM    ,dP' ",
    ".JML. `'  .JMML. `\"bmmd\"' .JMMmmmdP'   ",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModImage {
    pub lines: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ModGameMeta {
    pub game_id: String,
    pub script_name: String,
    pub script_path: PathBuf,
    pub name: String,
    pub description: String,
    pub detail: String,
    pub best_none: Option<String>,
    pub save: bool,
    pub min_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
}

#[derive(Clone, Debug)]
pub struct ModPackage {
    pub namespace: String,
    pub enabled: bool,
    pub debug_enabled: bool,
    pub safe_mode_enabled: bool,
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
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModState {
    pub api_version: u32,
    #[serde(default)]
    pub mods: HashMap<String, ModStateEntry>,
    #[serde(default)]
    pub scan_errors: Vec<ModScanError>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModStateEntry {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub debug_enabled: bool,
    #[serde(default = "default_true")]
    pub safe_mode_enabled: bool,
    #[serde(skip)]
    pub session_safe_mode_enabled: Option<bool>,
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
            safe_mode_enabled: true,
            session_safe_mode_enabled: None,
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

static MOD_STATE_STORE: LazyLock<Mutex<ModState>> = LazyLock::new(|| {
    Mutex::new(read_persisted_mod_state().unwrap_or_else(|| ModState {
        api_version: MOD_API_VERSION,
        ..Default::default()
    }))
});

fn sanitize_mod_save_file_stem(game_id: &str) -> String {
    let mut sanitized = String::with_capacity(game_id.len());
    for ch in game_id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            sanitized.push(ch);
        } else {
            sanitized.push('_');
        }
    }

    while sanitized.contains("__") {
        sanitized = sanitized.replace("__", "_");
    }

    let trimmed = sanitized.trim_matches('_');
    if trimmed.is_empty() {
        "mod_save".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn mod_root_dir() -> Result<PathBuf> {
    Ok(path_utils::app_data_dir()?.join("mod"))
}

pub fn mod_data_dir() -> Result<PathBuf> {
    mod_root_dir()
}

pub fn mod_cache_dir() -> Result<PathBuf> {
    path_utils::cache_dir()
}

pub fn mod_save_dir(namespace: &str) -> Result<PathBuf> {
    Ok(path_utils::mod_save_dir()?.join(namespace))
}

pub fn mod_save_path(namespace: &str, game_id: &str) -> Result<PathBuf> {
    Ok(mod_save_dir(namespace)?.join(format!("{}.json", sanitize_mod_save_file_stem(game_id))))
}
pub fn load_mod_state() -> ModState {
    MOD_STATE_STORE
        .lock()
        .map(|state| state.clone())
        .unwrap_or_else(|_| ModState {
            api_version: MOD_API_VERSION,
            ..Default::default()
        })
}

pub fn save_mod_state(state: &ModState) -> Result<()> {
    if let Ok(mut guard) = MOD_STATE_STORE.lock() {
        *guard = state.clone();
    }
    persist_mod_state(state)?;
    Ok(())
}

pub fn load_scan_cache() -> ModScanCache {
    read_persisted_scan_cache().unwrap_or_default()
}

pub fn save_scan_cache(_cache: &ModScanCache) -> Result<()> {
    persist_scan_cache(_cache)
}

pub fn set_mod_enabled(namespace: &str, enabled: bool) -> Result<()> {
    let mut state = load_mod_state();
    state.mods.entry(namespace.to_string()).or_default().enabled = enabled;
    save_mod_state(&state)
}

pub fn set_mod_debug_enabled(namespace: &str, enabled: bool) -> Result<()> {
    let mut state = load_mod_state();
    state
        .mods
        .entry(namespace.to_string())
        .or_default()
        .debug_enabled = enabled;
    save_mod_state(&state)
}

pub fn set_mod_safe_mode(namespace: &str, enabled: bool, persist: bool) -> Result<()> {
    let mut state = load_mod_state();
    let entry = state.mods.entry(namespace.to_string()).or_default();
    if persist {
        entry.safe_mode_enabled = enabled;
        entry.session_safe_mode_enabled = None;
        save_mod_state(&state)
    } else {
        entry.session_safe_mode_enabled = Some(enabled);
        if let Ok(mut guard) = MOD_STATE_STORE.lock() {
            *guard = state;
        }
        Ok(())
    }
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

pub fn update_mod_best_score(
    namespace: &str,
    game_id: &str,
    script_name: &str,
    score: JsonValue,
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

    let _ = namespace;
    let _ = level;
    let _ = message;
    Ok(())
}

pub fn scan_mods() -> Result<ModScanOutput> {
    let root = mod_data_dir()?;
    fs::create_dir_all(&root)?;
    fs::create_dir_all(mod_cache_dir()?)?;
    fs::create_dir_all(path_utils::mod_save_dir()?)?;

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
                    "package.json",
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

    Ok(ModScanOutput { packages })
}

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
    let state_entry = state.mods.entry(namespace.clone()).or_default();
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
    let thumbnail = image_from_meta(
        &namespace,
        package.package.icon.as_ref(),
        ImageKind::Thumbnail,
    )?;
    let banner = image_from_meta(&namespace, package.package.banner.as_ref(), ImageKind::Banner)?;

    let package_name = resolve_mod_text(
        &namespace,
        package
            .package
            .mod_name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(&package.package.package_name),
    );
    let author = package.package.author.clone();
    let version = package.package.version.clone();
    let enabled = state_entry.enabled;
    let debug_enabled = state_entry.debug_enabled;
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

    Ok(Some(ModPackage {
        namespace,
        enabled,
        debug_enabled,
        safe_mode_enabled,
        package_name,
        author,
        version,
        description,
        thumbnail,
        banner,
        games,
        errors,
    }))
}
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
        .name
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
        best_none,
        save: game_manifest.save,
        min_width: game_manifest.min_width.filter(|value| *value > 0),
        min_height: game_manifest.min_height.filter(|value| *value > 0),
        max_width: game_manifest.max_width.filter(|value| *value > 0),
        max_height: game_manifest.max_height.filter(|value| *value > 0),
    })
}

fn validate_mod_package_root(dir: &Path, package: &crate::game::manifest::PackageManifest) -> Result<()> {
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
        .name
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

fn validate_mod_structure(dir: &Path) -> Result<()> {
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

fn resolve_mod_entry_path(package_dir: &Path, entry: &str) -> PathBuf {
    if entry.starts_with("scripts/") || entry.starts_with("scripts\\") {
        package_dir.join(entry)
    } else {
        package_dir.join("scripts").join(entry)
    }
}

fn image_from_meta(namespace: &str, raw: Option<&JsonValue>, kind: ImageKind) -> Result<ModImage> {
    let image = match raw {
        Some(JsonValue::String(value)) => load_image_from_string(namespace, value, kind)?,
        Some(JsonValue::Array(value)) => {
            parse_ascii_image_array(value, kind).unwrap_or_else(|| default_image(kind))
        }
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
        if let Ok(image) = serde_json::from_str::<ModImage>(raw.trim_start_matches('\u{feff}')) {
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
        ImageRenderMode::Math => {
            render_charset_image(&dynamic, spec.color_mode, kind, &MATH_IMAGE_CHARS)
        }
        ImageRenderMode::Number => {
            render_charset_image(&dynamic, spec.color_mode, kind, &NUMBER_IMAGE_CHARS)
        }
        ImageRenderMode::Block => {
            render_charset_image(&dynamic, spec.color_mode, kind, &BLOCK_IMAGE_CHARS)
        }
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
        let mut line = if matches!(
            color_mode,
            ImageColorMode::Grayscale | ImageColorMode::Color
        ) {
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

        if matches!(
            color_mode,
            ImageColorMode::Grayscale | ImageColorMode::Color
        ) && current_color.is_some()
        {
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
        let mut line = if matches!(
            color_mode,
            ImageColorMode::Grayscale | ImageColorMode::Color
        ) {
            String::from("f%")
        } else {
            String::new()
        };
        let mut current_color: Option<String> = None;

        for x in 0..visual_w {
            let pixel = resized.get_pixel(x as u32, y as u32).0;
            let alpha = pixel[3] as f32 / 255.0;
            let luminance =
                ((0.299 * pixel[0] as f32) + (0.587 * pixel[1] as f32) + (0.114 * pixel[2] as f32))
                    * alpha;
            let index = ((luminance / 255.0) * (chars.len() - 1) as f32).round() as usize;
            let ch = if alpha <= 0.05 {
                ' '
            } else {
                chars[index.min(chars.len() - 1)]
            };

            if ch != ' ' {
                if let Some(color) = image_output_color(color_mode, [pixel[0], pixel[1], pixel[2]])
                {
                    if current_color.as_deref() != Some(color.as_str()) {
                        line.push_str(&format!("{{tc:{color}}}"));
                        current_color = Some(color);
                    }
                }
            }

            line.push(ch);
            line.push(ch);
        }

        if matches!(
            color_mode,
            ImageColorMode::Grayscale | ImageColorMode::Color
        ) && current_color.is_some()
        {
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

pub fn resolve_mod_text_for_display(namespace: &str, raw: &str) -> String {
    resolve_mod_text(namespace, raw)
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
    let current_code = i18n::current_language_code()
        .replace('-', "_")
        .to_lowercase();
    if let Some(value) = load_mod_lang_value(namespace, &current_code, key) {
        return value;
    }
    if let Some(value) = load_mod_lang_value(namespace, "en_us", key) {
        return value;
    }
    format!("[missing-i18n-key:{namespace}:{key}]")
}

fn load_mod_lang_value(namespace: &str, code: &str, key: &str) -> Option<String> {
    let lang_path = mod_data_dir()
        .ok()?
        .join(namespace)
        .join("assets")
        .join("lang")
        .join(format!("{code}.json"));
    let raw = fs::read_to_string(lang_path).ok()?;
    let json = serde_json::from_str::<JsonValue>(raw.trim_start_matches('\u{feff}')).ok()?;
    json.as_object()?
        .get(key)?
        .as_str()
        .map(|value| value.to_string())
}

fn resolve_asset_path(namespace: &str, path_str: &str) -> Result<PathBuf> {
    if path_str.starts_with('/') || path_str.starts_with('\\') {
        return Err(anyhow!("asset path must be relative"));
    }
    let asset_path = mod_data_dir()?
        .join(namespace)
        .join("assets")
        .join(path_str);
    let asset_root = mod_data_dir()?.join(namespace).join("assets");
    let normalized = asset_path.components().collect::<PathBuf>();
    if path_str
        .split(['/', '\\'])
        .any(|segment| segment == "." || segment == "..")
    {
        return Err(anyhow!("asset path cannot escape assets directory"));
    }
    if !normalized.starts_with(&asset_root) {
        return Err(anyhow!("asset path cannot escape assets directory"));
    }
    Ok(normalized)
}

fn default_image(kind: ImageKind) -> ModImage {
    let lines = match kind {
        ImageKind::Thumbnail => DEFAULT_THUMBNAIL_LINES
            .iter()
            .map(|line| (*line).to_string())
            .collect(),
        ImageKind::Banner => DEFAULT_BANNER_ASCII
            .iter()
            .map(|line| (*line).to_string())
            .collect(),
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
    value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
        && value.contains('.')
}

fn mtime_secs(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

fn mod_state_cache_file() -> Result<PathBuf> {
    Ok(mod_cache_dir()?.join("mod_state.json"))
}

fn scan_cache_file() -> Result<PathBuf> {
    Ok(mod_cache_dir()?.join("scan_cache.json"))
}

fn read_persisted_mod_state() -> Option<ModState> {
    let path = mod_state_cache_file().ok()?;
    let raw = fs::read_to_string(path).ok()?;
    let mut state = serde_json::from_str::<ModState>(raw.trim_start_matches('\u{feff}')).ok()?;
    state.api_version = MOD_API_VERSION;
    Some(state)
}

fn persist_mod_state(state: &ModState) -> Result<()> {
    let path = mod_state_cache_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

fn read_persisted_scan_cache() -> Option<ModScanCache> {
    let path = scan_cache_file().ok()?;
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<ModScanCache>(raw.trim_start_matches('\u{feff}')).ok()
}

fn persist_scan_cache(cache: &ModScanCache) -> Result<()> {
    let path = scan_cache_file()?;
    path_utils::ensure_parent_dir(&path)?;
    fs::write(path, serde_json::to_string_pretty(cache)?)?;
    Ok(())
}

fn scan_error(
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
