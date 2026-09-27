use std::cell::Cell;
use std::collections::HashMap;
use std::time::Duration;

use crate::host_engine::services::text_layout::TextWrapMode;
use crate::host_engine::services::{
  ActionMapEntry, BorderStyle, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId,
  HitAreaOptions, HitAreaService, I18nService, KeyState, LanguageRegistryEntry, LayoutService,
  LogService, LogSource, MouseButton, Rect, RenderService, RichTextParams, RuntimeObjectPool,
  RuntimeObjectPoolOwner, StorageService, TerminalColor, TextColor, UiEvent, UiObjectPool,
  UiObjectPoolOwner,
};

const GRID_START_Y: u16 = 3;

const CELL_HEIGHT: u16 = 3;

const MIN_CELL_WIDTH: u16 = 14;

const MAX_NAME_LEN: u16 = 20;

/// Screen layout of the language selection page.
pub(crate) struct LanguageSelectLayout {
  title_x: u16,
  title_y: u16,
  cell_rects: Vec<Rect>,
  cell_text_xs: Vec<u16>,
  page_start: usize,
  pages: usize,
  page_center: u16,
  page_y: u16,
  flip_forward_x: u16,
  flip_backward_max_x: u16,
  hint_x: u16,
  hint_y: u16,
}

/// Language selection page that shows the available language packs as a paged grid, navigable
/// by keyboard and mouse.
pub struct LanguageSelectUi {
  selected_index: usize,
  page: usize,
  registry: Vec<LanguageRegistryEntry>,
  runtime_cache: HashMap<String, HashMap<String, String>>,
  active_code: String,
  first_selection: bool,

  columns: Cell<usize>,
  per_page: Cell<usize>,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  back_area: HitAreaId,
  flip_forward_area: HitAreaId,
  flip_backward_area: HitAreaId,
  cell_areas: Vec<HitAreaId>,
}

impl UiObjectPoolOwner for LanguageSelectUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for LanguageSelectUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

/// Command emitted by the language selection page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LanguageSelectCommand {
  Confirm(String),
  ConfirmAndApply(String),
  Back,
  Exit,
}

impl LanguageSelectUi {
  /// Creates the language selection page from `registry`, sorted by name, and preloads the
  /// runtime language texts of every registered language.
  pub fn init(
    mut registry: Vec<LanguageRegistryEntry>,
    storage: &StorageService,
    log: &mut LogService,
    hit_area: &HitAreaService,
  ) -> Self {
    registry.sort_by(|a, b| a.name.cmp(&b.name));

    let stored_code = storage.read_language_code(log);
    let first_selection = stored_code
      .as_deref()
      .is_none_or(|code| !registry.iter().any(|entry| entry.code == code));
    let active_code = if first_selection {
      String::new()
    } else {
      stored_code.unwrap_or_default()
    };

    let selected_index = registry
      .iter()
      .position(|e| e.code == active_code)
      .or_else(|| {
        registry
          .iter()
          .position(|entry| entry.code == storage.default_language_code())
      })
      .unwrap_or(0);

    let runtime_cache = Self::preload_runtime_cache(storage, log, &registry);
    let mut objects = UiObjectPool::new();
    let back_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let flip_forward_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let flip_backward_area = hit_area.create(&mut objects, HitAreaOptions::default());
    let cell_areas = (0..registry.len())
      .map(|_| hit_area.create(&mut objects, HitAreaOptions::default()))
      .collect();

    Self {
      selected_index,
      page: 1,
      registry,
      runtime_cache,
      active_code,
      first_selection,
      columns: Cell::new(4),
      per_page: Cell::new(12),
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      back_area,
      flip_forward_area,
      flip_backward_area,
      cell_areas,
    }
  }

  pub fn is_first_selection(&self) -> bool {
    self.first_selection
  }

  fn preload_runtime_cache(
    storage: &StorageService,
    log: &mut LogService,
    registry: &[LanguageRegistryEntry],
  ) -> HashMap<String, HashMap<String, String>> {
    let mut cache = HashMap::new();
    for entry in registry {
      let path = storage.language_runtime_namespace_path(&entry.code, "language");
      if let Ok(content) = std::fs::read_to_string(&path)
        && let Ok(texts) = serde_json::from_str::<HashMap<String, String>>(&content)
      {
        cache.insert(entry.code.clone(), texts);
        continue;
      }
      log.warn_operation_failed(
        LogSource::I18n,
        "load_runtime_language",
        path.display().to_string(),
        "language resource is unavailable",
      );
    }
    cache
  }

  fn get_text(&self, key: &str) -> String {
    let code = self
      .registry
      .get(self.selected_index)
      .map(|e| e.code.as_str())
      .unwrap_or("en_us");
    self
      .runtime_cache
      .get(code)
      .and_then(|m| m.get(key))
      .cloned()
      .unwrap_or_else(|| key.to_string())
  }

  fn normalize_page(&mut self) {
    let per_page = self.per_page.get();
    if self.registry.is_empty() || per_page == 0 {
      return;
    }
    let pages = self.registry.len().max(1).div_ceil(per_page).max(1);
    self.page = self
      .selected_index
      .saturating_div(per_page)
      .saturating_add(1)
      .clamp(1, pages);
  }

  /// Returns the action map (key bindings) of the language selection page.
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "language_select.focus_up".to_string(),
        description: "Focus up".to_string(),
        keys: vec![vec!["up".to_string()]],
      },
      ActionMapEntry {
        action: "language_select.focus_down".to_string(),
        description: "Focus down".to_string(),
        keys: vec![vec!["down".to_string()]],
      },
      ActionMapEntry {
        action: "language_select.focus_left".to_string(),
        description: "Focus left".to_string(),
        keys: vec![vec!["left".to_string()]],
      },
      ActionMapEntry {
        action: "language_select.focus_right".to_string(),
        description: "Focus right".to_string(),
        keys: vec![vec!["right".to_string()]],
      },
      ActionMapEntry {
        action: "language_select.flip_forward".to_string(),
        description: "Previous page".to_string(),
        keys: vec![vec!["q".to_string()]],
      },
      ActionMapEntry {
        action: "language_select.flip_backward".to_string(),
        description: "Next page".to_string(),
        keys: vec![vec!["e".to_string()]],
      },
      ActionMapEntry {
        action: "language_select.confirm".to_string(),
        description: "Confirm language".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
      ActionMapEntry {
        action: "language_select.back".to_string(),
        description: "Go back".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
    ]
  }

  /// Handles a UI event and returns the confirm or back command it triggers, if any.
  pub fn handle_event(&mut self, event: &UiEvent) -> Option<LanguageSelectCommand> {
    match event {
      UiEvent::HitArea(HitAreaEvent::HoverEnter { id, .. }) => {
        self.selected_index = self.cell_areas.iter().position(|area| area == id)?;
        self.normalize_page();
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.flip_forward_area => self.flip_page(false),
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) if *id == self.flip_backward_area => self.flip_page(true),
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) => {
        let index = self.cell_areas.iter().position(|area| area == id)?;
        self.selected_index = index;
        self.confirm_focused()
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => Some(self.back_command()),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "language_select.focus_up" => {
          self.focus_up();
          None
        }
        "language_select.focus_down" => {
          self.focus_down();
          None
        }
        "language_select.focus_left" => {
          self.focus_left();
          None
        }
        "language_select.focus_right" => {
          self.focus_right();
          None
        }
        "language_select.flip_forward" => self.flip_page(false),
        "language_select.flip_backward" => self.flip_page(true),
        "language_select.confirm" => self.confirm_focused(),
        "language_select.back" => Some(self.back_command()),
        _ => None,
      },
      _ => None,
    }
  }

  fn confirm_focused(&mut self) -> Option<LanguageSelectCommand> {
    let code = self.registry.get(self.selected_index)?.code.clone();
    if self.first_selection && self.active_code == code {
      return Some(LanguageSelectCommand::ConfirmAndApply(code));
    }
    self.active_code = code.clone();
    Some(LanguageSelectCommand::Confirm(code))
  }

  fn back_command(&self) -> LanguageSelectCommand {
    if self.first_selection {
      LanguageSelectCommand::Exit
    } else {
      LanguageSelectCommand::Back
    }
  }

  fn confirm_hint_key(&self) -> &'static str {
    let focused_code = self
      .registry
      .get(self.selected_index)
      .map(|entry| &entry.code);
    if self.first_selection && focused_code == Some(&self.active_code) {
      "language.action.confirm.two"
    } else {
      "language.action.confirm"
    }
  }

  fn back_hint_key(&self) -> &'static str {
    if self.first_selection {
      "language.action.exit"
    } else {
      "language.action.back"
    }
  }

  fn flip_forward_rect(&self, layout: &LayoutService, pos: &LanguageSelectLayout) -> Option<Rect> {
    (self.page > 1).then(|| Rect {
      x: pos.flip_forward_x,
      y: pos.page_y,
      width: layout.get_text_width(
        &self.get_text("language.flip.forward"),
        Some(&self.build_key_params()),
      ),
      height: 1,
    })
  }

  fn flip_backward_rect(&self, layout: &LayoutService, pos: &LanguageSelectLayout) -> Option<Rect> {
    if self.page >= pos.pages {
      return None;
    }
    let key_params = self.build_key_params();
    let width = layout.get_text_width(&self.get_text("language.flip.backward"), Some(&key_params));
    Some(Rect {
      x: pos.flip_backward_max_x.saturating_sub(width),
      y: pos.page_y,
      width,
      height: 1,
    })
  }

  pub fn update(&mut self, dt: Duration) -> Option<LanguageSelectCommand> {
    let _ = dt;
    None
  }

  /// Draws the language selection page onto the host layer.
  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    _i18n: &I18nService,
    hit_area: &HitAreaService,
  ) {
    let positions = self.compute_positions(layout);
    self.draw_content(render, canvas, layout, &positions);
    let viewport = layout.developer_viewport_rect();
    hit_area.render_host(&mut self.objects, self.back_area, viewport, canvas);
    if let Some(rect) = self.flip_forward_rect(layout, &positions) {
      hit_area.render_host(&mut self.objects, self.flip_forward_area, rect, canvas);
    }
    if let Some(rect) = self.flip_backward_rect(layout, &positions) {
      hit_area.render_host(&mut self.objects, self.flip_backward_area, rect, canvas);
    }
    let (start, _, _, _) = self.page_bounds();
    for (id, rect) in self.cell_areas[start..]
      .iter()
      .copied()
      .zip(positions.cell_rects.iter().copied())
    {
      hit_area.render_host(&mut self.objects, id, rect, canvas);
    }
  }

  /// Computes the host coordinates of every element of the language selection page from the
  /// [`LayoutService`].
  pub fn compute_positions(&self, layout: &LayoutService) -> LanguageSelectLayout {
    let viewport = layout.developer_viewport_rect();
    let size = layout.developer_size();
    let term_w = size.width;
    let term_h = size.height;
    let name_widths: Vec<u16> = self
      .registry
      .iter()
      .map(|e| {
        let dp = DrawTextParams {
          text: e.name.clone(),
          max_width: Some(MAX_NAME_LEN),
          overflow_marker: Some("...".to_string()),
          wrap_mode: TextWrapMode::None,
          ..Default::default()
        };
        layout.get_draw_text_width(&dp)
      })
      .collect();
    let max_name_w = name_widths
      .iter()
      .max()
      .copied()
      .unwrap_or(MIN_CELL_WIDTH - 2)
      .max(MIN_CELL_WIDTH - 2);

    let cell_width = max_name_w + 2;
    let grid_available_h = term_h.saturating_sub(GRID_START_Y).saturating_sub(4);
    let rows = (grid_available_h / CELL_HEIGHT).max(1) as usize;
    let columns = (term_w / cell_width).max(1) as usize;
    let per_page = columns * rows;
    let pages = self.registry.len().max(1).div_ceil(per_page).max(1);
    self.columns.set(columns);
    self.per_page.set(per_page);

    let page = self.page.clamp(1, pages);
    let page_start = (page - 1) * per_page;
    let page_end = (page_start + per_page).min(self.registry.len());
    let visible_count = page_end - page_start;

    let grid_total_w = (columns as u16).saturating_mul(cell_width);
    let grid_x = viewport.x + term_w.saturating_sub(grid_total_w) / 2;

    let mut cell_rects = Vec::with_capacity(visible_count);
    let mut cell_text_xs = Vec::with_capacity(visible_count);
    for vi in 0..visible_count {
      let col = vi % columns;
      let row = vi / columns;
      let cx = grid_x + (col as u16) * cell_width;
      let cy = viewport.y + GRID_START_Y + (row as u16) * CELL_HEIGHT;
      cell_rects.push(Rect {
        x: cx,
        y: cy,
        width: cell_width,
        height: CELL_HEIGHT,
      });

      let nw = name_widths[page_start + vi];
      let inner_w = cell_width.saturating_sub(2);
      let tx = cx + 1 + (inner_w.saturating_sub(nw)) / 2;
      cell_text_xs.push(tx);
    }
    let title = self.get_text("language.title");
    let title_w = layout.get_text_width(&title, None);
    let title_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, title_w, 0));
    let page_y = viewport.y + term_h.saturating_sub(3);
    let page_center = viewport.x + term_w / 2;
    let flip_forward_x = viewport.x;
    let flip_backward_max_x = viewport.x.saturating_add(term_w);
    let key_params = self.build_key_params();
    let hint = format!(
      "{}  {}  {}  {}",
      self.get_text("language.action.focus"),
      self.get_text("language.action.flip"),
      self.get_text(self.confirm_hint_key()),
      self.get_text(self.back_hint_key()),
    );
    let hint_w = layout.get_text_width(&hint, Some(&key_params));
    let hint_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, hint_w, 0));
    let hint_y = viewport.y + term_h.saturating_sub(1);

    LanguageSelectLayout {
      title_x,
      title_y: viewport.y.saturating_add(1),
      cell_rects,
      cell_text_xs,
      page_start,
      pages,
      page_center,
      page_y,
      flip_forward_x,
      flip_backward_max_x,
      hint_x,
      hint_y,
    }
  }

  fn page_bounds(&self) -> (usize, usize, usize, usize) {
    let per_page = self.per_page.get();
    let cols = self.columns.get();
    let total = self.registry.len().max(1);
    let pages = total.div_ceil(per_page).max(1);
    let page = self.page.clamp(1, pages);
    let start = (page - 1) * per_page;
    let end = (start + per_page).min(total);
    (start, end, cols, pages)
  }

  fn focus_up(&mut self) {
    if self.registry.is_empty() {
      return;
    }
    let (start, _end, cols, _pages) = self.page_bounds();
    let col = self.selected_index % cols;

    if self.selected_index >= start + cols {
      self.selected_index -= cols;
    } else if self.page > 1 {
      self.page -= 1;
      let (prev_start, prev_end, _prev_cols, _) = self.page_bounds();
      self.selected_index = (prev_start + col).min(prev_end - 1);
    }
  }

  fn focus_down(&mut self) {
    if self.registry.is_empty() {
      return;
    }
    let (_start, end, cols, pages) = self.page_bounds();
    let col = self.selected_index % cols;
    let candidate = self.selected_index + cols;

    if candidate < end {
      self.selected_index = candidate;
    } else if self.page < pages {
      self.page += 1;
      let (next_start, next_end, _, _) = self.page_bounds();
      self.selected_index = (next_start + col).min(next_end - 1);
    }
  }

  fn focus_left(&mut self) {
    if self.registry.is_empty() {
      return;
    }
    let cols = self.columns.get();
    let col = self.selected_index % cols;

    if col > 0 {
      self.selected_index -= 1;
    } else if self.page > 1 {
      self.page -= 1;
      let (_prev_start, prev_end, _, _) = self.page_bounds();
      self.selected_index = prev_end - 1;
    }
  }

  fn focus_right(&mut self) {
    if self.registry.is_empty() {
      return;
    }
    let (_start, end, cols, pages) = self.page_bounds();
    let col = self.selected_index % cols;

    if col + 1 < cols && self.selected_index + 1 < end {
      self.selected_index += 1;
    } else if self.page < pages {
      self.page += 1;
      let (next_start, _, _, _) = self.page_bounds();
      self.selected_index = next_start;
    }
  }

  fn flip_page(&mut self, forward: bool) -> Option<LanguageSelectCommand> {
    let per_page = self.per_page.get();
    if per_page == 0 {
      return None;
    }
    let pages = self.registry.len().max(1).div_ceil(per_page).max(1);
    if forward {
      if self.page < pages {
        self.page += 1;
        self.selected_index = (self.page - 1) * per_page;
      }
    } else if self.page > 1 {
      self.page -= 1;
      self.selected_index = (self.page - 1) * per_page;
    }
    None
  }

  fn build_key_params(&self) -> RichTextParams {
    let action_map = Self::action_map();
    let mut key_actions: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    for entry in &action_map {
      key_actions.insert(entry.action.clone(), entry.keys.clone());
    }
    let aliases: &[(&str, &str)] = &[
      ("language.focus_up", "language_select.focus_up"),
      ("language.focus_down", "language_select.focus_down"),
      ("language.focus_left", "language_select.focus_left"),
      ("language.right", "language_select.focus_right"),
      ("language.flip_forward", "language_select.flip_forward"),
      ("language.flip_backward", "language_select.flip_backward"),
      ("language.confirm", "language_select.confirm"),
      ("language.back", "language_select.back"),
    ];
    for &(alias, action) in aliases {
      if let Some(keys) = key_actions.get(action) {
        key_actions.insert(alias.to_string(), keys.clone());
      }
    }
    RichTextParams {
      values: HashMap::new(),
      key_default_actions: key_actions.clone(),
      key_actions,
    }
  }

  fn draw_content(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    pos: &LanguageSelectLayout,
  ) {
    let key_params = self.build_key_params();
    let title = self.get_text("language.title");
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: pos.title_x,
        y: pos.title_y,
        text: format!("f%<fg:bright_magenta><b>{}</b></fg>", title),
        ..Default::default()
      },
    );
    for vi in 0..pos.cell_rects.len() {
      let gi = pos.page_start + vi;
      let entry = &self.registry[gi];
      let is_active = entry.code == self.active_code;
      let is_focused = gi == self.selected_index;

      let fg = if is_active {
        TextColor::Terminal(TerminalColor::BrightGreen)
      } else {
        TextColor::Terminal(TerminalColor::White)
      };
      if is_focused {
        let r = &pos.cell_rects[vi];
        render.draw_host_border_rect(
          canvas,
          r.x,
          r.y,
          r.width,
          r.height,
          &BorderStyle::Line,
          Some(TextColor::Terminal(TerminalColor::BrightCyan)),
          None,
          None,
          None,
        );
      }

      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: pos.cell_text_xs[vi],
          y: pos.cell_rects[vi].y + 1,
          text: entry.name.clone(),
          fg: Some(fg),
          max_width: Some(MAX_NAME_LEN),
          overflow_marker: Some("...".to_string()),
          wrap_mode: TextWrapMode::None,
          ..Default::default()
        },
      );
    }
    let cur_str = self.page.to_string();
    let tot_str = pos.pages.to_string();
    let cur_x = pos.page_center.saturating_sub(cur_str.len() as u16);
    let sep_x = pos.page_center;
    let tot_x = pos.page_center + 1;

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: cur_x,
        y: pos.page_y,
        text: cur_str,
        ..Default::default()
      },
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: sep_x,
        y: pos.page_y,
        text: "|".to_string(),
        ..Default::default()
      },
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: tot_x,
        y: pos.page_y,
        text: tot_str,
        ..Default::default()
      },
    );
    if self.page > 1 {
      let fwd = self.get_text("language.flip.forward");
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: pos.flip_forward_x,
          y: pos.page_y,
          text: fwd,
          params: Some(key_params.clone()),
          ..Default::default()
        },
      );
    }
    if self.page < pos.pages {
      let bwd = self.get_text("language.flip.backward");
      let bwd_w = layout.get_text_width(&bwd, Some(&key_params));
      let bwd_x = pos.flip_backward_max_x.saturating_sub(bwd_w);
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: bwd_x,
          y: pos.page_y,
          text: bwd,
          params: Some(key_params.clone()),
          ..Default::default()
        },
      );
    }
    let hint = format!(
      "{}  {}  {}  {}",
      self.get_text("language.action.focus"),
      self.get_text("language.action.flip"),
      self.get_text(self.confirm_hint_key()),
      self.get_text(self.back_hint_key()),
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: pos.hint_x,
        y: pos.hint_y,
        text: format!("f%<fg:rgb(85,87,83)>{}</fg>", hint),
        params: Some(key_params),
        ..Default::default()
      },
    );
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;

  fn test_ui(saved_code: Option<&str>) -> LanguageSelectUi {
    let root = std::env::temp_dir().join(format!(
      "tg_language_select_{}_{}",
      saved_code.unwrap_or("missing"),
      std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("data/profiles")).unwrap();
    for code in ["en_us", "zh_cn"] {
      let dir = root.join(format!("assets/language/{code}/runtime"));
      fs::create_dir_all(&dir).unwrap();
      fs::write(dir.join("language.json"), "{}").unwrap();
    }
    let storage = StorageService::from_root_for_test(root);
    if let Some(code) = saved_code {
      storage.write_language_code(code).unwrap();
    }
    LanguageSelectUi::init(
      vec![
        LanguageRegistryEntry {
          code: "en_us".to_string(),
          name: "English".to_string(),
          direction: "ltr".to_string(),
        },
        LanguageRegistryEntry {
          code: "zh_cn".to_string(),
          name: "简体中文".to_string(),
          direction: "ltr".to_string(),
        },
      ],
      &storage,
      &mut LogService::new(),
      &HitAreaService::new(),
    )
  }

  #[test]
  fn first_selection_requires_second_confirmation_and_exits_on_back() {
    let mut ui = test_ui(None);
    assert!(ui.is_first_selection());
    assert_eq!(ui.back_command(), LanguageSelectCommand::Exit);
    assert_eq!(ui.confirm_hint_key(), "language.action.confirm");

    assert_eq!(
      ui.confirm_focused(),
      Some(LanguageSelectCommand::Confirm("en_us".to_string()))
    );
    assert_eq!(ui.confirm_hint_key(), "language.action.confirm.two");
    assert_eq!(
      ui.confirm_focused(),
      Some(LanguageSelectCommand::ConfirmAndApply("en_us".to_string()))
    );
  }

  #[test]
  fn settings_entry_keeps_existing_back_and_confirm_behavior() {
    let mut ui = test_ui(Some("zh_cn"));
    assert!(!ui.is_first_selection());
    assert_eq!(ui.back_command(), LanguageSelectCommand::Back);
    assert_eq!(
      ui.confirm_focused(),
      Some(LanguageSelectCommand::Confirm("zh_cn".to_string()))
    );
  }

  #[test]
  fn unknown_saved_language_is_treated_as_first_selection() {
    let ui = test_ui(Some("unknown"));
    assert!(ui.is_first_selection());
  }
}
