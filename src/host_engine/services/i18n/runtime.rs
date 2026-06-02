use std::collections::HashMap;
use std::fs;

use super::service::I18nService;
use crate::host_engine::services::{LogService, LogSource, StorageService};

// 语言命名空间
const RUNTIME_NAMESPACES: &[&str] = &["ui", "key", "terminal", "error", "package", "dialog"];

impl I18nService {
  // 加载语言
  pub fn load_runtime_language(
    &mut self,
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
  ) {
    // 清理缓存
    self.clear_runtime_texts();

    // 加载当前语言
    let loaded_selected = self.load_runtime_namespaces(storage, log, language_code);

    // 成功了就设置语言代码
    if loaded_selected {
      self.set_current_language(language_code);
      return;
    }

    // 回退语言
    let fallback = storage.default_language_code();

    // 回退
    let loaded_fallback = self.load_runtime_namespaces(storage, log, fallback);

    // 设置语言代码
    self.set_current_language(fallback);

    // 回退成功则设置
    if loaded_fallback {
      return;
    }

    // 若都失败了则发出错误
    // TODO：这里到时候会接入把英语文件内嵌一份
    log.error(LogSource::I18n, "Failed to load runtime i18n language.");
  }

  // 批量加载命名空间
  fn load_runtime_namespaces(
    &mut self,
    storage: &StorageService,
    log: &mut LogService,
    language_code: &str,
  ) -> bool {
    // 尚未加载任何一个语言内容
    let mut loaded_any = false;

    // 遍历每一个命名空间来注册
    for namespace in RUNTIME_NAMESPACES {
      // 尝试注册
      if self.try_load_runtime_namespace(storage, language_code, namespace, log) {
        // 只要有一个成功就设置语言加载成功
        loaded_any = true;
      }
    }

    // 返回成功加载
    loaded_any
  }

  // 尝试加载语言文件
  fn try_load_runtime_namespace(
    &mut self,
    storage: &StorageService,
    language_code: &str,
    namespace: &str,
    log: &mut LogService,
  ) -> bool {
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
