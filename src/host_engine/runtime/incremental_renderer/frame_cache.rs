//! 增量渲染帧缓存

use std::collections::BTreeMap;

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::canvas_state::{
    CanvasCell, CanvasState,
};

/// 已渲染帧快照。按行组织以支持行级跳过和段合并。
#[derive(Clone, Debug, Default)]
pub struct FrameCache {
    width: u16,
    height: u16,
    rows: BTreeMap<u16, BTreeMap<u16, CanvasCell>>,
}

impl FrameCache {
    /// 从当前画布构造帧快照。
    pub fn from_canvas_state(canvas_state: &CanvasState) -> Self {
        let mut rows: BTreeMap<u16, BTreeMap<u16, CanvasCell>> = BTreeMap::new();
        for (x, y, cell) in canvas_state.cells() {
            rows.entry(y).or_default().insert(x, cell.clone());
        }
        Self {
            width: canvas_state.width(),
            height: canvas_state.height(),
            rows,
        }
    }

    /// 帧宽度。
    pub fn width(&self) -> u16 {
        self.width
    }

    /// 帧高度。
    pub fn height(&self) -> u16 {
        self.height
    }

    /// 获取指定行。
    pub fn row(&self, y: u16) -> Option<&BTreeMap<u16, CanvasCell>> {
        self.rows.get(&y)
    }

    /// 所有行迭代（按 y 升序）。
    pub fn rows(&self) -> impl Iterator<Item = (&u16, &BTreeMap<u16, CanvasCell>)> {
        self.rows.iter()
    }
}
