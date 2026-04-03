use unicode_width::UnicodeWidthStr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: Option<String>,
    pub bg: Option<String>,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: None,
            bg: None,
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
    ) {
        let mut cursor_x = x;
        for ch in text.chars() {
            if ch == '\n' {
                break;
            }
            self.set_cell(
                cursor_x,
                y,
                Cell {
                    ch,
                    fg: fg.clone(),
                    bg: bg.clone(),
                },
            );
            cursor_x = cursor_x.saturating_add(1);
        }
    }

    pub fn diff_count(&self, previous: &Self) -> usize {
        self.cells
            .iter()
            .zip(previous.cells.iter())
            .filter(|(left, right)| left != right)
            .count()
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
