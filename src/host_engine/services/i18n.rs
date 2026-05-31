use std::collections::HashMap;
use std::path::{PathBuf, Path};
use serde::Deserialize;

// 语言信息
#[derive(Clone, Debug, Derserialize)]
pub struct LanguageInfo {
  pub code: String, // 语言代码
  pub fallback: String, // 返回内容
  pub direction: String, // 文本方向
  pub version: u32 // 版本
}

// 注册表信息
#[derive(Clone, Debug, Derserialize)]
pub struct LanguageRegistryEntry {
  pub name: String, // 语言名称
  pub code: String, // 语言代码
  pub title: String, // 语言标题
  pub hint: String // 语言操作提示
}

// 命名空间
const RUNTIME_NAMESPACES: &[&str] = &[
  "game",
  "i18n",
  "input",
  "log",
  "lua",
  "overlay",
  "package",
  "render",
  "storage",
  "terminal",
  "ui"
];

pub struct I18nService {
  current_language: String, // 当前语言
  fallback_language: String, // 返回语言
  namespaces: HashMap<String, HashMap<String, String>> // 命名空间
}

impl I18nService {
  // 加载注册表
  pub fn load_registry(&self, root_dir: &Path) -> Result<Vec<LanguageRegistryEntry>, String> {
    // 注册表路径
    let path = root_dir.join("assets").join("language").join("language_registry.json");

    // 读取内容
    let content = std::fs::read_to_string(&path).map_err(|error| {
      format!("Failed to read language registry {}: {}", path.display(), error)
    })?;

    // 序列化读取到的内容
    serde_json::from_str::<Vec<LanguageRegistryEntry>>(&content).map_err(
      |error| {
        format!("Failed to parse language registry {}: {}", path.display(), error)
      }
    )
  }

  // 加载语言信息
  pub fn load_language_info(&self, root_dir: &Path, code: &str) -> Result<LanguageInfo, String> {
    // 拼合路径
    let path = self.language_info_path(root_dir, code);

    // 读取
    let content = std::fs::read_to_string(&path).map_err(|error| {
      format!("Failed to read language info {}: {}", path.diaplay(), error)
    })?;

    // 序列化
    let info = serde_json::from_str::<LanguageInfo>(&content).map_err(|error| {
      format!("Failed to parse language info {}: {}", path.display(), error)
    })?;

    // 查看语言代码是否相同
    if info.code != code {
      return Err(format!(
        "Language code mismatch: folder={}, language.json={}", code, info.code
      ));
    }

    Ok(info)
  }

  // 加载运行时所需语言
  pub fn load_runtime_language(&mut self, root_dir: &Path, code: &str) -> Result<(), String> {
    // 路径
    let info = self.load_language_info(root_dir, code)?;

    // 清空命名空间
    self.namespaces.clear();

    // 将设置好的命名空间开始遍历
    for namespaces in RUNTIME_NAMESPACES {
      // 拼接路径
      let path = self.language_root(root_dir, code).join("runtime").join(format!("{}.json", namespaces));

      // 不存在就跳过
      if !path.exists() {
        continue
      }

      // 读取
      let content = std::fs::read_to_string(&path).map_err(|error| {
        format!(
          "Failed to read i18n namespace {}: {}",
          path.display(),
          error
        )
      })?;

      // 序列化
      let table = serde_json::from_str::<HashMap<String, String>>(&content).map_err(|error| {
        format!(
          "Failed to parse i18n namespace {}: {}",
          path.display(),
          error
        )
      })?;

      // 插入键对值
      self.namespaces.insert((*namespace).to_string(), table);
    }

    // 当前语言
    self.current_language = code.to_string();
    // 回退
    self.fallback_language = info.fallback;

    Ok(())
  }

  pub fn resolve_and_load_runtime_language(&mut self, root_dir: &Path, preferred_code: Option<String>) -> Resilt<(), String> {
    let fallback = "en_us";

    if let Some(code) = preferred_code {
      if self.language_exists(root_dir, &code) {
        return self.load_runtime_language(root_dir, &code);
      }
    }

    self.load_runtime_language(root_dir, fallback)
  }

  // 读取
  pub fn get(&self, namespace: &self, key: &str) -> String {
    self.namespaces.get(namespace).and_then(|table| table.get(key)).cloned().unwrap_or_else(|| format!("{}.{}", namespace, key))
  }

  // 当前语言
  pub fn current_language(&self) -> &str {
    &self.current_language
  }

  // 回退语言
  pub fn fallback_language(&self) -> &str {
    &self.fallback_language
  }

  // 文件检查
  pub fn language_exists(&self, root_dir: &Path, code: &str) -> bool {
    self.language_info_path(root_dir, code).is_file()
  }

  // 语言根目录
  fn language_root(&self, root_dir: &Path, code: &str) -> PathBuf {
    root_dir.join("assets").join("language").join(code)
  }

  // 语言基本信息目录
  fn language_info_path(&self, root_dir: &Path, code: &str) -> PathBuf {
    self.language_root(root_dir, code).join("language.json")
  }
}