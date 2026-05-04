//! 官方 UI 包清单结构

use std::path::PathBuf;

use serde::Serialize;
use serde_json::Value;

/// 成功读取的官方 UI 包
#[derive(Clone, Debug, Serialize)]
pub struct OfficialUiPackage {
    pub id: String,
    pub root_dir: PathBuf,
    pub manifest: Value,
}

/// 官方 UI 包读取错误
#[derive(Clone, Debug, Default, Serialize)]
pub struct OfficialUiScanError {
    pub path: String,
    pub error: String,
}

/// 官方 UI 包注册表
#[derive(Clone, Debug, Default, Serialize)]
pub struct OfficialUiRegistry {
    pub packages: Vec<OfficialUiPackage>,
    pub errors: Vec<OfficialUiScanError>,
}
