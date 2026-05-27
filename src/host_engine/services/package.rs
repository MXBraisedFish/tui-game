// 引入官方标准库
use std::fs;
use std::path::{Path, PathBuf};

// 引入序列化库
use serde::Deserialize;

// 包类型枚举
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageType {
  Game, // 游戏包
  Screensaver, // 屏保包
  Boss // 老板包
}

// 包基本信息结构体
#[derive(Clone, Debug)]
pub struct PackageInfo {
  pub schema_version: u32, // 清单版本
  pub package_type: PackageType, // 包类型
  pub package: String, // 包名
  pub namespace: String, // 命名空间
  pub version: String, // 版本
  pub version_code: u32, // 版本真值
  pub api_min: u32, // api最小版本
  pub api_max: u32, // api最大版本
  pub entry: String, // 入口脚本（相对于scripts）
  pub title: String, // 包名称
  pub path: PathBuf // 路径
}

pub struct PackageService {
  games: Vec<PackageInfo>,
  screensavers: Vec<PackageInfo>,
  bosses: Vec<PackageInfo>,
  errors: Vec<String>
}

impl PackageService {
  pub fn new() -> Self {
    Self {
      games: Vec::new(),
      screensavers: Vec::new(),
      bosses: Vec::new(),
      errors: Vec::new()
    }
  }

  // 扫描所有目录
  pub fn scan_all(&mut self, root_dir: &Path) {
    // 清理内容
    self.games.clear();
    self.screensavers.clear();
    self.bosses.clear();
    self.errors.clear();

    // 扫描官方目录
    self.scan_dir(root_dir, "scripts/game", PackageType::Game);
    self.scan_dir(root_dir, "scripts/screensaver", PackageType::Screensaver);
    self.scan_dir(root_dir, "scripts/boss", PackageType::Boss);

    // 扫描第三方目录
    self.scan_dir(root_dir, "data/mod/game", PackageType::Game);
    self.scan_dir(root_dir, "data/mod/screensaver", PackageType::Screensaver);
    self.scan_dir(root_dir, "data/mod/boss", PackageType::Boss);
  }

  // 扫描路径
  fn scan_dir(&mut self, root_dir: &Path, relative_dir: &str, expected_type: PackageType) {
    // 拼接绝对路径
    let dir = root_dir.join(relative_dir);

    // 扫描父路径，确保父目录可读
    let entries = match fs::read_dir(&dir) {
      Ok(entries) => entries,
      Err(error) => {
        self.errors.push(format!(
          // TODO: 将语言国际化
          "failed to read package directory {}: {}",
          dir.display(),
          error
        ));
        return;
      }
    };

    // 遍历条目（忽略失败条目）
    for entry in entries.flatten() {
      // 提取路径
      let path = entry.path();

      // 非目录跳过
      if !path.is_dir() {
        continue;
      }

      // 读取包信息
      match self.read_package(&path, expected_type.clone()) {
        Ok(info) => self.insert(info), // 成功就插入到集合当中
        Err(error) => self.errors.push(format!("{}: {}", path.display(), error))
      }
    }
  }

  fn read_package(&self, package_dir: &Path, expected_type: PackageType) -> Result<PackageInfo, String> {
    // 组合package.json路径
    let manifest_path = package_dir.join("package.json");

    // 读取内容
    // TODO: 将语言国际化
    let content = fs::read_to_string(&manifest_path).map_err(|error| format!("faild to read package.json: {}", error))?;

    // 序列化内容
    // TODO: 将语言国际化
    let raw: RawPackageManifest = serde_json::from_str(&content).map_err(|error| format!("failed to parse package.json: {}", error))?;

    // 包类型
    let package_type = parse_package_type(&raw.package_type)?;

    // 判断包类型是否正确
    if package_type != expected_type {
      return Err(format!(
        // TODO: 将语言国际化
        "package type mismatch: expected {:?}, found {:?}",
        expected_type, package_type
      ))
    }

    // api支持版本
    let api = normalize_api(raw.api)?;

    // 包名称
    let title = raw.display.as_ref().and_then(|display| display.title.clone()).unwrap_or_else(|| raw.package.clone());

    // 返回当前包的信息
    Ok(PackageInfo { 
      schema_version: raw.schema_version, 
      package_type, 
      package: raw.package, 
      namespace: raw.namespace, 
      version: raw.version.unwrap_or_default(), 
      version_code: raw.version_code, 
      api_min: api.min, 
      api_max: api.max, 
      entry: raw.entry, 
      title, 
      path: package_dir.to_path_buf() })
  }

  // 信息插入
  fn insert(&mut self, info: PackageInfo) {
    match info.package_type {
      PackageType::Game => self.games.push(info),
      PackageType::Screensaver => self.screensavers.push(info),
      PackageType::Boss => self.bosses.push(info)
    }
  }

  // 获取游戏包
  pub fn games(&self) -> &[PackageInfo] {
    &self.games
  }

  // 获取屏保包
  pub fn screensavers(&self) -> &[PackageInfo] {
    &self.screensavers
  }

  // 获取老板包
  pub fn bosses(&self) -> &[PackageInfo] {
    &self.bosses
  }

  // 获取错误表
  pub fn errors(&self) -> &[String] {
    &self.errors
  }

  // 获取包总数
  pub fn total_count(&self) -> usize {
    self.games.len() + self.screensavers.len() + self.bosses.len()
  }
}

// 包配置清单结构体
#[derive(Debug, Deserialize)]
struct RawPackageManifest {
  #[serde(rename = "type")]
  package_type: String, // 包类型

  schema_version: u32, // 配置版本
  package: String, // 包名
  namespace: String, // 命名空间
  version: Option<String>, // 版本
  version_code: u32, // 版本真值
  api: RawApi, // api版本
  entry: String, // 入口
  display: Option<RawDisplay> // 展示信息
}

// API枚举
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawApi {
  Number(u32), // 单数字
  Range { min: u32, max: u32 } // 范围
}

// 显示信息
#[derive(Debug, Deserialize)]
struct RawDisplay {
  title: Option<String>, // 包名称
  description: Option<String>, // 包简介
  author: Option<String>, // 作者
  icon: Option<serde_json::Value>, // 图标
  banner: Option<serde_json::Value> // 横幅
}

struct ApiRange {
  min: u32,
  max: u32
}

fn normalize_api(api: RawApi) -> Result<ApiRange, String> {
  match api {
    RawApi::Number(version) => Ok(ApiRange {
      min: version,
      max: version
    }),
    RawApi::Range { min, max} => {
      if min > max {
        return Err(format!("invalid api range: min {} > max {}", min, max));
      }

      Ok(ApiRange { min, max }) 
    }
  }
}

fn parse_package_type(value: &str) -> Result<PackageType, String> {
  match value {
    "game" => Ok(PackageType::Game),
    "screensaver" => Ok(PackageType::Screensaver),
    "boss" => Ok(PackageType::Boss),
    other => Err(format!("unknow package type: {}", other))
  }
}
