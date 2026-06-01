use std::collections::HashMap;
use std::fs;

use super::service::I18nService;
use crate::host_engine::services::{LogService, LogSource, StorageService};

// 临时测试命名空间
const TEST_NAMESPACE: &str = "test";

impl I18nService {
  // 加载语言
  pub fn load_runtime_test_language(&mut self, storage: &StorageService, log: &mut LogService, language_code: &str) {
    // 清理缓存
    self.clear_runtime_texts();

    // 尝试加载对应的语言文件
    if self.try_load_runtime_namespace(storage, language_code, TEST_NAMESPACE, log) {
      self.set_current_language(language_code);
      return;
    }

    // 回退语言
    let fallback = storage.default_language_code();

    // 加载回退语言
    if self.try_load_runtime_namespace(storage, fallback, TEST_NAMESPACE, log) {
      self.set_current_language(fallback);
      return;
    }

    // 蛇者当前语言为回退
    self.set_current_language(fallback);

    // 若都没加载成功，发出错误
    log.error(
      LogSource::I18n,
      "Failed to load runtime i18n test namespace.",
    );
  }

  // 尝试加载语言文件
  fn try_load_runtime_namespace(&mut self, storage: &StorageService, language_code: &str, namespace: &str, log: &mut LogService) -> bool {
    // 构建路径
    let path = storage.language_runtime_namespace_path(language_code, namespace);

    // 读取
    let content = match fs::read_to_string(&path) {
      Ok(content) => content,
      Err(error) => {
        log.warn(
          LogSource::I18n,
          format!(
            "Failed to read i18n namespace {}: {}",
            path.display(),
            error,
          ),
        );

        return false;
      }
    };

    // 序列化内容
    let texts = match serde_json::from_str::<HashMap<String, String>>(&content) {
      Ok(texts) => texts,
      Err(error) => {
        log.warn(
          LogSource::I18n,
          format!(
            "Failed to parse i18n namespace {}: {}",
            path.display(),
            error,
          ),
        );

        return false;
      }
    };

    // 插入内容
    self.insert_runtime_namespace(namespace, texts);
    true
  }
}