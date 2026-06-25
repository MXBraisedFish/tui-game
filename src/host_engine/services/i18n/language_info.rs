use serde::Deserialize;
use std::fs;

use crate::host_engine::services::{LogService, LogSource, StorageService};

/// 语言信息（代码和文本方向）
#[derive(Clone, Debug, Deserialize)]
pub struct LanguageInfo {
  pub code: String,
  pub direction: String,
}

/// 从磁盘加载指定语言的信息文件
pub fn load_language_info(
  storage: &StorageService,
  log: &mut LogService,
  language_code: &str,
) -> Option<LanguageInfo> {
  let path = storage.language_info_path(language_code);

  let content = match fs::read_to_string(&path) {
    Ok(content) => content,
    Err(error) => {
      log.warn(
        LogSource::I18n,
        format!("Failed to read language info {}: {}", path.display(), error,),
      );
      return None;
    }
  };

  let info = match serde_json::from_str::<LanguageInfo>(&content) {
    Ok(info) => info,
    Err(error) => {
      log.warn(
        LogSource::I18n,
        format!(
          "Failed to parse language info {}: {}",
          path.display(),
          error,
        ),
      );
      return None;
    }
  };

  if info.code != language_code {
    log.warn(
      LogSource::I18n,
      format!(
        "Language code mismatch: folder={}, file={}",
        language_code, info.code,
      ),
    );
    return None;
  }

  Some(info)
}
