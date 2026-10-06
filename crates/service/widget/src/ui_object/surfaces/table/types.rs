//! Identifiers, configuration, states, and events shared by this module.

/// The identity of table within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TableId(
  /// The wrapped u64 value.
  pub u64,
);

/// Horizontal alignment of text within a table cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableAlign {
  /// The left setting for table align.
  Left,
  /// The center setting for table align.
  Center,
  /// The right setting for table align.
  Right,
}

/// The clipping or wrapping policy for text exceeding its column width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableOverflow {
  /// The clip setting for table overflow.
  Clip,
  /// The ellipsis setting for table overflow.
  Ellipsis,
  /// The wrap setting for table overflow.
  Wrap,
}

/// The border layout selected for a table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableBorderMode {
  /// The none setting for table border mode.
  None,
  /// The header only setting for table border mode.
  HeaderOnly,
  /// The full setting for table border mode.
  Full,
}

/// The table border style representation used by this module.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TableBorderStyle {
  /// The single setting for table border style.
  #[default]
  Single,
  /// The double setting for table border style.
  Double,
  /// The double outer single inner setting for table border style.
  DoubleOuterSingleInner,
}

/// The table column representation used by this module.
///
/// # Fields
///
/// * `key` - The lookup key.
/// * `title` - The title.
/// * `width` - The width in terminal columns.
/// * `min_width` - The min width in terminal columns.
/// * `align` - The align.
/// * `overflow` - The overflow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableColumn {
  /// The lookup key.
  pub key: String,
  /// The title.
  pub title: String,
  /// The width in terminal columns.
  pub width: u16,
  /// The min width in terminal columns.
  pub min_width: u16,
  /// The align.
  pub align: TableAlign,
  /// The overflow.
  pub overflow: TableOverflow,
}

impl TableColumn {
  /// Create a table column with the supplied fixed width.
  ///
  /// # Arguments
  ///
  /// * `key` - The lookup key.
  /// * `title` - The title.
  /// * `width` - The width in terminal columns.
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

  /// Set the table column alignment and return the updated column configuration.
  pub fn align(mut self, align: TableAlign) -> Self {
    self.align = align;
    self
  }

  /// Set the table column's minimum width and return the updated configuration.
  pub fn min_width(mut self, min_width: u16) -> Self {
    self.min_width = min_width.min(self.width);
    self
  }

  /// Set the table column overflow rule and return the updated configuration.
  pub fn overflow(mut self, overflow: TableOverflow) -> Self {
    self.overflow = overflow;
    self
  }
}

/// The table cell representation used by this module.
///
/// # Fields
///
/// * `text` - The text to process or display.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableCell {
  /// The text to process or display.
  pub text: String,
}

/// The table row representation used by this module.
///
/// # Fields
///
/// * `cells` - The ordered cells retained by this owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableRow {
  /// The ordered cells retained by this owner.
  pub cells: Vec<TableCell>,
}

impl TableRow {
  /// Create a table row whose cells contain the supplied text values.
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

/// The table style representation used by this module.
///
/// # Fields
///
/// * `border_mode` - The border mode.
/// * `border_style` - The border style.
/// * `column_gap` - The column gap.
/// * `show_header` - The show header.
/// * `show_empty_message` - The show empty message.
/// * `empty_message` - The empty message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableStyle {
  /// The border mode.
  pub border_mode: TableBorderMode,
  /// The border style.
  pub border_style: TableBorderStyle,
  /// The column gap.
  pub column_gap: u16,
  /// The show header.
  pub show_header: bool,
  /// The show empty message.
  pub show_empty_message: bool,
  /// The empty message.
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

/// Configuration values controlling table behavior.
///
/// # Fields
///
/// * `columns` - The ordered columns retained by this owner.
/// * `style` - The text style applied to the rendered content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableOptions {
  /// The ordered columns retained by this owner.
  pub columns: Vec<TableColumn>,
  /// The text style applied to the rendered content.
  pub style: TableStyle,
}

impl TableOptions {
  /// Create a table options initialized from `columns`.
  pub fn new(columns: Vec<TableColumn>) -> Self {
    Self {
      columns,
      style: TableStyle::default(),
    }
  }
}

/// Configuration values controlling table draw behavior.
///
/// # Fields
///
/// * `id` - The identifier of the owned object.
/// * `x` - The horizontal coordinate in terminal cells.
/// * `y` - The vertical coordinate in terminal cells.
/// * `width` - The width in terminal columns.
/// * `height` - The height in terminal rows.
/// * `rows` - The rows.
/// * `row_offset` - The row offset.
pub struct TableDrawParams<'a> {
  /// The identifier of the owned object.
  pub id: TableId,
  /// The horizontal coordinate in terminal cells.
  pub x: u16,
  /// The vertical coordinate in terminal cells.
  pub y: u16,
  /// The width in terminal columns.
  pub width: u16,
  /// The height in terminal rows.
  pub height: u16,
  /// The rows.
  pub rows: &'a [TableRow],
  /// The row offset.
  pub row_offset: usize,
}
