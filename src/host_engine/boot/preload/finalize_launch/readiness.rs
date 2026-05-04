//! 启动就绪快照

use std::path::PathBuf;

/// 启动前最终准备结果。
#[derive(Clone, Debug)]
pub struct LaunchReadiness {
    pub game_count: usize,
    pub game_scan_error_count: usize,
    pub official_ui_package_count: usize,
    pub official_ui_scan_error_count: usize,
    pub removed_game_cache_count: usize,
    pub image_cache_dir: PathBuf,
    pub todo_items: Vec<&'static str>,
}

impl LaunchReadiness {
    /// 当前是否存在尚未完成的启动拼合事项。
    pub fn has_todo_items(&self) -> bool {
        !self.todo_items.is_empty()
    }
}
