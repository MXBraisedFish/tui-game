use super::{HostState, MainHostState, OverlayStackState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeState {
    pub main_host: MainHostState,
    pub overlays: OverlayStackState,
}

// Runtime状态
impl RuntimeState {
    // 创建默认 Host 运行时（根 UI 树，无覆盖层）
    pub fn new_host_runtime() -> Self {
        Self {
            main_host: MainHostState::Host(HostState::new()),
            overlays: OverlayStackState::new(),
        }
    }

    // MainHost状态访问方法
    pub fn main_host(&self) -> &MainHostState {
        &self.main_host
    }

    // MainHost状态访问方法（可变）
    pub fn main_host_mut(&mut self) -> &mut MainHostState {
        &mut self.main_host
    }

    // 覆盖层状态访问方法
    pub fn overlays(&self) -> &OverlayStackState {
        &self.overlays
    }

    // 覆盖层状态访问方法（可变）
    pub fn overlays_mut(&mut self) -> &mut OverlayStackState {
        &mut self.overlays
    }

    // 检查是否有覆盖层
    pub fn has_overlay(&self) -> bool {
        !self.overlays.stack.is_empty()
    }

    // 切换MainHost状态
    pub fn set_main_host(&mut self, main_host: MainHostState) {
        self.main_host = main_host;
    }
}
