use std::fs;

use serde::Deserialize;

use crate::host_engine::services::{LogService, LogSource, StorageService};

// 语言注册表
#[derive(Clone, Debug, Deserialize)]
pub struct LanguageRegistryEntry {
  pub code: String,  // 语言代码
  pub name: String,  // 语言名称
  pub title: String, // 语言标题
  pub hint: String,  // 语言操作提示
}

// 加载语言注册表
pub fn load_language_registry(
  storage: &StorageService,
  log: &mut LogService,
) -> Vec<LanguageRegistryEntry> {
  // 路径
  let path = storage.language_registry_path();

  // 读取内容
  let content = match fs::read_to_string(&path) {
    Ok(content) => content,
    Err(error) => {
      log.warn(
        LogSource::I18n,
        format!(
          "Failed to read language registry {}: {}",
          path.display(),
          error,
        ),
      );

      return Vec::new();
    }
  };

  // 序列化内容
  match serde_json::from_str::<Vec<LanguageRegistryEntry>>(&content) {
    Ok(registry) => registry,
    Err(error) => {
      log.warn(
        LogSource::I18n,
        format!(
          "Failed to parse language registry {}: {}",
          path.display(),
          error,
        ),
      );

      Vec::new()
    }
  }
}
