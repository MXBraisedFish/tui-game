//! 虚拟画布状态

use std::collections::BTreeMap;

const DEFAULT_CANVAS_WIDTH: u16 = 80;
const DEFAULT_CANVAS_HEIGHT: u16 = 24;

/// 画布单元格。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanvasCell {
    pub text: String,
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub style: Option<i64>,
    pub is_continuation: bool,
}

impl Default for CanvasCell {
    fn default() -> Self {
        Self {
            text: " ".to_string(),
            fg: None,
            bg: None,
            style: None,
            is_continuation: false,
        }
    }
}

/// 虚拟画布。
#[derive(Clone, Debug)]
pub struct CanvasState {
    width: u16,
    height: u16,
    cells: BTreeMap<(u16, u16), CanvasCell>,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self::new(DEFAULT_CANVAS_WIDTH, DEFAULT_CANVAS_HEIGHT)
    }
}

impl CanvasState {
    /// 创建固定尺寸画布。
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            cells: BTreeMap::new(),
        }
    }

    /// 画布宽度。
    pub fn width(&self) -> u16 {
        self.width
    }

    /// 画布高度。
    pub fn height(&self) -> u16 {
        self.height
    }

    /// 清空画布。
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// 写入一个单元格。
    pub fn set_cell(&mut self, x: u16, y: u16, cell: CanvasCell) {
        if x < self.width && y < self.height {
            self.cells.insert((x, y), cell);
        }
    }

    /// 清空一个单元格。
    pub fn erase_cell(&mut self, x: u16, y: u16) {
        if x < self.width && y < self.height {
            self.cells.remove(&(x, y));
        }
    }
}
