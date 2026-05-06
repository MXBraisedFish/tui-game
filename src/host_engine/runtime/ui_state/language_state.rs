//! UI Language 状态聚合

use std::collections::BTreeMap;

use mlua::{Lua, Table, Value};

use crate::host_engine::boot::preload::cache_data::LanguageUiText;

/// 语言选择页面宿主与 Lua 双层状态。
#[derive(Clone, Debug)]
pub struct LanguageUiState {
    pub root_state: LanguageRootState,
    pub lua_state: LanguageLuaState,
}

impl LanguageUiState {
    /// 创建语言选择页面状态。
    pub fn new(
        current_language_code: String,
        language_ui_texts: BTreeMap<String, LanguageUiText>,
    ) -> Self {
        let selected_language_code = if language_ui_texts.contains_key(&current_language_code) {
            current_language_code.clone()
        } else {
            language_ui_texts
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "en_us".to_string())
        };
        let root_state = LanguageRootState {
            language_ui_texts,
            use_language_code: current_language_code,
            selected_language_code,
            pages: 1,
            page: 1,
            user_page: 0,
            jump: false,
        };
        let lua_state = LanguageLuaState::from_root_state(&root_state);
        Self {
            root_state,
            lua_state,
        }
    }

    /// 进入 Language 页面时重置 Lua state。
    pub fn reset_lua_state(&mut self) {
        if self
            .root_state
            .language_ui_texts
            .contains_key(&self.root_state.use_language_code)
        {
            self.root_state.selected_language_code = self.root_state.use_language_code.clone();
        }
        self.root_state.page = 1;
        self.root_state.user_page = 0;
        self.root_state.jump = false;
        self.lua_state = LanguageLuaState::from_root_state(&self.root_state);
    }

    /// 应用 Lua 返回状态。
    pub fn apply_lua_state(&mut self, lua_state: LanguageLuaState) -> LanguageLuaAction {
        self.lua_state = lua_state;
        if self
            .root_state
            .language_ui_texts
            .contains_key(&self.lua_state.selected_language_code)
        {
            self.root_state.selected_language_code = self.lua_state.selected_language_code.clone();
        }
        self.root_state.pages = self.lua_state.pages.max(1);
        self.root_state.page = self.lua_state.page.clamp(1, self.root_state.pages);
        self.root_state.user_page = if self.lua_state.jump {
            self.lua_state.user_page
        } else {
            0
        };
        self.root_state.jump = self.lua_state.jump;

        if self.lua_state.back {
            self.lua_state.back = false;
            return LanguageLuaAction::Back;
        }

        if self.lua_state.confirm {
            self.lua_state.confirm = false;
            self.root_state.use_language_code = self.root_state.selected_language_code.clone();
            return LanguageLuaAction::Confirm(self.root_state.use_language_code.clone());
        }

        LanguageLuaAction::None
    }
}

/// Language Lua 返回动作。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LanguageLuaAction {
    None,
    Back,
    Confirm(String),
}

/// Language 页面 Lua 运行状态。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanguageLuaState {
    pub selected_language_code: String,
    pub confirm: bool,
    pub back: bool,
    pub pages: i64,
    pub page: i64,
    pub user_page: i64,
    pub jump: bool,
}

impl LanguageLuaState {
    /// 从 root_state 创建 Lua state。
    pub fn from_root_state(root_state: &LanguageRootState) -> Self {
        Self {
            selected_language_code: root_state.selected_language_code.clone(),
            confirm: false,
            back: false,
            pages: root_state.pages.max(1),
            page: root_state.page.max(1),
            user_page: if root_state.jump {
                root_state.user_page
            } else {
                0
            },
            jump: root_state.jump,
        }
    }

    /// 转为 Lua 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("select", self.selected_language_code.as_str())?;
        table.set("confirm", false)?;
        table.set("back", false)?;
        table.set("pages", self.pages.max(1))?;
        table.set("page", self.page.max(1))?;
        table.set("user_page", if self.jump { self.user_page } else { 0 })?;
        table.set("jump", self.jump)?;
        Ok(table)
    }

    /// 从 Lua 返回值解析。
    pub fn from_lua_value(value: Value) -> mlua::Result<Self> {
        let table = match value {
            Value::Table(table) => table,
            _ => {
                return Err(mlua::Error::external(
                    "language lua state must be returned as table",
                ));
            }
        };
        Ok(Self {
            selected_language_code: table
                .get::<Option<String>>("select")?
                .unwrap_or_else(|| "en_us".to_string()),
            confirm: table.get::<Option<bool>>("confirm")?.unwrap_or(false),
            back: table.get::<Option<bool>>("back")?.unwrap_or(false),
            pages: table.get::<Option<i64>>("pages")?.unwrap_or(1).max(1),
            page: table.get::<Option<i64>>("page")?.unwrap_or(1).max(1),
            user_page: table.get::<Option<i64>>("user_page")?.unwrap_or(0).max(0),
            jump: table.get::<Option<bool>>("jump")?.unwrap_or(false),
        })
    }
}

/// Language 页面宿主根状态。
#[derive(Clone, Debug)]
pub struct LanguageRootState {
    pub language_ui_texts: BTreeMap<String, LanguageUiText>,
    pub use_language_code: String,
    pub selected_language_code: String,
    pub pages: i64,
    pub page: i64,
    pub user_page: i64,
    pub jump: bool,
}

impl LanguageRootState {
    /// 转为 Lua root_state 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set(
            "language",
            language_table(lua, self.selected_language_text())?,
        )?;
        table.set(
            "languages",
            language_name_map(lua, &self.language_ui_texts)?,
        )?;
        table.set(
            "language_order",
            language_order(lua, &self.language_ui_texts)?,
        )?;
        table.set("use", self.use_language_code.as_str())?;
        table.set("select", self.selected_language_code.as_str())?;
        table.set("pages", self.pages.max(1))?;
        table.set("page", self.page.max(1))?;
        table.set("user_page", if self.jump { self.user_page } else { 0 })?;
        table.set("jump", self.jump)?;
        Ok(table)
    }

    fn selected_language_text(&self) -> LanguageUiText {
        self.language_ui_texts
            .get(&self.selected_language_code)
            .or_else(|| self.language_ui_texts.get("en_us"))
            .cloned()
            .unwrap_or_default()
    }
}

fn language_table(lua: &Lua, language_text: LanguageUiText) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("LANGUAGE_UP_OPTION", language_text.key_language_up_option)?;
    table.set(
        "LANGUAGE_DOWN_OPTION",
        language_text.key_language_down_option,
    )?;
    table.set(
        "LANGUAGE_LEFT_OPTION",
        language_text.key_language_left_option,
    )?;
    table.set(
        "LANGUAGE_RIGHT_OPTION",
        language_text.key_language_right_option,
    )?;
    table.set("LANGUAGE_SELECT", language_text.key_language_select)?;
    table.set("LANGUAGE_CONFIRM", language_text.key_language_confirm)?;
    table.set("LANGUAGE_JUMP", language_text.key_language_jump)?;
    table.set("LANGUAGE_PREV_PAGE", language_text.key_language_prev_page)?;
    table.set("LANGUAGE_NEXT_PAGE", language_text.key_language_next_page)?;
    table.set(
        "LANGUAGE_BACK_CANCEL",
        language_text.key_language_back_cancel,
    )?;
    table.set("LANGUAGE_BACK", language_text.key_language_back)?;
    table.set("LANGUAGE_CANCEL", language_text.key_language_cancel)?;
    table.set("LANGUAGE_PAGE", language_text.key_language_page)?;
    table.set("LANGUAGE_FLIP", language_text.key_language_flip)?;
    table.set("LANGUAGE_TITLE", language_text.language_title)?;
    table.set("LANGUAGE_NAME", language_text.language_name)?;
    Ok(table)
}

fn language_name_map(
    lua: &Lua,
    language_ui_texts: &BTreeMap<String, LanguageUiText>,
) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (language_code, language_text) in language_ui_texts {
        table.set(language_code.as_str(), language_text.language_name.as_str())?;
    }
    Ok(table)
}

fn language_order(
    lua: &Lua,
    language_ui_texts: &BTreeMap<String, LanguageUiText>,
) -> mlua::Result<Table> {
    let mut languages = language_ui_texts
        .iter()
        .map(|(language_code, language_text)| {
            (language_code.as_str(), language_text.language_name.as_str())
        })
        .collect::<Vec<_>>();
    languages.sort_by(|left, right| left.1.cmp(right.1).then_with(|| left.0.cmp(right.0)));

    let table = lua.create_table()?;
    for (index, (language_code, _language_name)) in languages.iter().enumerate() {
        table.set(index + 1, *language_code)?;
    }
    Ok(table)
}
