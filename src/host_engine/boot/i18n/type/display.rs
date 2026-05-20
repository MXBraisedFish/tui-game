//! display.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static TITLE: MutableText = MutableText::new();
pub static TOGGLE_MOD_ON: MutableText = MutableText::new();
pub static TOGGLE_MOD_OFF: MutableText = MutableText::new();
pub static TOGGLE_AFK_SAVER_ON: MutableText = MutableText::new();
pub static TOGGLE_AFK_SAVER_OFF: MutableText = MutableText::new();
pub static TOGGLE_AFK_TIME_SECOND: MutableText = MutableText::new();
pub static TOGGLE_AFK_TIME_MINUTE: MutableText = MutableText::new();
pub static TOGGLE_AFK_TIME_NEVER: MutableText = MutableText::new();
pub static TOGGLE_SORT_ORDER: MutableText = MutableText::new();
pub static TOGGLE_SORT_RANDOM: MutableText = MutableText::new();
pub static TOGGLE_SORT_OFF: MutableText = MutableText::new();
pub static OPTION_INFO_ON: MutableText = MutableText::new();
pub static OPTION_INFO_OFF: MutableText = MutableText::new();
pub static TOGGLE_THEME_SYSTEM: MutableText = MutableText::new();
pub static OPTION_MOD: MutableText = MutableText::new();
pub static OPTION_THEME: MutableText = MutableText::new();
pub static OPTION_AFK_TIME: MutableText = MutableText::new();
pub static OPTION_AFK_SAVER: MutableText = MutableText::new();
pub static OPTION_INFO: MutableText = MutableText::new();
pub static OPTION_SAVER_SORT: MutableText = MutableText::new();
pub static OPTION_BOSS_SORT: MutableText = MutableText::new();
pub static OPTION_SAVER_LIST: MutableText = MutableText::new();
pub static OPTION_BOSS_LIST: MutableText = MutableText::new();
pub static OPTION_LIST_ON: MutableText = MutableText::new();
pub static OPTION_LIST_OFF: MutableText = MutableText::new();
pub static OPTION_LIST_MOD: MutableText = MutableText::new();

#[derive(Clone, Debug)]
pub struct DisplayText {
    pub title: String,
    pub toggle_mod_on: String,
    pub toggle_mod_off: String,
    pub toggle_afk_saver_on: String,
    pub toggle_afk_saver_off: String,
    pub toggle_afk_time_second: String,
    pub toggle_afk_time_minute: String,
    pub toggle_afk_time_never: String,
    pub toggle_sort_order: String,
    pub toggle_sort_random: String,
    pub toggle_sort_off: String,
    pub option_info_on: String,
    pub option_info_off: String,
    pub toggle_theme_system: String,
    pub option_mod: String,
    pub option_theme: String,
    pub option_afk_time: String,
    pub option_afk_saver: String,
    pub option_info: String,
    pub option_saver_sort: String,
    pub option_boss_sort: String,
    pub option_saver_list: String,
    pub option_boss_list: String,
    pub option_list_on: String,
    pub option_list_off: String,
    pub option_list_mod: String,
}

pub fn register(language_source: &LanguageSource) -> DisplayText {
    set_text(&TITLE, language_source, "display.title");
    set_text(&TOGGLE_MOD_ON, language_source, "display.toggle.mod.on");
    set_text(&TOGGLE_MOD_OFF, language_source, "display.toggle.mod.off");
    set_text(&TOGGLE_AFK_SAVER_ON, language_source, "display.toggle.afk.saver.on");
    set_text(&TOGGLE_AFK_SAVER_OFF, language_source, "display.toggle.afk.saver.off");
    set_text(&TOGGLE_AFK_TIME_SECOND, language_source, "display.toggle.afk.time.second");
    set_text(&TOGGLE_AFK_TIME_MINUTE, language_source, "display.toggle.afk.time.minute");
    set_text(&TOGGLE_AFK_TIME_NEVER, language_source, "display.toggle.afk.time.never");
    set_text(&TOGGLE_SORT_ORDER, language_source, "display.toggle.sort.order");
    set_text(&TOGGLE_SORT_RANDOM, language_source, "display.toggle.sort.random");
    set_text(&TOGGLE_SORT_OFF, language_source, "display.toggle.sort.off");
    set_text(&OPTION_INFO_ON, language_source, "display.option.info.on");
    set_text(&OPTION_INFO_OFF, language_source, "display.option.info.off");
    set_text(&TOGGLE_THEME_SYSTEM, language_source, "display.toggle.theme.system");
    set_text(&OPTION_MOD, language_source, "display.option.mod");
    set_text(&OPTION_THEME, language_source, "display.option.theme");
    set_text(&OPTION_AFK_TIME, language_source, "display.option.afk.time");
    set_text(&OPTION_AFK_SAVER, language_source, "display.option.afk.saver");
    set_text(&OPTION_INFO, language_source, "display.option.info");
    set_text(&OPTION_SAVER_SORT, language_source, "display.option.saver.sort");
    set_text(&OPTION_BOSS_SORT, language_source, "display.option.boss.sort");
    set_text(&OPTION_SAVER_LIST, language_source, "display.option.saver.list");
    set_text(&OPTION_BOSS_LIST, language_source, "display.option.boss.list");
    set_text(&OPTION_LIST_ON, language_source, "display.option.list.on");
    set_text(&OPTION_LIST_OFF, language_source, "display.option.list.off");
    set_text(&OPTION_LIST_MOD, language_source, "display.option.list.mod");

    DisplayText {
        title: text(&TITLE),
        toggle_mod_on: text(&TOGGLE_MOD_ON),
        toggle_mod_off: text(&TOGGLE_MOD_OFF),
        toggle_afk_saver_on: text(&TOGGLE_AFK_SAVER_ON),
        toggle_afk_saver_off: text(&TOGGLE_AFK_SAVER_OFF),
        toggle_afk_time_second: text(&TOGGLE_AFK_TIME_SECOND),
        toggle_afk_time_minute: text(&TOGGLE_AFK_TIME_MINUTE),
        toggle_afk_time_never: text(&TOGGLE_AFK_TIME_NEVER),
        toggle_sort_order: text(&TOGGLE_SORT_ORDER),
        toggle_sort_random: text(&TOGGLE_SORT_RANDOM),
        toggle_sort_off: text(&TOGGLE_SORT_OFF),
        option_info_on: text(&OPTION_INFO_ON),
        option_info_off: text(&OPTION_INFO_OFF),
        toggle_theme_system: text(&TOGGLE_THEME_SYSTEM),
        option_mod: text(&OPTION_MOD),
        option_theme: text(&OPTION_THEME),
        option_afk_time: text(&OPTION_AFK_TIME),
        option_afk_saver: text(&OPTION_AFK_SAVER),
        option_info: text(&OPTION_INFO),
        option_saver_sort: text(&OPTION_SAVER_SORT),
        option_boss_sort: text(&OPTION_BOSS_SORT),
        option_saver_list: text(&OPTION_SAVER_LIST),
        option_boss_list: text(&OPTION_BOSS_LIST),
        option_list_on: text(&OPTION_LIST_ON),
        option_list_off: text(&OPTION_LIST_OFF),
        option_list_mod: text(&OPTION_LIST_MOD),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
