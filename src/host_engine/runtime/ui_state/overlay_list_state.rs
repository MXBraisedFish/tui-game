//! UI Saver/Boss 列表状态聚合

use crate::host_engine::boot::environment::data_dirs;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use mlua::{Lua, Table, Value};
use serde_json::{Map, Value as JsonValue, json};
use unicode_width::UnicodeWidthStr;

use crate::host_engine::boot::preload::overlay_modules::{
    OverlayPackage, OverlayRegistry, OverlaySource,
};

const DEFAULT_LANGUAGE_CODE: &str = "en_us";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayListKind {
    Saver,
    Boss,
}

impl OverlayListKind {
    fn state_path(self) -> PathBuf {
        data_dirs::root_dir().join(match self {
            Self::Saver => "data/profiles/saver_state",
            Self::Boss => "data/profiles/boss_state",
        })
    }
    fn key_prefix(self) -> &'static str {
        match self {
            Self::Saver => "mod_saver_list",
            Self::Boss => "mod_boss_list",
        }
    }
    fn packages<'a>(self, registry: &'a OverlayRegistry) -> &'a [OverlayPackage] {
        match self {
            Self::Saver => &registry.savers,
            Self::Boss => &registry.bosses,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayListSortMode {
    Name,
    Author,
    Toggle,
    Debug,
}
impl OverlayListSortMode {
    fn from_str(value: &str) -> Self {
        match value {
            "author" => Self::Author,
            "toggle" => Self::Toggle,
            "debug" => Self::Debug,
            _ => Self::Name,
        }
    }
    fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Author => "author",
            Self::Toggle => "toggle",
            Self::Debug => "debug",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayListSortOrder {
    Asc,
    Desc,
}
impl OverlayListSortOrder {
    fn from_str(value: &str) -> Self {
        if value == "desc" {
            Self::Desc
        } else {
            Self::Asc
        }
    }
    fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayListDisplayMode {
    Full,
    Brief,
}
impl OverlayListDisplayMode {
    fn from_str(value: &str) -> Self {
        if value == "brief" {
            Self::Brief
        } else {
            Self::Full
        }
    }
    fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Brief => "brief",
        }
    }
}

#[derive(Clone, Debug)]
pub struct OverlayListUiState {
    pub root_state: OverlayListRootState,
    pub lua_state: OverlayListLuaState,
}
impl OverlayListUiState {
    pub fn new(
        kind: OverlayListKind,
        registry: OverlayRegistry,
        state: JsonValue,
        language_code: String,
    ) -> Self {
        let root_state = OverlayListRootState::new(kind, registry, state, language_code);
        let lua_state = OverlayListLuaState::from_root_state(&root_state);
        Self {
            root_state,
            lua_state,
        }
    }
    pub fn reset_lua_state(&mut self) {
        self.root_state.refresh_language();
        self.root_state.normalize_select();
        self.lua_state = OverlayListLuaState::from_root_state(&self.root_state);
    }
    pub fn refresh_language(&mut self, language_code: String) {
        self.root_state.language_code = language_code;
        self.root_state.refresh_language();
        self.root_state.refresh_display();
    }
    pub fn replace_registry_and_state(
        &mut self,
        registry: OverlayRegistry,
        state: JsonValue,
        language_code: String,
    ) {
        let selected = self.root_state.selected_uid.clone();
        let sort_order = self.root_state.sort_order;
        let sort_mode = self.root_state.sort_mode;
        let page = self.root_state.page;
        let list_mode = self.root_state.list_mode;
        self.root_state =
            OverlayListRootState::new(self.root_state.kind, registry, state, language_code);
        self.root_state.selected_uid = selected;
        self.root_state.sort_order = sort_order;
        self.root_state.sort_mode = sort_mode;
        self.root_state.page = page.max(1);
        self.root_state.list_mode = list_mode;
        self.root_state.sort_items();
        self.root_state.normalize_select();
        self.reset_lua_state();
    }
    pub fn apply_lua_state(&mut self, lua_state: OverlayListLuaState) -> OverlayListLuaAction {
        self.lua_state = lua_state;
        self.root_state.sort_mode = OverlayListSortMode::from_str(&self.lua_state.sort);
        self.root_state.sort_order = OverlayListSortOrder::from_str(&self.lua_state.order);
        self.root_state.pages = self.lua_state.pages.max(1);
        self.root_state.page = self.lua_state.page.clamp(1, self.root_state.pages);
        self.root_state.user_page = if self.lua_state.jump {
            self.lua_state.user_page
        } else {
            0
        };
        self.root_state.jump = self.lua_state.jump;
        self.root_state.info_scroll = self.lua_state.info_scroll.max(0);
        self.root_state.list_mode = OverlayListDisplayMode::from_str(&self.lua_state.list_mode);
        if self.lua_state.select.is_empty() {
            self.root_state.normalize_select();
        } else {
            self.root_state.selected_uid = self.lua_state.select.clone();
        }
        self.root_state.sort_items();
        self.root_state.normalize_select();
        if self.lua_state.back {
            self.lua_state.back = false;
            return OverlayListLuaAction::Back;
        }
        if self.lua_state.toggle_debug {
            self.lua_state.toggle_debug = false;
            return self.toggle_debug();
        }
        if self.lua_state.confirm {
            self.lua_state.confirm = false;
            return self.toggle_enabled();
        }
        OverlayListLuaAction::None
    }
    fn toggle_enabled(&mut self) -> OverlayListLuaAction {
        let uid = self.root_state.selected_uid.clone();
        if uid.is_empty() {
            return OverlayListLuaAction::None;
        }
        if let Some(item) = self.root_state.item_mut(&uid) {
            item.enabled = !item.enabled;
            self.root_state.write_item_state(&uid);
            let _ = persist_state(self.root_state.kind, &self.root_state.state);
            return OverlayListLuaAction::StateChanged(
                self.root_state.kind,
                self.root_state.state.clone(),
            );
        }
        OverlayListLuaAction::None
    }
    fn toggle_debug(&mut self) -> OverlayListLuaAction {
        let uid = self.root_state.selected_uid.clone();
        if uid.is_empty() {
            return OverlayListLuaAction::None;
        }
        if let Some(item) = self.root_state.item_mut(&uid) {
            item.debug = !item.debug;
            self.root_state.write_item_state(&uid);
            let _ = persist_state(self.root_state.kind, &self.root_state.state);
            return OverlayListLuaAction::StateChanged(
                self.root_state.kind,
                self.root_state.state.clone(),
            );
        }
        OverlayListLuaAction::None
    }
    pub fn reset_all_enabled_off(&mut self) -> OverlayListLuaAction {
        let uids = self
            .root_state
            .items
            .iter()
            .map(|i| i.uid.clone())
            .collect::<Vec<_>>();
        for uid in uids {
            if let Some(item) = self.root_state.item_mut(&uid) {
                item.enabled = false;
            }
            self.root_state.write_item_state(&uid);
        }
        let _ = persist_state(self.root_state.kind, &self.root_state.state);
        OverlayListLuaAction::StateChanged(self.root_state.kind, self.root_state.state.clone())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum OverlayListLuaAction {
    None,
    Back,
    StateChanged(OverlayListKind, JsonValue),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverlayListLuaState {
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
}
impl OverlayListLuaState {
    fn from_root_state(root: &OverlayListRootState) -> Self {
        Self {
            select: root.selected_uid.clone(),
            confirm: false,
            back: false,
            order: root.sort_order.as_str().to_string(),
            sort: root.sort_mode.as_str().to_string(),
            pages: root.pages.max(1),
            page: root.page.max(1),
            user_page: if root.jump { root.user_page } else { 0 },
            jump: root.jump,
            info_scroll: root.info_scroll.max(0),
            list_mode: root.list_mode.as_str().to_string(),
            toggle_debug: false,
        }
    }
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let t = lua.create_table()?;
        t.set("select", self.select.as_str())?;
        t.set("confirm", false)?;
        t.set("back", false)?;
        t.set("order", self.order.as_str())?;
        t.set("sort", self.sort.as_str())?;
        t.set("pages", self.pages.max(1))?;
        t.set("page", self.page.max(1))?;
        t.set("user_page", if self.jump { self.user_page } else { 0 })?;
        t.set("jump", self.jump)?;
        t.set("info_scroll", self.info_scroll.max(0))?;
        t.set("list_mode", self.list_mode.as_str())?;
        t.set("toggle_debug", false)?;
        Ok(t)
    }
    pub fn from_lua_value(value: Value) -> mlua::Result<Self> {
        let table = match value {
            Value::Table(t) => t,
            _ => {
                return Err(mlua::Error::external(
                    "overlay list lua state must be returned as table",
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
        })
    }
}

#[derive(Clone, Debug)]
pub struct OverlayListRootState {
    pub kind: OverlayListKind,
    pub language: Vec<(String, String)>,
    pub items: Vec<OverlayListItem>,
    pub state: JsonValue,
    pub language_code: String,
    pub selected_uid: String,
    pub sort_order: OverlayListSortOrder,
    pub sort_mode: OverlayListSortMode,
    pub pages: i64,
    pub page: i64,
    pub user_page: i64,
    pub jump: bool,
    pub info_scroll: i64,
    pub list_mode: OverlayListDisplayMode,
}
impl OverlayListRootState {
    fn new(
        kind: OverlayListKind,
        registry: OverlayRegistry,
        state: JsonValue,
        language_code: String,
    ) -> Self {
        let mut items = kind
            .packages(&registry)
            .iter()
            .filter(|p| p.source == OverlaySource::ThirdParty)
            .map(|p| OverlayListItem::from_package(p, &state, &language_code))
            .collect::<Vec<_>>();
        let selected_uid = items.first().map(|i| i.uid.clone()).unwrap_or_default();
        let mut root = Self {
            kind,
            language: overlay_language_pairs(kind),
            items: Vec::new(),
            state,
            language_code,
            selected_uid,
            sort_order: OverlayListSortOrder::Asc,
            sort_mode: OverlayListSortMode::Name,
            pages: 1,
            page: 1,
            user_page: 0,
            jump: false,
            info_scroll: 0,
            list_mode: OverlayListDisplayMode::Full,
        };
        root.items.append(&mut items);
        root.sort_items();
        root.normalize_select();
        root
    }
    fn refresh_language(&mut self) {
        self.language = overlay_language_pairs(self.kind);
    }
    fn refresh_display(&mut self) {
        for item in &mut self.items {
            item.refresh_display(&self.language_code);
        }
    }
    fn normalize_select(&mut self) {
        if self.items.is_empty() {
            self.selected_uid.clear();
            return;
        }
        if !self.items.iter().any(|i| i.uid == self.selected_uid) {
            self.selected_uid = self.items[0].uid.clone();
        }
    }
    fn sort_items(&mut self) {
        let mode = self.sort_mode;
        self.items.sort_by(|l, r| compare_item(l, r, mode));
        if self.sort_order == OverlayListSortOrder::Desc {
            self.items.reverse();
        }
    }
    fn item_mut(&mut self, uid: &str) -> Option<&mut OverlayListItem> {
        self.items.iter_mut().find(|i| i.uid == uid)
    }
    fn write_item_state(&mut self, uid: &str) {
        let Some(item) = self.items.iter().find(|i| i.uid == uid) else {
            return;
        };
        if !self.state.is_object() {
            self.state = JsonValue::Object(Map::new());
        }
        let Some(root) = self.state.as_object_mut() else {
            return;
        };
        root.insert(
            uid.to_string(),
            json!({"package":item.package,"enabled":item.enabled,"debug":item.debug}),
        );
    }
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let t = lua.create_table()?;
        t.set("language", pairs_to_table(lua, &self.language)?)?;
        t.set("mod_list", items_to_table(lua, &self.items)?)?;
        t.set("mod_info", self.selected_info(lua)?)?;
        t.set("order", self.sort_order.as_str())?;
        t.set("sort", self.sort_mode.as_str())?;
        t.set("select", self.selected_uid.as_str())?;
        t.set("pages", self.pages.max(1))?;
        t.set("page", self.page.max(1))?;
        t.set("user_page", if self.jump { self.user_page } else { 0 })?;
        t.set("jump", self.jump)?;
        t.set("info_scroll", self.info_scroll.max(0))?;
        t.set("list_mode", self.list_mode.as_str())?;
        Ok(t)
    }
    fn selected_info(&self, lua: &Lua) -> mlua::Result<Table> {
        match self
            .items
            .iter()
            .find(|i| i.uid == self.selected_uid)
            .or_else(|| self.items.first())
        {
            Some(i) => i.to_lua_table(lua),
            None => lua.create_table(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct OverlayListItem {
    pub uid: String,
    pub package: String,
    pub root_dir: PathBuf,
    pub package_name_raw: String,
    pub display_name_raw: String,
    pub introduction_raw: String,
    pub author_raw: String,
    pub version: String,
    pub package_name: String,
    pub display_name: String,
    pub introduction: String,
    pub author: String,
    pub enabled: bool,
    pub debug: bool,
    pub icon: Vec<String>,
    pub banner: Vec<String>,
}
impl OverlayListItem {
    fn from_package(package: &OverlayPackage, state: &JsonValue, language_code: &str) -> Self {
        let entry = state.get(package.uid.as_str());
        let mut item = Self {
            uid: package.uid.clone(),
            package: package.manifest.package.clone(),
            root_dir: package.root_dir.clone(),
            package_name_raw: package.manifest.package_name.clone(),
            display_name_raw: package.manifest.display_name.clone(),
            introduction_raw: package.manifest.introduction.clone(),
            author_raw: package.manifest.author.clone(),
            version: package.manifest.version.clone(),
            package_name: String::new(),
            display_name: String::new(),
            introduction: String::new(),
            author: String::new(),
            enabled: entry
                .and_then(|v| v.get("enabled"))
                .and_then(JsonValue::as_bool)
                .unwrap_or(true),
            debug: entry
                .and_then(|v| v.get("debug"))
                .and_then(JsonValue::as_bool)
                .unwrap_or(false),
            icon: image_lines(&package.uid, "icon", &package.manifest.icon),
            banner: image_lines(&package.uid, "banner", &package.manifest.banner),
        };
        item.refresh_display(language_code);
        item
    }
    fn refresh_display(&mut self, language_code: &str) {
        let texts = load_package_language_texts(&self.root_dir, language_code);
        self.package_name = resolve_package_text(&texts, &self.package_name_raw);
        self.display_name = resolve_package_text(&texts, &self.display_name_raw);
        self.introduction = resolve_package_text(&texts, &self.introduction_raw);
        self.author = resolve_package_text(&texts, &self.author_raw);
    }
    fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let t = lua.create_table()?;
        t.set("uid", self.uid.as_str())?;
        t.set("package", self.package.as_str())?;
        t.set("package_name", self.display_name.as_str())?;
        t.set("name", self.display_name.as_str())?;
        t.set("pack_name", self.package_name.as_str())?;
        t.set("introduction", self.introduction.as_str())?;
        t.set("author", self.author.as_str())?;
        t.set("version", self.version.as_str())?;
        t.set("enabled", self.enabled)?;
        t.set("debug", self.debug)?;
        t.set("icon", string_vec_to_table(lua, &self.icon)?)?;
        t.set("banner", string_vec_to_table(lua, &self.banner)?)?;
        Ok(t)
    }
}

fn compare_item(
    left: &OverlayListItem,
    right: &OverlayListItem,
    mode: OverlayListSortMode,
) -> Ordering {
    match mode {
        OverlayListSortMode::Name => {
            compare_text(left.display_name.as_str(), right.display_name.as_str())
        }
        OverlayListSortMode::Author => compare_text(left.author.as_str(), right.author.as_str()),
        OverlayListSortMode::Toggle => left.enabled.cmp(&right.enabled),
        OverlayListSortMode::Debug => left.debug.cmp(&right.debug),
    }
    .then_with(|| compare_text(left.display_name.as_str(), right.display_name.as_str()))
    .then_with(|| compare_text(left.author.as_str(), right.author.as_str()))
    .then_with(|| compare_text(left.package.as_str(), right.package.as_str()))
}
fn compare_text(left: &str, right: &str) -> Ordering {
    let l = left.to_lowercase();
    let r = right.to_lowercase();
    UnicodeWidthStr::width(l.as_str())
        .cmp(&UnicodeWidthStr::width(r.as_str()))
        .then_with(|| l.cmp(&r))
}

fn overlay_language_pairs(kind: OverlayListKind) -> Vec<(String, String)> {
    let prefix = kind.key_prefix();
    let get = |key: &str| resolve_current_i18n(&format!("{prefix}.{key}"));
    let get_key = |key: &str| resolve_current_i18n(&format!("key.{prefix}.{key}"));
    vec![
        ("MOD_LIST_PREV_OPTION".into(), get_key("prev_option")),
        ("MOD_LIST_NEXT_OPTION".into(), get_key("next_option")),
        ("MOD_LIST_PREV_PAGE".into(), get_key("prev_page")),
        ("MOD_LIST_NEXT_PAGE".into(), get_key("next_page")),
        ("MOD_LIST_SCROLL_UP".into(), get_key("scroll_up")),
        ("MOD_LIST_SCROLL_DOWN".into(), get_key("scroll_down")),
        ("MOD_LIST_JUMP".into(), get_key("jump")),
        ("MOD_LIST_ORDER".into(), get_key("order")),
        ("MOD_LIST_SORT".into(), get_key("sort")),
        ("MOD_LIST_BACK".into(), get_key("back")),
        ("MOD_LIST_TOGGLE_CONFIRM".into(), get_key("toggle_confirm")),
        ("MOD_LIST_BACK_CANCEL".into(), get_key("back_cancel")),
        ("MOD_LIST_TOGGLE".into(), get_key("toggle")),
        ("MOD_LIST_CONFIRM".into(), get_key("confirm")),
        ("MOD_LIST_CANCEL".into(), get_key("cancel")),
        ("MOD_LIST_SELECT".into(), get_key("select")),
        ("MOD_LIST_FLIP".into(), get_key("flip")),
        ("MOD_LIST_SCROLL".into(), get_key("scroll")),
        ("MOD_LIST_DEBUG".into(), get_key("debug")),
        ("MOD_LIST_LIST".into(), get_key("list")),
        ("MOD_LIST_LIST_TITLE".into(), get("list.title")),
        ("MOD_LIST_INFO_SORT_NAME".into(), get("info.sort.name")),
        ("MOD_LIST_INFO_SORT_AUTHOR".into(), get("info.sort.author")),
        ("MOD_LIST_INFO_SORT_TOGGLE".into(), get("info.sort.toggle")),
        ("MOD_LIST_INFO_SORT_DEBUG".into(), get("info.sort.debug")),
        (
            "MOD_LIST_INFO_ORDER_ASCENDING".into(),
            get("info.order.ascending"),
        ),
        (
            "MOD_LIST_INFO_ORDER_DESCENDING".into(),
            get("info.order.descending"),
        ),
        ("MOD_LIST_INFO_AUTHOR".into(), get("info.author")),
        ("MOD_LIST_INFO_VERSION".into(), get("info.version")),
        ("MOD_LIST_INFO_BASE".into(), get("info.base")),
        ("MOD_LIST_INFO_SAFE".into(), get("info.safe")),
        ("MOD_LIST_INFO_SAFE_SWITCH".into(), get("info.safe.switch")),
        ("MOD_LIST_INFO_SAFE_DEBUG".into(), get("info.safe.debug")),
        (
            "MOD_LIST_INFO_INTRODUCTION".into(),
            get("info.introduction"),
        ),
        ("MOD_LIST_INFO_TITLE".into(), get("info.title")),
        ("MOD_LIST_STATUS".into(), get("status")),
        ("MOD_LIST_NONE_MOD".into(), get("none.mod")),
        ("MOD_LIST_NONE_INFO".into(), get("none.info")),
        ("MOD_LIST_TOGGLE_MOD_ON".into(), get("toggle.mod.on")),
        ("MOD_LIST_TOGGLE_MOD_OFF".into(), get("toggle.mod.off")),
        (
            "MOD_LIST_TOGGLE_MOD_ON_BRIEF".into(),
            get("toggle.mod.on.brief"),
        ),
        (
            "MOD_LIST_TOGGLE_MOD_OFF_BRIEF".into(),
            get("toggle.mod.off.brief"),
        ),
        ("MOD_LIST_TOGGLE_DEBUG_ON".into(), get("toggle.debug.on")),
        ("MOD_LIST_TOGGLE_DEBUG_OFF".into(), get("toggle.debug.off")),
    ]
}

fn resolve_current_i18n(key: &str) -> String {
    let lang = read_language_code();
    let mut texts = read_host_language(&lang);
    if lang != DEFAULT_LANGUAGE_CODE {
        let fallback = read_host_language(DEFAULT_LANGUAGE_CODE);
        for (k, v) in fallback {
            texts.entry(k).or_insert(v);
        }
    }
    texts
        .get(key)
        .filter(|v| !v.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| format!("[Missing i18n key: {key}]"))
}
fn read_language_code() -> String {
    fs::read_to_string(data_dirs::root_dir().join("data/profiles/language.txt"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_LANGUAGE_CODE.to_string())
}
fn read_host_language(code: &str) -> HashMap<String, String> {
    fs::read_to_string(data_dirs::root_dir().join("assets/lang").join(format!("{code}.json")))
        .ok()
        .and_then(|s| {
            serde_json::from_str::<HashMap<String, String>>(s.trim_start_matches('\u{feff}')).ok()
        })
        .unwrap_or_default()
}
fn pairs_to_table(lua: &Lua, pairs: &[(String, String)]) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    for (k, v) in pairs {
        t.set(k.as_str(), v.as_str())?;
    }
    Ok(t)
}
fn items_to_table(lua: &Lua, items: &[OverlayListItem]) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    for (i, item) in items.iter().enumerate() {
        t.set(i + 1, item.to_lua_table(lua)?)?;
    }
    Ok(t)
}
fn string_vec_to_table(lua: &Lua, values: &[String]) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    for (i, v) in values.iter().enumerate() {
        t.set(i + 1, v.as_str())?;
    }
    Ok(t)
}
fn image_lines(uid: &str, slot: &str, raw: &JsonValue) -> Vec<String> {
    let cache = data_dirs::root_dir()
        .join("data/cache/images")
        .join(format!("{uid}.{slot}.json"));
    if let Some(lines) = fs::read_to_string(cache)
        .ok()
        .and_then(|s| serde_json::from_str::<JsonValue>(&s).ok())
        .and_then(|v| v.get("lines").cloned())
        .and_then(|v| v.as_array().cloned())
        .map(|a| {
            a.iter()
                .filter_map(JsonValue::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|l| !l.is_empty())
    {
        return lines;
    }
    value_to_lines(raw)
}
fn value_to_lines(value: &JsonValue) -> Vec<String> {
    match value {
        JsonValue::Array(v) => v
            .iter()
            .filter_map(JsonValue::as_str)
            .map(ToString::to_string)
            .collect(),
        JsonValue::String(s) => s.lines().map(ToString::to_string).collect(),
        _ => Vec::new(),
    }
}
fn load_package_language_texts(root: &Path, code: &str) -> HashMap<String, String> {
    let mut t = read_language_file(root, DEFAULT_LANGUAGE_CODE);
    if code != DEFAULT_LANGUAGE_CODE {
        t.extend(read_language_file(root, code));
    }
    t
}
fn read_language_file(root: &Path, code: &str) -> HashMap<String, String> {
    fs::read_to_string(root.join("assets/lang").join(format!("{code}.json")))
        .ok()
        .and_then(|s| {
            serde_json::from_str::<HashMap<String, String>>(s.trim_start_matches('\u{feff}')).ok()
        })
        .unwrap_or_default()
}
fn resolve_package_text(texts: &HashMap<String, String>, raw: &str) -> String {
    texts
        .get(raw)
        .filter(|v| !v.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| raw.to_string())
}
fn persist_state(kind: OverlayListKind, state: &JsonValue) -> std::io::Result<()> {
    let path = kind.state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(state)?)
}
