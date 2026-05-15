//! setting_keybind.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static LIST_GLOBAL: MutableText = MutableText::new();
pub static LIST_SYSTEM: MutableText = MutableText::new();
pub static LIST_GAME: MutableText = MutableText::new();
pub static SYSTEM_LIST_TITLE: MutableText = MutableText::new();
pub static SYSTEM_KEY_TITLE: MutableText = MutableText::new();
pub static SYSTEM_KEY_ANY: MutableText = MutableText::new();
pub static SYSTEM_SORT_NAME: MutableText = MutableText::new();
pub static SYSTEM_SORT_CONFLICT: MutableText = MutableText::new();
pub static SYSTEM_ORDER_ASCENDING: MutableText = MutableText::new();
pub static SYSTEM_ORDER_DESCENDING: MutableText = MutableText::new();
pub static SYSTEM_TABLE_ACTION: MutableText = MutableText::new();
pub static SYSTEM_TABLE_KEY1: MutableText = MutableText::new();
pub static SYSTEM_TABLE_KEY2: MutableText = MutableText::new();
pub static SYSTEM_TABLE_KEY3: MutableText = MutableText::new();
pub static SYSTEM_TABLE_KEY4: MutableText = MutableText::new();
pub static SYSTEM_CASE_SENSITIVE: MutableText = MutableText::new();
pub static SYSTEM_PAGE_HOME: MutableText = MutableText::new();
pub static SYSTEM_PAGE_SETTING: MutableText = MutableText::new();
pub static SYSTEM_PAGE_GAME_LIST: MutableText = MutableText::new();
pub static SYSTEM_PAGE_STORAGE_DETAILS: MutableText = MutableText::new();
pub static SYSTEM_PAGE_SETTING_KEYBIND: MutableText = MutableText::new();
pub static SYSTEM_PAGE_SETTING_MEMORY: MutableText = MutableText::new();
pub static SYSTEM_PAGE_SETTING_LANGUAGE: MutableText = MutableText::new();
pub static SYSTEM_PAGE_SETTING_MODS: MutableText = MutableText::new();
pub static SYSTEM_PAGE_SETTING_SECURITY: MutableText = MutableText::new();
pub static SYSTEM_PAGE_KEYBIND_SYSTEM: MutableText = MutableText::new();
pub static SYSTEM_PAGE_KEYBIND_GLOBAL: MutableText = MutableText::new();
pub static SYSTEM_PAGE_KEYBIND_GAME: MutableText = MutableText::new();

/// setting_keybind.* 文本集合
#[derive(Clone, Debug)]
pub struct SettingKeybindText {
    pub list_global: String,
    pub list_system: String,
    pub list_game: String,
    pub system_list_title: String,
    pub system_key_title: String,
    pub system_key_any: String,
    pub system_sort_name: String,
    pub system_sort_conflict: String,
    pub system_order_ascending: String,
    pub system_order_descending: String,
    pub system_table_action: String,
    pub system_table_key1: String,
    pub system_table_key2: String,
    pub system_table_key3: String,
    pub system_table_key4: String,
    pub system_case_sensitive: String,
    pub system_page_home: String,
    pub system_page_setting: String,
    pub system_page_game_list: String,
    pub system_page_storage_details: String,
    pub system_page_setting_keybind: String,
    pub system_page_setting_memory: String,
    pub system_page_setting_language: String,
    pub system_page_setting_mods: String,
    pub system_page_setting_security: String,
    pub system_page_keybind_system: String,
    pub system_page_keybind_global: String,
    pub system_page_keybind_game: String,
}

/// 注册 setting_keybind.* 文本
pub fn register(language_source: &LanguageSource) -> SettingKeybindText {
    set_text(&LIST_GLOBAL, language_source, "setting_keybind.list.global");
    set_text(&LIST_SYSTEM, language_source, "setting_keybind.list.system");
    set_text(&LIST_GAME, language_source, "setting_keybind.list.game");
    set_text(
        &SYSTEM_LIST_TITLE,
        language_source,
        "setting_keybind.system.list.title",
    );
    set_text(
        &SYSTEM_KEY_TITLE,
        language_source,
        "setting_keybind.system.key.title",
    );
    set_text(
        &SYSTEM_KEY_ANY,
        language_source,
        "setting_keybind.system.key.any",
    );
    set_text(
        &SYSTEM_SORT_NAME,
        language_source,
        "setting_keybind.system.sort.name",
    );
    set_text(
        &SYSTEM_SORT_CONFLICT,
        language_source,
        "setting_keybind.system.sort.conflict",
    );
    set_text(
        &SYSTEM_ORDER_ASCENDING,
        language_source,
        "setting_keybind.system.order.ascending",
    );
    set_text(
        &SYSTEM_ORDER_DESCENDING,
        language_source,
        "setting_keybind.system.order.descending",
    );
    set_text(
        &SYSTEM_TABLE_ACTION,
        language_source,
        "setting_keybind.system.table.action",
    );
    set_text(
        &SYSTEM_TABLE_KEY1,
        language_source,
        "setting_keybind.system.table.key1",
    );
    set_text(
        &SYSTEM_TABLE_KEY2,
        language_source,
        "setting_keybind.system.table.key2",
    );
    set_text(
        &SYSTEM_TABLE_KEY3,
        language_source,
        "setting_keybind.system.table.key3",
    );
    set_text(
        &SYSTEM_TABLE_KEY4,
        language_source,
        "setting_keybind.system.table.key4",
    );
    set_text(
        &SYSTEM_CASE_SENSITIVE,
        language_source,
        "setting_keybind.system.case_sensitive",
    );
    set_text(
        &SYSTEM_PAGE_HOME,
        language_source,
        "setting_keybind.system.page.home",
    );
    set_text(
        &SYSTEM_PAGE_SETTING,
        language_source,
        "setting_keybind.system.page.setting",
    );
    set_text(
        &SYSTEM_PAGE_GAME_LIST,
        language_source,
        "setting_keybind.system.page.game_list",
    );
    set_text(
        &SYSTEM_PAGE_STORAGE_DETAILS,
        language_source,
        "setting_keybind.system.page.storage_details",
    );
    set_text(
        &SYSTEM_PAGE_SETTING_KEYBIND,
        language_source,
        "setting_keybind.system.page.setting_keybind",
    );
    set_text(
        &SYSTEM_PAGE_SETTING_MEMORY,
        language_source,
        "setting_keybind.system.page.setting_memory",
    );
    set_text(
        &SYSTEM_PAGE_SETTING_LANGUAGE,
        language_source,
        "setting_keybind.system.page.setting_language",
    );
    set_text(
        &SYSTEM_PAGE_SETTING_MODS,
        language_source,
        "setting_keybind.system.page.setting_mods",
    );
    set_text(
        &SYSTEM_PAGE_SETTING_SECURITY,
        language_source,
        "setting_keybind.system.page.setting_security",
    );
    set_text(
        &SYSTEM_PAGE_KEYBIND_SYSTEM,
        language_source,
        "setting_keybind.system.page.keybind_system",
    );
    set_text(
        &SYSTEM_PAGE_KEYBIND_GLOBAL,
        language_source,
        "setting_keybind.system.page.keybind_global",
    );
    set_text(
        &SYSTEM_PAGE_KEYBIND_GAME,
        language_source,
        "setting_keybind.system.page.keybind_game",
    );

    SettingKeybindText {
        list_global: text(&LIST_GLOBAL),
        list_system: text(&LIST_SYSTEM),
        list_game: text(&LIST_GAME),
        system_list_title: text(&SYSTEM_LIST_TITLE),
        system_key_title: text(&SYSTEM_KEY_TITLE),
        system_key_any: text(&SYSTEM_KEY_ANY),
        system_sort_name: text(&SYSTEM_SORT_NAME),
        system_sort_conflict: text(&SYSTEM_SORT_CONFLICT),
        system_order_ascending: text(&SYSTEM_ORDER_ASCENDING),
        system_order_descending: text(&SYSTEM_ORDER_DESCENDING),
        system_table_action: text(&SYSTEM_TABLE_ACTION),
        system_table_key1: text(&SYSTEM_TABLE_KEY1),
        system_table_key2: text(&SYSTEM_TABLE_KEY2),
        system_table_key3: text(&SYSTEM_TABLE_KEY3),
        system_table_key4: text(&SYSTEM_TABLE_KEY4),
        system_case_sensitive: text(&SYSTEM_CASE_SENSITIVE),
        system_page_home: text(&SYSTEM_PAGE_HOME),
        system_page_setting: text(&SYSTEM_PAGE_SETTING),
        system_page_game_list: text(&SYSTEM_PAGE_GAME_LIST),
        system_page_storage_details: text(&SYSTEM_PAGE_STORAGE_DETAILS),
        system_page_setting_keybind: text(&SYSTEM_PAGE_SETTING_KEYBIND),
        system_page_setting_memory: text(&SYSTEM_PAGE_SETTING_MEMORY),
        system_page_setting_language: text(&SYSTEM_PAGE_SETTING_LANGUAGE),
        system_page_setting_mods: text(&SYSTEM_PAGE_SETTING_MODS),
        system_page_setting_security: text(&SYSTEM_PAGE_SETTING_SECURITY),
        system_page_keybind_system: text(&SYSTEM_PAGE_KEYBIND_SYSTEM),
        system_page_keybind_global: text(&SYSTEM_PAGE_KEYBIND_GLOBAL),
        system_page_keybind_game: text(&SYSTEM_PAGE_KEYBIND_GAME),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
