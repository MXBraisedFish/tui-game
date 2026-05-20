//! UI Memory 状态聚合

use crate::host_engine::boot::environment::data_dirs;
use std::fs;
use std::path::{Path, PathBuf};

use mlua::{Lua, Table, Value};

use crate::host_engine::boot::i18n;

/// Memory 页面选项数量。
pub const MEMORY_OPTION_COUNT: i64 = 3;

/// Memory 页面确认动作。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryConfirmAction {
    ClearCache,
    ClearData,
    ShowStorageDetails,
}

/// Memory 页面宿主与 Lua 双层状态。
#[derive(Clone, Debug)]
pub struct MemoryUiState {
    pub root_state: MemoryRootState,
    pub lua_state: MemoryLuaState,
}

impl MemoryUiState {
    /// 创建初始 Memory 状态。
    pub fn new() -> Self {
        let root_state = MemoryRootState::new(1);
        let lua_state = MemoryLuaState::new(root_state.select);
        Self {
            root_state,
            lua_state,
        }
    }

    /// 进入 Memory 页面时刷新目录快照并重置 Lua state。
    pub fn reset_lua_state(&mut self) {
        let select = self.root_state.select;
        self.root_state = MemoryRootState::new(select);
        self.root_state.normalize_select();
        self.lua_state = MemoryLuaState::new(self.root_state.select);
    }

    /// 刷新 Memory 页面语言文本。
    pub fn refresh_language(&mut self) {
        self.root_state.language = memory_language_pairs();
    }

    /// 应用 Lua 返回状态。
    pub fn apply_lua_state(&mut self, lua_state: MemoryLuaState) -> MemoryLuaAction {
        self.lua_state = lua_state;
        self.root_state.select = self.lua_state.select;
        self.root_state.normalize_select();

        if self.lua_state.back {
            self.lua_state.back = false;
            return MemoryLuaAction::Back;
        }

        if self.lua_state.confirm {
            self.lua_state.confirm = false;
            return MemoryLuaAction::Confirm(self.root_state.confirm_action());
        }

        MemoryLuaAction::None
    }
}

/// Memory Lua 返回动作。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryLuaAction {
    None,
    Back,
    Confirm(MemoryConfirmAction),
}

/// Memory 页面 Lua 运行状态。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryLuaState {
    pub select: i64,
    pub confirm: bool,
    pub back: bool,
}

impl MemoryLuaState {
    /// 创建新的 Lua state，确认与返回状态总是重置为 false。
    pub fn new(select: i64) -> Self {
        Self {
            select: normalize_memory_select(select),
            confirm: false,
            back: false,
        }
    }

    /// 转为 Lua 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("select", normalize_memory_select(self.select))?;
        table.set("confirm", false)?;
        table.set("back", false)?;
        Ok(table)
    }

    /// 从 Lua 返回值解析。
    pub fn from_lua_value(value: Value) -> mlua::Result<Self> {
        let table = match value {
            Value::Table(table) => table,
            _ => {
                return Err(mlua::Error::external(
                    "memory lua state must be returned as table",
                ));
            }
        };
        let select = table.get::<Option<i64>>("select")?.unwrap_or(1);
        let confirm = table.get::<Option<bool>>("confirm")?.unwrap_or(false);
        let back = table.get::<Option<bool>>("back")?.unwrap_or(false);
        Ok(Self {
            select: normalize_memory_select(select),
            confirm,
            back,
        })
    }
}

/// Memory 页面宿主根状态。
#[derive(Clone, Debug)]
pub struct MemoryRootState {
    pub language: Vec<(String, String)>,
    pub select: i64,
    pub directories: MemoryDirectories,
    pub sizes: MemorySizes,
}

impl MemoryRootState {
    /// 创建新的 Memory root state。
    pub fn new(select: i64) -> Self {
        let directories = MemoryDirectories::from_root(data_dirs::root_dir());
        let sizes = MemorySizes::from_directories(&directories);
        Self {
            language: memory_language_pairs(),
            select: normalize_memory_select(select),
            directories,
            sizes,
        }
    }

    /// 规范化当前选项。
    pub fn normalize_select(&mut self) {
        self.select = normalize_memory_select(self.select);
    }

    /// 选中项转为确认动作。
    pub fn confirm_action(&self) -> MemoryConfirmAction {
        match normalize_memory_select(self.select) {
            1 => MemoryConfirmAction::ClearCache,
            2 => MemoryConfirmAction::ClearData,
            3 => MemoryConfirmAction::ShowStorageDetails,
            _ => MemoryConfirmAction::ClearCache,
        }
    }

    /// 转为 Lua root_state 表。
    pub fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("language", pairs_to_table(lua, &self.language)?)?;
        table.set("select", normalize_memory_select(self.select))?;
        table.set("dir", self.directories.to_lua_table(lua)?)?;
        table.set("size", self.sizes.to_lua_table(lua)?)?;
        Ok(table)
    }
}

/// Memory 页面目录数据。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryDirectories {
    pub root_dir: PathBuf,
    pub data_dir: PathBuf,
    pub profiles_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: PathBuf,
    pub mod_dir: PathBuf,
}

impl MemoryDirectories {
    fn from_root(root_dir: PathBuf) -> Self {
        let data_dir = root_dir.join("data");
        Self {
            root_dir,
            profiles_dir: data_dir.join("profiles"),
            cache_dir: data_dir.join("cache"),
            log_dir: data_dir.join("log"),
            mod_dir: data_dir.join("mod"),
            data_dir,
        }
    }

    fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("root_dir", self.root_dir.display().to_string())?;
        table.set("data_dir", self.data_dir.display().to_string())?;
        table.set("profiles_dir", self.profiles_dir.display().to_string())?;
        table.set("cache_dir", self.cache_dir.display().to_string())?;
        table.set("log_dir", self.log_dir.display().to_string())?;
        table.set("mod_dir", self.mod_dir.display().to_string())?;
        Ok(table)
    }
}

/// Memory 页面目录大小数据。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemorySizes {
    pub root_size: u64,
    pub data_size: u64,
    pub profiles_size: u64,
    pub cache_size: u64,
    pub log_size: u64,
    pub mod_size: u64,
}

impl MemorySizes {
    fn from_directories(directories: &MemoryDirectories) -> Self {
        Self {
            root_size: directory_size(&directories.root_dir),
            data_size: directory_size(&directories.data_dir),
            profiles_size: directory_size(&directories.profiles_dir),
            cache_size: directory_size(&directories.cache_dir),
            log_size: directory_size(&directories.log_dir),
            mod_size: directory_size(&directories.mod_dir),
        }
    }

    fn to_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("root_size", self.root_size)?;
        table.set("data_size", self.data_size)?;
        table.set("profiles_size", self.profiles_size)?;
        table.set("cache_size", self.cache_size)?;
        table.set("log_size", self.log_size)?;
        table.set("mod_size", self.mod_size)?;
        Ok(table)
    }
}

/// 将 Memory 选项限制在 1-3。
pub fn normalize_memory_select(select: i64) -> i64 {
    if select < 1 {
        MEMORY_OPTION_COUNT
    } else if select > MEMORY_OPTION_COUNT {
        1
    } else {
        select
    }
}

fn memory_language_pairs() -> Vec<(String, String)> {
    let text = i18n::text();
    vec![
        (
            "MEMORY_PREV_OPTION".to_string(),
            text.key.memory_prev_option,
        ),
        (
            "MEMORY_NEXT_OPTION".to_string(),
            text.key.memory_next_option,
        ),
        ("MEMORY_SELECT".to_string(), text.key.memory_select),
        ("MEMORY_OPTION1".to_string(), text.key.memory_option1),
        ("MEMORY_OPTION2".to_string(), text.key.memory_option2),
        ("MEMORY_OPTION3".to_string(), text.key.memory_option3),
        ("MEMORY_CONFIRM".to_string(), text.key.memory_confirm),
        ("MEMORY_BACK".to_string(), text.key.memory_back),
        (
            "STORAGE_DETAILS_BACK".to_string(),
            text.key.storage_details_back,
        ),
        ("MEMORY_TITLE".to_string(), text.memory.title),
        ("MEMORY_CACHE".to_string(), text.memory.cache),
        ("MEMORY_DATA".to_string(), text.memory.data),
        ("MEMORY_SHOW".to_string(), text.memory.show),
        ("MEMORY_INFO_DIR".to_string(), text.memory.info_dir),
        ("MEMORY_INFO_SIZE".to_string(), text.memory.info_size),
        ("MEMORY_INFO_PATH".to_string(), text.memory.info_path),
        (
            "MEMORY_INFO_NAME_ROOT".to_string(),
            text.memory.info_name_root,
        ),
        (
            "MEMORY_INFO_NAME_DATA".to_string(),
            text.memory.info_name_data,
        ),
        (
            "MEMORY_INFO_NAME_CACHE".to_string(),
            text.memory.info_name_cache,
        ),
        (
            "MEMORY_INFO_NAME_PROFILES".to_string(),
            text.memory.info_name_profiles,
        ),
        (
            "MEMORY_INFO_NAME_LOG".to_string(),
            text.memory.info_name_log,
        ),
        (
            "MEMORY_INFO_NAME_MOD".to_string(),
            text.memory.info_name_mod,
        ),
        ("MEMORY_TIP".to_string(), text.memory.tip),
    ]
}

fn pairs_to_table(lua: &Lua, pairs: &[(String, String)]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (key, value) in pairs {
        table.set(key.as_str(), value.as_str())?;
    }
    Ok(table)
}

fn directory_size(path: &Path) -> u64 {
    if path.is_file() {
        return fs::metadata(path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
    }
    if !path.is_dir() {
        return 0;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };

    entries
        .filter_map(Result::ok)
        .map(|entry| directory_size(&entry.path()))
        .sum()
}
