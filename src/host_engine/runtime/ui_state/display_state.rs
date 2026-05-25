//! 显示设置 UI 状态聚合

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use mlua::{Lua, Table, Value};
use serde_json::Value as JsonValue;

use crate::host_engine::boot::i18n;
use crate::host_engine::boot::preload::overlay_modules::{
    OverlayPackage, OverlayRegistry, OverlaySource,
};
use crate::host_engine::boot::preload::persistent_data::display_profile::{
    DisplayOverlayProfile, DisplayProfile,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplayPanelKind {
    None,
    Screensaver,
    Boss,
}

impl DisplayPanelKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Screensaver => "screensaver",
            Self::Boss => "boss",
        }
    }
    fn from_str(value: &str) -> Self {
        match value {
            "screensaver" => Self::Screensaver,
            "boss" => Self::Boss,
            _ => Self::None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DisplayUiState {
    pub root_state: DisplayRootState,
    pub lua_state: DisplayLuaState,
}

impl DisplayUiState {
    pub fn new(
        display_state: JsonValue,
        overlay_registry: OverlayRegistry,
        screensaver_state: JsonValue,
        boss_state: JsonValue,
        language_code: String,
    ) -> Self {
        let profile = DisplayProfile::from_value(&display_state);
        let root_state = DisplayRootState::new(
            profile,
            overlay_registry,
            screensaver_state,
            boss_state,
            language_code,
        );
        let lua_state = DisplayLuaState::from_root_state(&root_state);
        Self {
            root_state,
            lua_state,
        }
    }

    pub fn reset_lua_state(&mut self) {
        self.root_state.refresh_language();
        self.root_state.normalize_select();
        self.lua_state = DisplayLuaState::from_root_state(&self.root_state);
    }

    pub fn refresh_language(&mut self, language_code: String) {
        self.root_state.language_code = language_code;
        self.root_state.refresh_language();
        self.root_state.refresh_items();
    }

    pub fn replace_overlay_data(
        &mut self,
        overlay_registry: OverlayRegistry,
        screensaver_state: JsonValue,
        boss_state: JsonValue,
    ) {
        self.root_state.overlay_registry = overlay_registry;
        self.root_state.screensaver_state = screensaver_state;
        self.root_state.boss_state = boss_state;
        self.root_state.refresh_items();
        self.root_state.normalize_list_select();
        self.reset_lua_state();
    }

    pub fn show_mod_badge(&self) -> bool {
        self.root_state.profile.mod_badge
    }

    pub fn idle_threshold(&self) -> u64 {
        self.root_state.profile.idle_threshold
    }

    pub fn should_auto_enter_screensaver(&self) -> bool {
        self.root_state.profile.idle_enter_screensaver
            && self.root_state.profile.screensaver_mode != "off"
            && self
                .root_state
                .display_candidates(DisplayPanelKind::Screensaver)
                .next()
                .is_some()
    }

    pub fn selected_overlay_uid(&mut self, panel: DisplayPanelKind) -> Option<String> {
        let mode = match panel {
            DisplayPanelKind::Screensaver => self.root_state.profile.screensaver_mode.as_str(),
            DisplayPanelKind::Boss => self.root_state.profile.boss_mode.as_str(),
            DisplayPanelKind::None => return None,
        };
        if mode == "off" {
            return None;
        }
        let candidates = self
            .root_state
            .display_candidates(panel)
            .cloned()
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            return None;
        }
        let selected = if mode == "random" {
            let index = random_index(candidates.len());
            candidates.get(index).cloned()
        } else {
            let profile = match panel {
                DisplayPanelKind::Screensaver => &mut self.root_state.profile.screensaver_list,
                DisplayPanelKind::Boss => &mut self.root_state.profile.boss_list,
                DisplayPanelKind::None => unreachable!(),
            };
            if profile.cursor >= candidates.len() {
                profile.cursor = 0;
            }
            let index = profile.cursor;
            profile.cursor = if candidates.is_empty() {
                0
            } else {
                (index + 1) % candidates.len()
            };
            let _ = self.root_state.persist_profile();
            candidates.get(index).cloned()
        };
        selected.map(|item| item.uid)
    }

    pub fn apply_lua_state(&mut self, lua_state: DisplayLuaState) -> DisplayLuaAction {
        self.lua_state = lua_state;
        self.root_state.select = self.lua_state.select.clamp(1, 9);
        self.root_state.panel = DisplayPanelKind::from_str(&self.lua_state.panel);
        self.root_state.list_select_uid = self.lua_state.list_select.clone();
        self.root_state.list_scroll = self.lua_state.list_scroll.max(0);
        self.root_state.move_mode = self.lua_state.move_mode;
        self.root_state.position_mode = self.lua_state.position_mode;
        self.root_state.position_input = self.lua_state.position_input.max(0);
        self.root_state.normalize_select();
        self.root_state.normalize_list_select();

        if self.lua_state.back {
            self.lua_state.back = false;
            if self.root_state.panel != DisplayPanelKind::None {
                self.root_state.panel = DisplayPanelKind::None;
                self.root_state.move_mode = false;
                self.root_state.position_mode = false;
                self.root_state.position_input = 0;
                self.lua_state.panel = DisplayPanelKind::None.as_str().to_string();
                self.lua_state.move_mode = false;
                self.lua_state.position_mode = false;
                self.lua_state.position_input = 0;
                let _ = self.root_state.persist_profile();
                return DisplayLuaAction::None;
            }
            return DisplayLuaAction::Back;
        }

        if self.root_state.panel != DisplayPanelKind::None {
            if self.lua_state.move_delta != 0 {
                self.root_state.move_selected_by(self.lua_state.move_delta);
                let _ = self.root_state.persist_profile();
                return DisplayLuaAction::StateChanged(self.root_state.profile.to_value());
            }
            if self.lua_state.position_target > 0 {
                self.root_state
                    .move_selected_to(self.lua_state.position_target as usize);
                self.root_state.position_mode = false;
                self.root_state.position_input = 0;
                let _ = self.root_state.persist_profile();
                return DisplayLuaAction::StateChanged(self.root_state.profile.to_value());
            }
            if self.lua_state.confirm {
                self.root_state.toggle_selected_display_enabled();
                let _ = self.root_state.persist_profile();
                return DisplayLuaAction::StateChanged(self.root_state.profile.to_value());
            }
            return DisplayLuaAction::None;
        }

        if self.lua_state.confirm {
            self.handle_normal_confirm();
            let _ = self.root_state.persist_profile();
            return DisplayLuaAction::StateChanged(self.root_state.profile.to_value());
        }

        DisplayLuaAction::None
    }

    fn handle_normal_confirm(&mut self) {
        match self.root_state.select {
            1 => self.root_state.profile.mod_badge = !self.root_state.profile.mod_badge,
            2 => self.root_state.profile.theme = "system".to_string(),
            3 => {
                self.root_state.profile.idle_threshold =
                    next_idle_threshold(self.root_state.profile.idle_threshold)
            }
            4 => {
                self.root_state.profile.idle_enter_screensaver = !self.root_state.profile.idle_enter_screensaver
            }
            5 => self.root_state.profile.host_status = !self.root_state.profile.host_status,
            6 => {
                self.root_state.profile.screensaver_mode = next_mode(&self.root_state.profile.screensaver_mode)
            }
            7 => self.root_state.profile.boss_mode = next_mode(&self.root_state.profile.boss_mode),
            8 => self.root_state.panel = DisplayPanelKind::Screensaver,
            9 => self.root_state.panel = DisplayPanelKind::Boss,
            _ => {}
        }
        self.root_state.normalize_list_select();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DisplayLuaAction {
    None,
    Back,
    StateChanged(JsonValue),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisplayLuaState {
    pub select: i64,
    pub confirm: bool,
    pub back: bool,
    pub panel: String,
    pub list_select: String,
    pub list_scroll: i64,
    pub move_mode: bool,
    pub move_delta: i64,
    pub position_mode: bool,
    pub position_input: i64,
    pub position_target: i64,
}

impl DisplayLuaState {
    fn from_root_state(root: &DisplayRootState) -> Self {
        Self {
            select: root.select.clamp(1, 9),
            confirm: false,
            back: false,
            panel: root.panel.as_str().to_string(),
            list_select: root.list_select_uid.clone(),
            list_scroll: root.list_scroll.max(0),
            move_mode: root.move_mode,
            move_delta: 0,
            position_mode: root.position_mode,
            position_input: root.position_input.max(0),
            position_target: 0,
        }
    }

    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("select", self.select.clamp(1, 9))?;
        table.set("confirm", false)?;
        table.set("back", false)?;
        table.set("panel", self.panel.as_str())?;
        table.set("list_select", self.list_select.as_str())?;
        table.set("list_scroll", self.list_scroll.max(0))?;
        table.set("move_mode", self.move_mode)?;
        table.set("move_delta", 0)?;
        table.set("position_mode", self.position_mode)?;
        table.set("position_input", self.position_input.max(0))?;
        table.set("position_target", 0)?;
        Ok(table)
    }

    pub fn from_lua_value(value: Value) -> mlua::Result<Self> {
        let table = match value {
            Value::Table(table) => table,
            _ => {
                return Err(mlua::Error::external(
                    "display lua state must be returned as table",
                ));
            }
        };
        Ok(Self {
            select: table.get::<Option<i64>>("select")?.unwrap_or(1).clamp(1, 9),
            confirm: table.get::<Option<bool>>("confirm")?.unwrap_or(false),
            back: table.get::<Option<bool>>("back")?.unwrap_or(false),
            panel: table
                .get::<Option<String>>("panel")?
                .unwrap_or_else(|| "none".to_string()),
            list_select: table
                .get::<Option<String>>("list_select")?
                .unwrap_or_default(),
            list_scroll: table.get::<Option<i64>>("list_scroll")?.unwrap_or(0).max(0),
            move_mode: table.get::<Option<bool>>("move_mode")?.unwrap_or(false),
            move_delta: table.get::<Option<i64>>("move_delta")?.unwrap_or(0),
            position_mode: table.get::<Option<bool>>("position_mode")?.unwrap_or(false),
            position_input: table
                .get::<Option<i64>>("position_input")?
                .unwrap_or(0)
                .max(0),
            position_target: table
                .get::<Option<i64>>("position_target")?
                .unwrap_or(0)
                .max(0),
        })
    }
}

#[derive(Clone, Debug)]
pub struct DisplayRootState {
    pub language: Vec<(String, String)>,
    pub profile: DisplayProfile,
    pub overlay_registry: OverlayRegistry,
    pub screensaver_state: JsonValue,
    pub boss_state: JsonValue,
    pub language_code: String,
    pub select: i64,
    pub panel: DisplayPanelKind,
    pub list_select_uid: String,
    pub list_scroll: i64,
    pub move_mode: bool,
    pub position_mode: bool,
    pub position_input: i64,
    pub screensaver_items: Vec<DisplayOverlayItem>,
    pub boss_items: Vec<DisplayOverlayItem>,
}

impl DisplayRootState {
    fn new(
        profile: DisplayProfile,
        overlay_registry: OverlayRegistry,
        screensaver_state: JsonValue,
        boss_state: JsonValue,
        language_code: String,
    ) -> Self {
        let mut root = Self {
            language: display_language_pairs(),
            profile,
            overlay_registry,
            screensaver_state,
            boss_state,
            language_code,
            select: 1,
            panel: DisplayPanelKind::None,
            list_select_uid: String::new(),
            list_scroll: 0,
            move_mode: false,
            position_mode: false,
            position_input: 0,
            screensaver_items: Vec::new(),
            boss_items: Vec::new(),
        };
        root.refresh_items();
        root.normalize_list_select();
        root
    }

    fn refresh_language(&mut self) {
        self.language = display_language_pairs();
    }

    fn refresh_items(&mut self) {
        self.profile.normalize();
        self.screensaver_items = build_overlay_items(
            &self.overlay_registry.screensavers,
            &self.screensaver_state,
            &mut self.profile.screensaver_list,
            &self.language_code,
        );
        self.boss_items = build_overlay_items(
            &self.overlay_registry.bosses,
            &self.boss_state,
            &mut self.profile.boss_list,
            &self.language_code,
        );
        let _ = self.persist_profile();
    }

    fn normalize_select(&mut self) {
        self.select = self.select.clamp(1, 9);
    }

    fn active_items(&self) -> &[DisplayOverlayItem] {
        match self.panel {
            DisplayPanelKind::Screensaver => &self.screensaver_items,
            DisplayPanelKind::Boss => &self.boss_items,
            DisplayPanelKind::None => &[],
        }
    }

    fn active_profile_mut(&mut self) -> Option<&mut DisplayOverlayProfile> {
        match self.panel {
            DisplayPanelKind::Screensaver => Some(&mut self.profile.screensaver_list),
            DisplayPanelKind::Boss => Some(&mut self.profile.boss_list),
            DisplayPanelKind::None => None,
        }
    }

    fn normalize_list_select(&mut self) {
        let items = self.active_items();
        if items.is_empty() {
            self.list_select_uid.clear();
            return;
        }
        if !items.iter().any(|item| item.uid == self.list_select_uid) {
            self.list_select_uid = items[0].uid.clone();
        }
    }

    fn display_candidates(
        &self,
        panel: DisplayPanelKind,
    ) -> impl Iterator<Item = &DisplayOverlayItem> {
        let items = match panel {
            DisplayPanelKind::Screensaver => &self.screensaver_items,
            DisplayPanelKind::Boss => &self.boss_items,
            DisplayPanelKind::None => &self.screensaver_items,
        };
        items
            .iter()
            .filter(|item| item.scan_enabled && item.display_enabled)
    }

    fn toggle_selected_display_enabled(&mut self) {
        let uid = self.list_select_uid.clone();
        let Some(profile) = self.active_profile_mut() else {
            return;
        };
        let enabled = profile.enabled.get(&uid).copied().unwrap_or(true);
        profile.enabled.insert(uid, !enabled);
        self.refresh_items();
    }

    fn move_selected_by(&mut self, delta: i64) {
        if delta == 0 || self.list_select_uid.is_empty() {
            return;
        }
        let uid = self.list_select_uid.clone();
        let Some(profile) = self.active_profile_mut() else {
            return;
        };
        let Some(index) = profile.order.iter().position(|item| item == &uid) else {
            return;
        };
        let target = if delta < 0 {
            index.saturating_sub(1)
        } else {
            (index + 1).min(profile.order.len().saturating_sub(1))
        };
        if target != index {
            profile.order.swap(index, target);
            self.refresh_items();
        }
    }

    fn move_selected_to(&mut self, target_one_based: usize) {
        if target_one_based == 0 || self.list_select_uid.is_empty() {
            return;
        }
        let uid = self.list_select_uid.clone();
        let Some(profile) = self.active_profile_mut() else {
            return;
        };
        let Some(index) = profile.order.iter().position(|item| item == &uid) else {
            return;
        };
        let item = profile.order.remove(index);
        let target = target_one_based.saturating_sub(1).min(profile.order.len());
        profile.order.insert(target, item);
        self.refresh_items();
    }

    fn persist_profile(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.profile.persist_default_path()
    }

    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("language", pairs_to_table(lua, &self.language)?)?;
        table.set("select", self.select.clamp(1, 9))?;
        table.set("panel", self.panel.as_str())?;
        table.set("list_select", self.list_select_uid.as_str())?;
        table.set("list_scroll", self.list_scroll.max(0))?;
        table.set("move_mode", self.move_mode)?;
        table.set("position_mode", self.position_mode)?;
        table.set("position_input", self.position_input.max(0))?;
        table.set("settings", profile_to_lua(lua, &self.profile)?)?;
        table.set("screensaver_list", items_to_table(lua, &self.screensaver_items)?)?;
        table.set("boss_list", items_to_table(lua, &self.boss_items)?)?;
        Ok(table)
    }
}

#[derive(Clone, Debug)]
pub struct DisplayOverlayItem {
    pub uid: String,
    pub name: String,
    pub package_name: String,
    pub source: String,
    pub is_mod: bool,
    pub scan_enabled: bool,
    pub display_enabled: bool,
}

fn build_overlay_items(
    packages: &[OverlayPackage],
    state: &JsonValue,
    profile: &mut DisplayOverlayProfile,
    language_code: &str,
) -> Vec<DisplayOverlayItem> {
    let mut by_uid = BTreeMap::new();
    for package in packages {
        let scan_enabled = package.source == OverlaySource::Office
            || state
                .get(package.uid.as_str())
                .and_then(|value| value.get("enabled"))
                .and_then(JsonValue::as_bool)
                .unwrap_or(true);
        if !scan_enabled {
            continue;
        }
        profile.enabled.entry(package.uid.clone()).or_insert(true);
        let texts = load_package_language_texts(&package.root_dir, language_code);
        by_uid.insert(
            package.uid.clone(),
            DisplayOverlayItem {
                uid: package.uid.clone(),
                name: resolve_package_text(&texts, &package.manifest.display_name),
                package_name: resolve_package_text(&texts, &package.manifest.package_name),
                source: package.source.as_str().to_string(),
                is_mod: package.source == OverlaySource::ThirdParty,
                scan_enabled,
                display_enabled: profile.enabled.get(&package.uid).copied().unwrap_or(true),
            },
        );
    }
    profile.order.retain(|uid| by_uid.contains_key(uid));
    profile.enabled.retain(|uid, _| by_uid.contains_key(uid));
    for package in packages {
        if by_uid.contains_key(&package.uid)
            && !profile.order.iter().any(|item| item == &package.uid)
        {
            profile.order.push(package.uid.clone());
        }
    }
    let items = profile
        .order
        .iter()
        .filter_map(|uid| by_uid.remove(uid))
        .collect::<Vec<_>>();
    if profile.cursor >= items.len() {
        profile.cursor = 0;
    }
    items
}

fn profile_to_lua(lua: &Lua, profile: &DisplayProfile) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("mod_badge", profile.mod_badge)?;
    table.set("theme", profile.theme.as_str())?;
    table.set("idle_threshold", profile.idle_threshold as i64)?;
    table.set("idle_enter_screensaver", profile.idle_enter_screensaver)?;
    table.set("host_status", profile.host_status)?;
    table.set("screensaver_mode", profile.screensaver_mode.as_str())?;
    table.set("boss_mode", profile.boss_mode.as_str())?;
    Ok(table)
}

fn items_to_table(lua: &Lua, items: &[DisplayOverlayItem]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, item) in items.iter().enumerate() {
        let row = lua.create_table()?;
        row.set("uid", item.uid.as_str())?;
        row.set("name", item.name.as_str())?;
        row.set("package_name", item.package_name.as_str())?;
        row.set("source", item.source.as_str())?;
        row.set("is_mod", item.is_mod)?;
        row.set("enabled", item.display_enabled)?;
        table.set(index + 1, row)?;
    }
    Ok(table)
}

fn pairs_to_table(lua: &Lua, pairs: &[(String, String)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (key, value) in pairs {
        table.set(key.as_str(), value.as_str())?;
    }
    Ok(table)
}

fn display_language_pairs() -> Vec<(String, String)> {
    let text = i18n::text();
    vec![
        ("DISPLAY_PREV_OPTION".into(), text.key.display_prev_option),
        ("DISPLAY_NEXT_OPTION".into(), text.key.display_next_option),
        ("DISPLAY_SCROLL_UP".into(), text.key.display_scroll_up),
        ("DISPLAY_SCROLL_DOWN".into(), text.key.display_scroll_down),
        ("DISPLAY_SCROLL".into(), text.key.display_scroll),
        ("DISPLAY_SELECT".into(), text.key.display_select),
        ("DISPLAY_BACK".into(), text.key.display_back),
        (
            "DISPLAY_TOGGLE_CONFIRM".into(),
            text.key.display_toggle_confirm,
        ),
        ("DISPLAY_TOGGLE".into(), text.key.display_toggle),
        ("DISPLAY_CONFIRM".into(), text.key.display_confirm),
        ("DISPLAY_ORDER".into(), text.key.display_order),
        ("DISPLAY_POSITION".into(), text.key.display_position),
        ("DISPLAY_OPTION1".into(), text.key.display_option1),
        ("DISPLAY_OPTION2".into(), text.key.display_option2),
        ("DISPLAY_OPTION3".into(), text.key.display_option3),
        ("DISPLAY_OPTION4".into(), text.key.display_option4),
        ("DISPLAY_OPTION5".into(), text.key.display_option5),
        ("DISPLAY_OPTION6".into(), text.key.display_option6),
        ("DISPLAY_OPTION7".into(), text.key.display_option7),
        ("DISPLAY_OPTION8".into(), text.key.display_option8),
        ("DISPLAY_OPTION9".into(), text.key.display_option9),
        ("DISPLAY_TITLE".into(), text.display.title),
        ("DISPLAY_TOGGLE_MOD_ON".into(), text.display.toggle_mod_on),
        ("DISPLAY_TOGGLE_MOD_OFF".into(), text.display.toggle_mod_off),
        (
            "DISPLAY_TOGGLE_AFK_SCREENSAVER_ON".into(),
            text.display.toggle_afk_screensaver_on,
        ),
        (
            "DISPLAY_TOGGLE_AFK_SCREENSAVER_OFF".into(),
            text.display.toggle_afk_screensaver_off,
        ),
        (
            "DISPLAY_TOGGLE_AFK_TIME_SECOND".into(),
            text.display.toggle_afk_time_second,
        ),
        (
            "DISPLAY_TOGGLE_AFK_TIME_MINUTE".into(),
            text.display.toggle_afk_time_minute,
        ),
        (
            "DISPLAY_TOGGLE_AFK_TIME_NEVER".into(),
            text.display.toggle_afk_time_never,
        ),
        (
            "DISPLAY_TOGGLE_SORT_ORDER".into(),
            text.display.toggle_sort_order,
        ),
        (
            "DISPLAY_TOGGLE_SORT_RANDOM".into(),
            text.display.toggle_sort_random,
        ),
        (
            "DISPLAY_TOGGLE_SORT_OFF".into(),
            text.display.toggle_sort_off,
        ),
        ("DISPLAY_OPTION_INFO_ON".into(), text.display.option_info_on),
        (
            "DISPLAY_OPTION_INFO_OFF".into(),
            text.display.option_info_off,
        ),
        (
            "DISPLAY_TOGGLE_THEME_SYSTEM".into(),
            text.display.toggle_theme_system,
        ),
        ("DISPLAY_OPTION_MOD".into(), text.display.option_mod),
        ("DISPLAY_OPTION_THEME".into(), text.display.option_theme),
        (
            "DISPLAY_OPTION_AFK_TIME".into(),
            text.display.option_afk_time,
        ),
        (
            "DISPLAY_OPTION_AFK_SCREENSAVER".into(),
            text.display.option_afk_screensaver,
        ),
        ("DISPLAY_OPTION_INFO".into(), text.display.option_info),
        (
            "DISPLAY_OPTION_SCREENSAVER_SORT".into(),
            text.display.option_screensaver_sort,
        ),
        (
            "DISPLAY_OPTION_BOSS_SORT".into(),
            text.display.option_boss_sort,
        ),
        (
            "DISPLAY_OPTION_SCREENSAVER_LIST".into(),
            text.display.option_screensaver_list,
        ),
        (
            "DISPLAY_OPTION_BOSS_LIST".into(),
            text.display.option_boss_list,
        ),
        ("DISPLAY_OPTION_LIST_ON".into(), text.display.option_list_on),
        (
            "DISPLAY_OPTION_LIST_OFF".into(),
            text.display.option_list_off,
        ),
        (
            "DISPLAY_OPTION_LIST_MOD".into(),
            text.display.option_list_mod,
        ),
    ]
}

fn next_idle_threshold(current: u64) -> u64 {
    match current {
        30 => 60,
        60 => 300,
        300 => 600,
        600 => 0,
        _ => 30,
    }
}

fn next_mode(current: &str) -> String {
    match current {
        "ordered" => "random".to_string(),
        "random" => "off".to_string(),
        _ => "ordered".to_string(),
    }
}

fn random_index(max: usize) -> usize {
    if max <= 1 {
        return 0;
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos() as usize)
        .unwrap_or_default();
    nanos % max
}

fn load_package_language_texts(root_dir: &Path, language_code: &str) -> BTreeMap<String, String> {
    let mut texts = read_language_file(root_dir, "en_us");
    if language_code != "en_us" {
        texts.extend(read_language_file(root_dir, language_code));
    }
    texts
}

fn read_language_file(root_dir: &Path, language_code: &str) -> BTreeMap<String, String> {
    let path = root_dir
        .join("assets/lang")
        .join(format!("{language_code}.json"));
    let Ok(raw_json) = fs::read_to_string(path) else {
        return BTreeMap::new();
    };
    serde_json::from_str::<BTreeMap<String, String>>(raw_json.trim_start_matches('\u{feff}'))
        .unwrap_or_default()
}

fn resolve_package_text(texts: &BTreeMap<String, String>, value: &str) -> String {
    texts
        .get(value)
        .filter(|text| !text.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| value.to_string())
}
