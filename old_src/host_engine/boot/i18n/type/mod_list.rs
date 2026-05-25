//! mod_game_list.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static LIST_TITLE: MutableText = MutableText::new();
pub static INFO_SORT_NAME: MutableText = MutableText::new();
pub static INFO_SORT_AUTHOR: MutableText = MutableText::new();
pub static INFO_SORT_SAFE_MODE: MutableText = MutableText::new();
pub static INFO_SORT_TOGGLE: MutableText = MutableText::new();
pub static INFO_SORT_DEBUG: MutableText = MutableText::new();
pub static INFO_ORDER_ASCENDING: MutableText = MutableText::new();
pub static INFO_ORDER_DESCENDING: MutableText = MutableText::new();
pub static INFO_AUTHOR: MutableText = MutableText::new();
pub static INFO_VERSION: MutableText = MutableText::new();
pub static INFO_BASE: MutableText = MutableText::new();
pub static INFO_SAFE: MutableText = MutableText::new();
pub static INFO_SAFE_SWITCH: MutableText = MutableText::new();
pub static INFO_SAFE_DEBUG: MutableText = MutableText::new();
pub static INFO_SAFE_WRITE: MutableText = MutableText::new();
pub static INFO_SAFE_SAFE_MODE: MutableText = MutableText::new();
pub static INFO_INTRODUCTION: MutableText = MutableText::new();
pub static INFO_TITLE: MutableText = MutableText::new();
pub static STATUS: MutableText = MutableText::new();
pub static NONE_MOD: MutableText = MutableText::new();
pub static NONE_INFO: MutableText = MutableText::new();
pub static TOGGLE_MOD_ON: MutableText = MutableText::new();
pub static TOGGLE_MOD_OFF: MutableText = MutableText::new();
pub static TOGGLE_MOD_ON_BRIEF: MutableText = MutableText::new();
pub static TOGGLE_MOD_OFF_BRIEF: MutableText = MutableText::new();
pub static TOGGLE_WRITE_ON: MutableText = MutableText::new();
pub static TOGGLE_WRITE_OFF: MutableText = MutableText::new();
pub static TOGGLE_DEBUG_ON: MutableText = MutableText::new();
pub static TOGGLE_DEBUG_OFF: MutableText = MutableText::new();
pub static TOGGLE_SAFE_MODE_ON: MutableText = MutableText::new();
pub static TOGGLE_SAFE_MODE_OFF_TEMPORARY: MutableText = MutableText::new();
pub static TOGGLE_SAFE_MODE_OFF_PERMANENT: MutableText = MutableText::new();

/// mod_game_list.* 文本集合
#[derive(Clone, Debug)]
pub struct ModListText {
    pub list_title: String,
    pub info_sort_name: String,
    pub info_sort_author: String,
    pub info_sort_safe_mode: String,
    pub info_sort_toggle: String,
    pub info_sort_debug: String,
    pub info_order_ascending: String,
    pub info_order_descending: String,
    pub info_author: String,
    pub info_version: String,
    pub info_base: String,
    pub info_safe: String,
    pub info_safe_switch: String,
    pub info_safe_debug: String,
    pub info_safe_write: String,
    pub info_safe_safe_mode: String,
    pub info_introduction: String,
    pub info_title: String,
    pub status: String,
    pub none_mod: String,
    pub none_info: String,
    pub toggle_mod_on: String,
    pub toggle_mod_off: String,
    pub toggle_mod_on_brief: String,
    pub toggle_mod_off_brief: String,
    pub toggle_write_on: String,
    pub toggle_write_off: String,
    pub toggle_debug_on: String,
    pub toggle_debug_off: String,
    pub toggle_safe_mode_on: String,
    pub toggle_safe_mode_off_temporary: String,
    pub toggle_safe_mode_off_permanent: String,
}

/// 注册 mod_game_list.* 文本
pub fn register(language_source: &LanguageSource) -> ModListText {
    set_text(&LIST_TITLE, language_source, "mod_game_list.list.title");
    set_text(
        &INFO_SORT_NAME,
        language_source,
        "mod_game_list.info.sort.name",
    );
    set_text(
        &INFO_SORT_AUTHOR,
        language_source,
        "mod_game_list.info.sort.author",
    );
    set_text(
        &INFO_SORT_SAFE_MODE,
        language_source,
        "mod_game_list.info.sort.safe_mode",
    );
    set_text(
        &INFO_SORT_TOGGLE,
        language_source,
        "mod_game_list.info.sort.toggle",
    );
    set_text(
        &INFO_SORT_DEBUG,
        language_source,
        "mod_game_list.info.sort.debug",
    );
    set_text(
        &INFO_ORDER_ASCENDING,
        language_source,
        "mod_game_list.info.order.ascending",
    );
    set_text(
        &INFO_ORDER_DESCENDING,
        language_source,
        "mod_game_list.info.order.descending",
    );
    set_text(&INFO_AUTHOR, language_source, "mod_game_list.info.author");
    set_text(&INFO_VERSION, language_source, "mod_game_list.info.version");
    set_text(&INFO_BASE, language_source, "mod_game_list.info.base");
    set_text(&INFO_SAFE, language_source, "mod_game_list.info.safe");
    set_text(
        &INFO_SAFE_SWITCH,
        language_source,
        "mod_game_list.info.safe.switch",
    );
    set_text(
        &INFO_SAFE_DEBUG,
        language_source,
        "mod_game_list.info.safe.debug",
    );
    set_text(
        &INFO_SAFE_WRITE,
        language_source,
        "mod_game_list.info.safe.write",
    );
    set_text(
        &INFO_SAFE_SAFE_MODE,
        language_source,
        "mod_game_list.info.safe.safe_mode",
    );
    set_text(
        &INFO_INTRODUCTION,
        language_source,
        "mod_game_list.info.introduction",
    );
    set_text(&INFO_TITLE, language_source, "mod_game_list.info.title");
    set_text(&STATUS, language_source, "mod_game_list.status");
    set_text(&NONE_MOD, language_source, "mod_game_list.none.mod");
    set_text(&NONE_INFO, language_source, "mod_game_list.none.info");
    set_text(
        &TOGGLE_MOD_ON,
        language_source,
        "mod_game_list.toggle.mod.on",
    );
    set_text(
        &TOGGLE_MOD_OFF,
        language_source,
        "mod_game_list.toggle.mod.off",
    );
    set_text(
        &TOGGLE_MOD_ON_BRIEF,
        language_source,
        "mod_game_list.toggle.mod.on.brief",
    );
    set_text(
        &TOGGLE_MOD_OFF_BRIEF,
        language_source,
        "mod_game_list.toggle.mod.off.brief",
    );
    set_text(
        &TOGGLE_WRITE_ON,
        language_source,
        "mod_game_list.toggle.write.on",
    );
    set_text(
        &TOGGLE_WRITE_OFF,
        language_source,
        "mod_game_list.toggle.write.off",
    );
    set_text(
        &TOGGLE_DEBUG_ON,
        language_source,
        "mod_game_list.toggle.debug.on",
    );
    set_text(
        &TOGGLE_DEBUG_OFF,
        language_source,
        "mod_game_list.toggle.debug.off",
    );
    set_text(
        &TOGGLE_SAFE_MODE_ON,
        language_source,
        "mod_game_list.toggle.safe_mode.on",
    );
    set_text(
        &TOGGLE_SAFE_MODE_OFF_TEMPORARY,
        language_source,
        "mod_game_list.toggle.safe_mode.off.temporary",
    );
    set_text(
        &TOGGLE_SAFE_MODE_OFF_PERMANENT,
        language_source,
        "mod_game_list.toggle.safe_mode.off.permanent",
    );

    ModListText {
        list_title: text(&LIST_TITLE),
        info_sort_name: text(&INFO_SORT_NAME),
        info_sort_author: text(&INFO_SORT_AUTHOR),
        info_sort_safe_mode: text(&INFO_SORT_SAFE_MODE),
        info_sort_toggle: text(&INFO_SORT_TOGGLE),
        info_sort_debug: text(&INFO_SORT_DEBUG),
        info_order_ascending: text(&INFO_ORDER_ASCENDING),
        info_order_descending: text(&INFO_ORDER_DESCENDING),
        info_author: text(&INFO_AUTHOR),
        info_version: text(&INFO_VERSION),
        info_base: text(&INFO_BASE),
        info_safe: text(&INFO_SAFE),
        info_safe_switch: text(&INFO_SAFE_SWITCH),
        info_safe_debug: text(&INFO_SAFE_DEBUG),
        info_safe_write: text(&INFO_SAFE_WRITE),
        info_safe_safe_mode: text(&INFO_SAFE_SAFE_MODE),
        info_introduction: text(&INFO_INTRODUCTION),
        info_title: text(&INFO_TITLE),
        status: text(&STATUS),
        none_mod: text(&NONE_MOD),
        none_info: text(&NONE_INFO),
        toggle_mod_on: text(&TOGGLE_MOD_ON),
        toggle_mod_off: text(&TOGGLE_MOD_OFF),
        toggle_mod_on_brief: text(&TOGGLE_MOD_ON_BRIEF),
        toggle_mod_off_brief: text(&TOGGLE_MOD_OFF_BRIEF),
        toggle_write_on: text(&TOGGLE_WRITE_ON),
        toggle_write_off: text(&TOGGLE_WRITE_OFF),
        toggle_debug_on: text(&TOGGLE_DEBUG_ON),
        toggle_debug_off: text(&TOGGLE_DEBUG_OFF),
        toggle_safe_mode_on: text(&TOGGLE_SAFE_MODE_ON),
        toggle_safe_mode_off_temporary: text(&TOGGLE_SAFE_MODE_OFF_TEMPORARY),
        toggle_safe_mode_off_permanent: text(&TOGGLE_SAFE_MODE_OFF_PERMANENT),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
