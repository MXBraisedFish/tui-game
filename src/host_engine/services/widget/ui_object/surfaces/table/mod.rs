mod state;
mod types;

use std::collections::HashSet;

pub(crate) use self::state::TableObjects;
use self::state::TableState;
pub use self::types::{
  TableAlign, TableBorderMode, TableBorderStyle, TableCell, TableColumn, TableDrawParams, TableId,
  TableOptions, TableOverflow, TableRow, TableStyle,
};
use crate::host_engine::services::text_layout::{self, DrawTextParams, TextWrapMode};
use crate::host_engine::services::ui::UiObjectPool;
use crate::host_engine::services::{CanvasService, SliceId};

pub struct TableService;

#[derive(Clone, Copy)]
enum TableTarget {
  Base,
  Slice(SliceId),
  Host,
}

impl TableService {
  pub fn new() -> Self {
    Self
  }

  pub fn create(&self, pool: &mut UiObjectPool, options: TableOptions) -> Option<TableId> {
    validate_options(&options).then(|| {
      let id = TableId(pool.tables.next_id);
      pool.tables.next_id += 1;
      pool.tables.tables.insert(id, TableState { options });
      id
    })
  }

  pub fn remove(&self, pool: &mut UiObjectPool, id: TableId) -> bool {
    pool.tables.tables.remove(&id).is_some()
  }

  pub fn exists(&self, pool: &UiObjectPool, id: TableId) -> bool {
    pool.tables.tables.contains_key(&id)
  }

  pub fn set_columns(
    &self,
    pool: &mut UiObjectPool,
    id: TableId,
    columns: Vec<TableColumn>,
  ) -> bool {
    let Some(state) = pool.tables.tables.get_mut(&id) else {
      return false;
    };
    let options = TableOptions {
      columns,
      style: state.options.style.clone(),
    };
    if !validate_options(&options) {
      return false;
    }
    state.options.columns = options.columns;
    true
  }

  pub fn set_style(&self, pool: &mut UiObjectPool, id: TableId, style: TableStyle) -> bool {
    let Some(state) = pool.tables.tables.get_mut(&id) else {
      return false;
    };
    state.options.style = style;
    true
  }

  pub fn options<'a>(&self, pool: &'a UiObjectPool, id: TableId) -> Option<&'a TableOptions> {
    Some(&pool.tables.tables.get(&id)?.options)
  }

  pub fn draw(
    &self,
    pool: &UiObjectPool,
    canvas: &mut CanvasService,
    params: TableDrawParams<'_>,
  ) -> bool {
    self.draw_to(pool, canvas, TableTarget::Base, params)
  }

  pub fn draw_on(
    &self,
    pool: &UiObjectPool,
    canvas: &mut CanvasService,
    slice: SliceId,
    params: TableDrawParams<'_>,
  ) -> bool {
    self.draw_to(pool, canvas, TableTarget::Slice(slice), params)
  }

  pub(crate) fn draw_host(
    &self,
    pool: &UiObjectPool,
    canvas: &mut CanvasService,
    params: TableDrawParams<'_>,
  ) -> bool {
    self.draw_to(pool, canvas, TableTarget::Host, params)
  }

  fn draw_to(
    &self,
    pool: &UiObjectPool,
    canvas: &mut CanvasService,
    target: TableTarget,
    params: TableDrawParams<'_>,
  ) -> bool {
    if params.width == 0 || params.height == 0 {
      return false;
    }
    let Some(options) = self.options(pool, params.id) else {
      return false;
    };
    if options.columns.is_empty() {
      return false;
    }

    let columns = effective_columns(&options.columns, &options.style, params.width);
    let mut y = params.y;
    let bottom = params.y.saturating_add(params.height);

    if options.style.border_mode == TableBorderMode::Full {
      if y >= bottom {
        return true;
      }
      self.draw_full_border_line(
        canvas,
        target,
        params.x,
        y,
        &columns,
        &options.style,
        BorderLine::Top,
      );
      y = y.saturating_add(1);
    }

    if options.style.show_header && y < bottom {
      self.draw_row(
        canvas,
        target,
        params.x,
        y,
        &columns,
        &options
          .columns
          .iter()
          .map(|column| TableCell {
            text: column.title.clone(),
          })
          .collect::<Vec<_>>(),
        &options.style,
      );
      y = y.saturating_add(1);
    }

    if options.style.show_header
      && matches!(
        options.style.border_mode,
        TableBorderMode::HeaderOnly | TableBorderMode::Full
      )
      && y < bottom
    {
      match options.style.border_mode {
        TableBorderMode::Full => {
          self.draw_full_border_line(
            canvas,
            target,
            params.x,
            y,
            &columns,
            &options.style,
            BorderLine::Middle,
          );
        }
        _ => self.draw_header_line(canvas, target, params.x, y, &columns, &options.style),
      }
      y = y.saturating_add(1);
    }

    if params.rows.is_empty() {
      if options.style.show_empty_message && y < bottom {
        self.draw_text(
          canvas,
          target,
          &DrawTextParams {
            x: content_x(params.x, &options.style),
            y,
            text: options.style.empty_message.clone(),
            max_width: Some(content_width(&columns, &options.style)),
            max_height: Some(1),
            overflow_marker: Some("...".to_string()),
            ..Default::default()
          },
        );
      }
    } else {
      for row in params.rows.iter().skip(params.row_offset) {
        if y >= bottom {
          break;
        }
        self.draw_row(
          canvas,
          target,
          params.x,
          y,
          &columns,
          &row.cells,
          &options.style,
        );
        y = y.saturating_add(1);
      }
    }

    if options.style.border_mode == TableBorderMode::Full && bottom > params.y {
      self.draw_full_border_line(
        canvas,
        target,
        params.x,
        bottom.saturating_sub(1),
        &columns,
        &options.style,
        BorderLine::Bottom,
      );
    }

    true
  }

  fn draw_row(
    &self,
    canvas: &mut CanvasService,
    target: TableTarget,
    x: u16,
    y: u16,
    columns: &[EffectiveColumn],
    cells: &[TableCell],
    style: &TableStyle,
  ) {
    let mut cursor = x;
    if style.border_mode == TableBorderMode::Full {
      self.draw_plain(canvas, target, cursor, y, border_chars(style).outer_v);
      cursor = cursor.saturating_add(1);
    }

    for (index, column) in columns.iter().enumerate() {
      let text = cells
        .get(index)
        .map(|cell| cell.text.as_str())
        .unwrap_or("");
      self.draw_cell(canvas, target, cursor, y, column, text);
      cursor = cursor.saturating_add(column.width);

      if style.border_mode == TableBorderMode::Full {
        let ch = if index + 1 == columns.len() {
          border_chars(style).outer_v
        } else {
          border_chars(style).inner_v
        };
        self.draw_plain(canvas, target, cursor, y, ch);
        cursor = cursor.saturating_add(1);
      } else if index + 1 < columns.len() {
        cursor = cursor.saturating_add(style.column_gap);
      }
    }
  }

  fn draw_cell(
    &self,
    canvas: &mut CanvasService,
    target: TableTarget,
    x: u16,
    y: u16,
    column: &EffectiveColumn,
    text: &str,
  ) {
    if column.width == 0 {
      return;
    }
    let text_x = x.saturating_add(u16::from(column.width > 1));
    let text_width = column.width.saturating_sub(u16::from(column.width > 1));
    if text_width == 0 {
      return;
    }
    let text = align_cell_text(text, text_width, column.align);
    self.draw_text(
      canvas,
      target,
      &DrawTextParams {
        x: text_x,
        y,
        text,
        max_width: Some(text_width),
        max_height: Some(1),
        wrap_mode: TextWrapMode::Normal,
        overflow_marker: match column.overflow {
          TableOverflow::Clip => None,
          TableOverflow::Ellipsis => Some("...".to_string()),
        },
        ..Default::default()
      },
    );
  }

  fn draw_header_line(
    &self,
    canvas: &mut CanvasService,
    target: TableTarget,
    x: u16,
    y: u16,
    columns: &[EffectiveColumn],
    style: &TableStyle,
  ) {
    let width = content_width(columns, style);
    self.draw_plain(
      canvas,
      target,
      x,
      y,
      &border_chars(style).inner_h.repeat(width as usize),
    );
  }

  fn draw_full_border_line(
    &self,
    canvas: &mut CanvasService,
    target: TableTarget,
    x: u16,
    y: u16,
    columns: &[EffectiveColumn],
    style: &TableStyle,
    line_kind: BorderLine,
  ) {
    let chars = border_chars(style);
    let (left, sep, right, h) = match line_kind {
      BorderLine::Top => (
        chars.top_left,
        chars.top_sep,
        chars.top_right,
        chars.outer_h,
      ),
      BorderLine::Middle => (
        chars.mid_left,
        chars.mid_sep,
        chars.mid_right,
        chars.inner_h,
      ),
      BorderLine::Bottom => (
        chars.bottom_left,
        chars.bottom_sep,
        chars.bottom_right,
        chars.outer_h,
      ),
    };
    let mut line = String::from(left);
    for (index, column) in columns.iter().enumerate() {
      line.push_str(&h.repeat(column.width as usize));
      line.push_str(if index + 1 == columns.len() {
        right
      } else {
        sep
      });
    }
    self.draw_plain(canvas, target, x, y, &line);
  }

  fn draw_plain(
    &self,
    canvas: &mut CanvasService,
    target: TableTarget,
    x: u16,
    y: u16,
    text: &str,
  ) {
    self.draw_text(
      canvas,
      target,
      &DrawTextParams {
        x,
        y,
        text: text.to_string(),
        max_height: Some(1),
        ..Default::default()
      },
    );
  }

  fn draw_text(&self, canvas: &mut CanvasService, target: TableTarget, params: &DrawTextParams) {
    match target {
      TableTarget::Base => canvas.text(params),
      TableTarget::Slice(id) => {
        canvas.text_on(id, params);
      }
      TableTarget::Host => canvas.host_text(params),
    }
  }
}

#[derive(Clone, Debug)]
struct EffectiveColumn {
  width: u16,
  min_width: u16,
  align: TableAlign,
  overflow: TableOverflow,
}

#[derive(Clone, Copy)]
enum BorderLine {
  Top,
  Middle,
  Bottom,
}

struct BorderChars {
  top_left: &'static str,
  top_sep: &'static str,
  top_right: &'static str,
  mid_left: &'static str,
  mid_sep: &'static str,
  mid_right: &'static str,
  bottom_left: &'static str,
  bottom_sep: &'static str,
  bottom_right: &'static str,
  outer_h: &'static str,
  inner_h: &'static str,
  outer_v: &'static str,
  inner_v: &'static str,
}

fn border_chars(style: &TableStyle) -> BorderChars {
  match style.border_style {
    TableBorderStyle::Single => BorderChars {
      top_left: "┌",
      top_sep: "┬",
      top_right: "┐",
      mid_left: "├",
      mid_sep: "┼",
      mid_right: "┤",
      bottom_left: "└",
      bottom_sep: "┴",
      bottom_right: "┘",
      outer_h: "─",
      inner_h: "─",
      outer_v: "│",
      inner_v: "│",
    },
    TableBorderStyle::Double => BorderChars {
      top_left: "╔",
      top_sep: "╦",
      top_right: "╗",
      mid_left: "╠",
      mid_sep: "╬",
      mid_right: "╣",
      bottom_left: "╚",
      bottom_sep: "╩",
      bottom_right: "╝",
      outer_h: "═",
      inner_h: "═",
      outer_v: "║",
      inner_v: "║",
    },
    TableBorderStyle::DoubleOuterSingleInner => BorderChars {
      top_left: "╔",
      top_sep: "╤",
      top_right: "╗",
      mid_left: "╟",
      mid_sep: "┼",
      mid_right: "╢",
      bottom_left: "╚",
      bottom_sep: "╧",
      bottom_right: "╝",
      outer_h: "═",
      inner_h: "─",
      outer_v: "║",
      inner_v: "│",
    },
  }
}

fn validate_options(options: &TableOptions) -> bool {
  if options.columns.is_empty() {
    return false;
  }
  let mut keys = HashSet::new();
  options.columns.iter().all(|column| {
    !column.key.is_empty()
      && keys.insert(column.key.as_str())
      && column.width > 0
      && column.min_width > 0
      && column.min_width <= column.width
  })
}

fn effective_columns(
  columns: &[TableColumn],
  style: &TableStyle,
  available_width: u16,
) -> Vec<EffectiveColumn> {
  let mut result = columns
    .iter()
    .map(|column| EffectiveColumn {
      width: column.width,
      min_width: column.min_width,
      align: column.align,
      overflow: column.overflow,
    })
    .collect::<Vec<_>>();

  while required_width(&result, style) > available_width {
    let mut changed = false;
    for index in (0..result.len()).rev() {
      if required_width(&result, style) <= available_width {
        break;
      }
      let column = &mut result[index];
      if column.width > column.min_width {
        column.width -= 1;
        changed = true;
      }
    }
    if !changed {
      break;
    }
  }
  result
}

fn required_width(columns: &[EffectiveColumn], style: &TableStyle) -> u16 {
  let columns_width = columns.iter().map(|column| column.width).sum::<u16>();
  match style.border_mode {
    TableBorderMode::Full => columns_width.saturating_add(columns.len() as u16 + 1),
    TableBorderMode::None | TableBorderMode::HeaderOnly => columns_width.saturating_add(
      style
        .column_gap
        .saturating_mul(columns.len().saturating_sub(1) as u16),
    ),
  }
}

fn content_x(x: u16, style: &TableStyle) -> u16 {
  x.saturating_add(u16::from(style.border_mode == TableBorderMode::Full))
}

fn content_width(columns: &[EffectiveColumn], style: &TableStyle) -> u16 {
  required_width(columns, style).saturating_sub(match style.border_mode {
    TableBorderMode::Full => 2,
    TableBorderMode::None | TableBorderMode::HeaderOnly => 0,
  })
}

fn align_cell_text(text: &str, width: u16, align: TableAlign) -> String {
  let text_width = text_layout::measure_draw_text(&DrawTextParams {
    text: text.to_string(),
    max_height: Some(1),
    ..Default::default()
  })
  .0;
  if text_width >= width {
    return text.to_string();
  }
  let pad = width - text_width;
  let (left, right) = match align {
    TableAlign::Left => (0, pad),
    TableAlign::Center => (pad / 2, pad - pad / 2),
    TableAlign::Right => (pad, 0),
  };
  pad_rich_text(text, left as usize, right as usize)
}

fn pad_rich_text(text: &str, left: usize, right: usize) -> String {
  let left = " ".repeat(left);
  let right = " ".repeat(right);
  if let Some(rest) = text.strip_prefix("f%") {
    format!("f%{left}{rest}{right}")
  } else {
    format!("{left}{text}{right}")
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::CanvasService;

  fn table() -> (TableService, UiObjectPool, TableId) {
    table_with_border_style(TableBorderStyle::Single)
  }

  fn table_with_border_style(
    border_style: TableBorderStyle,
  ) -> (TableService, UiObjectPool, TableId) {
    let service = TableService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(
        &mut pool,
        TableOptions {
          columns: vec![
            TableColumn::fixed("a", "A", 4),
            TableColumn::fixed("b", "B", 4),
          ],
          style: TableStyle {
            border_mode: TableBorderMode::Full,
            border_style,
            column_gap: 1,
            show_header: true,
            show_empty_message: true,
            empty_message: "empty".to_string(),
          },
        },
      )
      .unwrap();
    (service, pool, id)
  }

  #[test]
  fn create_validates_columns() {
    let service = TableService::new();
    let mut pool = UiObjectPool::new();
    assert!(
      service
        .create(&mut pool, TableOptions::new(vec![]))
        .is_none()
    );
    assert!(
      service
        .create(
          &mut pool,
          TableOptions::new(vec![
            TableColumn::fixed("a", "A", 4),
            TableColumn::fixed("a", "B", 4),
          ]),
        )
        .is_none()
    );
  }

  #[test]
  fn remove_clears_state() {
    let (service, mut pool, id) = table();
    assert!(service.exists(&pool, id));
    assert!(service.remove(&mut pool, id));
    assert!(!service.exists(&pool, id));
  }

  #[test]
  fn draw_full_table_writes_cells() {
    let (service, pool, id) = table();
    let mut canvas = CanvasService::new();
    let rows = vec![TableRow::from_texts(["one", "two"])];
    assert!(service.draw(
      &pool,
      &mut canvas,
      TableDrawParams {
        id,
        x: 0,
        y: 0,
        width: 11,
        height: 5,
        rows: &rows,
        row_offset: 0,
      },
    ));
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "┌");
    assert_eq!(canvas.cell_at(2, 3).unwrap().text, "o");
  }

  #[test]
  fn double_border_uses_double_chars() {
    let (service, pool, id) = table_with_border_style(TableBorderStyle::Double);
    let mut canvas = CanvasService::new();
    assert!(service.draw(
      &pool,
      &mut canvas,
      TableDrawParams {
        id,
        x: 0,
        y: 0,
        width: 11,
        height: 5,
        rows: &[],
        row_offset: 0,
      },
    ));
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "╔");
    assert_eq!(canvas.cell_at(5, 0).unwrap().text, "╦");
    assert_eq!(canvas.cell_at(0, 2).unwrap().text, "╠");
    assert_eq!(canvas.cell_at(5, 2).unwrap().text, "╬");
    assert_eq!(canvas.cell_at(0, 4).unwrap().text, "╚");
  }

  #[test]
  fn double_outer_single_inner_border_uses_mixed_chars() {
    let (service, pool, id) = table_with_border_style(TableBorderStyle::DoubleOuterSingleInner);
    let mut canvas = CanvasService::new();
    assert!(service.draw(
      &pool,
      &mut canvas,
      TableDrawParams {
        id,
        x: 0,
        y: 0,
        width: 11,
        height: 5,
        rows: &[],
        row_offset: 0,
      },
    ));
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "╔");
    assert_eq!(canvas.cell_at(5, 0).unwrap().text, "╤");
    assert_eq!(canvas.cell_at(0, 2).unwrap().text, "╟");
    assert_eq!(canvas.cell_at(5, 2).unwrap().text, "┼");
    assert_eq!(canvas.cell_at(5, 4).unwrap().text, "╧");
  }

  #[test]
  fn padding_keeps_rich_text_prefix() {
    assert_eq!(
      pad_rich_text("f%<fg:red>A</fg>", 1, 1),
      "f% <fg:red>A</fg> "
    );
  }
}
