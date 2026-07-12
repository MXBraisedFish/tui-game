use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, KeyState, LayoutService, MouseButton, Rect, RenderService,
  RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, StorageService, TableBorderMode,
  TableColumn, TableDrawParams, TableId, TableOptions, TableOverflow, TableRow, TableService,
  TableStyle, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

const ROW_LEN: usize = 6;
const NS: &str = "storage_management_view";
const SCROLL_STEP: Duration = Duration::from_millis(160);

const ROW_KEYS: &[&str] = &[
  "storage_management_view.name.root",
  "storage_management_view.name.data",
  "storage_management_view.name.cache",
  "storage_management_view.name.log",
  "storage_management_view.name.profile",
  "storage_management_view.name.mod",
];

const ROW_RELATIVE_PATHS: &[&str] = &[
  "",
  "data",
  "data/cache",
  "data/log",
  "data/profiles",
  "data/mod",
];

#[derive(Clone)]
struct StorageRow {
  label_key: &'static str,
  display_path: String,
  full_path: String,
  bytes: u64,
}

pub struct StorageManagementViewUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  back_area: HitAreaId,
  path_areas: [HitAreaId; ROW_LEN],
  table: TableId,
  rows: Vec<StorageRow>,
  root_scroll_x: u16,
  root_scroll_forward: bool,
  scroll_elapsed: Duration,
}

pub(crate) struct StorageManagementViewLayout {
  title_x: u16,
  title_y: u16,
  table: Rect,
  name_col: u16,
  size_col: u16,
  path_col: u16,
  tip_x: u16,
  tip_y: u16,
  hint_x: u16,
  hint_y: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StorageManagementViewCommand {
  Back,
  CopyAll(String),
  CopyPath(String),
}

impl UiObjectPoolOwner for StorageManagementViewUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for StorageManagementViewUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl StorageManagementViewUi {
  pub fn init(hit_area: &HitAreaService, table: &TableService) -> Self {
    let mut objects = UiObjectPool::new();
    let table_id = table
      .create(&mut objects, Self::table_options(8, 8, 20))
      .expect("storage management table options are valid");
    Self {
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      path_areas: std::array::from_fn(|_| hit_area.create(&mut objects, HitAreaOptions::default())),
      table: table_id,
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      rows: Vec::new(),
      root_scroll_x: 0,
      root_scroll_forward: true,
      scroll_elapsed: Duration::ZERO,
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "storage_management_view.back".to_string(),
        description: "Back to storage management".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
      ActionMapEntry {
        action: "storage_management_view.copy".to_string(),
        description: "Copy all storage paths".to_string(),
        keys: vec![vec!["ctrl".to_string(), "c".to_string()]],
      },
    ]
  }

  fn table_options(name_col: u16, size_col: u16, path_col: u16) -> TableOptions {
    TableOptions {
      columns: vec![
        TableColumn::fixed("name", "", name_col).overflow(TableOverflow::Ellipsis),
        TableColumn::fixed("size", "", size_col).overflow(TableOverflow::Ellipsis),
        TableColumn::fixed("path", "", path_col).overflow(TableOverflow::Ellipsis),
      ],
      style: TableStyle {
        border_mode: TableBorderMode::Full,
        border_style: Default::default(),
        column_gap: 0,
        show_header: true,
        show_empty_message: false,
        empty_message: String::new(),
      },
    }
  }

  fn table_columns(
    positions: &StorageManagementViewLayout,
    i18n: &I18nService,
  ) -> Vec<TableColumn> {
    vec![
      TableColumn::fixed(
        "name",
        format!(
          "f%<fg:bright_yellow>{}</fg>",
          i18n.get_runtime_text(NS, "storage_management_view.name")
        ),
        positions.name_col,
      ),
      TableColumn::fixed(
        "size",
        format!(
          "f%<fg:bright_yellow>{}</fg>",
          i18n.get_runtime_text(NS, "storage_management_view.size")
        ),
        positions.size_col,
      ),
      TableColumn::fixed(
        "path",
        format!(
          "f%<fg:bright_yellow>{}</fg>",
          i18n.get_runtime_text(NS, "storage_management_view.path")
        ),
        positions.path_col,
      ),
    ]
  }

  pub fn handle_event(
    &mut self,
    event: &UiEvent,
    i18n: &I18nService,
  ) -> Option<StorageManagementViewCommand> {
    match event {
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) => {
        let index = self.path_areas.iter().position(|area| area == id)?;
        self
          .rows
          .get(index)
          .map(|row| StorageManagementViewCommand::CopyPath(row.full_path.clone()))
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => Some(StorageManagementViewCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "storage_management_view.back" => Some(StorageManagementViewCommand::Back),
        "storage_management_view.copy" => Some(StorageManagementViewCommand::CopyAll(
          self.copy_all_text(i18n),
        )),
        _ => None,
      },
      _ => None,
    }
  }

  pub fn update(&mut self, dt: Duration, layout: &LayoutService, i18n: &I18nService) {
    self.scroll_elapsed += dt;
    if self.scroll_elapsed < SCROLL_STEP || self.rows.is_empty() {
      return;
    }
    self.scroll_elapsed = Duration::ZERO;

    let positions = self.compute_positions(layout, i18n);
    let max_scroll = self.root_max_scroll(layout, positions.path_col);
    if max_scroll == 0 {
      self.root_scroll_x = 0;
      self.root_scroll_forward = true;
      return;
    }
    if self.root_scroll_forward {
      self.root_scroll_x = self.root_scroll_x.saturating_add(1).min(max_scroll);
      self.root_scroll_forward = self.root_scroll_x < max_scroll;
    } else {
      self.root_scroll_x = self.root_scroll_x.saturating_sub(1);
      self.root_scroll_forward = self.root_scroll_x == 0;
    }
  }

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    storage: &StorageService,
    hit_area: &HitAreaService,
    table: &TableService,
  ) {
    if self.rows.is_empty() {
      self.refresh(storage);
    }

    let positions = self.compute_positions(layout, i18n);
    self.draw_content(render, canvas, layout, &positions, i18n, table);

    let viewport = layout.developer_viewport_rect();
    hit_area.render_host(&mut self.objects, self.back_area, viewport, canvas);
    let path_x = positions
      .table
      .x
      .saturating_add(1)
      .saturating_add(positions.name_col)
      .saturating_add(1)
      .saturating_add(positions.size_col)
      .saturating_add(1);
    for (index, id) in self.path_areas.into_iter().enumerate() {
      hit_area.render_host(
        &mut self.objects,
        id,
        Rect {
          x: path_x,
          y: positions
            .table
            .y
            .saturating_add(3)
            .saturating_add(index as u16),
          width: positions.path_col,
          height: 1,
        },
        canvas,
      );
    }
  }

  fn refresh(&mut self, storage: &StorageService) {
    let root = absolute_path(storage.root_dir());
    self.rows = ROW_KEYS
      .iter()
      .zip(ROW_RELATIVE_PATHS)
      .map(|(&label_key, relative)| {
        let full = if relative.is_empty() {
          root.clone()
        } else {
          absolute_path(&storage.path(relative))
        };
        StorageRow {
          label_key,
          display_path: if relative.is_empty() {
            display_path_string(&full)
          } else {
            format!("[root]/{}", relative.replace('\\', "/"))
          },
          bytes: dir_size(&full),
          full_path: system_path_string(&full),
        }
      })
      .collect();
  }

  fn compute_positions(
    &self,
    layout: &LayoutService,
    i18n: &I18nService,
  ) -> StorageManagementViewLayout {
    let params = self.build_key_params();
    let viewport = layout.developer_viewport_rect();
    let max_width = layout
      .developer_width()
      .saturating_sub(32)
      .max(layout.developer_width().min(48));

    let title = i18n.get_runtime_text(NS, "storage_management_view.title");
    let title_w = layout.get_text_width(&title, None);
    let title_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, title_w, 0));
    let title_y = viewport.y.saturating_add(1);

    let hint = self.hint(i18n);
    let hint_w = layout.get_text_width(&hint, Some(&params));
    let hint_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, hint_w, 0));
    let hint_y = viewport
      .y
      .saturating_add(layout.developer_height().saturating_sub(1));

    let tip = i18n.get_runtime_text(NS, "storage_management_view.tip");
    let tip_w = layout.get_text_width(&tip, None);
    let tip_x = viewport
      .x
      .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, tip_w, 0));
    let tip_y = hint_y.saturating_sub(1);

    let name_col = self
      .rows
      .iter()
      .map(|row| layout.get_text_width(&i18n.get_runtime_text(NS, row.label_key), None))
      .chain([layout.get_text_width(
        &i18n.get_runtime_text(NS, "storage_management_view.name"),
        None,
      )])
      .max()
      .unwrap_or(8)
      .saturating_add(2);
    let size_col = self
      .rows
      .iter()
      .map(|row| layout.get_text_width(&self.format_size(row.bytes, i18n), None))
      .chain([layout.get_text_width(
        &i18n.get_runtime_text(NS, "storage_management_view.size"),
        None,
      )])
      .max()
      .unwrap_or(8)
      .saturating_add(2);
    let desired_path = self
      .rows
      .iter()
      .map(|row| layout.get_text_width(&row.display_path, None))
      .chain([layout.get_text_width(
        &i18n.get_runtime_text(NS, "storage_management_view.path"),
        None,
      )])
      .max()
      .unwrap_or(20)
      .saturating_add(2)
      .min(80);
    let desired_width = name_col
      .saturating_add(size_col)
      .saturating_add(desired_path)
      .saturating_add(4);
    let table_width = desired_width.min(max_width).max(32.min(max_width));
    let fixed = name_col.saturating_add(size_col).saturating_add(4);
    let path_col = table_width.saturating_sub(fixed).max(8);
    let table_width = fixed.saturating_add(path_col);
    let table_height = ROW_LEN as u16 + 4;
    let table_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, table_width, 0));
    let upper = title_y.saturating_add(2);
    let lower = tip_y.saturating_sub(1);
    let available = lower.saturating_sub(upper);
    let table_y = if available > table_height {
      upper.saturating_add((available - table_height) / 2)
    } else {
      upper
    };

    StorageManagementViewLayout {
      title_x,
      title_y,
      table: Rect {
        x: table_x,
        y: table_y,
        width: table_width,
        height: table_height,
      },
      name_col,
      size_col,
      path_col,
      tip_x,
      tip_y,
      hint_x,
      hint_y,
    }
  }

  fn draw_content(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    positions: &StorageManagementViewLayout,
    i18n: &I18nService,
    table: &TableService,
  ) {
    self.draw_text(
      render,
      canvas,
      positions.title_x,
      positions.title_y,
      format!(
        "f%<fg:bright_magenta>{}</fg>",
        i18n.get_runtime_text(NS, "storage_management_view.title")
      ),
    );
    self.draw_table(canvas, layout, positions, i18n, table);
    self.draw_text(
      render,
      canvas,
      positions.tip_x,
      positions.tip_y,
      format!(
        "f%<fg:white>{}</fg>",
        i18n.get_runtime_text(NS, "storage_management_view.tip")
      ),
    );
    self.draw_text_with_params(
      render,
      canvas,
      positions.hint_x,
      positions.hint_y,
      self.hint(i18n),
      Some(self.build_key_params()),
    );
  }

  fn draw_table(
    &mut self,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    positions: &StorageManagementViewLayout,
    i18n: &I18nService,
    table: &TableService,
  ) {
    let _ = table.set_columns(
      &mut self.objects,
      self.table,
      Self::table_columns(positions, i18n),
    );
    let rows = self
      .rows
      .iter()
      .enumerate()
      .map(|(index, row)| {
        TableRow::from_texts([
          format!(
            "f%<fg:bright_cyan>{}</fg>",
            i18n.get_runtime_text(NS, row.label_key)
          ),
          self.format_size(row.bytes, i18n),
          self.visible_path(index, row, layout, positions.path_col),
        ])
      })
      .collect::<Vec<_>>();
    let _ = table.draw_host(
      &self.objects,
      canvas,
      TableDrawParams {
        id: self.table,
        x: positions.table.x,
        y: positions.table.y,
        width: positions.table.width,
        height: positions.table.height,
        rows: &rows,
        row_offset: 0,
      },
    );
  }

  fn visible_path(
    &self,
    index: usize,
    row: &StorageRow,
    layout: &LayoutService,
    width: u16,
  ) -> String {
    if index != 0 {
      return row.display_path.clone();
    }
    let content_width = width.saturating_sub(1);
    scroll_window(&row.display_path, self.root_scroll_x, content_width, layout)
  }

  fn root_max_scroll(&self, layout: &LayoutService, width: u16) -> u16 {
    let Some(row) = self.rows.first() else {
      return 0;
    };
    layout
      .get_text_width(&row.display_path, None)
      .saturating_sub(width.saturating_sub(1))
  }

  fn format_size(&self, bytes: u64, i18n: &I18nService) -> String {
    let mut value = bytes as f64 / 1024.0;
    let mut key = "storage_management_view.size.k";
    if value >= 1024.0 {
      value /= 1024.0;
      key = "storage_management_view.size.m";
    }
    if value >= 1024.0 {
      value /= 1024.0;
      key = "storage_management_view.size.g";
    }
    format!("{:.1} {}", value, i18n.get_runtime_text(NS, key))
  }

  fn copy_all_text(&self, i18n: &I18nService) -> String {
    self
      .rows
      .iter()
      .map(|row| {
        format!(
          "{}: {}",
          i18n.get_runtime_text(NS, row.label_key),
          row.full_path
        )
      })
      .collect::<Vec<_>>()
      .join("\n")
  }

  fn hint(&self, i18n: &I18nService) -> String {
    format!(
      "f%<fg:rgb(85,87,83)>{}  {}</fg>",
      i18n.get_runtime_text(NS, "storage_management_view.action.copy"),
      i18n.get_runtime_text(NS, "storage_management_view.action.back")
    )
  }

  fn build_key_params(&self) -> RichTextParams {
    RichTextParams::from_action_map(&Self::action_map(), "storage_management_view.")
  }

  fn draw_text(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    x: u16,
    y: u16,
    text: impl Into<String>,
  ) {
    self.draw_text_with_params(render, canvas, x, y, text, None);
  }

  fn draw_text_with_params(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    x: u16,
    y: u16,
    text: impl Into<String>,
    params: Option<RichTextParams>,
  ) {
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x,
        y,
        text: text.into(),
        params,
        ..Default::default()
      },
    );
  }
}

fn absolute_path(path: &Path) -> PathBuf {
  path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn system_path_string(path: &Path) -> String {
  let value = path.to_string_lossy().to_string();
  strip_windows_verbatim_prefix(&value)
}

fn display_path_string(path: &Path) -> String {
  system_path_string(path).replace('\\', "/")
}

fn strip_windows_verbatim_prefix(path: &str) -> String {
  if let Some(rest) = path.strip_prefix(r"\\?\UNC\") {
    return format!(r"\\{}", rest);
  }
  path.strip_prefix(r"\\?\").unwrap_or(path).to_string()
}

fn dir_size(path: &Path) -> u64 {
  let Ok(meta) = fs::metadata(path) else {
    return 0;
  };
  if meta.is_file() {
    return meta.len();
  }
  let Ok(entries) = fs::read_dir(path) else {
    return 0;
  };
  entries
    .filter_map(Result::ok)
    .map(|entry| dir_size(&entry.path()))
    .sum()
}

fn scroll_window(text: &str, offset: u16, width: u16, layout: &LayoutService) -> String {
  if width == 0 {
    return String::new();
  }
  let mut skipped = 0u16;
  let mut taken = 0u16;
  let mut result = String::new();
  for ch in text.chars() {
    let s = ch.to_string();
    let w = layout.get_text_width(&s, None).max(1);
    if skipped.saturating_add(w) <= offset {
      skipped = skipped.saturating_add(w);
      continue;
    }
    if taken.saturating_add(w) > width {
      break;
    }
    taken = taken.saturating_add(w);
    result.push(ch);
  }
  result
}
