//! 官方 UI 包预加载入口

mod manifest;
mod scanner;

pub use manifest::{OfficialUiPackage, OfficialUiRegistry, OfficialUiScanError};

/// 读取官方 UI 包。
///
/// 当前阶段只建立入口和基础扫描能力，后续 Lua UI 脚本接入时再扩展 manifest 校验与加载逻辑。
pub fn load() -> Result<OfficialUiRegistry, Box<dyn std::error::Error>> {
    scanner::scan_official_ui()
}
