//! 文件写入执行

use std::fs;
use std::path::Path;

/// 写入文本内容，按需创建父目录。
pub fn write_text(path: &Path, content: &str) -> bool {
    if let Some(parent_dir) = path.parent()
        && fs::create_dir_all(parent_dir).is_err()
    {
        return false;
    }

    fs::write(path, content).is_ok()
}
