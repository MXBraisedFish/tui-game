/// 虚拟画布，供游戏逻辑绘制像素/文本

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub const ALIGN_NO_WRAP: i64 = 0;
pub const ALIGN_LEFT: i64 = 1;
pub const ALIGN_CENTER: i64 = 2;
pub const ALIGN_RIGHT: i64 = 3;

pub const STYLE_BOLD: i64 = 0;
pub const STYLE_ITALIC: i64 = 1;
pub const STYLE_UNDERLINE: i64 = 2;
pub const STYLE_STRIKE: i64 = 3;
pub const STYLE_BLINK: i64 = 4;
pub const STYLE_REVERSE: i64 = 5;
pub const STYLE_HIDDEN: i64 = 6;
pub const STYLE_DIM: i64 = 7;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub style: Option<i64>,
    pub continuation: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: None,
            bg: None,
            style: None,
            continuation: false,
        }
    }
}

/// 游戏逻辑写入的虚拟画布。
#[derive(Clone, Debug)]
pub struct Canvas {
    width: u16,
    height: u16,
    pub(crate) cells: Vec<Cell>,
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        let len = usize::from(width) * usize::from(height);
        Self {
            width,
            height,
            cells: vec![Cell::default(); len],
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.cells = vec![Cell::default(); usize::from(width) * usize::from(height)];
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
    }

    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) {
        if let Some(index) = self.index(x, y) {
            self.cells[index] = cell;
        }
    }

    pub fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, cell: Cell) {
        for row in 0..height {
            for col in 0..width {
                self.set_cell(x.saturating_add(col), y.saturating_add(row), cell.clone());
            }
        }
    }

    pub fn draw_text(
        &mut self,
        x: u16,
        y: u16,
        text: &str,
        fg: Option<String>,
        bg: Option<String>,
        style: Option<i64>,
        align: i64,
    ) {
        if align == ALIGN_NO_WRAP {
            let escaped = text.replace('\n', "\\n");
            self.draw_text_line(i64::from(x), y, &escaped, fg, bg, style);
            return;
        }

        let first_line = text.split('\n').next().unwrap_or("");
        let first_width = UnicodeWidthStr::width(first_line) as i64;

        for (row_offset, line) in text.split('\n').enumerate() {
            let line_width = UnicodeWidthStr::width(line) as i64;
            let start_x = match align {
                ALIGN_CENTER => i64::from(x) + ((first_width - line_width) / 2),
                ALIGN_RIGHT => i64::from(x) + (first_width - line_width),
                _ => i64::from(x),
            };
            let draw_y = y.saturating_add(row_offset as u16);
            self.draw_text_line(start_x, draw_y, line, fg.clone(), bg.clone(), style);
        }
    }

    fn draw_text_line(
        &mut self,
        start_x: i64,
        y: u16,
        text: &str,
        fg: Option<String>,
        bg: Option<String>,
        style: Option<i64>,
    ) {
        let mut cursor_x = start_x;
        for ch in text.chars() {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0) as i64;
            if ch_width == 0 {
                continue;
            }
            if cursor_x >= 0 {
                self.set_cell(
                    cursor_x as u16,
                    y,
                    Cell {
                        ch,
                        fg: fg.clone(),
                        bg: bg.clone(),
                        style,
                        continuation: false,
                    },
                );
                if ch_width > 1 {
                    for extra in 1..ch_width {
                        let continuation_x = cursor_x + extra;
                        if continuation_x >= 0 {
                            self.set_cell(
                                continuation_x as u16,
                                y,
                                Cell {
                                    ch: ' ',
                                    fg: fg.clone(),
                                    bg: bg.clone(),
                                    style,
                                    continuation: true,
                                },
                            );
                        }
                    }
                }
            }
            cursor_x += ch_width;
        }
    }

    pub fn diff_count(&self, previous: &Self) -> usize {
        self.cells
            .iter()
            .zip(previous.cells.iter())
            .filter(|(left, right)| left != right)
            .count()
    }

    pub fn row(&self, y: u16) -> Option<&[Cell]> {
        if y >= self.height {
            return None;
        }
        let start = usize::from(y) * usize::from(self.width);
        let end = start + usize::from(self.width);
        self.cells.get(start..end)
    }

    pub fn measure_text(text: &str) -> (u16, u16) {
        let mut width = 0usize;
        let mut height = 0u16;
        for line in text.split('\n') {
            height = height.saturating_add(1);
            width = width.max(UnicodeWidthStr::width(line));
        }
        (width as u16, height.max(1))
    }

    fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(usize::from(y) * usize::from(self.width) + usize::from(x))
    }
}
