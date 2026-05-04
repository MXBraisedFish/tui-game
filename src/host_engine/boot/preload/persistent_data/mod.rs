//! 持久化数据预读取入口

mod loader;
mod profile_data;

pub use profile_data::PersistentData;

/// 读取 data/profiles 下的持久化数据。
///
/// 此阶段只负责读取和校验，不读取 cache/，也不与游戏模块扫描结果做合并。
pub fn load() -> Result<PersistentData, Box<dyn std::error::Error>> {
    loader::load_persistent_data()
}
