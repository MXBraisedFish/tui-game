use serde::Deserialize;
use std::fs;

use crate::host_engine::services::{LogService, LogSource, StorageService};

// 语言信息
#[derive(Clone, Debug, Deserialize)]
pub struct LanguageInfo {
  pub code: String,      // 语言代码
  pub direction: String, // 语言方向
}

// 读取语言信息文件
// language.json
pub fn load_language_info(
  storage: &StorageService,
  log: &mut LogService,
  language_code: &str,
) -> Option<LanguageInfo> {
  // 路径
  let path = storage.language_info_path(language_code);

  // 读取内容
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

  // 序列化
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

  // 如果语言代码不相同，跳过
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

  // 成功就返回
  Some(info)
}
