//! security.* 语言文本注册

use crate::host_engine::boot::i18n::i18n::{LanguageSource, resolve_text};
use crate::host_engine::boot::i18n::pseudo_text::MutableText;

pub static TITLE: MutableText = MutableText::new();
pub static TOGGLE_MOD_ON: MutableText = MutableText::new();
pub static TOGGLE_MOD_OFF: MutableText = MutableText::new();
pub static TOGGLE_SAFE_MODE_ON: MutableText = MutableText::new();
pub static TOGGLE_SAFE_MODE_OFF_PERMANENT: MutableText = MutableText::new();
pub static DEFAULT_SAFE_MODE: MutableText = MutableText::new();
pub static DEFAULT_MOD: MutableText = MutableText::new();
pub static RESET_SAFE_MODE: MutableText = MutableText::new();
pub static RESET_MOD: MutableText = MutableText::new();

/// security.* 文本集合
#[derive(Clone, Debug)]
pub struct SecurityText {
    pub title: String,
    pub toggle_mod_on: String,
    pub toggle_mod_off: String,
    pub toggle_safe_mode_on: String,
    pub toggle_safe_mode_off_permanent: String,
    pub default_safe_mode: String,
    pub default_mod: String,
    pub reset_safe_mode: String,
    pub reset_mod: String,
}

/// 注册 security.* 文本
pub fn register(language_source: &LanguageSource) -> SecurityText {
    set_text(&TITLE, language_source, "security.title");
    set_text(&TOGGLE_MOD_ON, language_source, "security.toggle.mod.on");
    set_text(&TOGGLE_MOD_OFF, language_source, "security.toggle.mod.off");
    set_text(
        &TOGGLE_SAFE_MODE_ON,
        language_source,
        "security.toggle.safe_mode.on",
    );
    set_text(
        &TOGGLE_SAFE_MODE_OFF_PERMANENT,
        language_source,
        "security.toggle.safe_mode.off.permanent",
    );
    set_text(
        &DEFAULT_SAFE_MODE,
        language_source,
        "security.default.safe_mode",
    );
    set_text(&DEFAULT_MOD, language_source, "security.default.mod");
    set_text(
        &RESET_SAFE_MODE,
        language_source,
        "security.reset.safe_mode",
    );
    set_text(&RESET_MOD, language_source, "security.reset.mod");

    SecurityText {
        title: text(&TITLE),
        toggle_mod_on: text(&TOGGLE_MOD_ON),
        toggle_mod_off: text(&TOGGLE_MOD_OFF),
        toggle_safe_mode_on: text(&TOGGLE_SAFE_MODE_ON),
        toggle_safe_mode_off_permanent: text(&TOGGLE_SAFE_MODE_OFF_PERMANENT),
        default_safe_mode: text(&DEFAULT_SAFE_MODE),
        default_mod: text(&DEFAULT_MOD),
        reset_safe_mode: text(&RESET_SAFE_MODE),
        reset_mod: text(&RESET_MOD),
    }
}

fn set_text(cell: &'static MutableText, language_source: &LanguageSource, key: &str) {
    cell.set(resolve_text(language_source, key));
}

fn text(cell: &'static MutableText) -> String {
    cell.get()
}
