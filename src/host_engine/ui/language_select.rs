use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, I18nService, InputActionEvent, KeyState,
  LanguageRegistryEntry, LayoutService, MouseButton, MouseEvent, MouseEventKind, Rect,
  RenderService,
};

/// 布局计算结果
pub(crate) struct LanguageSelectLayout {
  title_x: u16,
  title_y: u16,
  item_rects: Vec<Rect>,
  item_xs: Vec<u16>,
  hint_x: u16,
  hint_y: u16,
}

#[derive(Clone, Debug)]
pub struct LanguageSelectUi {
  selected_index: usize,
  registry: Vec<LanguageRegistryEntry>,
}

#[derive(Clone, Debug)]
pub enum LanguageSelectCommand {
  /// 确认选择（携带语言代码）
  Confirm(String),
  /// 退出程序
  Exit,
}

impl LanguageSelectUi {
  pub fn init(registry: Vec<LanguageRegistryEntry>) -> Self {
    Self {
      selected_index: 0,
      registry,
    }
  }

  /// 当前选中的语言代码（供外部保存用）。
  pub fn selected_code(&self) -> &str {
    &self.registry[self.selected_index].code
  }

  // ── 输入绑定 ──

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "language_select.focus_up".to_string(),
        description: "Focus previous language".to_string(),
        keys: vec![vec!["up".to_string()]],
      },
      ActionMapEntry {
        action: "language_select.focus_down".to_string(),
        description: "Focus next language".to_string(),
        keys: vec![vec!["down".to_string()]],
      },
      ActionMapEntry {
        action: "language_select.confirm".to_string(),
        description: "Confirm language selection".to_string(),
        keys: vec![vec!["enter".to_string()]],
      },
      ActionMapEntry {
        action: "language_select.exit".to_string(),
        description: "Exit program".to_string(),
        keys: vec![vec!["esc".to_string()]],
      },
    ]
  }

  // ── 输入处理 ──

  pub fn handle_event(&mut self, event: &InputActionEvent) -> Option<LanguageSelectCommand> {
    if event.state != KeyState::Pressed {
      return None;
    }

    match event.action.as_str() {
      "language_select.focus_up" => {
        self.focus_previous();
        None
      }
      "language_select.focus_down" => {
        self.focus_next();
        None
      }
      "language_select.confirm" => {
        let code = self.selected_code().to_string();
        Some(LanguageSelectCommand::Confirm(code))
      }
      "language_select.exit" => Some(LanguageSelectCommand::Exit),
      _ => None,
    }
  }

  /// 鼠标事件：hover 聚焦、左键确认、右键退出。
  pub fn handle_mouse_event(
    &mut self,
    event: &MouseEvent,
    positions: &LanguageSelectLayout,
  ) -> Option<LanguageSelectCommand> {
    match event.kind {
      MouseEventKind::Move | MouseEventKind::Hold => {
        if let Some(index) = Self::hit_test(positions, event.x, event.y) {
          self.selected_index = index;
        }
        None
      }
      MouseEventKind::Press => match event.button {
        Some(MouseButton::Left) => {
          if Self::hit_test(positions, event.x, event.y).is_some() {
            let code = self.selected_code().to_string();
            return Some(LanguageSelectCommand::Confirm(code));
          }
          None
        }
        Some(MouseButton::Right) => Some(LanguageSelectCommand::Exit),
        _ => None,
      },
      _ => None,
    }
  }

  pub fn update(&mut self, dt: Duration) -> Option<LanguageSelectCommand> {
    let _ = dt;
    None
  }

  // ── 渲染 ──

  pub fn render(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    _i18n: &I18nService,
  ) {
    let positions = self.compute_positions(layout);
    self.draw_content(render, canvas, &positions);
  }

  pub fn compute_positions(&self, layout: &LayoutService) -> LanguageSelectLayout {
    let entry = &self.registry[self.selected_index];

    // title — y=1 居中
    let title_w = layout.get_text_width(&entry.title, None);
    let title_x = layout.resolve_x(LayoutService::ALIGN_CENTER, title_w, 0);
    let title_y: u16 = 1;

    // 每个语言名（含装饰）单独居中的 x
    let item_texts: Vec<String> = (0..self.registry.len())
      .map(|i| self.item_display_name(i))
      .collect();
    let item_widths: Vec<u16> = item_texts
      .iter()
      .map(|t| layout.get_text_width(t, None))
      .collect();
    let item_xs: Vec<u16> = item_widths
      .iter()
      .map(|&w| layout.resolve_x(LayoutService::ALIGN_CENTER, w, 0))
      .collect();

    let items_height = self.registry.len() as u16;

    // hint — 底部居中
    let hint_w = layout.get_text_width(&entry.hint, None);
    let hint_x = layout.resolve_x(LayoutService::ALIGN_CENTER, hint_w, 0);
    let hint_y = layout.get_terminal_size().height.saturating_sub(1);

    // 语言列表垂直居中（title 和 hint 之间）
    let available = hint_y.saturating_sub(title_y).saturating_sub(1);
    let menu_y = if available > items_height {
      title_y
        .saturating_add(1)
        .saturating_add((available - items_height) / 2)
    } else {
      title_y.saturating_add(1)
    };

    let item_rects: Vec<Rect> = (0..self.registry.len())
      .map(|i| Rect {
        x: item_xs[i],
        y: menu_y.saturating_add(i as u16),
        width: item_widths[i],
        height: 1,
      })
      .collect();

    LanguageSelectLayout {
      title_x,
      title_y,
      item_rects,
      item_xs,
      hint_x,
      hint_y,
    }
  }

  // ── 内部辅助 ──

  /// 返回索引处的显示文本（选中项带箭头，非选中项等宽占位）。
  fn item_display_name(&self, index: usize) -> String {
    let name = &self.registry[index].name;
    if index == self.selected_index {
      format!("❯ {} ❮", name)
    } else {
      format!("   {}   ", name)
    }
  }

  fn focus_previous(&mut self) {
    if self.selected_index == 0 {
      self.selected_index = self.registry.len().saturating_sub(1);
    } else {
      self.selected_index -= 1;
    }
  }

  fn focus_next(&mut self) {
    self.selected_index = (self.selected_index + 1) % self.registry.len().max(1);
  }

  fn hit_test(positions: &LanguageSelectLayout, x: u16, y: u16) -> Option<usize> {
    positions
      .item_rects
      .iter()
      .position(|rect| rect.contains(x, y))
  }

  fn draw_content(
    &self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    positions: &LanguageSelectLayout,
  ) {
    let entry = &self.registry[self.selected_index];

    // 标题
    render.draw_text(
      canvas,
      &DrawTextParams {
        x: positions.title_x,
        y: positions.title_y,
        text: format!("f%<fg:bright_magenta><b>{}</b></fg>", entry.title),
        ..Default::default()
      },
    );

    // 语言名列表
    for i in 0..self.registry.len() {
      let display = self.item_display_name(i);
      let text = if i == self.selected_index {
        format!("f%<fg:bright_cyan>{}</fg>", display)
      } else {
        display
      };

      render.draw_text(
        canvas,
        &DrawTextParams {
          x: positions.item_xs[i],
          y: positions.item_rects[i].y,
          text,
          ..Default::default()
        },
      );
    }

    // 操作提示
    render.draw_text(
      canvas,
      &DrawTextParams {
        x: positions.hint_x,
        y: positions.hint_y,
        text: format!("f%<fg:rgb(85,87,83)>{}</fg>", entry.hint),
        ..Default::default()
      },
    );
  }
}
