//! 持久化数据预读取入口
// TODO: 迁移至 storage::ProfileStore，完成后删除此文件

pub mod display_profile;
pub mod keybind_profile;
mod profile_data;
pub mod security_profile;

pub use profile_data::PersistentData;

/// 读取 data/profiles 下的持久化数据。
///
/// 此阶段只负责读取和校验，不读取 cache/，也不与游戏模块扫描结果做合并。
pub fn load() -> Result<PersistentData, Box<dyn std::error::Error>> {
    let store = crate::host_engine::storage::profile_store::ProfileStore::open()?;
    let game_state = store.game_state_value();
    let saver_state = store.saver_state_value();
    let boss_state = store.boss_state_value();
    Ok(PersistentData {
        saves: store.saves,
        best_scores: store.best_scores,
        language_code: store.language,
        keybinds: store.keybinds,
        game_state,
        saver_state,
        boss_state,
        security_state: store.security.to_value(),
        display_state: store.display.to_value(),
    })
}
