// 游戏注册表，管理所有已发现游戏的描述符集合。负责扫描包、合并官方和 Mod 游戏、应用用户保存的键位覆盖、过滤禁用的 Mod、生成最终 GameDescriptor

use std::collections::BTreeMap; // 维持动作顺序
use std::fs; // 目录扫描
use std::path::PathBuf; // 路径

use anyhow::Result; // 错误处理

use crate::core::stats; // 成绩数据清理
use crate::core::save as runtime_save; // 加载官方游戏的键位绑定
use crate::game::action::ActionBinding; // 动作类型
use crate::game::action::ActionKeys; // 动作类型
use crate::game::package::{GamePackageSource, load_package}; // 包加载
use crate::mods; // 读取 Mod 启用状态和键位覆盖
use crate::utils::path_utils; // 获取官方游戏目录
use crate::utils::host_log; // 日志

// 与 GamePackageSource 类似，用于 GameDescriptor 的 source 字段
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameSourceKind {
    Official,
    Mod,
}

// 包级别的元数据，用于资源访问和日志
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageDescriptor {
    pub root_dir: PathBuf,
    pub namespace: String,
    pub package_name: String,
    pub mod_name: Option<String>,
    pub author: String,
    pub version: String,
    pub debug_enabled: bool,
    pub source: GameSourceKind,
}

// 注册表中存储的完整游戏描述，包含显示字段（用于 UI）和运行时需要的路径、动作绑定等
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameDescriptor {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub display_description: String,
    pub detail: String,
    pub display_detail: String,
    pub author: String,
    pub display_author: String,
    pub introduction: Option<String>,
    pub icon: Option<serde_json::Value>,
    pub banner: Option<serde_json::Value>,
    pub best_none: Option<String>,
    pub display_best_none: Option<String>,
    pub has_best_score: bool,
    pub save: bool,
    pub api: Option<serde_json::Value>,
    pub entry: String,
    pub write: bool,
    pub case_sensitive: bool,
    pub min_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
    pub default_actions: BTreeMap<String, ActionBinding>,
    pub actions: BTreeMap<String, ActionBinding>,
    pub target_fps: u16,
    pub entry_path: PathBuf,
    pub source: GameSourceKind,
    pub package: Option<PackageDescriptor>,
    pub display_package_name: Option<String>,
    pub display_package_name_allows_rich: bool,
    pub display_package_author: Option<String>,
    pub display_package_version: Option<String>,
}

impl GameDescriptor {
    // 获取包信息
    pub fn package_info(&self) -> Option<&PackageDescriptor> {
        self.package.as_ref()
    }

    // 判断是否为 Mod 游戏
    pub fn is_mod_game(&self) -> bool {
        matches!(self.source, GameSourceKind::Mod)
    }
}

// 简单容器，提供扫描、查找、遍历等方法
#[derive(Clone, Debug, Default)]
pub struct GameRegistry {
    games: Vec<GameDescriptor>,
}

impl GameRegistry {
    // 创建空注册表
    pub fn empty() -> Self {
        Self { games: Vec::new() }
    }

    // 从已收集的列表创建
    pub fn from_games(games: Vec<GameDescriptor>) -> Self {
        Self { games }
    }

    // 扫描官方和 Mod 目录，去重，清理无效成绩记录
    pub fn scan_all() -> Result<Self> {
        let mut games = Vec::new();
        games.extend(scan_manifest_games(GamePackageSource::Official)?);
        games.extend(scan_manifest_games(GamePackageSource::Mod)?);

        let mut dedup = Vec::new();
        for game in games {
            if !dedup
                .iter()
                .any(|existing: &GameDescriptor| existing.id == game.id)
            {
                dedup.push(game);
            }
        }

        let valid_ids = dedup.iter().map(|game| game.id.clone()).collect::<Vec<_>>();
        let _ = stats::prune_runtime_scores(valid_ids);

        Ok(Self { games: dedup })
    }

    // 获取所有游戏引用
    pub fn games(&self) -> &[GameDescriptor] {
        &self.games
    }

    // 获取所有权
    pub fn into_games(self) -> Vec<GameDescriptor> {
        self.games
    }

    // 根据 ID 查找游戏
    pub fn find(&self, id: &str) -> Option<&GameDescriptor> {
        self.games.iter().find(|game| game.id == id)
    }
}

// 扫描指定来源目录，加载包，过滤禁用 Mod，生成 GameDescriptor 列表
fn scan_manifest_games(source: GamePackageSource) -> Result<Vec<GameDescriptor>> {
    let base_dir = match source {
        GamePackageSource::Official => path_utils::official_games_dir()?,
        GamePackageSource::Mod => mods::mod_data_dir()?,
    };

    let mut games = Vec::new();
    let mut entries: Vec<PathBuf> = fs::read_dir(&base_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir() && path.join("package.json").exists())
        .collect();
    entries.sort();
    let mod_state = if matches!(source, GamePackageSource::Mod) {
        Some(mods::load_mod_state())
    } else {
        None
    };
    for root_dir in entries {
        let fallback_log_object = package_log_object_from_root(&root_dir);
        let _log_object_guard = host_log::scoped_log_object(fallback_log_object);
        let package = match load_package(&root_dir, source.clone()) {
            Ok(package) => package,
            Err(err) => {
                let err_text = err.to_string();
                host_log::append_host_error("host.error.raw", &[("err", &err_text)]);
                continue;
            }
        };
        let enabled = mod_state
            .as_ref()
            .and_then(|state| state.mods.get(&package.package.namespace))
            .map(|entry| entry.enabled)
            .unwrap_or(true);
        if matches!(source, GamePackageSource::Mod) && !enabled {
            continue;
        }
        let debug_enabled = mod_state
            .as_ref()
            .and_then(|state| state.mods.get(&package.package.namespace))
            .map(|entry| entry.debug_enabled)
            .unwrap_or(false);
        let package_descriptor = PackageDescriptor {
            root_dir: package.root_dir.clone(),
            namespace: package.package.namespace.clone(),
            package_name: package.package.package_name.clone(),
            mod_name: package.package.mod_name.clone(),
            author: package.package.author.clone(),
            version: package.package.version.clone(),
            debug_enabled,
            source: match source {
                GamePackageSource::Official => GameSourceKind::Official,
                GamePackageSource::Mod => GameSourceKind::Mod,
            },
        };

        for game in package.games {
            let has_best_score = game.best_none.is_some();
            let (name, description, detail, author, introduction, icon, banner) =
                resolve_display_fields(&package.package, &game);
            let entry = game.entry.clone();
            let game_id = game.id.clone();
            let actions = apply_saved_keybindings(
                &game.actions,
                package_descriptor.namespace.as_str(),
                &game_id,
                &entry,
                &source,
                game.case_sensitive,
            );
            if matches!(source, GamePackageSource::Mod) {
                if !has_best_score {
                    host_log::append_host_warning(
                        "host.warning.best_none_null_ignored",
                        &[("mod_uid", game_id.as_str())],
                    );
                }
                if !game.save {
                    host_log::append_host_warning(
                        "host.warning.save_false_ignored",
                        &[("mod_uid", game_id.as_str())],
                    );
                }
            }
            games.push(GameDescriptor {
                id: game_id,
                name,
                display_name: String::new(),
                description,
                display_description: String::new(),
                detail,
                display_detail: String::new(),
                author,
                display_author: String::new(),
                introduction,
                icon,
                banner,
                best_none: game.best_none,
                display_best_none: None,
                has_best_score,
                save: game.save,
                api: game.api,
                entry: entry.clone(),
                write: game.write,
                case_sensitive: game.case_sensitive,
                min_width: game.min_width,
                min_height: game.min_height,
                max_width: game.max_width,
                max_height: game.max_height,
                default_actions: game.actions.clone(),
                actions,
                target_fps: sanitize_target_fps(game.runtime.target_fps),
                entry_path: resolve_entry_path(&package.root_dir, &entry, &source),
                source: package_descriptor.source.clone(),
                package: Some(package_descriptor.clone()),
                display_package_name: None,
                display_package_name_allows_rich: false,
                display_package_author: None,
                display_package_version: None,
            });
        }
    }
    Ok(games)
}

// 生成包的日志对象标识符
fn package_log_object_from_root(root_dir: &PathBuf) -> String {
    let name = root_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if name.trim().is_empty() {
        return "宿主".to_string();
    }
    format!("tui_game_{}", name)
}

// 将用户保存的键位覆盖应用到游戏动作（官方用 runtime_save，Mod 用 mods::read_mod_keybindings）
fn apply_saved_keybindings(
    actions: &BTreeMap<String, ActionBinding>,
    namespace: &str,
    game_id: &str,
    script_name: &str,
    source: &GamePackageSource,
    case_sensitive: bool,
) -> BTreeMap<String, ActionBinding> {
    let overrides = match source {
        GamePackageSource::Official => runtime_save::load_keybindings(game_id).unwrap_or_default(),
        GamePackageSource::Mod => mods::read_mod_keybindings(namespace, game_id),
    };
    if overrides.is_empty() {
        return actions.clone();
    }

    let mut out = actions.clone();
    for (action_name, keys) in overrides {
        let Some(binding) = out.get_mut(&action_name) else {
            continue;
        };
        let keys = compact_action_keys(keys, case_sensitive);
        binding.key = match keys.len() {
            0 => ActionKeys::Multiple(Vec::new()),
            1 => ActionKeys::Single(keys[0].clone()),
            _ => ActionKeys::Multiple(keys),
        };
    }
    let _ = script_name;
    out
}

// 去重并截断键列表（最多 5 个），大小写敏感由参数控制
fn compact_action_keys(keys: Vec<String>, case_sensitive: bool) -> Vec<String> {
    let mut out = Vec::new();
    for key in keys.into_iter().filter(|key| !key.trim().is_empty()) {
        let exists = out.iter().any(|existing: &String| {
            if case_sensitive {
                existing == &key
            } else {
                existing.eq_ignore_ascii_case(&key)
            }
        });
        if !exists {
            out.push(key);
        }
        if out.len() >= 5 {
            break;
        }
    }
    out
}

// 验证 FPS 值，非法值回退到 60 并记录警告
fn sanitize_target_fps(value: Option<u16>) -> u16 {
    match value {
        Some(30) => 30,
        Some(60) => 60,
        Some(120) => 120,
        Some(actual_fps) => {
            host_log::append_host_warning(
                "host.warning.invalid_fps_fallback",
                &[("actual_fps", &actual_fps.to_string())],
            );
            60
        }
        None => 60,
    }
}

// 优先使用包级别的显示字段（如 game_name），否则使用游戏清单的字段
fn resolve_display_fields(
    package: &crate::game::manifest::PackageManifest,
    game: &crate::game::manifest::GameManifest,
) -> (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<serde_json::Value>,
    Option<serde_json::Value>,
) {
    let name = package
        .game_name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| game.name.clone());
    let description = if package.description.trim().is_empty() {
        game.description.clone()
    } else {
        package.description.clone()
    };
    let detail = package
        .detail
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| game.detail.clone());
    let author = if package.author.trim().is_empty() {
        game.author.clone()
    } else {
        package.author.clone()
    };
    let introduction = package
        .introduction
        .clone()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| game.introduction.clone());
    let icon = package.icon.clone().or(game.icon.clone());
    let banner = package.banner.clone().or(game.banner.clone());

    (name, description, detail, author, introduction, icon, banner)
}

// 解析入口脚本的绝对路径，Mod 游戏默认放在 scripts/ 下
fn resolve_entry_path(root_dir: &PathBuf, entry: &str, source: &GamePackageSource) -> PathBuf {
    if matches!(source, GamePackageSource::Mod)
        && !entry.starts_with("scripts/")
        && !entry.starts_with("scripts\\")
    {
        root_dir.join("scripts").join(entry)
    } else {
        root_dir.join(entry)
    }
}
