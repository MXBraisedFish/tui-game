#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TableId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableAlign {
  Left,
  Center,
  Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableOverflow {
  Clip,
  Ellipsis,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableBorderMode {
  None,
  HeaderOnly,
  Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableBorderStyle {
  Single,
  Double,
  DoubleOuterSingleInner,
}

impl Default for TableBorderStyle {
  fn default() -> Self {
    Self::Single
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableColumn {
  pub key: String,
  pub title: String,
  pub width: u16,
  pub min_width: u16,
  pub align: TableAlign,
  pub overflow: TableOverflow,
}

impl TableColumn {
  pub fn fixed(key: impl Into<String>, title: impl Into<String>, width: u16) -> Self {
    Self {
      key: key.into(),
      title: title.into(),
      width,
      min_width: width.min(4),
      align: TableAlign::Left,
      overflow: TableOverflow::Ellipsis,
    }
  }

  pub fn align(mut self, align: TableAlign) -> Self {
    self.align = align;
    self
  }

  pub fn min_width(mut self, min_width: u16) -> Self {
    self.min_width = min_width.min(self.width);
    self
  }

  pub fn overflow(mut self, overflow: TableOverflow) -> Self {
    self.overflow = overflow;
    self
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableCell {
  pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableRow {
  pub cells: Vec<TableCell>,
}

impl TableRow {
  pub fn from_texts<I, S>(texts: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self {
      cells: texts
        .into_iter()
        .map(|text| TableCell { text: text.into() })
        .collect(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableStyle {
  pub border_mode: TableBorderMode,
  pub border_style: TableBorderStyle,
  pub column_gap: u16,
  pub show_header: bool,
  pub show_empty_message: bool,
  pub empty_message: String,
}

impl Default for TableStyle {
  fn default() -> Self {
    Self {
      border_mode: TableBorderMode::HeaderOnly,
      border_style: TableBorderStyle::Single,
      column_gap: 2,
      show_header: true,
      show_empty_message: true,
      empty_message: "No data".to_string(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableOptions {
  pub columns: Vec<TableColumn>,
  pub style: TableStyle,
}

impl TableOptions {
  pub fn new(columns: Vec<TableColumn>) -> Self {
    Self {
      columns,
      style: TableStyle::default(),
    }
  }
}

pub struct TableDrawParams<'a> {
  pub id: TableId,
  pub x: u16,
  pub y: u16,
  pub width: u16,
  pub height: u16,
  pub rows: &'a [TableRow],
  pub row_offset: usize,
}
