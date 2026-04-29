pub fn load_cli_language() {
  // 读取偏好
    let preference = load_language_preference().unwrap_or("en_us");

    // 解析文件
    let dict = load_command_lang_json(preference)
        .or_else(|| load_command_lang_json("en_us"))
        .unwrap_or_default();

    // 写入常量
    // 从 dict 提取每个键写入 pub static 常量
    // 例如: HELP_TEXT = dict["help_text"]
    //       VERSION_TEXT = dict["version_text"]
    //       ERROR_UNKNOWN = dict["error_unknown_command"]
    //       ...
}