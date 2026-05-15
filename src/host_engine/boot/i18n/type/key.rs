//! key.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static HOME_PREV_OPTION: MutableText = MutableText::new();
pub static HOME_NEXT_OPTION: MutableText = MutableText::new();
pub static HOME_SELECT: MutableText = MutableText::new();
pub static HOME_CONFIRM: MutableText = MutableText::new();
pub static HOME_OPTION1: MutableText = MutableText::new();
pub static HOME_OPTION2: MutableText = MutableText::new();
pub static HOME_OPTION3: MutableText = MutableText::new();
pub static HOME_OPTION4: MutableText = MutableText::new();
pub static HOME_OPTION5: MutableText = MutableText::new();
pub static SETTING_PREV_OPTION: MutableText = MutableText::new();
pub static SETTING_NEXT_OPTION: MutableText = MutableText::new();
pub static SETTING_SELECT: MutableText = MutableText::new();
pub static SETTING_CONFIRM: MutableText = MutableText::new();
pub static SETTING_OPTION1: MutableText = MutableText::new();
pub static SETTING_OPTION2: MutableText = MutableText::new();
pub static SETTING_OPTION3: MutableText = MutableText::new();
pub static SETTING_OPTION4: MutableText = MutableText::new();
pub static SETTING_OPTION5: MutableText = MutableText::new();
pub static SETTING_BACK: MutableText = MutableText::new();
pub static GAME_LIST_PREV_OPTION: MutableText = MutableText::new();
pub static GAME_LIST_NEXT_OPTION: MutableText = MutableText::new();
pub static GAME_LIST_PREV_PAGE: MutableText = MutableText::new();
pub static GAME_LIST_NEXT_PAGE: MutableText = MutableText::new();
pub static GAME_LIST_SCROLL_UP: MutableText = MutableText::new();
pub static GAME_LIST_SCROLL_DOWN: MutableText = MutableText::new();
pub static GAME_LIST_JUMP: MutableText = MutableText::new();
pub static GAME_LIST_ORDER: MutableText = MutableText::new();
pub static GAME_LIST_SORT: MutableText = MutableText::new();
pub static GAME_LIST_BACK: MutableText = MutableText::new();
pub static GAME_LIST_BACK_CANCEL: MutableText = MutableText::new();
pub static GAME_LIST_START_CONFIRM: MutableText = MutableText::new();
pub static GAME_LIST_START: MutableText = MutableText::new();
pub static GAME_LIST_CONFIRM: MutableText = MutableText::new();
pub static GAME_LIST_CANCEL: MutableText = MutableText::new();
pub static GAME_LIST_SELECT: MutableText = MutableText::new();
pub static GAME_LIST_FLIP: MutableText = MutableText::new();
pub static GAME_LIST_SCROLL: MutableText = MutableText::new();
pub static MOD_PREV_OPTION: MutableText = MutableText::new();
pub static MOD_NEXT_OPTION: MutableText = MutableText::new();
pub static MOD_LIST_OPTION1: MutableText = MutableText::new();
pub static MOD_LIST_OPTION2: MutableText = MutableText::new();
pub static MOD_LIST_OPTION3: MutableText = MutableText::new();
pub static MOD_HUB_SELECT: MutableText = MutableText::new();
pub static MOD_HUB_CONFIRM: MutableText = MutableText::new();
pub static MOD_HUB_BACK: MutableText = MutableText::new();
pub static MOD_HUB_TITLE: MutableText = MutableText::new();
pub static MOD_LIST_PREV_OPTION: MutableText = MutableText::new();
pub static MOD_LIST_NEXT_OPTION: MutableText = MutableText::new();
pub static MOD_LIST_PREV_PAGE: MutableText = MutableText::new();
pub static MOD_LIST_NEXT_PAGE: MutableText = MutableText::new();
pub static MOD_LIST_SCROLL_UP: MutableText = MutableText::new();
pub static MOD_LIST_SCROLL_DOWN: MutableText = MutableText::new();
pub static MOD_LIST_JUMP: MutableText = MutableText::new();
pub static MOD_LIST_ORDER: MutableText = MutableText::new();
pub static MOD_LIST_SORT: MutableText = MutableText::new();
pub static MOD_LIST_BACK: MutableText = MutableText::new();
pub static MOD_LIST_TOGGLE_CONFIRM: MutableText = MutableText::new();
pub static MOD_LIST_BACK_CANCEL: MutableText = MutableText::new();
pub static MOD_LIST_TOGGLE: MutableText = MutableText::new();
pub static MOD_LIST_CONFIRM: MutableText = MutableText::new();
pub static MOD_LIST_CANCEL: MutableText = MutableText::new();
pub static MOD_LIST_SELECT: MutableText = MutableText::new();
pub static MOD_LIST_FLIP: MutableText = MutableText::new();
pub static MOD_LIST_SCROLL: MutableText = MutableText::new();
pub static MOD_LIST_DEBUG: MutableText = MutableText::new();
pub static MOD_LIST_LIST: MutableText = MutableText::new();
pub static MOD_LIST_SAFE_MODE: MutableText = MutableText::new();
pub static MOD_SECURITY_CLOSE_PERMANENT: MutableText = MutableText::new();
pub static MOD_SECURITY_CLOSE_TEMPORARY: MutableText = MutableText::new();
pub static MOD_SECURITY_CANCEL: MutableText = MutableText::new();
pub static DEFAULT_SECURITY_CLOSE_PERMANENT: MutableText = MutableText::new();
pub static DEFAULT_SECURITY_CANCEL: MutableText = MutableText::new();
pub static SECURITY_PREV_OPTION: MutableText = MutableText::new();
pub static SECURITY_NEXT_OPTION: MutableText = MutableText::new();
pub static SECURITY_SELECT: MutableText = MutableText::new();
pub static SECURITY_CLOSE_PERMANENT: MutableText = MutableText::new();
pub static SECURITY_BACK: MutableText = MutableText::new();
pub static SECURITY_TOGGLE_CONFIRM: MutableText = MutableText::new();
pub static SECURITY_TOGGLE: MutableText = MutableText::new();
pub static SECURITY_CONFIRM: MutableText = MutableText::new();
pub static SECURITY_OPTION1: MutableText = MutableText::new();
pub static SECURITY_OPTION2: MutableText = MutableText::new();
pub static SECURITY_OPTION3: MutableText = MutableText::new();
pub static SECURITY_OPTION4: MutableText = MutableText::new();
pub static SECURITY_OPTION5: MutableText = MutableText::new();
pub static SECURITY_OPTION6: MutableText = MutableText::new();
pub static SECURITY_OPTION7: MutableText = MutableText::new();
pub static SECURITY_OPTION8: MutableText = MutableText::new();
pub static LANGUAGE_UP_OPTION: MutableText = MutableText::new();
pub static LANGUAGE_DOWN_OPTION: MutableText = MutableText::new();
pub static LANGUAGE_LEFT_OPTION: MutableText = MutableText::new();
pub static LANGUAGE_RIGHT_OPTION: MutableText = MutableText::new();
pub static LANGUAGE_SELECT: MutableText = MutableText::new();
pub static LANGUAGE_CONFIRM: MutableText = MutableText::new();
pub static LANGUAGE_JUMP: MutableText = MutableText::new();
pub static LANGUAGE_PREV_PAGE: MutableText = MutableText::new();
pub static LANGUAGE_NEXT_PAGE: MutableText = MutableText::new();
pub static LANGUAGE_BACK_CANCEL: MutableText = MutableText::new();
pub static LANGUAGE_BACK: MutableText = MutableText::new();
pub static LANGUAGE_CANCEL: MutableText = MutableText::new();
pub static LANGUAGE_PAGE: MutableText = MutableText::new();
pub static LANGUAGE_FLIP: MutableText = MutableText::new();
pub static MEMORY_PREV_OPTION: MutableText = MutableText::new();
pub static MEMORY_NEXT_OPTION: MutableText = MutableText::new();
pub static MEMORY_SELECT: MutableText = MutableText::new();
pub static MEMORY_OPTION1: MutableText = MutableText::new();
pub static MEMORY_OPTION2: MutableText = MutableText::new();
pub static MEMORY_OPTION3: MutableText = MutableText::new();
pub static MEMORY_CONFIRM: MutableText = MutableText::new();
pub static MEMORY_BACK: MutableText = MutableText::new();
pub static SETTING_KEYBIND_LIST_OPTION1: MutableText = MutableText::new();
pub static SETTING_KEYBIND_LIST_OPTION2: MutableText = MutableText::new();
pub static SETTING_KEYBIND_LIST_OPTION3: MutableText = MutableText::new();
pub static SETTING_KEYBIND_LIST_PREV_OPTION: MutableText = MutableText::new();
pub static SETTING_KEYBIND_LIST_NEXT_OPTION: MutableText = MutableText::new();
pub static SETTING_KEYBIND_LIST_SELECT: MutableText = MutableText::new();
pub static SETTING_KEYBIND_LIST_CONFIRM: MutableText = MutableText::new();
pub static SETTING_KEYBIND_LIST_BACK: MutableText = MutableText::new();
pub static SETTING_KEYBIND_LIST_TITLE: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_PREV_OPTION: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_NEXT_OPTION: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_SELECT: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_PREV_PAGE: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_NEXT_PAGE: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_SCROLL_UP: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_SCROLL_DOWN: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_SCROLL: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_JUMP: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_ORDER: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_SORT: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_CONFIRM: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_LIST_BACK: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_BACK: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_LIST: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_KEY1: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_KEY2: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_KEY3: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_KEY4: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_TIP_DELETE: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_TIP_ADD_MODIFY: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_ADD: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_MODIFY: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_ADD_SHIFT: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_MODIFY_SHIFT: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_DELETE: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_KEY_MODE: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_RESET_ONLY: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_RESET_GAME: MutableText = MutableText::new();
pub static SETTING_KEYBIND_SYSTEM_RESET_PAGE: MutableText = MutableText::new();
pub static STORAGE_DETAILS_BACK: MutableText = MutableText::new();
pub static CLEAR_DATA_CONFIRM: MutableText = MutableText::new();
pub static CLEAR_DATA_CANCEL: MutableText = MutableText::new();
pub static CLEAR_CACHE_CONFIRM: MutableText = MutableText::new();
pub static CLEAR_CACHE_CANCEL: MutableText = MutableText::new();
pub static SIZE_RETURN: MutableText = MutableText::new();

/// key.* 文本集合
#[derive(Clone, Debug)]
pub struct KeyText {
    pub home_prev_option: String,
    pub home_next_option: String,
    pub home_select: String,
    pub home_confirm: String,
    pub home_option1: String,
    pub home_option2: String,
    pub home_option3: String,
    pub home_option4: String,
    pub home_option5: String,
    pub setting_prev_option: String,
    pub setting_next_option: String,
    pub setting_select: String,
    pub setting_confirm: String,
    pub setting_option1: String,
    pub setting_option2: String,
    pub setting_option3: String,
    pub setting_option4: String,
    pub setting_option5: String,
    pub setting_back: String,
    pub game_list_prev_option: String,
    pub game_list_next_option: String,
    pub game_list_prev_page: String,
    pub game_list_next_page: String,
    pub game_list_scroll_up: String,
    pub game_list_scroll_down: String,
    pub game_list_jump: String,
    pub game_list_order: String,
    pub game_list_sort: String,
    pub game_list_back: String,
    pub game_list_back_cancel: String,
    pub game_list_start_confirm: String,
    pub game_list_start: String,
    pub game_list_confirm: String,
    pub game_list_cancel: String,
    pub game_list_select: String,
    pub game_list_flip: String,
    pub game_list_scroll: String,
    pub mod_prev_option: String,
    pub mod_next_option: String,
    pub mod_list_option1: String,
    pub mod_list_option2: String,
    pub mod_list_option3: String,
    pub mod_hub_select: String,
    pub mod_hub_confirm: String,
    pub mod_hub_back: String,
    pub mod_hub_title: String,
    pub mod_list_prev_option: String,
    pub mod_list_next_option: String,
    pub mod_list_prev_page: String,
    pub mod_list_next_page: String,
    pub mod_list_scroll_up: String,
    pub mod_list_scroll_down: String,
    pub mod_list_jump: String,
    pub mod_list_order: String,
    pub mod_list_sort: String,
    pub mod_list_back: String,
    pub mod_list_toggle_confirm: String,
    pub mod_list_back_cancel: String,
    pub mod_list_toggle: String,
    pub mod_list_confirm: String,
    pub mod_list_cancel: String,
    pub mod_list_select: String,
    pub mod_list_flip: String,
    pub mod_list_scroll: String,
    pub mod_list_debug: String,
    pub mod_list_list: String,
    pub mod_list_safe_mode: String,
    pub mod_security_close_permanent: String,
    pub mod_security_close_temporary: String,
    pub mod_security_cancel: String,
    pub default_security_close_permanent: String,
    pub default_security_cancel: String,
    pub security_prev_option: String,
    pub security_next_option: String,
    pub security_select: String,
    pub security_close_permanent: String,
    pub security_back: String,
    pub security_toggle_confirm: String,
    pub security_toggle: String,
    pub security_confirm: String,
    pub security_option1: String,
    pub security_option2: String,
    pub security_option3: String,
    pub security_option4: String,
    pub security_option5: String,
    pub security_option6: String,
    pub security_option7: String,
    pub security_option8: String,
    pub language_up_option: String,
    pub language_down_option: String,
    pub language_left_option: String,
    pub language_right_option: String,
    pub language_select: String,
    pub language_confirm: String,
    pub language_jump: String,
    pub language_prev_page: String,
    pub language_next_page: String,
    pub language_back_cancel: String,
    pub language_back: String,
    pub language_cancel: String,
    pub language_page: String,
    pub language_flip: String,
    pub memory_prev_option: String,
    pub memory_next_option: String,
    pub memory_select: String,
    pub memory_option1: String,
    pub memory_option2: String,
    pub memory_option3: String,
    pub memory_confirm: String,
    pub memory_back: String,
    pub setting_keybind_list_option1: String,
    pub setting_keybind_list_option2: String,
    pub setting_keybind_list_option3: String,
    pub setting_keybind_list_prev_option: String,
    pub setting_keybind_list_next_option: String,
    pub setting_keybind_list_select: String,
    pub setting_keybind_list_confirm: String,
    pub setting_keybind_list_back: String,
    pub setting_keybind_list_title: String,
    pub setting_keybind_system_prev_option: String,
    pub setting_keybind_system_next_option: String,
    pub setting_keybind_system_select: String,
    pub setting_keybind_system_prev_page: String,
    pub setting_keybind_system_next_page: String,
    pub setting_keybind_system_scroll_up: String,
    pub setting_keybind_system_scroll_down: String,
    pub setting_keybind_system_scroll: String,
    pub setting_keybind_system_jump: String,
    pub setting_keybind_system_order: String,
    pub setting_keybind_system_sort: String,
    pub setting_keybind_system_confirm: String,
    pub setting_keybind_system_list_back: String,
    pub setting_keybind_system_back: String,
    pub setting_keybind_system_list: String,
    pub setting_keybind_system_key1: String,
    pub setting_keybind_system_key2: String,
    pub setting_keybind_system_key3: String,
    pub setting_keybind_system_key4: String,
    pub setting_keybind_system_tip_delete: String,
    pub setting_keybind_system_tip_add_modify: String,
    pub setting_keybind_system_add: String,
    pub setting_keybind_system_modify: String,
    pub setting_keybind_system_add_shift: String,
    pub setting_keybind_system_modify_shift: String,
    pub setting_keybind_system_delete: String,
    pub setting_keybind_system_key_mode: String,
    pub setting_keybind_system_reset_only: String,
    pub setting_keybind_system_reset_game: String,
    pub setting_keybind_system_reset_page: String,
    pub storage_details_back: String,
    pub clear_data_confirm: String,
    pub clear_data_cancel: String,
    pub clear_cache_confirm: String,
    pub clear_cache_cancel: String,
    pub size_return: String,
}

/// 注册 key.* 文本
pub fn register(language_source: &LanguageSource) -> KeyText {
    set_text(&HOME_PREV_OPTION, language_source, "key.home.prev_option");
    set_text(&HOME_NEXT_OPTION, language_source, "key.home.next_option");
    set_text(&HOME_SELECT, language_source, "key.home.select");
    set_text(&HOME_CONFIRM, language_source, "key.home.confirm");
    set_text(&HOME_OPTION1, language_source, "key.home.option1");
    set_text(&HOME_OPTION2, language_source, "key.home.option2");
    set_text(&HOME_OPTION3, language_source, "key.home.option3");
    set_text(&HOME_OPTION4, language_source, "key.home.option4");
    set_text(&HOME_OPTION5, language_source, "key.home.option5");
    set_text(
        &SETTING_PREV_OPTION,
        language_source,
        "key.setting.prev_option",
    );
    set_text(
        &SETTING_NEXT_OPTION,
        language_source,
        "key.setting.next_option",
    );
    set_text(&SETTING_SELECT, language_source, "key.setting.select");
    set_text(&SETTING_CONFIRM, language_source, "key.setting.confirm");
    set_text(&SETTING_OPTION1, language_source, "key.setting.option1");
    set_text(&SETTING_OPTION2, language_source, "key.setting.option2");
    set_text(&SETTING_OPTION3, language_source, "key.setting.option3");
    set_text(&SETTING_OPTION4, language_source, "key.setting.option4");
    set_text(&SETTING_OPTION5, language_source, "key.setting.option5");
    set_text(&SETTING_BACK, language_source, "key.setting.back");
    set_text(
        &GAME_LIST_PREV_OPTION,
        language_source,
        "key.game_list.prev_option",
    );
    set_text(
        &GAME_LIST_NEXT_OPTION,
        language_source,
        "key.game_list.next_option",
    );
    set_text(
        &GAME_LIST_PREV_PAGE,
        language_source,
        "key.game_list.prev_page",
    );
    set_text(
        &GAME_LIST_NEXT_PAGE,
        language_source,
        "key.game_list.next_page",
    );
    set_text(
        &GAME_LIST_SCROLL_UP,
        language_source,
        "key.game_list.scroll_up",
    );
    set_text(
        &GAME_LIST_SCROLL_DOWN,
        language_source,
        "key.game_list.scroll_down",
    );
    set_text(&GAME_LIST_JUMP, language_source, "key.game_list.jump");
    set_text(&GAME_LIST_ORDER, language_source, "key.game_list.order");
    set_text(&GAME_LIST_SORT, language_source, "key.game_list.sort");
    set_text(&GAME_LIST_BACK, language_source, "key.game_list.back");
    set_text(
        &GAME_LIST_BACK_CANCEL,
        language_source,
        "key.game_list.back_cancel",
    );
    set_text(
        &GAME_LIST_START_CONFIRM,
        language_source,
        "key.game_list.start_confirm",
    );
    set_text(&GAME_LIST_START, language_source, "key.game_list.start");
    set_text(&GAME_LIST_CONFIRM, language_source, "key.game_list.confirm");
    set_text(&GAME_LIST_CANCEL, language_source, "key.game_list.cancel");
    set_text(&GAME_LIST_SELECT, language_source, "key.game_list.select");
    set_text(&GAME_LIST_FLIP, language_source, "key.game_list.flip");
    set_text(&GAME_LIST_SCROLL, language_source, "key.game_list.scroll");
    set_text(&MOD_PREV_OPTION, language_source, "key.mod.prev_option");
    set_text(&MOD_NEXT_OPTION, language_source, "key.mod.next_option");
    set_text(&MOD_LIST_OPTION1, language_source, "key.mod.list.option1");
    set_text(&MOD_LIST_OPTION2, language_source, "key.mod.list.option2");
    set_text(&MOD_LIST_OPTION3, language_source, "key.mod.list.option3");
    set_text(&MOD_HUB_SELECT, language_source, "key.mod.list.select");
    set_text(&MOD_HUB_CONFIRM, language_source, "key.mod.list.confirm");
    set_text(&MOD_HUB_BACK, language_source, "key.mod.list.back");
    set_text(&MOD_HUB_TITLE, language_source, "key.mod.list.title");
    set_text(
        &MOD_LIST_PREV_OPTION,
        language_source,
        "key.mod_game_list.prev_option",
    );
    set_text(
        &MOD_LIST_NEXT_OPTION,
        language_source,
        "key.mod_game_list.next_option",
    );
    set_text(
        &MOD_LIST_PREV_PAGE,
        language_source,
        "key.mod_game_list.prev_page",
    );
    set_text(
        &MOD_LIST_NEXT_PAGE,
        language_source,
        "key.mod_game_list.next_page",
    );
    set_text(
        &MOD_LIST_SCROLL_UP,
        language_source,
        "key.mod_game_list.scroll_up",
    );
    set_text(
        &MOD_LIST_SCROLL_DOWN,
        language_source,
        "key.mod_game_list.scroll_down",
    );
    set_text(&MOD_LIST_JUMP, language_source, "key.mod_game_list.jump");
    set_text(&MOD_LIST_ORDER, language_source, "key.mod_game_list.order");
    set_text(&MOD_LIST_SORT, language_source, "key.mod_game_list.sort");
    set_text(&MOD_LIST_BACK, language_source, "key.mod_game_list.back");
    set_text(
        &MOD_LIST_TOGGLE_CONFIRM,
        language_source,
        "key.mod_game_list.toggle_confirm",
    );
    set_text(
        &MOD_LIST_BACK_CANCEL,
        language_source,
        "key.mod_game_list.back_cancel",
    );
    set_text(
        &MOD_LIST_TOGGLE,
        language_source,
        "key.mod_game_list.toggle",
    );
    set_text(
        &MOD_LIST_CONFIRM,
        language_source,
        "key.mod_game_list.confirm",
    );
    set_text(
        &MOD_LIST_CANCEL,
        language_source,
        "key.mod_game_list.cancel",
    );
    set_text(
        &MOD_LIST_SELECT,
        language_source,
        "key.mod_game_list.select",
    );
    set_text(&MOD_LIST_FLIP, language_source, "key.mod_game_list.flip");
    set_text(
        &MOD_LIST_SCROLL,
        language_source,
        "key.mod_game_list.scroll",
    );
    set_text(&MOD_LIST_DEBUG, language_source, "key.mod_game_list.debug");
    set_text(&MOD_LIST_LIST, language_source, "key.mod_game_list.list");
    set_text(
        &MOD_LIST_SAFE_MODE,
        language_source,
        "key.mod_game_list.safe_mode",
    );
    set_text(
        &MOD_SECURITY_CLOSE_PERMANENT,
        language_source,
        "key.mod_security.close.permanent",
    );
    set_text(
        &MOD_SECURITY_CLOSE_TEMPORARY,
        language_source,
        "key.mod_security.close.temporary",
    );
    set_text(
        &MOD_SECURITY_CANCEL,
        language_source,
        "key.mod_security.cancel",
    );
    set_text(
        &DEFAULT_SECURITY_CLOSE_PERMANENT,
        language_source,
        "key.default_security.close.permanent",
    );
    set_text(
        &DEFAULT_SECURITY_CANCEL,
        language_source,
        "key.default_security.cancel",
    );
    set_text(
        &SECURITY_PREV_OPTION,
        language_source,
        "key.security.prev_option",
    );
    set_text(
        &SECURITY_NEXT_OPTION,
        language_source,
        "key.security.next_option",
    );
    set_text(&SECURITY_SELECT, language_source, "key.security.select");
    set_text(
        &SECURITY_CLOSE_PERMANENT,
        language_source,
        "key.security.close.permanent",
    );
    set_text(&SECURITY_BACK, language_source, "key.security.back");
    set_text(
        &SECURITY_TOGGLE_CONFIRM,
        language_source,
        "key.security.toggle_confirm",
    );
    set_text(&SECURITY_TOGGLE, language_source, "key.security.toggle");
    set_text(&SECURITY_CONFIRM, language_source, "key.security.confirm");
    set_text(&SECURITY_OPTION1, language_source, "key.security.option1");
    set_text(&SECURITY_OPTION2, language_source, "key.security.option2");
    set_text(&SECURITY_OPTION3, language_source, "key.security.option3");
    set_text(&SECURITY_OPTION4, language_source, "key.security.option4");
    set_text(&SECURITY_OPTION5, language_source, "key.security.option5");
    set_text(&SECURITY_OPTION6, language_source, "key.security.option6");
    set_text(&SECURITY_OPTION7, language_source, "key.security.option7");
    set_text(&SECURITY_OPTION8, language_source, "key.security.option8");
    set_text(
        &LANGUAGE_UP_OPTION,
        language_source,
        "key.language.up_option",
    );
    set_text(
        &LANGUAGE_DOWN_OPTION,
        language_source,
        "key.language.down_option",
    );
    set_text(
        &LANGUAGE_LEFT_OPTION,
        language_source,
        "key.language.left_option",
    );
    set_text(
        &LANGUAGE_RIGHT_OPTION,
        language_source,
        "key.language.right_option",
    );
    set_text(&LANGUAGE_SELECT, language_source, "key.language.select");
    set_text(&LANGUAGE_CONFIRM, language_source, "key.language.confirm");
    set_text(&LANGUAGE_JUMP, language_source, "key.language.jump");
    set_text(
        &LANGUAGE_PREV_PAGE,
        language_source,
        "key.language.prev_page",
    );
    set_text(
        &LANGUAGE_NEXT_PAGE,
        language_source,
        "key.language.next_page",
    );
    set_text(
        &LANGUAGE_BACK_CANCEL,
        language_source,
        "key.language.back_cancel",
    );
    set_text(&LANGUAGE_BACK, language_source, "key.language.back");
    set_text(&LANGUAGE_CANCEL, language_source, "key.language.cancel");
    set_text(&LANGUAGE_PAGE, language_source, "key.language.page");
    set_text(&LANGUAGE_FLIP, language_source, "key.language.flip");
    set_text(
        &MEMORY_PREV_OPTION,
        language_source,
        "key.memory.prev_option",
    );
    set_text(
        &MEMORY_NEXT_OPTION,
        language_source,
        "key.memory.next_option",
    );
    set_text(&MEMORY_SELECT, language_source, "key.memory.select");
    set_text(&MEMORY_OPTION1, language_source, "key.memory.option1");
    set_text(&MEMORY_OPTION2, language_source, "key.memory.option2");
    set_text(&MEMORY_OPTION3, language_source, "key.memory.option3");
    set_text(&MEMORY_CONFIRM, language_source, "key.memory.confirm");
    set_text(&MEMORY_BACK, language_source, "key.memory.back");
    set_text(
        &SETTING_KEYBIND_LIST_OPTION1,
        language_source,
        "key.setting_keybind.list.option1",
    );
    set_text(
        &SETTING_KEYBIND_LIST_OPTION2,
        language_source,
        "key.setting_keybind.list.option2",
    );
    set_text(
        &SETTING_KEYBIND_LIST_OPTION3,
        language_source,
        "key.setting_keybind.list.option3",
    );
    set_text(
        &SETTING_KEYBIND_LIST_PREV_OPTION,
        language_source,
        "key.setting_keybind.list.prev_option",
    );
    set_text(
        &SETTING_KEYBIND_LIST_NEXT_OPTION,
        language_source,
        "key.setting_keybind.list.next_option",
    );
    set_text(
        &SETTING_KEYBIND_LIST_SELECT,
        language_source,
        "key.setting_keybind.list.select",
    );
    set_text(
        &SETTING_KEYBIND_LIST_CONFIRM,
        language_source,
        "key.setting_keybind.list.confirm",
    );
    set_text(
        &SETTING_KEYBIND_LIST_BACK,
        language_source,
        "key.setting_keybind.list.back",
    );
    set_text(
        &SETTING_KEYBIND_LIST_TITLE,
        language_source,
        "key.setting_keybind.list.title",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_PREV_OPTION,
        language_source,
        "key.setting_keybind.system.prev_option",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_NEXT_OPTION,
        language_source,
        "key.setting_keybind.system.next_option",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_SELECT,
        language_source,
        "key.setting_keybind.system.select",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_PREV_PAGE,
        language_source,
        "key.setting_keybind.system.prev_page",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_NEXT_PAGE,
        language_source,
        "key.setting_keybind.system.next_page",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_SCROLL_UP,
        language_source,
        "key.setting_keybind.system.scroll_up",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_SCROLL_DOWN,
        language_source,
        "key.setting_keybind.system.scroll_down",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_SCROLL,
        language_source,
        "key.setting_keybind.system.scroll",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_JUMP,
        language_source,
        "key.setting_keybind.system.jump",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_ORDER,
        language_source,
        "key.setting_keybind.system.order",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_SORT,
        language_source,
        "key.setting_keybind.system.sort",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_CONFIRM,
        language_source,
        "key.setting_keybind.system.confirm",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_LIST_BACK,
        language_source,
        "key.setting_keybind.system.list_back",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_BACK,
        language_source,
        "key.setting_keybind.system.back",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_LIST,
        language_source,
        "key.setting_keybind.system.list",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_KEY1,
        language_source,
        "key.setting_keybind.system.key1",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_KEY2,
        language_source,
        "key.setting_keybind.system.key2",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_KEY3,
        language_source,
        "key.setting_keybind.system.key3",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_KEY4,
        language_source,
        "key.setting_keybind.system.key4",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_TIP_DELETE,
        language_source,
        "key.setting_keybind.system.tip.delete",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_TIP_ADD_MODIFY,
        language_source,
        "key.setting_keybind.system.tip.add_modify",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_ADD,
        language_source,
        "key.setting_keybind.system.add",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_MODIFY,
        language_source,
        "key.setting_keybind.system.modify",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_ADD_SHIFT,
        language_source,
        "key.setting_keybind.system.add.shift",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_MODIFY_SHIFT,
        language_source,
        "key.setting_keybind.system.modify.shift",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_DELETE,
        language_source,
        "key.setting_keybind.system.delete",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_KEY_MODE,
        language_source,
        "key.setting_keybind.system.key_mode",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_RESET_ONLY,
        language_source,
        "key.setting_keybind.system.reset.only",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_RESET_GAME,
        language_source,
        "key.setting_keybind.system.reset.game",
    );
    set_text(
        &SETTING_KEYBIND_SYSTEM_RESET_PAGE,
        language_source,
        "key.setting_keybind.system.reset.page",
    );
    set_text(
        &STORAGE_DETAILS_BACK,
        language_source,
        "key.storage_details.back",
    );
    set_text(
        &CLEAR_DATA_CONFIRM,
        language_source,
        "key.clear_data.confirm",
    );
    set_text(&CLEAR_DATA_CANCEL, language_source, "key.clear_data.cancel");
    set_text(
        &CLEAR_CACHE_CONFIRM,
        language_source,
        "key.clear_cache.confirm",
    );
    set_text(
        &CLEAR_CACHE_CANCEL,
        language_source,
        "key.clear_cache.cancel",
    );
    set_text(&SIZE_RETURN, language_source, "key.size.return");

    KeyText {
        home_prev_option: text(&HOME_PREV_OPTION),
        home_next_option: text(&HOME_NEXT_OPTION),
        home_select: text(&HOME_SELECT),
        home_confirm: text(&HOME_CONFIRM),
        home_option1: text(&HOME_OPTION1),
        home_option2: text(&HOME_OPTION2),
        home_option3: text(&HOME_OPTION3),
        home_option4: text(&HOME_OPTION4),
        home_option5: text(&HOME_OPTION5),
        setting_prev_option: text(&SETTING_PREV_OPTION),
        setting_next_option: text(&SETTING_NEXT_OPTION),
        setting_select: text(&SETTING_SELECT),
        setting_confirm: text(&SETTING_CONFIRM),
        setting_option1: text(&SETTING_OPTION1),
        setting_option2: text(&SETTING_OPTION2),
        setting_option3: text(&SETTING_OPTION3),
        setting_option4: text(&SETTING_OPTION4),
        setting_option5: text(&SETTING_OPTION5),
        setting_back: text(&SETTING_BACK),
        game_list_prev_option: text(&GAME_LIST_PREV_OPTION),
        game_list_next_option: text(&GAME_LIST_NEXT_OPTION),
        game_list_prev_page: text(&GAME_LIST_PREV_PAGE),
        game_list_next_page: text(&GAME_LIST_NEXT_PAGE),
        game_list_scroll_up: text(&GAME_LIST_SCROLL_UP),
        game_list_scroll_down: text(&GAME_LIST_SCROLL_DOWN),
        game_list_jump: text(&GAME_LIST_JUMP),
        game_list_order: text(&GAME_LIST_ORDER),
        game_list_sort: text(&GAME_LIST_SORT),
        game_list_back: text(&GAME_LIST_BACK),
        game_list_back_cancel: text(&GAME_LIST_BACK_CANCEL),
        game_list_start_confirm: text(&GAME_LIST_START_CONFIRM),
        game_list_start: text(&GAME_LIST_START),
        game_list_confirm: text(&GAME_LIST_CONFIRM),
        game_list_cancel: text(&GAME_LIST_CANCEL),
        game_list_select: text(&GAME_LIST_SELECT),
        game_list_flip: text(&GAME_LIST_FLIP),
        game_list_scroll: text(&GAME_LIST_SCROLL),
        mod_prev_option: text(&MOD_PREV_OPTION),
        mod_next_option: text(&MOD_NEXT_OPTION),
        mod_list_option1: text(&MOD_LIST_OPTION1),
        mod_list_option2: text(&MOD_LIST_OPTION2),
        mod_list_option3: text(&MOD_LIST_OPTION3),
        mod_hub_select: text(&MOD_HUB_SELECT),
        mod_hub_confirm: text(&MOD_HUB_CONFIRM),
        mod_hub_back: text(&MOD_HUB_BACK),
        mod_hub_title: text(&MOD_HUB_TITLE),
        mod_list_prev_option: text(&MOD_LIST_PREV_OPTION),
        mod_list_next_option: text(&MOD_LIST_NEXT_OPTION),
        mod_list_prev_page: text(&MOD_LIST_PREV_PAGE),
        mod_list_next_page: text(&MOD_LIST_NEXT_PAGE),
        mod_list_scroll_up: text(&MOD_LIST_SCROLL_UP),
        mod_list_scroll_down: text(&MOD_LIST_SCROLL_DOWN),
        mod_list_jump: text(&MOD_LIST_JUMP),
        mod_list_order: text(&MOD_LIST_ORDER),
        mod_list_sort: text(&MOD_LIST_SORT),
        mod_list_back: text(&MOD_LIST_BACK),
        mod_list_toggle_confirm: text(&MOD_LIST_TOGGLE_CONFIRM),
        mod_list_back_cancel: text(&MOD_LIST_BACK_CANCEL),
        mod_list_toggle: text(&MOD_LIST_TOGGLE),
        mod_list_confirm: text(&MOD_LIST_CONFIRM),
        mod_list_cancel: text(&MOD_LIST_CANCEL),
        mod_list_select: text(&MOD_LIST_SELECT),
        mod_list_flip: text(&MOD_LIST_FLIP),
        mod_list_scroll: text(&MOD_LIST_SCROLL),
        mod_list_debug: text(&MOD_LIST_DEBUG),
        mod_list_list: text(&MOD_LIST_LIST),
        mod_list_safe_mode: text(&MOD_LIST_SAFE_MODE),
        mod_security_close_permanent: text(&MOD_SECURITY_CLOSE_PERMANENT),
        mod_security_close_temporary: text(&MOD_SECURITY_CLOSE_TEMPORARY),
        mod_security_cancel: text(&MOD_SECURITY_CANCEL),
        default_security_close_permanent: text(&DEFAULT_SECURITY_CLOSE_PERMANENT),
        default_security_cancel: text(&DEFAULT_SECURITY_CANCEL),
        security_prev_option: text(&SECURITY_PREV_OPTION),
        security_next_option: text(&SECURITY_NEXT_OPTION),
        security_select: text(&SECURITY_SELECT),
        security_close_permanent: text(&SECURITY_CLOSE_PERMANENT),
        security_back: text(&SECURITY_BACK),
        security_toggle_confirm: text(&SECURITY_TOGGLE_CONFIRM),
        security_toggle: text(&SECURITY_TOGGLE),
        security_confirm: text(&SECURITY_CONFIRM),
        security_option1: text(&SECURITY_OPTION1),
        security_option2: text(&SECURITY_OPTION2),
        security_option3: text(&SECURITY_OPTION3),
        security_option4: text(&SECURITY_OPTION4),
        security_option5: text(&SECURITY_OPTION5),
        security_option6: text(&SECURITY_OPTION6),
        security_option7: text(&SECURITY_OPTION7),
        security_option8: text(&SECURITY_OPTION8),
        language_up_option: text(&LANGUAGE_UP_OPTION),
        language_down_option: text(&LANGUAGE_DOWN_OPTION),
        language_left_option: text(&LANGUAGE_LEFT_OPTION),
        language_right_option: text(&LANGUAGE_RIGHT_OPTION),
        language_select: text(&LANGUAGE_SELECT),
        language_confirm: text(&LANGUAGE_CONFIRM),
        language_jump: text(&LANGUAGE_JUMP),
        language_prev_page: text(&LANGUAGE_PREV_PAGE),
        language_next_page: text(&LANGUAGE_NEXT_PAGE),
        language_back_cancel: text(&LANGUAGE_BACK_CANCEL),
        language_back: text(&LANGUAGE_BACK),
        language_cancel: text(&LANGUAGE_CANCEL),
        language_page: text(&LANGUAGE_PAGE),
        language_flip: text(&LANGUAGE_FLIP),
        memory_prev_option: text(&MEMORY_PREV_OPTION),
        memory_next_option: text(&MEMORY_NEXT_OPTION),
        memory_select: text(&MEMORY_SELECT),
        memory_option1: text(&MEMORY_OPTION1),
        memory_option2: text(&MEMORY_OPTION2),
        memory_option3: text(&MEMORY_OPTION3),
        memory_confirm: text(&MEMORY_CONFIRM),
        memory_back: text(&MEMORY_BACK),
        setting_keybind_list_option1: text(&SETTING_KEYBIND_LIST_OPTION1),
        setting_keybind_list_option2: text(&SETTING_KEYBIND_LIST_OPTION2),
        setting_keybind_list_option3: text(&SETTING_KEYBIND_LIST_OPTION3),
        setting_keybind_list_prev_option: text(&SETTING_KEYBIND_LIST_PREV_OPTION),
        setting_keybind_list_next_option: text(&SETTING_KEYBIND_LIST_NEXT_OPTION),
        setting_keybind_list_select: text(&SETTING_KEYBIND_LIST_SELECT),
        setting_keybind_list_confirm: text(&SETTING_KEYBIND_LIST_CONFIRM),
        setting_keybind_list_back: text(&SETTING_KEYBIND_LIST_BACK),
        setting_keybind_list_title: text(&SETTING_KEYBIND_LIST_TITLE),
        setting_keybind_system_prev_option: text(&SETTING_KEYBIND_SYSTEM_PREV_OPTION),
        setting_keybind_system_next_option: text(&SETTING_KEYBIND_SYSTEM_NEXT_OPTION),
        setting_keybind_system_select: text(&SETTING_KEYBIND_SYSTEM_SELECT),
        setting_keybind_system_prev_page: text(&SETTING_KEYBIND_SYSTEM_PREV_PAGE),
        setting_keybind_system_next_page: text(&SETTING_KEYBIND_SYSTEM_NEXT_PAGE),
        setting_keybind_system_scroll_up: text(&SETTING_KEYBIND_SYSTEM_SCROLL_UP),
        setting_keybind_system_scroll_down: text(&SETTING_KEYBIND_SYSTEM_SCROLL_DOWN),
        setting_keybind_system_scroll: text(&SETTING_KEYBIND_SYSTEM_SCROLL),
        setting_keybind_system_jump: text(&SETTING_KEYBIND_SYSTEM_JUMP),
        setting_keybind_system_order: text(&SETTING_KEYBIND_SYSTEM_ORDER),
        setting_keybind_system_sort: text(&SETTING_KEYBIND_SYSTEM_SORT),
        setting_keybind_system_confirm: text(&SETTING_KEYBIND_SYSTEM_CONFIRM),
        setting_keybind_system_list_back: text(&SETTING_KEYBIND_SYSTEM_LIST_BACK),
        setting_keybind_system_back: text(&SETTING_KEYBIND_SYSTEM_BACK),
        setting_keybind_system_list: text(&SETTING_KEYBIND_SYSTEM_LIST),
        setting_keybind_system_key1: text(&SETTING_KEYBIND_SYSTEM_KEY1),
        setting_keybind_system_key2: text(&SETTING_KEYBIND_SYSTEM_KEY2),
        setting_keybind_system_key3: text(&SETTING_KEYBIND_SYSTEM_KEY3),
        setting_keybind_system_key4: text(&SETTING_KEYBIND_SYSTEM_KEY4),
        setting_keybind_system_tip_delete: text(&SETTING_KEYBIND_SYSTEM_TIP_DELETE),
        setting_keybind_system_tip_add_modify: text(&SETTING_KEYBIND_SYSTEM_TIP_ADD_MODIFY),
        setting_keybind_system_add: text(&SETTING_KEYBIND_SYSTEM_ADD),
        setting_keybind_system_modify: text(&SETTING_KEYBIND_SYSTEM_MODIFY),
        setting_keybind_system_add_shift: text(&SETTING_KEYBIND_SYSTEM_ADD_SHIFT),
        setting_keybind_system_modify_shift: text(&SETTING_KEYBIND_SYSTEM_MODIFY_SHIFT),
        setting_keybind_system_delete: text(&SETTING_KEYBIND_SYSTEM_DELETE),
        setting_keybind_system_key_mode: text(&SETTING_KEYBIND_SYSTEM_KEY_MODE),
        setting_keybind_system_reset_only: text(&SETTING_KEYBIND_SYSTEM_RESET_ONLY),
        setting_keybind_system_reset_game: text(&SETTING_KEYBIND_SYSTEM_RESET_GAME),
        setting_keybind_system_reset_page: text(&SETTING_KEYBIND_SYSTEM_RESET_PAGE),
        storage_details_back: text(&STORAGE_DETAILS_BACK),
        clear_data_confirm: text(&CLEAR_DATA_CONFIRM),
        clear_data_cancel: text(&CLEAR_DATA_CANCEL),
        clear_cache_confirm: text(&CLEAR_CACHE_CONFIRM),
        clear_cache_cancel: text(&CLEAR_CACHE_CANCEL),
        size_return: text(&SIZE_RETURN),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
