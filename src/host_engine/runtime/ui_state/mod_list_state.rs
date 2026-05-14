//! UI ModList 状态聚合

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use mlua::{Lua, Table, Value};
use serde_json::{Map, Value as JsonValue, json};
use unicode_width::UnicodeWidthStr;

use crate::host_engine::boot::i18n;
use crate::host_engine::boot::preload::game_modules::{
    GameModule, GameModuleRegistry, GameModuleSource,
};

const DEFAULT_LANGUAGE_CODE: &str = "en_us";

/// 模组列表排序方式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModListSortMode {
    Name,
    Author,
    SafeMode,
    Toggle,
}

impl ModListSortMode {
    fn from_str(value: &str) -> Self {
        match value {
            "author" => Self::Author,
            "safe_mode" => Self::SafeMode,
            "toggle" => Self::Toggle,
            _ => Self::Name,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Author => "author",
            Self::SafeMode => "safe_mode",
            Self::Toggle => "toggle",
        }
    }
}

/// 模组列表排序顺序。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModListSortOrder {
    Asc,
    Desc,
}

impl ModListSortOrder {
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

/// 模组列表宿主与 Lua 双层状态。
#[derive(Clone, Debug)]
pub struct ModListUiState {
    pub root_state: ModListRootState,
    pub lua_state: ModListLuaState,
}

impl ModListUiState {
    /// 创建模组列表状态。
    pub fn new(registry: GameModuleRegistry, mod_state: JsonValue, language_code: String) -> Self {
        let root_state = ModListRootState::new(registry, mod_state, language_code);
        let lua_state = ModListLuaState::from_root_state(&root_state);
        Self {
            root_state,
            lua_state,
        }
    }

    /// 进入模组列表时重置 transient Lua state。
    pub fn reset_lua_state(&mut self) {
        self.root_state.refresh_language();
        self.root_state.normalize_select();
        self.lua_state = ModListLuaState::from_root_state(&self.root_state);
    }

    /// 刷新语言。
    pub fn refresh_language(&mut self, language_code: String) {
        self.root_state.language_code = language_code;
        self.root_state.refresh_language();
        self.root_state.refresh_mod_display();
    }

    /// 应用 Lua 返回状态。
    pub fn apply_lua_state(&mut self, lua_state: ModListLuaState) -> ModListLuaAction {
        self.lua_state = lua_state;
        self.root_state.sort_mode = ModListSortMode::from_str(self.lua_state.sort.as_str());
        self.root_state.sort_order = ModListSortOrder::from_str(self.lua_state.order.as_str());
        self.root_state.pages = self.lua_state.pages.max(1);
        self.root_state.page = self.lua_state.page.clamp(1, self.root_state.pages);
        self.root_state.user_page = if self.lua_state.jump {
            self.lua_state.user_page
        } else {
            0
        };
        self.root_state.jump = self.lua_state.jump;
        self.root_state.info_scroll = self.lua_state.info_scroll.max(0);
        self.root_state.list_mode = ModListDisplayMode::from_str(self.lua_state.list_mode.as_str());
        if self.lua_state.select.is_empty() {
            self.root_state.normalize_select();
        } else {
            self.root_state.selected_uid = self.lua_state.select.clone();
        }
        self.root_state.sort_mods();
        self.root_state.normalize_select();

        if self.lua_state.back {
            self.lua_state.back = false;
            return ModListLuaAction::Back;
        }

        if self.lua_state.toggle_debug {
            self.lua_state.toggle_debug = false;
            return self.toggle_debug();
        }

        if self.lua_state.toggle_safe_mode {
            self.lua_state.toggle_safe_mode = false;
            return self.request_safe_mode_change();
        }

        if self.lua_state.confirm {
            self.lua_state.confirm = false;
            return self.toggle_enabled();
        }

        ModListLuaAction::None
    }

    fn toggle_enabled(&mut self) -> ModListLuaAction {
        let selected_uid = self.root_state.selected_uid.clone();
        if selected_uid.is_empty() {
            return ModListLuaAction::None;
        }
        if let Some(item) = self.root_state.item_mut(selected_uid.as_str()) {
            item.enabled = !item.enabled;
            self.root_state.write_item_state(selected_uid.as_str());
            let _ = persist_mod_state(&self.root_state.mod_state);
            return ModListLuaAction::StateChanged(self.root_state.mod_state.clone());
        }
        ModListLuaAction::None
    }

    fn toggle_debug(&mut self) -> ModListLuaAction {
        let selected_uid = self.root_state.selected_uid.clone();
        if selected_uid.is_empty() {
            return ModListLuaAction::None;
        }
        if let Some(item) = self.root_state.item_mut(selected_uid.as_str()) {
            item.debug = !item.debug;
            self.root_state.write_item_state(selected_uid.as_str());
            let _ = persist_mod_state(&self.root_state.mod_state);
            return ModListLuaAction::StateChanged(self.root_state.mod_state.clone());
        }
        ModListLuaAction::None
    }

    fn request_safe_mode_change(&mut self) -> ModListLuaAction {
        let selected_uid = self.root_state.selected_uid.clone();
        if selected_uid.is_empty() {
            return ModListLuaAction::None;
        }
        if let Some(item) = self.root_state.item_mut(selected_uid.as_str()) {
            if item.safe_mode {
                return ModListLuaAction::OpenSafeModeWarning(selected_uid);
            }
            item.safe_mode = true;
            item.safe_mode_permanent = false;
            self.root_state.write_item_state(selected_uid.as_str());
            let _ = persist_mod_state(&self.root_state.mod_state);
            return ModListLuaAction::StateChanged(self.root_state.mod_state.clone());
        }
        ModListLuaAction::None
    }

    /// 关闭指定模组包安全模式。
    ///
    /// `permanent = true` 会写入持久化文件；否则只更新本次运行内存状态。
    pub fn close_safe_mode(&mut self, uid: &str, permanent: bool) -> ModListLuaAction {
        if uid.is_empty() {
            return ModListLuaAction::None;
        }
        if let Some(item) = self.root_state.item_mut(uid) {
            item.safe_mode = false;
            item.safe_mode_permanent = permanent;
            self.root_state.write_item_state(uid);
            if permanent {
                let _ = persist_mod_state(&self.root_state.mod_state);
            }
            return ModListLuaAction::StateChanged(self.root_state.mod_state.clone());
        }
        ModListLuaAction::None
    }

    /// 重置所有模组包安全模式为开启。
    pub fn reset_all_safe_mode_on(&mut self) -> ModListLuaAction {
        let uids = self
            .root_state
            .mods
            .iter()
            .map(|item| item.uid.clone())
            .collect::<Vec<_>>();
        for uid in &uids {
            if let Some(item) = self.root_state.item_mut(uid.as_str()) {
                item.safe_mode = true;
                item.safe_mode_permanent = false;
            }
            self.root_state.write_item_state(uid.as_str());
        }
        let _ = persist_mod_state(&self.root_state.mod_state);
        ModListLuaAction::StateChanged(self.root_state.mod_state.clone())
    }

    /// 重置所有模组包启用状态为禁用。
    pub fn reset_all_enabled_off(&mut self) -> ModListLuaAction {
        let uids = self
            .root_state
            .mods
            .iter()
            .map(|item| item.uid.clone())
            .collect::<Vec<_>>();
        for uid in &uids {
            if let Some(item) = self.root_state.item_mut(uid.as_str()) {
                item.enabled = false;
            }
            self.root_state.write_item_state(uid.as_str());
        }
        let _ = persist_mod_state(&self.root_state.mod_state);
        ModListLuaAction::StateChanged(self.root_state.mod_state.clone())
    }

    /// 指定模组包显示名。
    pub fn package_name(&self, uid: &str) -> String {
        self.root_state
            .mods
            .iter()
            .find(|item| item.uid == uid)
            .map(|item| item.package_name.clone())
            .unwrap_or_else(|| uid.to_string())
    }

    /// 当前选中模组包 UID。
    pub fn selected_uid(&self) -> String {
        self.root_state.selected_uid.clone()
    }
}

/// 模组列表 Lua 返回动作。
#[derive(Clone, Debug, PartialEq)]
pub enum ModListLuaAction {
    None,
    Back,
    OpenSafeModeWarning(String),
    StateChanged(JsonValue),
}

/// 模组列表展示模式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModListDisplayMode {
    Full,
    Brief,
}

impl ModListDisplayMode {
    fn from_str(value: &str) -> Self {
        match value {
            "brief" => Self::Brief,
            _ => Self::Full,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Brief => "brief",
        }
    }
}

/// 模组列表 Lua state。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModListLuaState {
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
    pub list_mode: String,
    pub toggle_debug: bool,
    pub toggle_safe_mode: bool,
}

impl ModListLuaState {
    fn from_root_state(root_state: &ModListRootState) -> Self {
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
            list_mode: root_state.list_mode.as_str().to_string(),
            toggle_debug: false,
            toggle_safe_mode: false,
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
        table.set("list_mode", self.list_mode.as_str())?;
        table.set("toggle_debug", false)?;
        table.set("toggle_safe_mode", false)?;
        Ok(table)
    }

    /// 从 Lua 返回值解析。
    pub fn from_lua_value(value: Value) -> mlua::Result<Self> {
        let table = match value {
            Value::Table(table) => table,
            _ => {
                return Err(mlua::Error::external(
                    "mod list lua state must be returned as table",
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
                .unwrap_or_else(|| "name".to_string()),
            pages: table.get::<Option<i64>>("pages")?.unwrap_or(1).max(1),
            page: table.get::<Option<i64>>("page")?.unwrap_or(1).max(1),
            user_page: table.get::<Option<i64>>("user_page")?.unwrap_or(0).max(0),
            jump: table.get::<Option<bool>>("jump")?.unwrap_or(false),
            info_scroll: table.get::<Option<i64>>("info_scroll")?.unwrap_or(0).max(0),
            list_mode: table
                .get::<Option<String>>("list_mode")?
                .unwrap_or_else(|| "full".to_string()),
            toggle_debug: table.get::<Option<bool>>("toggle_debug")?.unwrap_or(false),
            toggle_safe_mode: table
                .get::<Option<bool>>("toggle_safe_mode")?
                .unwrap_or(false),
        })
    }
}

/// 模组列表 root_state。
#[derive(Clone, Debug)]
pub struct ModListRootState {
    pub language: Vec<(String, String)>,
    pub mods: Vec<ModListItem>,
    pub mod_state: JsonValue,
    pub language_code: String,
    pub selected_uid: String,
    pub sort_order: ModListSortOrder,
    pub sort_mode: ModListSortMode,
    pub pages: i64,
    pub page: i64,
    pub user_page: i64,
    pub jump: bool,
    pub info_scroll: i64,
    pub list_mode: ModListDisplayMode,
}

impl ModListRootState {
    fn new(registry: GameModuleRegistry, mod_state: JsonValue, language_code: String) -> Self {
        let mut mods = registry
            .games
            .iter()
            .filter(|game_module| game_module.source == GameModuleSource::Mod)
            .map(|game_module| {
                ModListItem::from_game_module(game_module, &mod_state, language_code.as_str())
            })
            .collect::<Vec<_>>();
        let selected_uid = mods
            .first()
            .map(|item| item.uid.clone())
            .unwrap_or_default();
        let mut root_state = Self {
            language: mod_list_language_pairs(),
            mods: Vec::new(),
            mod_state,
            language_code,
            selected_uid,
            sort_order: ModListSortOrder::Asc,
            sort_mode: ModListSortMode::Name,
            pages: 1,
            page: 1,
            user_page: 0,
            jump: false,
            info_scroll: 0,
            list_mode: ModListDisplayMode::Full,
        };
        root_state.mods.append(&mut mods);
        root_state.sort_mods();
        root_state.normalize_select();
        root_state
    }

    fn refresh_language(&mut self) {
        self.language = mod_list_language_pairs();
    }

    fn refresh_mod_display(&mut self) {
        for item in &mut self.mods {
            item.refresh_display(self.language_code.as_str());
        }
    }

    fn normalize_select(&mut self) {
        if self.mods.is_empty() {
            self.selected_uid.clear();
            return;
        }
        if !self.mods.iter().any(|item| item.uid == self.selected_uid) {
            self.selected_uid = self.mods[0].uid.clone();
        }
    }

    fn sort_mods(&mut self) {
        let sort_mode = self.sort_mode;
        self.mods
            .sort_by(|left, right| compare_mod(left, right, sort_mode));
        if self.sort_order == ModListSortOrder::Desc {
            self.mods.reverse();
        }
    }

    fn item_mut(&mut self, uid: &str) -> Option<&mut ModListItem> {
        self.mods.iter_mut().find(|item| item.uid == uid)
    }

    fn write_item_state(&mut self, uid: &str) {
        let Some(item) = self.mods.iter().find(|item| item.uid == uid) else {
            return;
        };
        if !self.mod_state.is_object() {
            self.mod_state = JsonValue::Object(Map::new());
        }
        let Some(root) = self.mod_state.as_object_mut() else {
            return;
        };
        root.insert(
            uid.to_string(),
            json!({
                "package": item.package,
                "enabled": item.enabled,
                "debug": item.debug,
                "safe_mode": item.safe_mode,
                "safe_mode_permanent": item.safe_mode_permanent
            }),
        );
    }

    /// 转为 Lua root_state 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("language", pairs_to_table(lua, &self.language)?)?;
        table.set("mod_list", mod_list_to_table(lua, &self.mods)?)?;
        table.set("mod_info", self.selected_mod_info(lua)?)?;
        table.set("order", self.sort_order.as_str())?;
        table.set("sort", self.sort_mode.as_str())?;
        table.set("select", self.selected_uid.as_str())?;
        table.set("pages", self.pages.max(1))?;
        table.set("page", self.page.max(1))?;
        table.set("user_page", if self.jump { self.user_page } else { 0 })?;
        table.set("jump", self.jump)?;
        table.set("info_scroll", self.info_scroll.max(0))?;
        table.set("list_mode", self.list_mode.as_str())?;
        Ok(table)
    }

    fn selected_mod_info(&self, lua: &Lua) -> mlua::Result<Table> {
        match self
            .mods
            .iter()
            .find(|item| item.uid == self.selected_uid)
            .or_else(|| self.mods.first())
        {
            Some(item) => item.to_lua_table(lua),
            None => lua.create_table(),
        }
    }
}

/// 模组列表项。
#[derive(Clone, Debug)]
pub struct ModListItem {
    pub uid: String,
    pub package: String,
    pub root_dir: PathBuf,
    pub package_name_raw: String,
    pub introduction_raw: String,
    pub author_raw: String,
    pub version: String,
    pub package_name: String,
    pub introduction: String,
    pub author: String,
    pub enabled: bool,
    pub debug: bool,
    pub safe_mode: bool,
    pub safe_mode_permanent: bool,
    pub write: bool,
    pub icon: Vec<String>,
    pub banner: Vec<String>,
}

impl ModListItem {
    fn from_game_module(
        game_module: &GameModule,
        mod_state: &JsonValue,
        language_code: &str,
    ) -> Self {
        let state = mod_state.get(game_module.uid.as_str());
        let mut item = Self {
            uid: game_module.uid.clone(),
            package: game_module.package.package.clone(),
            root_dir: game_module.root_dir.clone(),
            package_name_raw: game_module.package.package_name.clone(),
            introduction_raw: game_module.package.introduction.clone(),
            author_raw: game_module.package.author.clone(),
            version: game_module.package.version.clone(),
            package_name: String::new(),
            introduction: String::new(),
            author: String::new(),
            enabled: state
                .and_then(|value| value.get("enabled"))
                .and_then(JsonValue::as_bool)
                .unwrap_or(true),
            debug: state
                .and_then(|value| value.get("debug"))
                .and_then(JsonValue::as_bool)
                .unwrap_or(false),
            safe_mode: state
                .and_then(|value| value.get("safe_mode"))
                .and_then(JsonValue::as_bool)
                .unwrap_or(true),
            safe_mode_permanent: state
                .and_then(|value| value.get("safe_mode_permanent"))
                .and_then(JsonValue::as_bool)
                .unwrap_or_else(|| {
                    state
                        .and_then(|value| value.get("safe_mode"))
                        .and_then(JsonValue::as_bool)
                        .map(|safe_mode| !safe_mode)
                        .unwrap_or(false)
                }),
            write: game_module.game.write,
            icon: image_lines(game_module.uid.as_str(), "icon", &game_module.package.icon),
            banner: image_lines(
                game_module.uid.as_str(),
                "banner",
                &game_module.package.banner,
            ),
        };
        item.refresh_display(language_code);
        item
    }

    fn refresh_display(&mut self, language_code: &str) {
        let language_texts = load_package_language_texts(&self.root_dir, language_code);
        self.package_name = resolve_package_text(&language_texts, &self.package_name_raw);
        self.introduction = resolve_package_text(&language_texts, &self.introduction_raw);
        self.author = resolve_package_text(&language_texts, &self.author_raw);
    }

    fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("uid", self.uid.as_str())?;
        table.set("package", self.package.as_str())?;
        table.set("package_name", self.package_name.as_str())?;
        table.set("name", self.package_name.as_str())?;
        table.set("introduction", self.introduction.as_str())?;
        table.set("author", self.author.as_str())?;
        table.set("version", self.version.as_str())?;
        table.set("enabled", self.enabled)?;
        table.set("debug", self.debug)?;
        table.set("safe_mode", self.safe_mode)?;
        table.set("safe_mode_permanent", self.safe_mode_permanent)?;
        table.set("write", self.write)?;
        table.set("icon", string_vec_to_table(lua, &self.icon)?)?;
        table.set("banner", string_vec_to_table(lua, &self.banner)?)?;
        Ok(table)
    }
}

fn compare_mod(left: &ModListItem, right: &ModListItem, sort_mode: ModListSortMode) -> Ordering {
    match sort_mode {
        ModListSortMode::Name => {
            compare_text_by_width_then_dictionary(left.package_name.as_str(), right.package_name.as_str())
        }
        ModListSortMode::Author => {
            compare_text_by_width_then_dictionary(left.author.as_str(), right.author.as_str())
        }
        ModListSortMode::SafeMode => left.safe_mode.cmp(&right.safe_mode),
        ModListSortMode::Toggle => left.enabled.cmp(&right.enabled),
    }
    .then_with(|| {
        compare_text_by_width_then_dictionary(left.package_name.as_str(), right.package_name.as_str())
    })
    .then_with(|| {
        compare_text_by_width_then_dictionary(left.author.as_str(), right.author.as_str())
    })
    .then_with(|| {
        compare_text_by_width_then_dictionary(left.package.as_str(), right.package.as_str())
    })
}

fn compare_text_by_width_then_dictionary(left: &str, right: &str) -> Ordering {
    let left_text = left.to_lowercase();
    let right_text = right.to_lowercase();
    let left_width = UnicodeWidthStr::width(left_text.as_str());
    let right_width = UnicodeWidthStr::width(right_text.as_str());

    left_width
        .cmp(&right_width)
        .then_with(|| left_text.cmp(&right_text))
}

fn mod_list_language_pairs() -> Vec<(String, String)> {
    let text = i18n::text();
    vec![
        (
            "MOD_LIST_PREV_OPTION".to_string(),
            text.key.mod_list_prev_option,
        ),
        (
            "MOD_LIST_NEXT_OPTION".to_string(),
            text.key.mod_list_next_option,
        ),
        (
            "MOD_LIST_PREV_PAGE".to_string(),
            text.key.mod_list_prev_page,
        ),
        (
            "MOD_LIST_NEXT_PAGE".to_string(),
            text.key.mod_list_next_page,
        ),
        (
            "MOD_LIST_SCROLL_UP".to_string(),
            text.key.mod_list_scroll_up,
        ),
        (
            "MOD_LIST_SCROLL_DOWN".to_string(),
            text.key.mod_list_scroll_down,
        ),
        ("MOD_LIST_JUMP".to_string(), text.key.mod_list_jump),
        ("MOD_LIST_ORDER".to_string(), text.key.mod_list_order),
        ("MOD_LIST_SORT".to_string(), text.key.mod_list_sort),
        ("MOD_LIST_BACK".to_string(), text.key.mod_list_back),
        (
            "MOD_LIST_TOGGLE_CONFIRM".to_string(),
            text.key.mod_list_toggle_confirm,
        ),
        (
            "MOD_LIST_BACK_CANCEL".to_string(),
            text.key.mod_list_back_cancel,
        ),
        ("MOD_LIST_TOGGLE".to_string(), text.key.mod_list_toggle),
        ("MOD_LIST_CONFIRM".to_string(), text.key.mod_list_confirm),
        ("MOD_LIST_CANCEL".to_string(), text.key.mod_list_cancel),
        ("MOD_LIST_SELECT".to_string(), text.key.mod_list_select),
        ("MOD_LIST_FLIP".to_string(), text.key.mod_list_flip),
        ("MOD_LIST_SCROLL".to_string(), text.key.mod_list_scroll),
        ("MOD_LIST_DEBUG".to_string(), text.key.mod_list_debug),
        ("MOD_LIST_LIST".to_string(), text.key.mod_list_list),
        (
            "MOD_LIST_SAFE_MODE".to_string(),
            text.key.mod_list_safe_mode,
        ),
        ("MOD_LIST_LIST_TITLE".to_string(), text.mod_list.list_title),
        (
            "MOD_LIST_INFO_SORT_NAME".to_string(),
            text.mod_list.info_sort_name,
        ),
        (
            "MOD_LIST_INFO_SORT_AUTHOR".to_string(),
            text.mod_list.info_sort_author,
        ),
        (
            "MOD_LIST_INFO_SORT_SAFE_MODE".to_string(),
            text.mod_list.info_sort_safe_mode,
        ),
        (
            "MOD_LIST_INFO_SORT_TOGGLE".to_string(),
            text.mod_list.info_sort_toggle,
        ),
        (
            "MOD_LIST_INFO_ORDER_ASCENDING".to_string(),
            text.mod_list.info_order_ascending,
        ),
        (
            "MOD_LIST_INFO_ORDER_DESCENDING".to_string(),
            text.mod_list.info_order_descending,
        ),
        (
            "MOD_LIST_INFO_AUTHOR".to_string(),
            text.mod_list.info_author,
        ),
        (
            "MOD_LIST_INFO_VERSION".to_string(),
            text.mod_list.info_version,
        ),
        ("MOD_LIST_INFO_BASE".to_string(), text.mod_list.info_base),
        ("MOD_LIST_INFO_SAFE".to_string(), text.mod_list.info_safe),
        (
            "MOD_LIST_INFO_SAFE_SWITCH".to_string(),
            text.mod_list.info_safe_switch,
        ),
        (
            "MOD_LIST_INFO_SAFE_DEBUG".to_string(),
            text.mod_list.info_safe_debug,
        ),
        (
            "MOD_LIST_INFO_SAFE_WRITE".to_string(),
            text.mod_list.info_safe_write,
        ),
        (
            "MOD_LIST_INFO_SAFE_SAFE_MODE".to_string(),
            text.mod_list.info_safe_safe_mode,
        ),
        (
            "MOD_LIST_INFO_INTRODUCTION".to_string(),
            text.mod_list.info_introduction,
        ),
        ("MOD_LIST_INFO_TITLE".to_string(), text.mod_list.info_title),
        ("MOD_LIST_STATUS".to_string(), text.mod_list.status),
        ("MOD_LIST_NONE_MOD".to_string(), text.mod_list.none_mod),
        ("MOD_LIST_NONE_INFO".to_string(), text.mod_list.none_info),
        (
            "MOD_LIST_TOGGLE_MOD_ON".to_string(),
            text.mod_list.toggle_mod_on,
        ),
        (
            "MOD_LIST_TOGGLE_MOD_OFF".to_string(),
            text.mod_list.toggle_mod_off,
        ),
        (
            "MOD_LIST_TOGGLE_MOD_ON_BRIEF".to_string(),
            text.mod_list.toggle_mod_on_brief,
        ),
        (
            "MOD_LIST_TOGGLE_MOD_OFF_BRIEF".to_string(),
            text.mod_list.toggle_mod_off_brief,
        ),
        (
            "MOD_LIST_TOGGLE_WRITE_ON".to_string(),
            text.mod_list.toggle_write_on,
        ),
        (
            "MOD_LIST_TOGGLE_WRITE_OFF".to_string(),
            text.mod_list.toggle_write_off,
        ),
        (
            "MOD_LIST_TOGGLE_DEBUG_ON".to_string(),
            text.mod_list.toggle_debug_on,
        ),
        (
            "MOD_LIST_TOGGLE_DEBUG_OFF".to_string(),
            text.mod_list.toggle_debug_off,
        ),
        (
            "MOD_LIST_TOGGLE_SAFE_MODE_ON".to_string(),
            text.mod_list.toggle_safe_mode_on,
        ),
        (
            "MOD_LIST_TOGGLE_SAFE_MODE_OFF_TEMPORARY".to_string(),
            text.mod_list.toggle_safe_mode_off_temporary,
        ),
        (
            "MOD_LIST_TOGGLE_SAFE_MODE_OFF_PERMANENT".to_string(),
            text.mod_list.toggle_safe_mode_off_permanent,
        ),
    ]
}

fn pairs_to_table(lua: &Lua, pairs: &[(String, String)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (key, value) in pairs {
        table.set(key.as_str(), value.as_str())?;
    }
    Ok(table)
}

fn mod_list_to_table(lua: &Lua, mods: &[ModListItem]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, item) in mods.iter().enumerate() {
        table.set(index + 1, item.to_lua_table(lua)?)?;
    }
    Ok(table)
}

fn string_vec_to_table(lua: &Lua, values: &[String]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, value) in values.iter().enumerate() {
        table.set(index + 1, value.as_str())?;
    }
    Ok(table)
}

fn image_lines(uid: &str, slot: &str, raw_value: &JsonValue) -> Vec<String> {
    let cache_path = root_dir()
        .join("data/cache/images")
        .join(format!("{uid}.{slot}.json"));
    if let Some(lines) = fs::read_to_string(cache_path)
        .ok()
        .and_then(|raw_json| serde_json::from_str::<JsonValue>(&raw_json).ok())
        .and_then(|value| value.get("lines").cloned())
        .and_then(|value| value.as_array().cloned())
        .map(|values| {
            values
                .iter()
                .filter_map(JsonValue::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|lines| !lines.is_empty())
    {
        return lines;
    }

    if is_image_reference(raw_value) {
        return Vec::new();
    }

    value_to_lines(raw_value)
}

fn is_image_reference(value: &JsonValue) -> bool {
    value
        .as_str()
        .map(|text| {
            let text = text.trim();
            text.starts_with("image:") || text.starts_with("color:image:")
        })
        .unwrap_or(false)
}

fn value_to_lines(value: &JsonValue) -> Vec<String> {
    match value {
        JsonValue::Array(values) => values
            .iter()
            .filter_map(JsonValue::as_str)
            .map(ToString::to_string)
            .collect(),
        JsonValue::String(text) => text.lines().map(ToString::to_string).collect(),
        _ => Vec::new(),
    }
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

fn persist_mod_state(mod_state: &JsonValue) -> std::io::Result<()> {
    let path = root_dir().join("data/profiles/mod_state.json");
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(path, serde_json::to_string_pretty(mod_state)?)
}

fn root_dir() -> PathBuf {
    std::env::current_dir()
        .ok()
        .filter(|path| path.join("assets").exists() || path.join("Cargo.toml").exists())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(PathBuf::from))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}
