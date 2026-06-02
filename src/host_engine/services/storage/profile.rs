use std::fs;

use super::layout;
use super::service::StorageService;

impl StorageService {
  // 读取语言代码
  pub fn read_language_code(&self) -> Option<String> {
    let content = fs::read_to_string(self.profile_language_path()).ok()?;
    let code = content.trim();

    // 判断是否为空
    if code.is_empty() {
      None
    } else {
      Some(code.to_string())
    }
  }

  // 写入持久化语言代码
  pub fn write_language_code(&self, language_code: &str) -> std::io::Result<()> {
    fs::write(self.profile_language_path(), language_code.trim())
  }

  // 默认语言代码
  pub fn default_language_code(&self) -> &'static str {
    layout::DEFAULT_LANGUAGE_CODE
  }
}
