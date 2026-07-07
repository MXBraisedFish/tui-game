/// 宿主版本号，来源于 Cargo 包版本。
pub const HOST_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 当前宿主支持的包 API 版本。
pub const HOST_API_VERSION: u32 = 1;

/// 当前宿主支持的 package.json 清单版本。
pub const PACKAGE_MANIFEST_VERSION: u32 = 1;
