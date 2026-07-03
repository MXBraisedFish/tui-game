pub use super::widget::{UiEvent, UiObjectPool, UiObjectPoolOwner};

/// UI 服务（无状态标记类型）
#[derive(Clone, Debug)]
pub struct UiService;

impl UiService {
  pub fn new() -> Self {
    Self
  }
}
