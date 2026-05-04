//! 帧差异计算

use std::collections::BTreeSet;

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::canvas_state::CanvasCell;

use super::frame_cache::FrameCache;

/// 单个终端输出变化。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderChange {
    pub x: u16,
    pub y: u16,
    pub cell: CanvasCell,
}

/// 计算上一帧到当前帧的变化列表。
pub fn diff_frames(previous_frame: &FrameCache, current_frame: &FrameCache) -> Vec<RenderChange> {
    let mut positions = BTreeSet::new();
    positions.extend(previous_frame.cells().keys().copied());
    positions.extend(current_frame.cells().keys().copied());

    positions
        .into_iter()
        .filter_map(|(x, y)| {
            let previous_cell = previous_frame.cells().get(&(x, y));
            let current_cell = current_frame.cells().get(&(x, y));
            if previous_cell == current_cell {
                return None;
            }

            match current_cell {
                Some(cell) if cell.is_continuation => {
                    if previous_cell.is_some() {
                        Some(RenderChange {
                            x,
                            y,
                            cell: CanvasCell::default(),
                        })
                    } else {
                        None
                    }
                }
                Some(cell) => Some(RenderChange {
                    x,
                    y,
                    cell: cell.clone(),
                }),
                None => Some(RenderChange {
                    x,
                    y,
                    cell: CanvasCell::default(),
                }),
            }
        })
        .collect()
}
