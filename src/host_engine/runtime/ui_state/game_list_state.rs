//! UI GameList 状态聚合

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use mlua::{Lua, Table, Value};
use serde_json::Value as JsonValue;

use crate::host_engine::boot::i18n;
use crate::host_engine::boot::preload::game_modules::{
    GameModule, GameModuleRegistry, GameModuleSource,
};

const DEFAULT_LANGUAGE_CODE: &str = "en_us";

/// 游戏列表排序方式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameListSortMode {
    OfficialMod,
    Name,
    Author,
}

impl GameListSortMode {
    fn from_str(value: &str) -> Self {
        match value {
            "name" => Self::Name,
            "author" => Self::Author,
            _ => Self::OfficialMod,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::OfficialMod => "official_mod",
            Self::Name => "name",
            Self::Author => "author",
        }
    }
}

/// 游戏列表排序顺序。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameListSortOrder {
    Asc,
    Desc,
}

impl GameListSortOrder {
    fn from_str(value: &str) -> Self {
        match value {
            "desc" => Self::Desc,
            _ => Self::Asc,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

/// 游戏列表宿主与 Lua 双层状态。
#[derive(Clone, Debug)]
pub struct GameListUiState {
    pub root_state: GameListRootState,
    pub lua_state: GameListLuaState,
}

impl GameListUiState {
    /// 创建游戏列表状态。
    pub fn new(
        registry: GameModuleRegistry,
        best_scores: JsonValue,
        language_code: String,
    ) -> Self {
        let root_state = GameListRootState::new(registry, best_scores, language_code);
        let lua_state = GameListLuaState::from_root_state(&root_state);
        Self {
            root_state,
            lua_state,
        }
    }

    /// 进入游戏列表时重置 transient Lua state。
    pub fn reset_lua_state(&mut self) {
        self.root_state.refresh_language();
        self.root_state.normalize_select();
        self.lua_state = GameListLuaState::from_root_state(&self.root_state);
    }

    /// 刷新语言。
    pub fn refresh_language(&mut self, language_code: String) {
        self.root_state.language_code = language_code;
        self.root_state.refresh_language();
        self.root_state.refresh_game_display();
    }

    /// 应用 Lua 返回状态。
    pub fn apply_lua_state(&mut self, lua_state: GameListLuaState) -> GameListLuaAction {
        self.lua_state = lua_state;
        self.root_state.sort_mode = GameListSortMode::from_str(self.lua_state.sort.as_str());
        self.root_state.sort_order = GameListSortOrder::from_str(self.lua_state.order.as_str());
        self.root_state.pages = self.lua_state.pages.max(1);
        self.root_state.page = self.lua_state.page.clamp(1, self.root_state.pages);
        self.root_state.user_page = if self.lua_state.jump {
            self.lua_state.user_page
        } else {
            0
        };
        self.root_state.jump = self.lua_state.jump;
        self.root_state.info_scroll = self.lua_state.info_scroll.max(0);
        if self.lua_state.select.is_empty() {
            self.root_state.normalize_select();
        } else {
            self.root_state.selected_uid = self.lua_state.select.clone();
        }
        self.root_state.sort_games();
        self.root_state.normalize_select();

        if self.lua_state.back {
            self.lua_state.back = false;
            return GameListLuaAction::Back;
        }

        if self.lua_state.confirm {
            self.lua_state.confirm = false;
            return GameListLuaAction::Confirm(self.root_state.selected_uid.clone());
        }

        GameListLuaAction::None
    }
}

/// 游戏列表 Lua 返回动作。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameListLuaAction {
    None,
    Back,
    Confirm(String),
}

/// 游戏列表 Lua state。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameListLuaState {
    pub select: String,
    pub confirm: bool,
    pub back: bool,
    pub order: String,
    pub sort: String,
    pub pages: i64,
    pub page: i64,
    pub user_page: i64,
    pub jump: bool,
    pub info_scroll: i64,
}

impl GameListLuaState {
    fn from_root_state(root_state: &GameListRootState) -> Self {
        Self {
            select: root_state.selected_uid.clone(),
            confirm: false,
            back: false,
            order: root_state.sort_order.as_str().to_string(),
            sort: root_state.sort_mode.as_str().to_string(),
            pages: root_state.pages.max(1),
            page: root_state.page.max(1),
            user_page: if root_state.jump {
                root_state.user_page
            } else {
                0
            },
            jump: root_state.jump,
            info_scroll: root_state.info_scroll.max(0),
        }
    }

    /// 转为 Lua 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("select", self.select.as_str())?;
        table.set("confirm", false)?;
        table.set("back", false)?;
        table.set("order", self.order.as_str())?;
        table.set("sort", self.sort.as_str())?;
        table.set("pages", self.pages.max(1))?;
        table.set("page", self.page.max(1))?;
        table.set("user_page", if self.jump { self.user_page } else { 0 })?;
        table.set("jump", self.jump)?;
        table.set("info_scroll", self.info_scroll.max(0))?;
        Ok(table)
    }

    /// 从 Lua 返回值解析。
    pub fn from_lua_value(value: Value) -> mlua::Result<Self> {
        let table = match value {
            Value::Table(table) => table,
            _ => {
                return Err(mlua::Error::external(
                    "game list lua state must be returned as table",
                ));
            }
        };
        Ok(Self {
            select: table.get::<Option<String>>("select")?.unwrap_or_default(),
            confirm: table.get::<Option<bool>>("confirm")?.unwrap_or(false),
            back: table.get::<Option<bool>>("back")?.unwrap_or(false),
            order: table
                .get::<Option<String>>("order")?
                .unwrap_or_else(|| "asc".to_string()),
            sort: table
                .get::<Option<String>>("sort")?
                .unwrap_or_else(|| "official_mod".to_string()),
            pages: table.get::<Option<i64>>("pages")?.unwrap_or(1).max(1),
            page: table.get::<Option<i64>>("page")?.unwrap_or(1).max(1),
            user_page: table.get::<Option<i64>>("user_page")?.unwrap_or(0).max(0),
            jump: table.get::<Option<bool>>("jump")?.unwrap_or(false),
            info_scroll: table.get::<Option<i64>>("info_scroll")?.unwrap_or(0).max(0),
        })
    }
}

/// 游戏列表 root_state。
#[derive(Clone, Debug)]
pub struct GameListRootState {
    pub language: Vec<(String, String)>,
    pub games: Vec<GameListItem>,
    pub best_scores: JsonValue,
    pub language_code: String,
    pub selected_uid: String,
    pub sort_order: GameListSortOrder,
    pub sort_mode: GameListSortMode,
    pub pages: i64,
    pub page: i64,
    pub user_page: i64,
    pub jump: bool,
    pub info_scroll: i64,
}

impl GameListRootState {
    fn new(registry: GameModuleRegistry, best_scores: JsonValue, language_code: String) -> Self {
        let mut games = registry
            .games
            .iter()
            .map(|game_module| GameListItem::from_game_module(game_module, language_code.as_str()))
            .collect::<Vec<_>>();
        let selected_uid = games
            .first()
            .map(|game| game.uid.clone())
            .unwrap_or_default();
        let mut root_state = Self {
            language: game_list_language_pairs(),
            games: Vec::new(),
            best_scores,
            language_code,
            selected_uid,
            sort_order: GameListSortOrder::Asc,
            sort_mode: GameListSortMode::OfficialMod,
            pages: 1,
            page: 1,
            user_page: 0,
            jump: false,
            info_scroll: 0,
        };
        root_state.games.append(&mut games);
        root_state.sort_games();
        root_state.normalize_select();
        root_state
    }

    fn refresh_language(&mut self) {
        self.language = game_list_language_pairs();
    }

    fn refresh_game_display(&mut self) {
        for game in &mut self.games {
            game.refresh_display(self.language_code.as_str());
        }
    }

    fn normalize_select(&mut self) {
        if self.games.is_empty() {
            self.selected_uid.clear();
            return;
        }
        if !self.games.iter().any(|game| game.uid == self.selected_uid) {
            self.selected_uid = self.games[0].uid.clone();
        }
    }

    fn sort_games(&mut self) {
        let sort_mode = self.sort_mode;
        self.games
            .sort_by(|left, right| compare_game(left, right, sort_mode));
        if self.sort_order == GameListSortOrder::Desc {
            self.games.reverse();
        }
    }

    /// 转为 Lua root_state 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("language", pairs_to_table(lua, &self.language)?)?;
        table.set("game_list", game_list_to_table(lua, &self.games)?)?;
        table.set("game_info", self.selected_game_info(lua)?)?;
        table.set("order", self.sort_order.as_str())?;
        table.set("sort", self.sort_mode.as_str())?;
        table.set("select", self.selected_uid.as_str())?;
        table.set("pages", self.pages.max(1))?;
        table.set("page", self.page.max(1))?;
        table.set("user_page", if self.jump { self.user_page } else { 0 })?;
        table.set("jump", self.jump)?;
        table.set("info_scroll", self.info_scroll.max(0))?;
        Ok(table)
    }

    fn selected_game_info(&self, lua: &Lua) -> mlua::Result<Table> {
        let selected_game = self
            .games
            .iter()
            .find(|game| game.uid == self.selected_uid)
            .or_else(|| self.games.first());
        let table = match selected_game {
            Some(game) => game.to_lua_table(lua)?,
            None => lua.create_table()?,
        };
        if let Some(game) = selected_game {
            if let Some(best_score) = self.best_scores.get(game.uid.as_str()) {
                table.set("best_score", best_score_to_text(best_score))?;
            }
        }
        Ok(table)
    }
}

/// 游戏列表项。
#[derive(Clone, Debug)]
pub struct GameListItem {
    pub uid: String,
    pub source: GameModuleSource,
    pub source_label: String,
    pub package: String,
    pub mod_name_raw: String,
    pub introduction_raw: String,
    pub author_raw: String,
    pub game_name_raw: String,
    pub description_raw: String,
    pub detail_raw: String,
    pub version: String,
    pub root_dir: PathBuf,
    pub mod_name: String,
    pub introduction: String,
    pub author: String,
    pub game_name: String,
    pub description: String,
    pub detail: String,
}

impl GameListItem {
    fn from_game_module(game_module: &GameModule, language_code: &str) -> Self {
        let mut item = Self {
            uid: game_module.uid.clone(),
            source: game_module.source,
            source_label: game_module.source_label.clone(),
            package: game_module.package.package.clone(),
            mod_name_raw: game_module.package.mod_name.clone(),
            introduction_raw: game_module.package.introduction.clone(),
            author_raw: game_module.package.author.clone(),
            game_name_raw: game_module.package.game_name.clone(),
            description_raw: game_module.package.description.clone(),
            detail_raw: game_module.package.detail.clone(),
            version: game_module.package.version.clone(),
            root_dir: game_module.root_dir.clone(),
            mod_name: String::new(),
            introduction: String::new(),
            author: String::new(),
            game_name: String::new(),
            description: String::new(),
            detail: String::new(),
        };
        item.refresh_display(language_code);
        item
    }

    fn refresh_display(&mut self, language_code: &str) {
        let language_texts = load_package_language_texts(&self.root_dir, language_code);
        self.mod_name = resolve_package_text(&language_texts, &self.mod_name_raw);
        self.introduction = resolve_package_text(&language_texts, &self.introduction_raw);
        self.author = resolve_package_text(&language_texts, &self.author_raw);
        self.game_name = resolve_package_text(&language_texts, &self.game_name_raw);
        self.description = resolve_package_text(&language_texts, &self.description_raw);
        self.detail = resolve_package_text(&language_texts, &self.detail_raw);
    }

    fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("uid", self.uid.as_str())?;
        table.set("name", self.game_name.as_str())?;
        table.set("game_name", self.game_name.as_str())?;
        table.set("mod_name", self.mod_name.as_str())?;
        table.set("introduction", self.introduction.as_str())?;
        table.set("author", self.author.as_str())?;
        table.set("description", self.description.as_str())?;
        table.set("detail", self.detail.as_str())?;
        table.set("source", self.source_label.as_str())?;
        table.set("version", self.version.as_str())?;
        table.set("package", self.package.as_str())?;
        Ok(table)
    }
}

fn compare_game(
    left: &GameListItem,
    right: &GameListItem,
    sort_mode: GameListSortMode,
) -> Ordering {
    let primary = match sort_mode {
        GameListSortMode::OfficialMod => source_rank(left.source).cmp(&source_rank(right.source)),
        GameListSortMode::Name => compare_text(left.game_name.as_str(), right.game_name.as_str()),
        GameListSortMode::Author => compare_text(left.author.as_str(), right.author.as_str()),
    };
    primary
        .then_with(|| compare_text(left.game_name.as_str(), right.game_name.as_str()))
        .then_with(|| left.uid.cmp(&right.uid))
}

fn source_rank(source: GameModuleSource) -> u8 {
    match source {
        GameModuleSource::Office => 0,
        GameModuleSource::Mod => 1,
    }
}

fn compare_text(left: &str, right: &str) -> Ordering {
    left.to_lowercase().cmp(&right.to_lowercase())
}

fn game_list_language_pairs() -> Vec<(String, String)> {
    let text = i18n::text();
    vec![
        (
            "GAME_LIST_PREV_OPTION".to_string(),
            text.key.game_list_prev_option,
        ),
        (
            "GAME_LIST_NEXT_OPTION".to_string(),
            text.key.game_list_next_option,
        ),
        (
            "GAME_LIST_PREV_PAGE".to_string(),
            text.key.game_list_prev_page,
        ),
        (
            "GAME_LIST_NEXT_PAGE".to_string(),
            text.key.game_list_next_page,
        ),
        (
            "GAME_LIST_SCROLL_UP".to_string(),
            text.key.game_list_scroll_up,
        ),
        (
            "GAME_LIST_SCROLL_DOWN".to_string(),
            text.key.game_list_scroll_down,
        ),
        ("GAME_LIST_JUMP".to_string(), text.key.game_list_jump),
        ("GAME_LIST_ORDER".to_string(), text.key.game_list_order),
        ("GAME_LIST_SORT".to_string(), text.key.game_list_sort),
        ("GAME_LIST_BACK".to_string(), text.key.game_list_back),
        (
            "GAME_LIST_BACK_CANCEL".to_string(),
            text.key.game_list_back_cancel,
        ),
        (
            "GAME_LIST_START_CONFIRM".to_string(),
            text.key.game_list_start_confirm,
        ),
        ("GAME_LIST_START".to_string(), text.key.game_list_start),
        ("GAME_LIST_CONFIRM".to_string(), text.key.game_list_confirm),
        ("GAME_LIST_CANCEL".to_string(), text.key.game_list_cancel),
        ("GAME_LIST_SELECT".to_string(), text.key.game_list_select),
        ("GAME_LIST_FLIP".to_string(), text.key.game_list_flip),
        ("GAME_LIST_SCROLL".to_string(), text.key.game_list_scroll),
        (
            "GAME_LIST_LIST_TITLE".to_string(),
            text.game_list.list_title,
        ),
        (
            "GAME_LIST_INFO_SORT_NAME".to_string(),
            text.game_list.info_sort_name,
        ),
        (
            "GAME_LIST_INFO_SORT_MOD_OFFICIAL".to_string(),
            text.game_list.info_sort_mod_official,
        ),
        (
            "GAME_LIST_INFO_SORT_AUTHOR".to_string(),
            text.game_list.info_sort_author,
        ),
        (
            "GAME_LIST_INFO_ORDER_ASCENDING".to_string(),
            text.game_list.info_order_ascending,
        ),
        (
            "GAME_LIST_INFO_ORDER_DESCENDING".to_string(),
            text.game_list.info_order_descending,
        ),
        (
            "GAME_LIST_INFO_TITLE".to_string(),
            text.game_list.info_title,
        ),
        ("GAME_LIST_SOURCE_MOD".to_string(), text.game_list.mod_label),
        ("GAME_LIST_NONE_GAME".to_string(), text.game_list.none_game),
        ("GAME_LIST_NONE_INFO".to_string(), text.game_list.none_info),
    ]
}

fn pairs_to_table(lua: &Lua, pairs: &[(String, String)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (key, value) in pairs {
        table.set(key.as_str(), value.as_str())?;
    }
    Ok(table)
}

fn game_list_to_table(lua: &Lua, games: &[GameListItem]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, game) in games.iter().enumerate() {
        table.set(index + 1, game.to_lua_table(lua)?)?;
    }
    Ok(table)
}

fn best_score_to_text(best_score: &JsonValue) -> String {
    if let Some(best_string) = best_score.get("best_string").and_then(JsonValue::as_str) {
        return best_string.to_string();
    }
    if let Some(text) = best_score.as_str() {
        return text.to_string();
    }
    serde_json::to_string(best_score).unwrap_or_default()
}

fn load_package_language_texts(root_dir: &Path, language_code: &str) -> HashMap<String, String> {
    let mut texts = read_language_file(root_dir, DEFAULT_LANGUAGE_CODE);
    if language_code != DEFAULT_LANGUAGE_CODE {
        texts.extend(read_language_file(root_dir, language_code));
    }
    texts
}

fn read_language_file(root_dir: &Path, language_code: &str) -> HashMap<String, String> {
    let language_path = root_dir
        .join("assets")
        .join("lang")
        .join(format!("{language_code}.json"));
    let Ok(raw_json) = fs::read_to_string(language_path) else {
        return HashMap::new();
    };
    serde_json::from_str::<HashMap<String, String>>(raw_json.trim_start_matches('\u{feff}'))
        .unwrap_or_default()
}

fn resolve_package_text(language_texts: &HashMap<String, String>, raw_value: &str) -> String {
    language_texts
        .get(raw_value)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| raw_value.to_string())
}
