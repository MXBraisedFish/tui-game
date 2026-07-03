use crate::host_engine::services::MouseButton;

/// 点击区域唯一标识
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HitAreaId(pub u64);

/// 点击区域配置选项
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HitAreaOptions {
  pub hover_move: bool,
  pub drag: bool,
}

/// 点击区域事件
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HitAreaEvent {
  HoverEnter {
    id: HitAreaId,
    x: u16,
    y: u16,
  },
  HoverMove {
    id: HitAreaId,
    x: u16,
    y: u16,
  },
  HoverLeave {
    id: HitAreaId,
    x: u16,
    y: u16,
  },
  Press {
    id: HitAreaId,
    button: MouseButton,
    x: u16,
    y: u16,
  },
  Release {
    id: HitAreaId,
    button: MouseButton,
    x: u16,
    y: u16,
  },
  Click {
    id: HitAreaId,
    button: MouseButton,
    x: u16,
    y: u16,
  },
  Drag {
    id: HitAreaId,
    button: MouseButton,
    x: u16,
    y: u16,
    dx: i32,
    dy: i32,
  },
}
