use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::host_engine::services::log::{LogService, LogSource};

// ── 宿主常量 ──

pub const HOST_API_VERSION: u32 = 1;
pub const HOST_SCHEMA_VERSION: u32 = 1;
const VALID_TARGET_FPS: &[u32] = &[30, 60, 120];

// ── 公开类型 ──

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageSource {
  Official,
  Mod,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageType {
  Game,
  Screensaver,
}

#[derive(Clone, Debug)]
pub struct PackageInfo {
  pub source: PackageSource,
  pub dir_name: String,
  pub mod_id: String,
  pub package_type: PackageType,
  pub version: String,
  pub version_code: u32,
  pub api_min: u32,
  pub api_max: u32,
  pub entry: String,
  pub display: PackageDisplay,
  pub runtime: PackageRuntime,
  pub game: Option<GameConfig>,
  pub screensaver: Option<ScreensaverConfig>,
  pub path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct PackageDisplay {
  pub title: String,
  pub description: String,
  pub author: String,
  pub icon: IconOrImage,
  pub banner: IconOrImage,
}

#[derive(Clone, Debug)]
pub enum IconOrImage {
  Image(String),
  ArtArray,
}

#[derive(Clone, Debug)]
pub struct PackageRuntime {
  pub min_width: u32,
  pub min_height: u32,
}

#[derive(Clone, Debug)]
pub struct GameConfig {
  pub name: String,
  pub detail: String,
  pub write: bool,
  pub mouse: bool,
  pub target_fps: u32,
  pub save: bool,
  pub score: Option<ScoreConfig>,
  pub actions: HashMap<String, ActionConfig>,
}

#[derive(Clone, Debug)]
pub struct ScoreConfig {
  pub enabled: bool,
  pub empty_text: String,
}

#[derive(Clone, Debug)]
pub struct ActionConfig {
  pub description: String,
  pub keys: Vec<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct ScreensaverConfig {
  pub name: String,
}

// ── 服务 ──

pub struct PackageService {
  games: Vec<PackageInfo>,
  screensavers: Vec<PackageInfo>,
}

impl PackageService {
  pub fn new() -> Self {
    Self {
      games: Vec::new(),
      screensavers: Vec::new(),
    }
  }

  pub fn scan_all(&mut self, root_dir: &Path, log: &mut LogService) {
    self.games.clear();
    self.screensavers.clear();

    let mut errors = 0u32;
    let mut duplicates = 0u32;

    self.scan_dir(
      root_dir,
      "scripts/game",
      PackageType::Game,
      PackageSource::Official,
      log,
      &mut errors,
      &mut duplicates,
    );
    self.scan_dir(
      root_dir,
      "scripts/screensaver",
      PackageType::Screensaver,
      PackageSource::Official,
      log,
      &mut errors,
      &mut duplicates,
    );
    self.scan_dir(
      root_dir,
      "data/mod/game",
      PackageType::Game,
      PackageSource::Mod,
      log,
      &mut errors,
      &mut duplicates,
    );
    self.scan_dir(
      root_dir,
      "data/mod/screensaver",
      PackageType::Screensaver,
      PackageSource::Mod,
      log,
      &mut errors,
      &mut duplicates,
    );

    log.info(
      LogSource::Pack,
      format!(
        "Scanned {} packages ({} games, {} screensavers), {} errors, {} duplicates skipped",
        self.total_count(),
        self.games.len(),
        self.screensavers.len(),
        errors,
        duplicates,
      ),
    );
  }

  fn scan_dir(
    &mut self,
    root_dir: &Path,
    relative: &str,
    expected_type: PackageType,
    source: PackageSource,
    log: &mut LogService,
    errors: &mut u32,
    duplicates: &mut u32,
  ) {
    let dir = root_dir.join(relative);
    let entries = match std::fs::read_dir(&dir) {
      Ok(e) => e,
      Err(_) => return,
    };

    for entry in entries.flatten() {
      let path = entry.path();
      if !path.is_dir() {
        continue;
      }
      let dir_name = path.file_name().unwrap().to_string_lossy().to_string();

      match self.read_package(&path, &dir_name, &expected_type, &source) {
        Ok(info) => {
          if self.has_mod_id(&info.mod_id) {
            log.warn(
              LogSource::Pack,
              format!(
                "Duplicate mod_id '{}' in '{}', keeping first",
                info.mod_id, dir_name,
              ),
            );
            *duplicates += 1;
            continue;
          }
          self.insert(info);
        }
        Err(msg) => {
          log.warn(
            LogSource::Pack,
            format!("Skipping '{}/{}': {}", relative, dir_name, msg),
          );
          *errors += 1;
        }
      }
    }
  }

  fn read_package(
    &self,
    dir: &Path,
    dir_name: &str,
    expected_type: &PackageType,
    source: &PackageSource,
  ) -> Result<PackageInfo, String> {
    let json_path = dir.join("package.json");
    let content = std::fs::read_to_string(&json_path)
      .map_err(|e| format!("Cannot read package.json: {}", e))?;

    let raw: RawPackageJson =
      serde_json::from_str(&content).map_err(|e| format!("Invalid JSON: {}", e))?;

    // ── 校验 ──

    if raw.mod_id.trim().is_empty() {
      return Err("mod_id is empty".into());
    }

    if raw.schema_version != HOST_SCHEMA_VERSION {
      return Err(format!(
        "schema_version {} != host {}",
        raw.schema_version, HOST_SCHEMA_VERSION
      ));
    }

    let pkg_type = parse_package_type(&raw.package_type)?;
    if pkg_type != *expected_type {
      return Err(format!(
        "Type mismatch: manifest says {:?}, directory expects {:?}",
        pkg_type, expected_type
      ));
    }

    if raw.version_code == 0 {
      return Err("version_code must be > 0".into());
    }

    if raw.api.min > raw.api.max {
      return Err(format!(
        "api.min ({}) > api.max ({})",
        raw.api.min, raw.api.max
      ));
    }
    if raw.api.min > HOST_API_VERSION {
      return Err(format!(
        "api.min ({}) > host API ({})",
        raw.api.min, HOST_API_VERSION
      ));
    }
    if raw.api.max < HOST_API_VERSION {
      return Err(format!(
        "api.max ({}) < host API ({})",
        raw.api.max, HOST_API_VERSION
      ));
    }

    let entry = resolve_entry(dir, &raw.entry)?;

    let display = raw.display.unwrap_or_default();
    if display.title.trim().is_empty() {
      return Err("display.title is empty".into());
    }

    let runtime = raw.runtime.unwrap_or_default();

    let game = match pkg_type {
      PackageType::Game => {
        let g = raw.game.ok_or("Missing 'game' config for game type")?;
        if !VALID_TARGET_FPS.contains(&g.target_fps) {
          return Err(format!(
            "target_fps must be one of {:?}, got {}",
            VALID_TARGET_FPS, g.target_fps
          ));
        }
        let mut actions = HashMap::new();
        if let Some(raw_actions) = g.actions {
          for (name, a) in raw_actions {
            if a.keys.is_empty() {
              return Err(format!("action '{}' has empty keys", name));
            }
            actions.insert(
              name,
              ActionConfig {
                description: a.description.unwrap_or_default(),
                keys: a.keys,
              },
            );
          }
        }
        Some(GameConfig {
          name: g.name.unwrap_or_default(),
          detail: g.detail.unwrap_or_default(),
          write: g.write.unwrap_or(false),
          mouse: g.mouse.unwrap_or(false),
          target_fps: g.target_fps,
          save: g.save.unwrap_or(false),
          score: g.score.map(|s| ScoreConfig {
            enabled: s.enabled,
            empty_text: s.empty_text.unwrap_or_default(),
          }),
          actions,
        })
      }
      PackageType::Screensaver => None,
    };

    let screensaver = match pkg_type {
      PackageType::Screensaver => {
        let s = raw.screensaver.ok_or("Missing 'screensaver' config")?;
        Some(ScreensaverConfig {
          name: s.name.unwrap_or_default(),
        })
      }
      PackageType::Game => None,
    };

    Ok(PackageInfo {
      source: source.clone(),
      dir_name: dir_name.to_string(),
      mod_id: raw.mod_id,
      package_type: pkg_type,
      version: raw.version.unwrap_or_default(),
      version_code: raw.version_code,
      api_min: raw.api.min,
      api_max: raw.api.max,
      entry,
      display: PackageDisplay {
        title: display.title,
        description: display.description.unwrap_or_default(),
        author: display.author.unwrap_or_default(),
        icon: parse_icon_or_image(&display.icon),
        banner: parse_icon_or_image(&display.banner),
      },
      runtime: PackageRuntime {
        min_width: runtime.min_width.unwrap_or(0),
        min_height: runtime.min_height.unwrap_or(0),
      },
      game,
      screensaver,
      path: dir.to_path_buf(),
    })
  }

  fn insert(&mut self, info: PackageInfo) {
    match info.package_type {
      PackageType::Game => self.games.push(info),
      PackageType::Screensaver => self.screensavers.push(info),
    }
  }

  fn has_mod_id(&self, id: &str) -> bool {
    self.games.iter().any(|p| p.mod_id == id) || self.screensavers.iter().any(|p| p.mod_id == id)
  }

  pub fn games(&self) -> &[PackageInfo] {
    &self.games
  }
  pub fn screensavers(&self) -> &[PackageInfo] {
    &self.screensavers
  }
  pub fn total_count(&self) -> usize {
    self.games.len() + self.screensavers.len()
  }
}

// ── JSON 反序列化结构 ──

#[derive(Deserialize)]
struct RawPackageJson {
  mod_id: String,
  schema_version: u32,
  #[serde(rename = "type")]
  package_type: String,
  version: Option<String>,
  version_code: u32,
  api: RawApiRange,
  entry: String,
  display: Option<RawDisplay>,
  runtime: Option<RawRuntime>,
  game: Option<RawGameConfig>,
  screensaver: Option<RawScreensaverConfig>,
}

#[derive(Deserialize)]
struct RawApiRange {
  min: u32,
  max: u32,
}

#[derive(Deserialize, Default)]
struct RawDisplay {
  title: String,
  description: Option<String>,
  author: Option<String>,
  #[serde(default)]
  icon: Option<String>,
  #[serde(default)]
  banner: Option<String>,
}

#[derive(Deserialize, Default)]
struct RawRuntime {
  min_width: Option<u32>,
  min_height: Option<u32>,
}

#[derive(Deserialize)]
struct RawGameConfig {
  name: Option<String>,
  detail: Option<String>,
  write: Option<bool>,
  mouse: Option<bool>,
  target_fps: u32,
  save: Option<bool>,
  score: Option<RawScoreConfig>,
  actions: Option<HashMap<String, RawActionConfig>>,
}

#[derive(Deserialize)]
struct RawScoreConfig {
  enabled: bool,
  empty_text: Option<String>,
}

#[derive(Deserialize)]
struct RawActionConfig {
  description: Option<String>,
  keys: Vec<Vec<String>>,
}

#[derive(Deserialize)]
struct RawScreensaverConfig {
  name: Option<String>,
}

// ── 辅助 ──

fn parse_package_type(s: &str) -> Result<PackageType, String> {
  match s {
    "game" => Ok(PackageType::Game),
    "screensaver" => Ok(PackageType::Screensaver),
    other => Err(format!("Unknown package type: '{}'", other)),
  }
}

fn parse_icon_or_image(raw: &Option<String>) -> IconOrImage {
  match raw {
    Some(s) if s.starts_with("image:") => {
      IconOrImage::Image(s.strip_prefix("image:").unwrap().to_string())
    }
    Some(_) => IconOrImage::ArtArray,
    None => IconOrImage::ArtArray,
  }
}

fn resolve_entry(pkg_dir: &Path, entry: &str) -> Result<String, String> {
  let scripts = pkg_dir.join("scripts");
  let candidates: &[PathBuf] = if entry.ends_with(".lua") {
    &[scripts.join(entry)]
  } else {
    &[scripts.join(format!("{}.lua", entry)), scripts.join(entry)]
  };
  for c in candidates {
    if c.exists() {
      return Ok(c.file_name().unwrap().to_string_lossy().to_string());
    }
  }
  Err(format!(
    "Entry '{}' not found in scripts/ (tried .lua and raw)",
    entry
  ))
}
