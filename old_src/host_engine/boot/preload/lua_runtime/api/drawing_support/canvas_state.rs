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
    pub styles: Vec<i64>,
    pub is_continuation: bool,
}

impl Default for CanvasCell {
    fn default() -> Self {
        Self {
            text: " ".to_string(),
            fg: None,
            bg: None,
            styles: Vec::new(),
            is_continuation: false,
        }
    }
}

/// 虚拟画布。按行组织，稀疏存储——仅保存已写入的单元格。
#[derive(Clone, Debug)]
pub struct CanvasState {
    width: u16,
    height: u16,
    rows: BTreeMap<u16, BTreeMap<u16, CanvasCell>>,
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
            rows: BTreeMap::new(),
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

    /// 调整画布尺寸。
    ///
    /// 尺寸变化后由当前 UI 重新渲染，因此这里直接清空旧单元格，避免旧尺寸内容残留。
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.clear();
    }

    /// 清空画布。
    pub fn clear(&mut self) {
        self.rows.clear();
    }

    /// 写入一个单元格。
    pub fn set_cell(&mut self, x: u16, y: u16, cell: CanvasCell) {
        if x < self.width && y < self.height {
            self.rows.entry(y).or_default().insert(x, cell);
        }
    }

    /// 清空一个单元格。
    pub fn erase_cell(&mut self, x: u16, y: u16) {
        if x < self.width && y < self.height {
            if let Some(row) = self.rows.get_mut(&y) {
                row.remove(&x);
                if row.is_empty() {
                    self.rows.remove(&y);
                }
            }
        }
    }

    /// 获取指定行的所有单元格。
    pub fn row(&self, y: u16) -> Option<&BTreeMap<u16, CanvasCell>> {
        self.rows.get(&y)
    }

    /// 所有行的迭代器（按 y 升序）。
    pub fn rows(&self) -> impl Iterator<Item = (&u16, &BTreeMap<u16, CanvasCell>)> {
        self.rows.iter()
    }

    /// 迭代所有已写入单元格（按 y 升序，同 y 按 x 升序）。
    pub fn cells(&self) -> impl Iterator<Item = (u16, u16, &CanvasCell)> {
        self.rows
            .iter()
            .flat_map(|(y, row)| row.iter().map(move |(x, cell)| (*x, *y, cell)))
    }

    /// 获取指定位置的单元格。
    pub fn get_cell(&self, x: u16, y: u16) -> Option<&CanvasCell> {
        self.rows.get(&y).and_then(|row| row.get(&x))
    }
}
