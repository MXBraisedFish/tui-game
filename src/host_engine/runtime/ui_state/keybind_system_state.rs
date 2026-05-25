//! System keybind UI state aggregation.

use crate::host_engine::boot::environment::data_dirs;
use std::fs;

use mlua::{Lua, Table, Value};
use serde_json::{Map, Value as JsonValue, json};
use unicode_width::UnicodeWidthStr;

use crate::host_engine::boot::i18n;
use crate::host_engine::boot::preload::persistent_data::keybind_profile;
use crate::host_engine::constant::MAX_ACTION_KEYS;

const DEFAULT_SORT: &str = "name";
const DEFAULT_ORDER: &str = "asc";
const FOCUS_LIST: &str = "list";
const FOCUS_KEYS: &str = "keys";
const MODE_ADD: &str = "add";
const MODE_DELETE: &str = "delete";

#[derive(Clone, Debug)]
pub struct KeybindSystemUiState {
    pub root_state: KeybindSystemRootState,
    pub lua_state: KeybindSystemLuaState,
}

impl KeybindSystemUiState {
    pub fn new(manifest: JsonValue, keybinds: JsonValue) -> Self {
        let root_state = KeybindSystemRootState::new(manifest, keybinds);
        let lua_state = KeybindSystemLuaState::from_root_state(&root_state);
        Self {
            root_state,
            lua_state,
        }
    }

    pub fn reset_lua_state(&mut self) {
        self.root_state.refresh_language();
        self.root_state.rebuild_pages();
        self.root_state.normalize_selection();
        self.lua_state = KeybindSystemLuaState::from_root_state(&self.root_state);
    }

    pub fn refresh_language(&mut self) {
        self.root_state.refresh_language();
    }

    pub fn refresh_keybinds(&mut self, keybinds: JsonValue) {
        self.root_state.keybinds = normalize_keybind_root(keybinds);
        self.root_state.rebuild_pages();
        self.root_state.normalize_selection();
        self.lua_state = KeybindSystemLuaState::from_root_state(&self.root_state);
    }

    pub fn apply_lua_state(&mut self, lua_state: KeybindSystemLuaState) -> KeybindSystemLuaAction {
        self.lua_state = lua_state;
        self.root_state.apply_view_state(&self.lua_state);

        if let Some(update) = self.lua_state.pending_update.clone() {
            self.root_state.apply_update(update);
        }

        self.root_state.rebuild_pages();
        self.root_state.normalize_selection();

        if self.lua_state.back && !self.root_state.has_empty_actions() {
            self.lua_state.back = false;
            return KeybindSystemLuaAction::Back(self.root_state.keybinds.clone());
        }

        KeybindSystemLuaAction::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum KeybindSystemLuaAction {
    None,
    Back(JsonValue),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeybindSystemLuaState {
    pub select: String,
    pub action_select: String,
    pub confirm: bool,
    pub back: bool,
    pub order: String,
    pub sort: String,
    pub pages: i64,
    pub page: i64,
    pub user_page: i64,
    pub jump: bool,
    pub action_scroll: i64,
    pub focus: String,
    pub mode: String,
    pub key_slot: i64,
    pub waiting_slot: i64,
    pub pending_update: Option<KeybindPendingUpdate>,
}

impl KeybindSystemLuaState {
    fn from_root_state(root_state: &KeybindSystemRootState) -> Self {
        Self {
            select: root_state.selected_page.clone(),
            action_select: root_state.selected_action.clone(),
            confirm: false,
            back: false,
            order: root_state.order.clone(),
            sort: root_state.sort.clone(),
            pages: root_state.pages.max(1),
            page: root_state.page.max(1),
            user_page: if root_state.jump {
                root_state.user_page
            } else {
                0
            },
            jump: root_state.jump,
            action_scroll: root_state.action_scroll.max(0),
            focus: root_state.focus.clone(),
            mode: root_state.mode.clone(),
            key_slot: root_state.key_slot.clamp(1, MAX_ACTION_KEYS as i64),
            waiting_slot: 0,
            pending_update: None,
        }
    }

    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("select", self.select.as_str())?;
        table.set("action_select", self.action_select.as_str())?;
        table.set("confirm", false)?;
        table.set("back", false)?;
        table.set("order", self.order.as_str())?;
        table.set("sort", self.sort.as_str())?;
        table.set("pages", self.pages.max(1))?;
        table.set("page", self.page.max(1))?;
        table.set("user_page", if self.jump { self.user_page } else { 0 })?;
        table.set("jump", self.jump)?;
        table.set("action_scroll", self.action_scroll.max(0))?;
        table.set("focus", self.focus.as_str())?;
        table.set("mode", self.mode.as_str())?;
        table.set("key_slot", self.key_slot.clamp(1, MAX_ACTION_KEYS as i64))?;
        table.set(
            "waiting_slot",
            self.waiting_slot.clamp(0, MAX_ACTION_KEYS as i64),
        )?;
        table.set("pending_update", Value::Nil)?;
        Ok(table)
    }

    pub fn from_lua_value(value: Value) -> mlua::Result<Self> {
        let table = match value {
            Value::Table(table) => table,
            _ => {
                return Err(mlua::Error::external(
                    "system keybind lua state must be returned as table",
                ));
            }
        };

        Ok(Self {
            select: table.get::<Option<String>>("select")?.unwrap_or_default(),
            action_select: table
                .get::<Option<String>>("action_select")?
                .unwrap_or_default(),
            confirm: table.get::<Option<bool>>("confirm")?.unwrap_or(false),
            back: table.get::<Option<bool>>("back")?.unwrap_or(false),
            order: normalize_order(
                table
                    .get::<Option<String>>("order")?
                    .unwrap_or_else(|| DEFAULT_ORDER.to_string()),
            ),
            sort: normalize_sort(
                table
                    .get::<Option<String>>("sort")?
                    .unwrap_or_else(|| DEFAULT_SORT.to_string()),
            ),
            pages: table.get::<Option<i64>>("pages")?.unwrap_or(1).max(1),
            page: table.get::<Option<i64>>("page")?.unwrap_or(1).max(1),
            user_page: table.get::<Option<i64>>("user_page")?.unwrap_or(0).max(0),
            jump: table.get::<Option<bool>>("jump")?.unwrap_or(false),
            action_scroll: table
                .get::<Option<i64>>("action_scroll")?
                .unwrap_or(0)
                .max(0),
            focus: normalize_focus(
                table
                    .get::<Option<String>>("focus")?
                    .unwrap_or_else(|| FOCUS_LIST.to_string()),
            ),
            mode: normalize_mode(
                table
                    .get::<Option<String>>("mode")?
                    .unwrap_or_else(|| MODE_ADD.to_string()),
            ),
            key_slot: table
                .get::<Option<i64>>("key_slot")?
                .unwrap_or(1)
                .clamp(1, MAX_ACTION_KEYS as i64),
            waiting_slot: table
                .get::<Option<i64>>("waiting_slot")?
                .unwrap_or(0)
                .clamp(0, MAX_ACTION_KEYS as i64),
            pending_update: parse_pending_update(table.get::<Option<Table>>("pending_update")?)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeybindPendingUpdate {
    op: String,
    page: String,
    action: String,
    slot: i64,
    key: String,
}

#[derive(Clone, Debug)]
pub struct KeybindSystemRootState {
    pub language: Vec<(String, String)>,
    manifest: JsonValue,
    keybinds: JsonValue,
    pages_list: Vec<SystemPageItem>,
    selected_page: String,
    selected_action: String,
    order: String,
    sort: String,
    pages: i64,
    page: i64,
    user_page: i64,
    jump: bool,
    action_scroll: i64,
    focus: String,
    mode: String,
    key_slot: i64,
}

impl KeybindSystemRootState {
    fn new(manifest: JsonValue, keybinds: JsonValue) -> Self {
        let mut state = Self {
            language: keybind_system_language_pairs(),
            manifest,
            keybinds: normalize_keybind_root(keybinds),
            pages_list: Vec::new(),
            selected_page: String::new(),
            selected_action: String::new(),
            order: DEFAULT_ORDER.to_string(),
            sort: DEFAULT_SORT.to_string(),
            pages: 1,
            page: 1,
            user_page: 0,
            jump: false,
            action_scroll: 0,
            focus: FOCUS_LIST.to_string(),
            mode: MODE_ADD.to_string(),
            key_slot: 1,
        };
        state.rebuild_pages();
        state.normalize_selection();
        state
    }

    fn refresh_language(&mut self) {
        self.language = keybind_system_language_pairs();
    }

    fn apply_view_state(&mut self, lua_state: &KeybindSystemLuaState) {
        self.selected_page = lua_state.select.clone();
        self.selected_action = lua_state.action_select.clone();
        self.order = normalize_order(lua_state.order.clone());
        self.sort = normalize_sort(lua_state.sort.clone());
        self.pages = lua_state.pages.max(1);
        self.page = lua_state.page.clamp(1, self.pages);
        self.user_page = if lua_state.jump {
            lua_state.user_page.max(0)
        } else {
            0
        };
        self.jump = lua_state.jump;
        self.action_scroll = lua_state.action_scroll.max(0);
        self.focus = normalize_focus(lua_state.focus.clone());
        self.mode = normalize_mode(lua_state.mode.clone());
        self.key_slot = lua_state.key_slot.clamp(1, MAX_ACTION_KEYS as i64);
    }

    fn rebuild_pages(&mut self) {
        self.pages_list =
            build_system_pages(&self.manifest, &self.keybinds, &self.sort, &self.order);
    }

    fn normalize_selection(&mut self) {
        if self.pages_list.is_empty() {
            self.selected_page.clear();
            self.selected_action.clear();
            return;
        }

        if !self
            .pages_list
            .iter()
            .any(|page| page.id == self.selected_page)
        {
            self.selected_page = self.pages_list[0].id.clone();
        }

        let actions = self.selected_actions();
        if actions.is_empty() {
            self.selected_action.clear();
        } else if !actions
            .iter()
            .any(|action| action.id == self.selected_action)
        {
            self.selected_action = actions[0].id.clone();
        }
    }

    fn selected_actions(&self) -> Vec<SystemActionItem> {
        self.pages_list
            .iter()
            .find(|page| page.id == self.selected_page)
            .map(|page| page.actions.clone())
            .unwrap_or_default()
    }

    fn has_empty_actions(&self) -> bool {
        self.pages_list.iter().any(|page| page.has_empty)
    }

    fn apply_update(&mut self, update: KeybindPendingUpdate) {
        if update.op == "page_reset" {
            if !update.page.trim().is_empty() {
                self.reset_page(update.page.as_str());
            }
            return;
        }

        if update.page.trim().is_empty() || update.action.trim().is_empty() {
            return;
        }

        match update.op.as_str() {
            "bind" => self.bind_key(
                update.page.as_str(),
                update.action.as_str(),
                update.slot,
                update.key.as_str(),
            ),
            "delete" => self.delete_key(update.page.as_str(), update.action.as_str(), update.slot),
            "reset" => self.reset_action(update.page.as_str(), update.action.as_str()),
            _ => {}
        }
    }

    fn reset_page(&mut self, page: &str) {
        let action_names: Vec<String> = self
            .manifest
            .get("actions")
            .and_then(JsonValue::as_object)
            .and_then(|actions| actions.get(page))
            .and_then(JsonValue::as_object)
            .map(|actions| actions.keys().cloned().collect())
            .unwrap_or_default();
        for action_name in &action_names {
            self.reset_action(page, action_name.as_str());
        }
    }

    fn bind_key(&mut self, page: &str, action: &str, slot: i64, key: &str) {
        let key = normalize_key(key);
        if key.is_empty() {
            return;
        }
        self.remove_key_from_page(page, key.as_str(), Some(action));
        let entry = self.ensure_entry(page, action);
        let mut keys = key_strings(entry.get("key_user").unwrap_or(&JsonValue::Null));
        keys.retain(|old_key| normalize_key(old_key) != key);
        let slot_index = (slot.clamp(1, MAX_ACTION_KEYS as i64) - 1) as usize;
        if slot_index > keys.len() {
            keys.push(key);
        } else if slot_index == keys.len() {
            keys.push(key);
        } else {
            keys[slot_index] = key;
        }
        keys.truncate(MAX_ACTION_KEYS);
        entry.insert("key_user".to_string(), key_value_from_strings(keys));
    }

    fn delete_key(&mut self, page: &str, action: &str, slot: i64) {
        let entry = self.ensure_entry(page, action);
        let mut keys = key_strings(entry.get("key_user").unwrap_or(&JsonValue::Null));
        let slot_index = (slot.clamp(1, MAX_ACTION_KEYS as i64) - 1) as usize;
        if slot_index < keys.len() {
            keys.remove(slot_index);
        }
        entry.insert("key_user".to_string(), key_value_from_strings(keys));
    }

    fn reset_action(&mut self, page: &str, action: &str) {
        let default_key = default_action_key(&self.manifest, page, action);
        self.remove_default_conflicts(page, action, &default_key);
        let entry = self.ensure_entry(page, action);
        entry.insert("key_user".to_string(), compact_key_value(default_key));
    }

    fn remove_default_conflicts(&mut self, page: &str, action: &str, key_value: &JsonValue) {
        for key in key_strings(key_value) {
            self.remove_key_from_page(page, normalize_key(key.as_str()).as_str(), Some(action));
        }
    }

    fn remove_key_from_page(&mut self, page: &str, key: &str, keep_action: Option<&str>) {
        let Some(page_object) = self.system_page_mut(page) else {
            return;
        };
        for (action_name, action_value) in page_object.iter_mut() {
            if keep_action.is_some_and(|keep_action| keep_action == action_name) {
                continue;
            }
            let Some(action_object) = action_value.as_object_mut() else {
                continue;
            };
            let keys = key_strings(action_object.get("key_user").unwrap_or(&JsonValue::Null))
                .into_iter()
                .filter(|old_key| normalize_key(old_key) != key)
                .collect::<Vec<_>>();
            action_object.insert("key_user".to_string(), key_value_from_strings(keys));
        }
    }

    fn ensure_entry(&mut self, page: &str, action: &str) -> &mut Map<String, JsonValue> {
        let default_key = default_action_key(&self.manifest, page, action);
        let default_name = default_action_name(&self.manifest, page, action);
        let page_object = self.ensure_system_page(page);
        let entry = page_object
            .entry(action.to_string())
            .or_insert_with(|| keybind_profile::keybind_entry(&default_key, default_name.as_str()));
        if !entry.is_object() {
            *entry = keybind_profile::keybind_entry(&default_key, default_name.as_str());
        }
        let entry_object = entry.as_object_mut().expect("entry must be object");
        entry_object
            .entry("key".to_string())
            .or_insert_with(|| compact_key_value(default_key.clone()));
        entry_object
            .entry("key_name".to_string())
            .or_insert_with(|| JsonValue::String(default_name));
        entry_object
            .entry("key_user".to_string())
            .or_insert_with(|| compact_key_value(default_key));
        entry_object
    }

    fn ensure_system_page(&mut self, page: &str) -> &mut Map<String, JsonValue> {
        let root_object = self.keybinds.as_object_mut().expect("keybind root object");
        let system = root_object
            .entry(keybind_profile::SYSTEM_SECTION.to_string())
            .or_insert_with(|| JsonValue::Object(Map::new()));
        if !system.is_object() {
            *system = JsonValue::Object(Map::new());
        }
        let system_object = system.as_object_mut().expect("system section object");
        let page_value = system_object
            .entry(page.to_string())
            .or_insert_with(|| JsonValue::Object(Map::new()));
        if !page_value.is_object() {
            *page_value = JsonValue::Object(Map::new());
        }
        page_value.as_object_mut().expect("system page object")
    }

    fn system_page_mut(&mut self, page: &str) -> Option<&mut Map<String, JsonValue>> {
        self.keybinds
            .get_mut(keybind_profile::SYSTEM_SECTION)
            .and_then(JsonValue::as_object_mut)
            .and_then(|system| system.get_mut(page))
            .and_then(JsonValue::as_object_mut)
    }

    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("language", pairs_to_table(lua, &self.language)?)?;
        table.set("page_list", page_list_to_table(lua, &self.pages_list)?)?;
        table.set(
            "action_list",
            action_list_to_table(lua, &self.selected_actions())?,
        )?;
        table.set("select", self.selected_page.as_str())?;
        table.set("action_select", self.selected_action.as_str())?;
        table.set("order", self.order.as_str())?;
        table.set("sort", self.sort.as_str())?;
        table.set("pages", self.pages.max(1))?;
        table.set("page", self.page.max(1))?;
        table.set("user_page", if self.jump { self.user_page } else { 0 })?;
        table.set("jump", self.jump)?;
        table.set("action_scroll", self.action_scroll.max(0))?;
        table.set("focus", self.focus.as_str())?;
        table.set("mode", self.mode.as_str())?;
        table.set("key_slot", self.key_slot.clamp(1, MAX_ACTION_KEYS as i64))?;
        table.set("case_sensitive", false)?;
        Ok(table)
    }
}

#[derive(Clone, Debug)]
struct SystemPageItem {
    id: String,
    name: String,
    has_empty: bool,
    has_conflict: bool,
    actions: Vec<SystemActionItem>,
}

#[derive(Clone, Debug)]
struct SystemActionItem {
    id: String,
    name: String,
    key: JsonValue,
    key_user: JsonValue,
    key_display: JsonValue,
    empty: bool,
    conflict: bool,
}

fn build_system_pages(
    manifest: &JsonValue,
    keybinds: &JsonValue,
    sort: &str,
    order: &str,
) -> Vec<SystemPageItem> {
    let Some(actions) = manifest.get("actions").and_then(JsonValue::as_object) else {
        return Vec::new();
    };

    let language_map = keybind_system_language_map();
    let mut pages = Vec::new();
    for (page_id, page_actions) in actions {
        if page_id.starts_with("warning_") {
            continue;
        }
        let Some(page_actions) = page_actions.as_object() else {
            continue;
        };
        let mut action_items = Vec::new();
        for (action_id, action_value) in page_actions {
            let default_key = compact_key_value(truncate_key_value(
                &action_value.get("key").cloned().unwrap_or(JsonValue::Null),
            ));
            let key_user = stored_system_key(keybinds, page_id, action_id)
                .cloned()
                .map(|key_value| compact_key_value(truncate_key_value(&key_value)))
                .unwrap_or_else(|| default_key.clone());
            let empty = key_strings(&key_user).is_empty();
            action_items.push(SystemActionItem {
                id: action_id.clone(),
                name: resolve_language_name(
                    action_value
                        .get("name")
                        .and_then(JsonValue::as_str)
                        .unwrap_or(action_id),
                    &language_map,
                ),
                key_display: build_display_value(&default_key, &key_user),
                key: default_key,
                key_user,
                empty,
                conflict: false,
            });
        }
        mark_conflicts(&mut action_items);
        let has_empty = action_items.iter().any(|item| item.empty);
        let has_conflict = action_items.iter().any(|item| item.conflict);
        pages.push(SystemPageItem {
            id: page_id.clone(),
            name: resolve_page_name(page_id),
            has_empty,
            has_conflict,
            actions: action_items,
        });
    }

    sort_pages(&mut pages, sort, order);
    pages
}

fn sort_pages(pages: &mut [SystemPageItem], sort: &str, order: &str) {
    if sort == "conflict" {
        pages.sort_by(|left, right| {
            right
                .has_conflict
                .cmp(&left.has_conflict)
                .then_with(|| compare_text(left.name.as_str(), right.name.as_str()))
        });
    } else {
        pages.sort_by(|left, right| compare_text(left.name.as_str(), right.name.as_str()));
    }
    if order == "desc" {
        pages.reverse();
    }
}

fn mark_conflicts(actions: &mut [SystemActionItem]) {
    let mut seen: Map<String, JsonValue> = Map::new();
    for action in actions.iter() {
        for key in key_strings(&action.key_user) {
            let normalized_key = normalize_key(key.as_str());
            let value = seen
                .entry(normalized_key)
                .or_insert_with(|| JsonValue::Array(Vec::new()));
            if let Some(owners) = value.as_array_mut() {
                owners.push(JsonValue::String(action.id.clone()));
            }
        }
    }

    for action in actions.iter_mut() {
        action.conflict = key_strings(&action.key_user).into_iter().any(|key| {
            seen.get(normalize_key(key.as_str()).as_str())
                .and_then(JsonValue::as_array)
                .is_some_and(|owners| owners.len() > 1)
        });
    }
}

fn resolve_language_name(name: &str, language_map: &Map<String, JsonValue>) -> String {
    language_map
        .get(name)
        .and_then(JsonValue::as_str)
        .unwrap_or(name)
        .to_string()
}

fn resolve_page_name(page_id: &str) -> String {
    let text = i18n::text();
    match page_id {
        "home" => text.setting_keybind.system_page_home,
        "setting" => text.setting_keybind.system_page_setting,
        "game_list" => text.setting_keybind.system_page_game_list,
        "storage_details" => text.setting_keybind.system_page_storage_details,
        "setting_keybind" => text.setting_keybind.system_page_setting_keybind,
        "setting_memory" => text.setting_keybind.system_page_setting_memory,
        "setting_language" => text.setting_keybind.system_page_setting_language,
        "setting_mods" => text.setting_keybind.system_page_setting_mods,
        "setting_security" => text.setting_keybind.system_page_setting_security,
        "setting_display" => text.setting_keybind.system_page_setting_display,
        "mod_game_list" => text.mod_hub.game,
        "mod_screensaver_list" => text.mod_hub.screensaver,
        "mod_boss_list" => text.mod_hub.boss,
        "keybind_system" => text.setting_keybind.system_page_keybind_system,
        "keybind_global" => text.setting_keybind.system_page_keybind_global,
        "keybind_game" => text.setting_keybind.system_page_keybind_game,
        _ => page_id.to_string(),
    }
}

fn keybind_system_language_map() -> Map<String, JsonValue> {
    let text = i18n::text();
    let mut map: Map<String, JsonValue> = keybind_system_language_pairs()
        .into_iter()
        .map(|(key, value)| (key, JsonValue::String(value)))
        .collect();

    macro_rules! insert {
        ($key:expr, $value:expr) => {
            map.insert($key.to_string(), JsonValue::String($value));
        };
    }

    insert!("HOME_PREV_OPTION", text.key.home_prev_option);
    insert!("HOME_NEXT_OPTION", text.key.home_next_option);
    insert!("HOME_CONFIRM", text.key.home_confirm);
    insert!("HOME_OPTION1", text.key.home_option1);
    insert!("HOME_OPTION2", text.key.home_option2);
    insert!("HOME_OPTION3", text.key.home_option3);
    insert!("HOME_OPTION4", text.key.home_option4);
    insert!("HOME_OPTION5", text.key.home_option5);
    insert!("SETTING_PREV_OPTION", text.key.setting_prev_option);
    insert!("SETTING_NEXT_OPTION", text.key.setting_next_option);
    insert!("SETTING_CONFIRM", text.key.setting_confirm);
    insert!("SETTING_OPTION1", text.key.setting_option1);
    insert!("SETTING_OPTION2", text.key.setting_option2);
    insert!("SETTING_OPTION3", text.key.setting_option3);
    insert!("SETTING_OPTION4", text.key.setting_option4);
    insert!("SETTING_OPTION5", text.key.setting_option5);
    insert!("SETTING_OPTION6", text.key.setting_option6);
    insert!("SETTING_BACK", text.key.setting_back);
    insert!("GAME_LIST_PREV_OPTION", text.key.game_list_prev_option);
    insert!("GAME_LIST_NEXT_OPTION", text.key.game_list_next_option);
    insert!("GAME_LIST_PREV_PAGE", text.key.game_list_prev_page);
    insert!("GAME_LIST_NEXT_PAGE", text.key.game_list_next_page);
    insert!("GAME_LIST_SCROLL_UP", text.key.game_list_scroll_up);
    insert!("GAME_LIST_SCROLL_DOWN", text.key.game_list_scroll_down);
    insert!("GAME_LIST_JUMP", text.key.game_list_jump);
    insert!("GAME_LIST_ORDER", text.key.game_list_order);
    insert!("GAME_LIST_SORT", text.key.game_list_sort);
    insert!("GAME_LIST_BACK", text.key.game_list_back);
    insert!("GAME_LIST_BACK_CANCEL", text.key.game_list_back_cancel);
    insert!("GAME_LIST_START_CONFIRM", text.key.game_list_start_confirm);
    insert!("MOD_LIST_PREV_OPTION", text.key.mod_list_prev_option);
    insert!("MOD_LIST_NEXT_OPTION", text.key.mod_list_next_option);
    insert!("MOD_LIST_PREV_PAGE", text.key.mod_list_prev_page);
    insert!("MOD_LIST_NEXT_PAGE", text.key.mod_list_next_page);
    insert!("MOD_LIST_SCROLL_UP", text.key.mod_list_scroll_up);
    insert!("MOD_LIST_SCROLL_DOWN", text.key.mod_list_scroll_down);
    insert!("MOD_LIST_JUMP", text.key.mod_list_jump);
    insert!("MOD_LIST_TOGGLE_CONFIRM", text.key.mod_list_toggle_confirm);
    insert!("MOD_LIST_BACK_CANCEL", text.key.mod_list_back_cancel);
    insert!("MOD_LIST_ORDER", text.key.mod_list_order);
    insert!("MOD_LIST_SORT", text.key.mod_list_sort);
    insert!("MOD_LIST_DEBUG", text.key.mod_list_debug);
    insert!("MOD_LIST_LIST", text.key.mod_list_list);
    insert!("MOD_LIST_SAFE_MODE", text.key.mod_list_safe_mode);
    insert!("MOD_PREV_OPTION", text.key.mod_prev_option);
    insert!("MOD_NEXT_OPTION", text.key.mod_next_option);
    insert!("MOD_LIST_OPTION1", text.key.mod_list_option1);
    insert!("MOD_LIST_OPTION2", text.key.mod_list_option2);
    insert!("MOD_LIST_OPTION3", text.key.mod_list_option3);
    insert!("MOD_HUB_SELECT", text.key.mod_hub_select);
    insert!("MOD_HUB_CONFIRM", text.key.mod_hub_confirm);
    insert!("MOD_HUB_BACK", text.key.mod_hub_back);
    insert!("LANGUAGE_UP_OPTION", text.key.language_up_option);
    insert!("LANGUAGE_DOWN_OPTION", text.key.language_down_option);
    insert!("LANGUAGE_LEFT_OPTION", text.key.language_left_option);
    insert!("LANGUAGE_RIGHT_OPTION", text.key.language_right_option);
    insert!("LANGUAGE_CONFIRM", text.key.language_confirm);
    insert!("LANGUAGE_JUMP", text.key.language_jump);
    insert!("LANGUAGE_PREV_PAGE", text.key.language_prev_page);
    insert!("LANGUAGE_NEXT_PAGE", text.key.language_next_page);
    insert!("LANGUAGE_BACK_CANCEL", text.key.language_back_cancel);
    insert!("MEMORY_PREV_OPTION", text.key.memory_prev_option);
    insert!("MEMORY_NEXT_OPTION", text.key.memory_next_option);
    insert!("MEMORY_CONFIRM", text.key.memory_confirm);
    insert!("MEMORY_OPTION1", text.key.memory_option1);
    insert!("MEMORY_OPTION2", text.key.memory_option2);
    insert!("MEMORY_OPTION3", text.key.memory_option3);
    insert!("MEMORY_BACK", text.key.memory_back);
    insert!("SECURITY_PREV_OPTION", text.key.security_prev_option);
    insert!("SECURITY_NEXT_OPTION", text.key.security_next_option);
    insert!("SECURITY_TOGGLE_CONFIRM", text.key.security_toggle_confirm);
    insert!("SECURITY_OPTION1", text.key.security_option1);
    insert!("SECURITY_OPTION2", text.key.security_option2);
    insert!("SECURITY_OPTION3", text.key.security_option3);
    insert!("SECURITY_OPTION4", text.key.security_option4);
    insert!("SECURITY_OPTION5", text.key.security_option5);
    insert!("SECURITY_OPTION6", text.key.security_option6);
    insert!("SECURITY_OPTION7", text.key.security_option7);
    insert!("SECURITY_OPTION8", text.key.security_option8);
    insert!("SECURITY_BACK", text.key.security_back);
    insert!(
        "SETTING_KEYBIND_LIST_PREV_OPTION",
        text.key.setting_keybind_list_prev_option
    );
    insert!(
        "SETTING_KEYBIND_LIST_NEXT_OPTION",
        text.key.setting_keybind_list_next_option
    );
    insert!(
        "SETTING_KEYBIND_LIST_CONFIRM",
        text.key.setting_keybind_list_confirm
    );
    insert!(
        "SETTING_KEYBIND_LIST_OPTION1",
        text.key.setting_keybind_list_option1
    );
    insert!(
        "SETTING_KEYBIND_LIST_OPTION2",
        text.key.setting_keybind_list_option2
    );
    insert!(
        "SETTING_KEYBIND_LIST_OPTION3",
        text.key.setting_keybind_list_option3
    );
    insert!(
        "SETTING_KEYBIND_LIST_BACK",
        text.key.setting_keybind_list_back
    );
    insert!("DISPLAY_PREV_OPTION", text.key.display_prev_option);
    insert!("DISPLAY_NEXT_OPTION", text.key.display_next_option);
    insert!("DISPLAY_SCROLL_UP", text.key.display_scroll_up);
    insert!("DISPLAY_SCROLL_DOWN", text.key.display_scroll_down);
    insert!("DISPLAY_CONFIRM", text.key.display_confirm);
    insert!("DISPLAY_BACK", text.key.display_back);
    insert!("DISPLAY_ORDER", text.key.display_order);
    insert!("DISPLAY_POSITION", text.key.display_position);
    insert!("DISPLAY_OPTION1", text.key.display_option1);
    insert!("DISPLAY_OPTION2", text.key.display_option2);
    insert!("DISPLAY_OPTION3", text.key.display_option3);
    insert!("DISPLAY_OPTION4", text.key.display_option4);
    insert!("DISPLAY_OPTION5", text.key.display_option5);
    insert!("DISPLAY_OPTION6", text.key.display_option6);
    insert!("DISPLAY_OPTION7", text.key.display_option7);
    insert!("DISPLAY_OPTION8", text.key.display_option8);
    insert!("DISPLAY_OPTION9", text.key.display_option9);
    insert!("STORAGE_DETAILS_BACK", text.key.storage_details_back);
    map
}

fn keybind_system_language_pairs() -> Vec<(String, String)> {
    let text = i18n::text();
    vec![
        (
            "SETTING_KEYBIND_SYSTEM_PREV_OPTION".to_string(),
            text.key.setting_keybind_system_prev_option,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_NEXT_OPTION".to_string(),
            text.key.setting_keybind_system_next_option,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_SELECT".to_string(),
            text.key.setting_keybind_system_select,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_PREV_PAGE".to_string(),
            text.key.setting_keybind_system_prev_page,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_NEXT_PAGE".to_string(),
            text.key.setting_keybind_system_next_page,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_SCROLL_UP".to_string(),
            text.key.setting_keybind_system_scroll_up,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_SCROLL_DOWN".to_string(),
            text.key.setting_keybind_system_scroll_down,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_SCROLL".to_string(),
            text.key.setting_keybind_system_scroll,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_JUMP".to_string(),
            text.key.setting_keybind_system_jump,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_ORDER".to_string(),
            text.key.setting_keybind_system_order,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_SORT".to_string(),
            text.key.setting_keybind_system_sort,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_CONFIRM".to_string(),
            text.key.setting_keybind_system_confirm,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_LIST_BACK".to_string(),
            text.key.setting_keybind_system_list_back,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_BACK".to_string(),
            text.key.setting_keybind_system_back,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_LIST".to_string(),
            text.key.setting_keybind_system_list,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_KEY1".to_string(),
            text.key.setting_keybind_system_key1,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_KEY2".to_string(),
            text.key.setting_keybind_system_key2,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_KEY3".to_string(),
            text.key.setting_keybind_system_key3,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_KEY4".to_string(),
            text.key.setting_keybind_system_key4,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_TIP_DELETE".to_string(),
            text.key.setting_keybind_system_tip_delete,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_TIP_ADD_MODIFY".to_string(),
            text.key.setting_keybind_system_tip_add_modify,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_ADD".to_string(),
            text.key.setting_keybind_system_add,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_MODIFY".to_string(),
            text.key.setting_keybind_system_modify,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_ADD_SHIFT".to_string(),
            text.key.setting_keybind_system_add_shift,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_MODIFY_SHIFT".to_string(),
            text.key.setting_keybind_system_modify_shift,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_DELETE".to_string(),
            text.key.setting_keybind_system_delete,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_KEY_MODE".to_string(),
            text.key.setting_keybind_system_key_mode,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_RESET_ONLY".to_string(),
            text.key.setting_keybind_system_reset_only,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_RESET_GAME".to_string(),
            text.key.setting_keybind_system_reset_game,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_RESET_PAGE".to_string(),
            text.key.setting_keybind_system_reset_page,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_LIST_TITLE".to_string(),
            text.setting_keybind.system_list_title,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_KEY_TITLE".to_string(),
            text.setting_keybind.system_key_title,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_KEY_ANY".to_string(),
            text.setting_keybind.system_key_any,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_SORT_NAME".to_string(),
            text.setting_keybind.system_sort_name,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_SORT_CONFLICT".to_string(),
            text.setting_keybind.system_sort_conflict,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_ORDER_ASCENDING".to_string(),
            text.setting_keybind.system_order_ascending,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_ORDER_DESCENDING".to_string(),
            text.setting_keybind.system_order_descending,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_TABLE_ACTION".to_string(),
            text.setting_keybind.system_table_action,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_TABLE_KEY1".to_string(),
            text.setting_keybind.system_table_key1,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_TABLE_KEY2".to_string(),
            text.setting_keybind.system_table_key2,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_TABLE_KEY3".to_string(),
            text.setting_keybind.system_table_key3,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_TABLE_KEY4".to_string(),
            text.setting_keybind.system_table_key4,
        ),
        (
            "SETTING_KEYBIND_SYSTEM_CASE_SENSITIVE".to_string(),
            text.setting_keybind.system_case_sensitive,
        ),
    ]
}

fn parse_pending_update(table: Option<Table>) -> mlua::Result<Option<KeybindPendingUpdate>> {
    let Some(table) = table else {
        return Ok(None);
    };
    Ok(Some(KeybindPendingUpdate {
        op: table.get::<Option<String>>("op")?.unwrap_or_default(),
        page: table.get::<Option<String>>("page")?.unwrap_or_default(),
        action: table.get::<Option<String>>("action")?.unwrap_or_default(),
        slot: table
            .get::<Option<i64>>("slot")?
            .unwrap_or(1)
            .clamp(1, MAX_ACTION_KEYS as i64),
        key: table.get::<Option<String>>("key")?.unwrap_or_default(),
    }))
}

fn default_action_key(manifest: &JsonValue, page: &str, action: &str) -> JsonValue {
    manifest
        .get("actions")
        .and_then(JsonValue::as_object)
        .and_then(|actions| actions.get(page))
        .and_then(JsonValue::as_object)
        .and_then(|page_actions| page_actions.get(action))
        .and_then(|action_value| action_value.get("key"))
        .cloned()
        .map(|value| compact_key_value(truncate_key_value(&value)))
        .unwrap_or_else(|| JsonValue::Array(Vec::new()))
}

fn default_action_name(manifest: &JsonValue, page: &str, action: &str) -> String {
    manifest
        .get("actions")
        .and_then(JsonValue::as_object)
        .and_then(|actions| actions.get(page))
        .and_then(JsonValue::as_object)
        .and_then(|page_actions| page_actions.get(action))
        .and_then(|action_value| action_value.get("name"))
        .and_then(JsonValue::as_str)
        .unwrap_or(action)
        .to_string()
}

fn stored_system_key<'a>(
    keybinds: &'a JsonValue,
    page: &str,
    action: &str,
) -> Option<&'a JsonValue> {
    keybind_profile::system_section(keybinds)
        .and_then(|system| system.get(page))
        .and_then(JsonValue::as_object)
        .and_then(|page| page.get(action))
        .and_then(|action| action.get("key_user"))
}

fn normalize_keybind_root(keybinds: JsonValue) -> JsonValue {
    let mut root = keybinds.as_object().cloned().unwrap_or_default();
    root.entry(keybind_profile::GLOBAL_SECTION.to_string())
        .or_insert_with(|| JsonValue::Object(Map::new()));
    root.entry(keybind_profile::SYSTEM_SECTION.to_string())
        .or_insert_with(|| JsonValue::Object(Map::new()));
    root.entry(keybind_profile::GAME_SECTION.to_string())
        .or_insert_with(|| JsonValue::Object(Map::new()));
    JsonValue::Object(root)
}

fn normalize_order(order: String) -> String {
    if order == "desc" {
        "desc".to_string()
    } else {
        DEFAULT_ORDER.to_string()
    }
}

fn normalize_sort(sort: String) -> String {
    if sort == "conflict" {
        "conflict".to_string()
    } else {
        DEFAULT_SORT.to_string()
    }
}

fn normalize_focus(focus: String) -> String {
    if focus == FOCUS_KEYS {
        FOCUS_KEYS.to_string()
    } else {
        FOCUS_LIST.to_string()
    }
}

fn normalize_mode(mode: String) -> String {
    if mode == MODE_DELETE {
        MODE_DELETE.to_string()
    } else {
        MODE_ADD.to_string()
    }
}

fn normalize_key(key: &str) -> String {
    key.trim().to_ascii_lowercase()
}

fn truncate_key_value(key_value: &JsonValue) -> JsonValue {
    match key_value {
        JsonValue::Array(keys) => {
            JsonValue::Array(keys.iter().take(MAX_ACTION_KEYS).cloned().collect())
        }
        _ => key_value.clone(),
    }
}

fn compact_key_value(key_value: JsonValue) -> JsonValue {
    key_value_from_strings(key_strings(&key_value))
}

fn key_value_from_strings(keys: Vec<String>) -> JsonValue {
    let mut output = Vec::new();
    for key in keys {
        let key = normalize_key(key.as_str());
        if !key.is_empty() && !output.iter().any(|old_key: &String| old_key == &key) {
            output.push(key);
        }
        if output.len() >= MAX_ACTION_KEYS {
            break;
        }
    }
    match output.as_slice() {
        [] => JsonValue::Array(Vec::new()),
        [key] => JsonValue::String(key.clone()),
        _ => JsonValue::Array(output.into_iter().map(JsonValue::String).collect()),
    }
}

fn key_strings(key_value: &JsonValue) -> Vec<String> {
    match key_value {
        JsonValue::String(key) if !key.trim().is_empty() => vec![normalize_key(key)],
        JsonValue::Array(keys) => keys
            .iter()
            .take(MAX_ACTION_KEYS)
            .filter_map(JsonValue::as_str)
            .map(normalize_key)
            .filter(|key| !key.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

fn build_display_value(key: &JsonValue, key_user: &JsonValue) -> JsonValue {
    json!({
        "key": display_key_value(key),
        "key_user": display_key_value(key_user)
    })
}

fn display_key_value(key_value: &JsonValue) -> JsonValue {
    match key_value {
        JsonValue::String(key) => JsonValue::String(display_semantic_key(key)),
        JsonValue::Array(keys) => JsonValue::Array(
            keys.iter()
                .take(MAX_ACTION_KEYS)
                .map(display_key_value)
                .collect(),
        ),
        _ => JsonValue::Array(Vec::new()),
    }
}

fn display_semantic_key(key: &str) -> String {
    let key = key.trim();
    if key.len() == 1 {
        let character = key.chars().next().unwrap_or_default();
        if character.is_ascii_lowercase() {
            return character.to_ascii_uppercase().to_string();
        }
        return character.to_string();
    }

    match key {
        "f1" => "F1",
        "f2" => "F2",
        "f3" => "F3",
        "f4" => "F4",
        "f5" => "F5",
        "f6" => "F6",
        "f7" => "F7",
        "f8" => "F8",
        "f9" => "F9",
        "f10" => "F10",
        "f11" => "F11",
        "f12" => "F12",
        "up" => "↑",
        "down" => "↓",
        "left" => "←",
        "right" => "→",
        "pageup" => "PgUp",
        "pagedown" => "PgDn",
        "enter" => "Enter",
        "backspace" => "Bksp",
        "del" => "Del",
        "ins" => "Ins",
        "back_tab" => "BTab",
        "space" => "Space",
        "left_ctrl" => "LCtrl",
        "right_ctrl" => "RCtrl",
        "left_shift" => "LShift",
        "right_shift" => "RShift",
        "shift" => "Shift",
        "left_alt" => "LAlt",
        "right_alt" => "RAlt",
        "left_meta" => "LMeta",
        "right_meta" => "RMeta",
        "capslock" => "Caps",
        "numlock" => "Num",
        "scrolllock" => "Scrl",
        "esc" => "Esc",
        "printscreen" => "Prtsc",
        other => other,
    }
    .to_string()
}

fn compare_text(left: &str, right: &str) -> std::cmp::Ordering {
    let left_text = left.to_lowercase();
    let right_text = right.to_lowercase();
    UnicodeWidthStr::width(left_text.as_str())
        .cmp(&UnicodeWidthStr::width(right_text.as_str()))
        .then_with(|| left_text.cmp(&right_text))
}

fn pairs_to_table(lua: &Lua, pairs: &[(String, String)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (key, value) in pairs {
        table.set(key.as_str(), value.as_str())?;
    }
    Ok(table)
}

fn page_list_to_table(lua: &Lua, pages: &[SystemPageItem]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, page) in pages.iter().enumerate() {
        let item = lua.create_table()?;
        item.set("id", page.id.as_str())?;
        item.set("name", page.name.as_str())?;
        item.set("has_empty", page.has_empty)?;
        item.set("has_conflict", page.has_conflict)?;
        table.set(index + 1, item)?;
    }
    Ok(table)
}

fn action_list_to_table(lua: &Lua, actions: &[SystemActionItem]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, action) in actions.iter().enumerate() {
        let item = lua.create_table()?;
        item.set("id", action.id.as_str())?;
        item.set("name", action.name.as_str())?;
        item.set("key", json_to_lua_value(lua, &action.key)?)?;
        item.set("key_user", json_to_lua_value(lua, &action.key_user)?)?;
        item.set("key_display", json_to_lua_value(lua, &action.key_display)?)?;
        item.set("empty", action.empty)?;
        item.set("conflict", action.conflict)?;
        table.set(index + 1, item)?;
    }
    Ok(table)
}

fn json_to_lua_value(lua: &Lua, value: &JsonValue) -> mlua::Result<Value> {
    match value {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(value) => Ok(Value::Boolean(*value)),
        JsonValue::Number(value) => {
            if let Some(value) = value.as_i64() {
                Ok(Value::Integer(value))
            } else if let Some(value) = value.as_f64() {
                Ok(Value::Number(value))
            } else {
                Ok(Value::Nil)
            }
        }
        JsonValue::String(value) => Ok(Value::String(lua.create_string(value)?)),
        JsonValue::Array(values) => {
            let table = lua.create_table()?;
            for (index, item) in values.iter().enumerate() {
                table.set(index + 1, json_to_lua_value(lua, item)?)?;
            }
            Ok(Value::Table(table))
        }
        JsonValue::Object(values) => {
            let table = lua.create_table()?;
            for (key, item) in values {
                table.set(key.as_str(), json_to_lua_value(lua, item)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

pub fn persist_system_keybinds(keybinds: &JsonValue) -> std::io::Result<()> {
    let path = data_dirs::root_dir().join("data/profiles/keybind.json");
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(path, serde_json::to_string_pretty(keybinds)?)?;
    Ok(())
}
