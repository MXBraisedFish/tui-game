# 步骤 10：PackageService — 扫描并列出包

## 目标

给 `PackageService` 添加扫描 `scripts/` 和 `data/mod/` 目录的能力，
解析 `package.json` 清单，暴露列表供 UI 使用。

目前只做扫描，不做启用/禁用，不包装旧类型。

## 背景

旧扫描代码在：
- `package/scanner.rs` — `Scanner::scan_directories(kind)` 返回 `Vec<PackagePath>`
- `package/manifest.rs` — `parse_manifest(path)` 返回 `RawPackageManifest`

我们复用旧 `package` 模块的 `Scanner::scan_directories` 和 `parse_manifest`
以避免重写 JSON 解析，但将它们干净地包装在 `PackageService` 中。

## 操作

### 10.1 定义 `PackageInfo` 结构体

在 `src/app/services/package.rs` 中：

```rust
use std::path::PathBuf;
use crate::host_engine::package::package_id::{PackageId, PackageKind, PackageSource};
use crate::host_engine::package::scanner::Scanner;
use crate::host_engine::package::manifest::parse_manifest;

/// 服务层的轻量包摘要。
#[derive(Clone, Debug)]
pub struct PackageInfo {
    pub id: PackageId,
    pub path: PathBuf,
    pub package_name: String,
    pub display_name: String,
    pub author: String,
    pub version: String,
    pub entry: String,
    pub enabled: bool,
}
```

### 10.2 实现扫描

```rust
pub struct PackageService {
    games: Vec<PackageInfo>,
    screensavers: Vec<PackageInfo>,
    bosses: Vec<PackageInfo>,
    scan_errors: Vec<String>,
}

impl PackageService {
    pub fn new() -> Self {
        Self {
            games: Vec::new(),
            screensavers: Vec::new(),
            bosses: Vec::new(),
            scan_errors: Vec::new(),
        }
    }

    pub fn scan_all(&mut self) {
        self.scan_kind(PackageKind::Game);
        self.scan_kind(PackageKind::Screensaver);
        self.scan_kind(PackageKind::Boss);
    }

    fn scan_kind(&mut self, kind: PackageKind) {
        let packages = match Scanner::scan_directories(kind) {
            Ok(list) => list,
            Err(e) => {
                self.scan_errors.push(format!("{:?}: {}", kind, e));
                return;
            }
        };
        for pkg_path in packages {
            match self.read_package_info(&pkg_path.path, pkg_path.source, kind) {
                Ok(info) => {
                    match kind {
                        PackageKind::Game => self.games.push(info),
                        PackageKind::Screensaver => self.screensavers.push(info),
                        PackageKind::Boss => self.bosses.push(info),
                        _ => {}
                    }
                }
                Err(e) => {
                    self.scan_errors.push(format!("{}: {}", pkg_path.path.display(), e));
                }
            }
        }
    }

    fn read_package_info(&self, path: &PathBuf, source: PackageSource, kind: PackageKind)
        -> Result<PackageInfo, String>
    {
        let manifest = parse_manifest(&path.join("package.json"))
            .map_err(|e| format!("parse error: {}", e))?;
        let package_name = manifest.package.unwrap_or_default();
        let uid = format!("{}:{}:{}", source_to_str(source), kind_to_str(kind), package_name);
        Ok(PackageInfo {
            id: PackageId::new(source, kind, uid),
            path: path.clone(),
            package_name: package_name.clone(),
            display_name: manifest.package_name.unwrap_or(package_name),
            author: manifest.author.unwrap_or_default(),
            version: manifest.version.unwrap_or_default(),
            entry: manifest.entry.unwrap_or_default(),
            enabled: true,
        })
    }

    pub fn games(&self) -> &[PackageInfo] { &self.games }
    pub fn screensavers(&self) -> &[PackageInfo] { &self.screensavers }
    pub fn bosses(&self) -> &[PackageInfo] { &self.bosses }
    pub fn errors(&self) -> &[String] { &self.scan_errors }
    pub fn total_count(&self) -> usize {
        self.games.len() + self.screensavers.len() + self.bosses.len()
    }
}
```

### 10.3 在 boot 中调用扫描

```rust
pub fn prepare() -> BootOutput {
    // ...
    println!("[Boot] Scanning packages...");
    services.package.scan_all();
    println!("[Boot] Found {} packages ({} games, {} screensavers, {} bosses)",
        services.package.total_count(),
        services.package.games().len(),
        services.package.screensavers().len(),
        services.package.bosses().len(),
    );
    // ...
}
```

### 10.4 在 render 中显示包数量

```rust
let pkg_info = format!(
    "Packages: {} games, {} screensavers, {} bosses",
    services.package.games().len(),
    services.package.screensavers().len(),
    services.package.bosses().len(),
);
services.render.draw_centered(3, &pkg_info);
```

## 验证

```bash
cargo build
cargo run
```

- 启动输出显示包数量
- 帧渲染显示包统计
- 缺失目录不崩溃
