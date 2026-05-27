# 步骤 6：StorageService — 环境目录创建

## 目标

让 `StorageService` 持有环境目录准备逻辑。这是第一个有真实逻辑的服务。

## 背景

旧 `boot::environment` 模块处理：
- `host_dirs::verify()` — 检查 scripts/、assets/ 等是否存在
- `data_dirs::ensure()` — 创建 data/cache/、data/profiles/、data/mod/ 等
- `repair` — 空桩

这些都是存储/环境关注点，属于 `StorageService`。
目前只迁移目录准备，文件级别默认值后面再迁移。

## 操作

### 6.1 更新 `src/app/services/storage.rs`

```rust
use std::fs;
use std::path::PathBuf;

pub struct StorageService {
    root_dir: PathBuf,
}

impl StorageService {
    pub fn new() -> Self {
        let root_dir = resolve_root_dir();
        let service = Self { root_dir };
        service.ensure_directories();
        service
    }

    /// 确保所有运行时数据目录存在。
    fn ensure_directories(&self) {
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
            "scripts/boss",
        ];

        for dir in &dirs {
            let path = self.root_dir.join(dir);
            if let Err(e) = fs::create_dir_all(&path) {
                eprintln!("[Boot] Warning: failed to create {}: {}", path.display(), e);
            }
        }

        // 确保关键文件存在并写入默认值
        let files = [
            ("data/cache/mod_scan_cache.json", "{}"),
            ("data/cache/screensaver_scan_cache", "{}"),
            ("data/cache/boss_scan_cache", "{}"),
            ("data/cache/language_ui_cache.json", "{}"),
            ("data/profiles/language.txt", "en_us"),
            ("data/log/tui_log.txt", ""),
        ];

        for (file, default) in &files {
            let path = self.root_dir.join(file);
            if !path.exists() {
                if let Some(parent) = path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&path, default);
            }
        }
    }

    pub fn root_dir(&self) -> &PathBuf {
        &self.root_dir
    }
}

fn resolve_root_dir() -> PathBuf {
    std::env::current_dir()
        .ok()
        .filter(|path| path.join("assets").exists() || path.join("Cargo.toml").exists())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(Path::to_path_buf))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}
```

### 6.2 更新 `src/app/boot.rs`

```rust
pub fn prepare() -> BootOutput {
    println!("[Boot] Preparing engine...");
    let services = EngineServices::new();
    let world = RuntimeWorld::new();
    println!("[Boot] Storage root: {}", services.storage.root_dir().display());
    BootOutput { services, world }
}
```

## 验证

```bash
cargo build
cargo run
```

检查 `data/` 和 `scripts/` 目录是否在缺失时被创建。
启动输出显示存储根路径。
