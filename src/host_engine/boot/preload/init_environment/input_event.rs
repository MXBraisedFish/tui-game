//! 宿主输入事件类型

use super::resize_watcher::ResizeEvent;

/// 宿主输入事件
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostInputEvent {
    Key { key: String },
    Resize(ResizeEvent),
    ExitRequested,
}
