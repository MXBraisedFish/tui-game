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
