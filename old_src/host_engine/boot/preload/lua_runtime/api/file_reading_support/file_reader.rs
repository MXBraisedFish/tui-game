//! 文件读取辅助

use std::fs;
use std::path::Path;

/// 读取 UTF-8 文本并去除 BOM。
pub fn read_text(path: &Path) -> mlua::Result<String> {
    fs::read_to_string(path)
        .map(|text| text.trim_start_matches('\u{feff}').to_string())
        .map_err(|error| mlua::Error::external(format!("failed to read file: {error}")))
}

/// 确认文件存在。
pub fn ensure_file_exists(path: &Path) -> mlua::Result<()> {
    if path.is_file() {
        Ok(())
    } else {
        Err(mlua::Error::external(format!(
            "target file not found: {}",
            path.display()
        )))
    }
}
