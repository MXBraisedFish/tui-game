//! 增量渲染帧缓存

use std::collections::BTreeMap;

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::canvas_state::{
    CanvasCell, CanvasState,
};

/// 已渲染帧快照。
#[derive(Clone, Debug, Default)]
pub struct FrameCache {
    width: u16,
    height: u16,
    cells: BTreeMap<(u16, u16), CanvasCell>,
}

impl FrameCache {
    /// 从当前画布构造帧快照。
    pub fn from_canvas_state(canvas_state: &CanvasState) -> Self {
        let cells = canvas_state
            .cells()
            .map(|(position, cell)| (*position, cell.clone()))
            .collect();
        Self {
            width: canvas_state.width(),
            height: canvas_state.height(),
            cells,
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

    /// 帧内单元格。
    pub fn cells(&self) -> &BTreeMap<(u16, u16), CanvasCell> {
        &self.cells
    }
}
