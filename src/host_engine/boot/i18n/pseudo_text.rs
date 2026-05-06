//! 可更新伪常量文本

use std::sync::RwLock;

use once_cell::sync::OnceCell;

/// 可更新文本存储。
///
/// 外层 `OnceCell` 只初始化锁，内层 `RwLock<String>` 允许运行期覆盖文本。
pub struct MutableText {
    value: OnceCell<RwLock<String>>,
}

impl MutableText {
    /// 创建空文本。
    pub const fn new() -> Self {
        Self {
            value: OnceCell::new(),
        }
    }

    /// 覆盖当前文本。
    pub fn set(&'static self, value: String) {
        let lock = self.value.get_or_init(|| RwLock::new(String::new()));
        if let Ok(mut current_value) = lock.write() {
            *current_value = value;
        }
    }

    /// 读取当前文本快照。
    pub fn get(&'static self) -> String {
        self.value
            .get()
            .and_then(|lock| lock.read().ok().map(|value| value.clone()))
            .unwrap_or_default()
    }
}
