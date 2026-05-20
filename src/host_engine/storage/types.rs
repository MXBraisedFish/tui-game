//! storage 层共享类型

use serde::{Deserialize, Serialize};

/// 语言选择 UI 所需的文本片段。
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LanguageUiText {
    pub key_language_up_option: String,
    pub key_language_down_option: String,
    pub key_language_left_option: String,
    pub key_language_right_option: String,
    pub key_language_select: String,
    pub key_language_confirm: String,
    pub key_language_jump: String,
    pub key_language_prev_page: String,
    pub key_language_next_page: String,
    pub key_language_back_cancel: String,
    pub key_language_back: String,
    pub key_language_cancel: String,
    pub key_language_page: String,
    pub key_language_flip: String,
    pub language_title: String,
    pub language_name: String,
}
