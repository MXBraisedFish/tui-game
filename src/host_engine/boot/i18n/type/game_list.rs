//! game_list.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static LIST_TITLE: MutableText = MutableText::new();
pub static INFO_SORT_NAME: MutableText = MutableText::new();
pub static INFO_SORT_MOD_OFFICIAL: MutableText = MutableText::new();
pub static INFO_SORT_AUTHOR: MutableText = MutableText::new();
pub static INFO_ORDER_ASCENDING: MutableText = MutableText::new();
pub static INFO_ORDER_DESCENDING: MutableText = MutableText::new();
pub static INFO_MOD: MutableText = MutableText::new();
pub static INFO_AUTHOR: MutableText = MutableText::new();
pub static INFO_VERSION: MutableText = MutableText::new();
pub static INFO_TITLE: MutableText = MutableText::new();
pub static MOD: MutableText = MutableText::new();
pub static NONE_GAME: MutableText = MutableText::new();
pub static NONE_INFO: MutableText = MutableText::new();

/// game_list.* 文本集合
#[derive(Clone, Debug)]
pub struct GameListText {
    pub list_title: String,
    pub info_sort_name: String,
    pub info_sort_mod_official: String,
    pub info_sort_author: String,
    pub info_order_ascending: String,
    pub info_order_descending: String,
    pub info_mod: String,
    pub info_author: String,
    pub info_version: String,
    pub info_title: String,
    pub mod_label: String,
    pub none_game: String,
    pub none_info: String,
}

/// 注册 game_list.* 文本
pub fn register(language_source: &LanguageSource) -> GameListText {
    set_text(&LIST_TITLE, language_source, "game_list.list.title");
    set_text(&INFO_SORT_NAME, language_source, "game_list.info.sort.name");
    set_text(
        &INFO_SORT_MOD_OFFICIAL,
        language_source,
        "game_list.info.sort.mod_official",
    );
    set_text(
        &INFO_SORT_AUTHOR,
        language_source,
        "game_list.info.sort.author",
    );
    set_text(
        &INFO_ORDER_ASCENDING,
        language_source,
        "game_list.info.order.ascending",
    );
    set_text(
        &INFO_ORDER_DESCENDING,
        language_source,
        "game_list.info.order.descending",
    );
    set_text(&INFO_MOD, language_source, "game_list.info.mod");
    set_text(&INFO_AUTHOR, language_source, "game_list.info.author");
    set_text(&INFO_VERSION, language_source, "game_list.info.version");
    set_text(&INFO_TITLE, language_source, "game_list.info.title");
    set_text(&MOD, language_source, "game_list.source.mod");
    set_text(&NONE_GAME, language_source, "game_list.none.game");
    set_text(&NONE_INFO, language_source, "game_list.none.info");

    GameListText {
        list_title: text(&LIST_TITLE),
        info_sort_name: text(&INFO_SORT_NAME),
        info_sort_mod_official: text(&INFO_SORT_MOD_OFFICIAL),
        info_sort_author: text(&INFO_SORT_AUTHOR),
        info_order_ascending: text(&INFO_ORDER_ASCENDING),
        info_order_descending: text(&INFO_ORDER_DESCENDING),
        info_mod: text(&INFO_MOD),
        info_author: text(&INFO_AUTHOR),
        info_version: text(&INFO_VERSION),
        info_title: text(&INFO_TITLE),
        mod_label: text(&MOD),
        none_game: text(&NONE_GAME),
        none_info: text(&NONE_INFO),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
