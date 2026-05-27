// 引入官方文件操作和路径操作库
use std::fs;
use std::path::PathBuf;

pub struct StorageService {
  root_dir: PathBuf
}

impl StorageService {
  pub fn new() -> Self {
    // 获取根目录
    let root_dir = resolve_root_dir();
    // 创建对象
    let service = Self { root_dir };

    // 确保目录
    service.ensure_directories();

    // 返回
    service
  }

  // 确保应有的目录存在
  fn ensure_directories(&self) {
    // 目录数组
    let dirs = [
      "data",
      "data/cache",
      "data/cache/images",
      "data/profiles",
      "data/log",
      "data/mod",
      "data/mod/game",
      "data/mod/screensaver",
      "data/mod/boss",
      "scripts",
      "scripts/game",
      "scripts/screensaver",
      "scripts/boss"
    ];

    // 遍历目录数组
    for dir in dirs {
      // 和根目录组合最终的绝对路径
      let path = self.root_dir.join(dir);
      // 如果创建目录有错误捕获并打印
      if let Err(error) = fs::create_dir_all(&path) {
        // TODO: 这里的警告应该国际化或者写入日志而不是直接打印
        eprintln!(
          "[Boot] Warning: failed to create {}: {}",
          path.display(),
          error
        )
      }
    }

    // 文件数组，以及默认文本
    let files = [
      ("data/cache/game_scan_cache.json", "{}"),
      ("data/cache/screensaver_scan_cache.json", "{}"),
      ("data/cache/boss_scan_cache.json", "{}"),
      ("data/cache/language_ui_cache.json", "{}"),
      ("data/profiles/language.txt", "en_us"),
      ("data/log/tui_log.txt", "")
    ];

    // 遍历文件和默认文本
    for (file, default_context) in files {
      // 组合绝对路径
      let path = self.root_dir.join(file);
      
      // 文件检查
      match fs::metadata(&path) {
        // 如果存在检查是否为文件且内容是否为空
        Ok(metadata) => {
          if metadata.is_file() && metadata.len() > 0 {
            continue;
          }
        }
        // 如果不是找不到而是其它就抛出异常（例如权限不足、符号链接等）
        Err(error) => {
          if error.kind() != std::io::ErrorKind::NotFound {
            // TODO: 这里的警告应该国际化或者写入日志而不是直接打印
            eprintln!(
              "[Boot] Warning: cannot access {}: {}",
              path.display(),
              error
            )
          }
        }
      }

      // 确保父目录的存在
      if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
      }

      // 创建文件并写入默认内容
      if let Err(error) = fs::write(&path, &default_context) {
        // TODO: 这里的警告应该国际化或者写入日志而不是直接打印
        eprintln!(
          "[Boot] Warning: failed to create {}: {}",
          path.display(),
          error
        )
      }
    }
  }

  pub fn root_dir(&self) -> &PathBuf {
    &self.root_dir
  }
}

fn resolve_root_dir() -> PathBuf {
    // 开发模式：检查当前目录是否是工程项目根
    // 判断是否有assets/目录或有Carg.toml文件
    if let Ok(current_dir) = std::env::current_dir() {
      if current_dir.join("assets").exists() || current_dir.join("Cargo.toml").exists() {
          return current_dir;
      }
    }
    
    // 发布模式：使用可执行文件所在目录
    if let Ok(exe_path) = std::env::current_exe() {
      if let Some(exe_dir) = exe_path.parent() {
        return exe_dir.to_path_buf();
      }
    }
    
    // 保底，使用当前目录
    PathBuf::from(".")
}