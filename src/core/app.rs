use crate::game::registry::GameRegistry;

/// 统一运行时下的应用级共享状态。
#[derive(Default)]
pub struct AppContext {
    pub registry: GameRegistry,
}
