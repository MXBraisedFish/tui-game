// 虚拟画布，供游戏逻辑绘制像素/文本。支持带样式的单元格（前景色、背景色、样式标记）、对齐方式、宽字符处理（如中文、Emoji）

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr}; // 计算字符/字符串显示宽度（处理全角字符）

pub const ALIGN_NO_WRAP: i64 = 0; // 不换行，转义换行符为 \n 显示
pub const ALIGN_LEFT: i64 = 1; // 左对齐
pub const ALIGN_CENTER: i64 = 2; // 居中对齐（基于第一行宽度）
pub const ALIGN_RIGHT: i64 = 3; // 右对齐

pub const STYLE_BOLD: i64 = 0; // 粗体
pub const STYLE_ITALIC: i64 = 1; // 斜体
pub const STYLE_UNDERLINE: i64 = 2; // 下划线
pub const STYLE_STRIKE: i64 = 3; // 删除线
pub const STYLE_BLINK: i64 = 4; // 闪烁
pub const STYLE_REVERSE: i64 = 5; // 反转颜色
pub const STYLE_HIDDEN: i64 = 6; // 隐藏
pub const STYLE_DIM: i64 = 7; // 暗淡

// 画布的最小单位。支持 continuation 标记用于绘制宽度>1的字符（如中文）
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cell {
    pub ch: char, // 字符
    pub fg: Option<String>, // 前景色（CSS 颜色名或 #RRGGBB）
    pub bg: Option<String>, // 背景色
    pub style: Option<i64>, // 样式位掩码（可组合）
    pub continuation: bool, // 是否为宽字符的后续占位单元格
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

// 游戏可写的虚拟画布，与终端实际尺寸解耦
#[derive(Clone, Debug)]
pub struct Canvas {
    width: u16, // 列数
    height: u16, // 行数
    pub(crate) cells: Vec<Cell>, // 行优先存储
}

impl Canvas {
    // 创建指定尺寸的画布，所有单元格初始为默认 Cell
    pub fn new(width: u16, height: u16) -> Self {
        let len = usize::from(width) * usize::from(height);
        Self {
            width,
            height,
            cells: vec![Cell::default(); len],
        }
    }

    // 获取宽度
    pub fn width(&self) -> u16 {
        self.width
    }

    // 获取高度
    pub fn height(&self) -> u16 {
        self.height
    }

    // 重新分配画布，丢弃原有内容
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.cells = vec![Cell::default(); usize::from(width) * usize::from(height)];
    }

    // 将所有单元格重置为默认值（空格，无色，无样式）
    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
    }

    // 坐标，单元格
    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) {
        if let Some(index) = self.index(x, y) {
            self.cells[index] = cell;
        }
    }

    // 矩形区域，单元格模板
    pub fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, cell: Cell) {
        for row in 0..height {
            for col in 0..width {
                self.set_cell(x.saturating_add(col), y.saturating_add(row), cell.clone());
            }
        }
    }

    // 起始坐标，文本，颜色，样式，对齐方式
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

    // 起始 x（可为负数），行 y，文本
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

    // 统计当前画布与之前画布不同的单元格数量（用于增量渲染优化）
    pub fn diff_count(&self, previous: &Self) -> usize {
        self.cells
            .iter()
            .zip(previous.cells.iter())
            .filter(|(left, right)| left != right)
            .count()
    }

    // 获取某一行的切片
    pub fn row(&self, y: u16) -> Option<&[Cell]> {
        if y >= self.height {
            return None;
        }
        let start = usize::from(y) * usize::from(self.width);
        let end = start + usize::from(self.width);
        self.cells.get(start..end)
    }

    // 静态方法，测量文本渲染所需宽度和高度（按显示宽度，非字符数）
    pub fn measure_text(text: &str) -> (u16, u16) {
        let mut width = 0usize;
        let mut height = 0u16;
        for line in text.split('\n') {
            height = height.saturating_add(1);
            width = width.max(UnicodeWidthStr::width(line));
        }
        (width as u16, height.max(1))
    }

    // 将二维坐标转换为 cells 数组索引
    fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(usize::from(y) * usize::from(self.width) + usize::from(x))
    }
}
