//! 帧差异计算（按行比较 + 行内段合并）

use std::collections::BTreeSet;

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::canvas_state::CanvasCell;

use super::frame_cache::FrameCache;

/// 单个渲染段——一行中连续、样式相同的已变化单元格合并结果。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderSegment {
    pub x: u16,
    pub y: u16,
    pub text: String,
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub styles: Vec<i64>,
}

/// 计算上一帧到当前帧的变化段列表。
pub fn diff_frames(previous_frame: &FrameCache, current_frame: &FrameCache) -> Vec<RenderSegment> {
    let mut segments = Vec::new();

    let mut all_y: BTreeSet<u16> = BTreeSet::new();
    all_y.extend(previous_frame.rows().map(|(y, _)| *y));
    all_y.extend(current_frame.rows().map(|(y, _)| *y));

    for y in all_y {
        let prev_row = previous_frame.row(y);
        let curr_row = current_frame.row(y);

        // 整行未变化，跳过
        if prev_row == curr_row {
            continue;
        }

        let mut all_x: BTreeSet<u16> = BTreeSet::new();
        if let Some(row) = prev_row {
            all_x.extend(row.keys().copied());
        }
        if let Some(row) = curr_row {
            all_x.extend(row.keys().copied());
        }

        let mut x_iter = all_x.into_iter().peekable();
        while let Some(x) = x_iter.next() {
            let prev_cell = prev_row.and_then(|row| row.get(&x));
            let curr_cell = curr_row.and_then(|row| row.get(&x));

            if prev_cell == curr_cell {
                continue;
            }

            let default_cell = CanvasCell::default();
            let cell = match curr_cell {
                Some(cell) if cell.is_continuation => continue,
                Some(cell) => cell,
                None => &default_cell,
            };

            let segment_fg = cell.fg.clone();
            let segment_bg = cell.bg.clone();
            let segment_styles = cell.styles.clone();
            let mut text = cell.text.clone();
            let mut next_x = x + 1;

            // 向后合并相同样式的连续变化单元格
            while let Some(&nx) = x_iter.peek() {
                if nx != next_x {
                    break;
                }
                let np = prev_row.and_then(|row| row.get(&nx));
                let nc = curr_row.and_then(|row| row.get(&nx));
                if np == nc {
                    // 连续且未变，停止合并当前段
                    break;
                }
                let default_for_merge = CanvasCell::default();
                let next_cell = match nc {
                    Some(c) if c.is_continuation => {
                        x_iter.next();
                        next_x += 1;
                        continue;
                    }
                    Some(c) => c,
                    None => &default_for_merge,
                };
                if next_cell.fg != segment_fg
                    || next_cell.bg != segment_bg
                    || next_cell.styles != segment_styles
                {
                    break;
                }
                text.push_str(&next_cell.text);
                x_iter.next();
                next_x += 1;
            }

            segments.push(RenderSegment {
                x,
                y,
                text,
                fg: segment_fg,
                bg: segment_bg,
                styles: segment_styles,
            });
        }
    }

    segments
}
