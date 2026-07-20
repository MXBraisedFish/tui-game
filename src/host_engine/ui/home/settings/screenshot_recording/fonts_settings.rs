use crate::host_engine::services::{
  ActionMapEntry, BorderStyle, CanvasService, DrawTextParams, I18nService, KeyState, LayoutService,
  Overflow, Rect, RenderService, RichTextParams, ScrollBoxId, ScrollBoxOptions, ScrollBoxService,
  ScrollbarLayout, ScrollbarPolicy, ScrollbarVisibility, TextColor, TextInputEvent, TextInputId,
  TextInputMode, TextInputOptions, TextInputRenderParams, TextInputService, UiEvent, UiObjectPool,
};

const NS: &str = "fonts_settings";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FontsSettingsCommand {
  Back(Vec<String>),
  StartAdd,
  StartModify,
  FinishEdit(String),
  CancelEdit,
  Scroll(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FontEditMode {
  Add,
  Modify(usize),
}

pub struct FontsSettingsUi {
  scroll: ScrollBoxId,
  input: TextInputId,
  fonts: Vec<String>,
  selected: usize,
  locked: bool,
  edit_mode: Option<FontEditMode>,
}

impl FontsSettingsUi {
  pub fn create(
    objects: &mut UiObjectPool,
    text_input: &TextInputService,
    scroll_box: &ScrollBoxService,
  ) -> Self {
    let scroll = scroll_box
      .create(
        objects,
        ScrollBoxOptions {
          rect: Rect::default(),
          content_width: 1,
          content_height: 1,
          overflow_x: Overflow::Hidden,
          overflow_y: Overflow::Auto,
          scrollbar: ScrollbarPolicy {
            vertical: ScrollbarVisibility::Auto,
            horizontal: ScrollbarVisibility::Never,
          },
          scrollbar_layout: ScrollbarLayout::Inside,
          ..Default::default()
        },
      )
      .expect("failed to create font settings scroll box");
    let input = text_input.create(
      objects,
      TextInputOptions {
        mode: TextInputMode::SingleLine,
        mouse: true,
        ..Default::default()
      },
    );
    Self {
      scroll,
      input,
      fonts: Vec::new(),
      selected: 0,
      locked: false,
      edit_mode: None,
    }
  }

  pub fn enter(&mut self, fonts: Vec<String>) {
    self.fonts = fonts;
    self.selected = self.selected.min(self.fonts.len().saturating_sub(1));
    self.locked = false;
    self.edit_mode = None;
  }

  pub fn start_add(&mut self, objects: &mut UiObjectPool, text_input: &mut TextInputService) {
    self.edit_mode = Some(FontEditMode::Add);
    let _ = text_input.clear(objects, self.input);
    let _ = text_input.focus(objects, self.input);
  }

  pub fn start_modify(&mut self, objects: &mut UiObjectPool, text_input: &mut TextInputService) {
    let Some(value) = self.fonts.get(self.selected).cloned() else {
      return;
    };
    self.edit_mode = Some(FontEditMode::Modify(self.selected));
    let _ = text_input.set_text(objects, self.input, &value);
    let _ = text_input.focus(objects, self.input);
  }

  pub fn finish_edit(
    &mut self,
    objects: &mut UiObjectPool,
    text_input: &mut TextInputService,
    value: String,
  ) {
    let value = value.trim();
    if !value.is_empty() {
      match self.edit_mode {
        Some(FontEditMode::Add) => {
          self.fonts.push(value.to_string());
          self.selected = self.fonts.len() - 1;
        }
        Some(FontEditMode::Modify(index)) if index < self.fonts.len() => {
          self.fonts[index] = value.to_string();
          self.selected = index;
        }
        _ => {}
      }
    }
    self.edit_mode = None;
    let _ = text_input.blur(objects);
  }

  pub fn cancel_edit(&mut self, objects: &mut UiObjectPool, text_input: &mut TextInputService) {
    self.edit_mode = None;
    let _ = text_input.blur(objects);
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    [
      ("fonts_settings.focus_up.move_up", "up"),
      ("fonts_settings.focus_down.move_down", "down"),
      ("fonts_settings.scroll_up", "w"),
      ("fonts_settings.scroll_down", "s"),
      ("fonts_settings.confirm", "enter"),
      ("fonts_settings.back", "esc"),
      ("fonts_settings.lock_unlock", "b"),
      ("fonts_settings.export_preview", "v"),
      ("fonts_settings.add", "a"),
      ("fonts_settings.del", "d"),
      ("fonts_settings.modify", "f"),
    ]
    .into_iter()
    .map(|(action, key)| ActionMapEntry {
      action: action.to_string(),
      description: action.to_string(),
      keys: vec![vec![key.to_string()]],
    })
    .collect()
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<FontsSettingsCommand> {
    if self.edit_mode.is_some() {
      return match event {
        UiEvent::TextInput(TextInputEvent::Submit { id, value }) if *id == self.input => {
          Some(FontsSettingsCommand::FinishEdit(value.clone()))
        }
        UiEvent::TextInput(TextInputEvent::Cancel { id, .. }) if *id == self.input => {
          Some(FontsSettingsCommand::CancelEdit)
        }
        _ => None,
      };
    }
    let UiEvent::Action(event) = event else {
      return None;
    };
    if event.state != KeyState::Pressed {
      return None;
    }
    match event.action.as_str() {
      "fonts_settings.focus_up.move_up" => self.move_selection(-1),
      "fonts_settings.focus_down.move_down" => self.move_selection(1),
      "fonts_settings.scroll_up" => return Some(FontsSettingsCommand::Scroll(-3)),
      "fonts_settings.scroll_down" => return Some(FontsSettingsCommand::Scroll(3)),
      "fonts_settings.lock_unlock" if !self.fonts.is_empty() => self.locked = !self.locked,
      "fonts_settings.add" => return Some(FontsSettingsCommand::StartAdd),
      "fonts_settings.modify" if !self.fonts.is_empty() => {
        return Some(FontsSettingsCommand::StartModify);
      }
      "fonts_settings.del" if !self.fonts.is_empty() => {
        self.fonts.remove(self.selected);
        self.selected = self.selected.min(self.fonts.len().saturating_sub(1));
      }
      "fonts_settings.back" | "screenshot_settings.back" => {
        return Some(FontsSettingsCommand::Back(self.fonts.clone()));
      }
      _ => {}
    }
    None
  }

  fn move_selection(&mut self, delta: isize) {
    if self.fonts.is_empty() {
      return;
    }
    let next = (self.selected as isize + delta).clamp(0, self.fonts.len() as isize - 1) as usize;
    if self.locked && next != self.selected {
      self.fonts.swap(self.selected, next);
    }
    self.selected = next;
  }

  pub fn scroll(
    &mut self,
    objects: &mut UiObjectPool,
    service: &ScrollBoxService,
    layout: &LayoutService,
    dy: i32,
  ) {
    let _ = service.scroll_by(objects, self.scroll, 0, dy, layout);
  }

  pub fn prepare(
    &mut self,
    objects: &mut UiObjectPool,
    service: &ScrollBoxService,
    layout: &LayoutService,
  ) {
    let viewport = layout.developer_viewport_rect();
    let hint_height = 2;
    let rect = Rect {
      x: 1,
      y: 2,
      width: viewport.width.saturating_sub(2),
      height: viewport.height.saturating_sub(2 + hint_height),
    };
    let _ = service.set_rect(objects, self.scroll, rect, layout);
    let _ = service.set_content_size(
      objects,
      self.scroll,
      rect.width.saturating_sub(2).max(1),
      (self.fonts.len() as u16)
        .max(rect.height.saturating_sub(2))
        .max(1),
      layout,
    );
    if rect.height > 2 {
      let visible = rect.height.saturating_sub(2) as usize;
      let top = service.scroll_y(objects, self.scroll).unwrap_or(0) as usize;
      if self.selected < top {
        let _ = service.scroll_to(objects, self.scroll, 0, self.selected as u16, layout);
      } else if self.selected >= top.saturating_add(visible) {
        let y = self.selected.saturating_add(1).saturating_sub(visible);
        let _ = service.scroll_to(objects, self.scroll, 0, y as u16, layout);
      }
    }
  }

  pub fn render(
    &mut self,
    objects: &mut UiObjectPool,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    text_input: &TextInputService,
  ) -> Option<(u16, u16)> {
    let viewport = layout.developer_viewport_rect();
    let title = i18n.get_runtime_text(NS, "fonts_settings.title");
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: viewport.x
          + viewport
            .width
            .saturating_sub(layout.get_text_width(&title, None))
            / 2,
        y: viewport.y,
        text: format!("f%<fg:bright_magenta><b>{title}</b></fg>"),
        ..Default::default()
      },
    );
    let frame = Rect {
      x: viewport.x,
      y: viewport.y.saturating_add(1),
      width: viewport.width,
      height: viewport.height.saturating_sub(3),
    };
    render.draw_host_border_rect(
      canvas,
      frame.x,
      frame.y,
      frame.width,
      frame.height,
      &BorderStyle::Line,
      None,
      None,
      None,
      None,
    );

    if self.fonts.is_empty() {
      let no = i18n.get_runtime_text(NS, "fonts_settings.no");
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: frame.x + frame.width.saturating_sub(layout.get_text_width(&no, None)) / 2,
          y: frame.y + frame.height / 2,
          text: format!("f%<fg:rgb(85,87,83)>{no}</fg>"),
          ..Default::default()
        },
      );
    } else {
      for (index, font) in self.fonts.iter().enumerate() {
        let number = format!("{:>4}", index + 1);
        render.draw_text_in_scroll_box(
          canvas,
          self.scroll,
          &DrawTextParams {
            x: 0,
            y: index as u16,
            text: format!("f%<fg:rgb(170,170,170)>{number}</fg>"),
            ..Default::default()
          },
        );
        if index == self.selected {
          let color = if self.locked {
            "bright_red"
          } else {
            "bright_cyan"
          };
          render.draw_text_in_scroll_box(
            canvas,
            self.scroll,
            &DrawTextParams {
              x: 4,
              y: index as u16,
              text: format!("f%<fg:{color}>▌</fg>"),
              ..Default::default()
            },
          );
        }
        render.draw_text_in_scroll_box(
          canvas,
          self.scroll,
          &DrawTextParams {
            x: 6,
            y: index as u16,
            text: font.clone(),
            max_width: Some(frame.width.saturating_sub(9)),
            max_height: Some(1),
            overflow_marker: Some("...".to_string()),
            ..Default::default()
          },
        );
      }
    }

    let hint = self.hint(i18n);
    let params = RichTextParams::from_action_map(&Self::action_map(), "fonts_settings.");
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: viewport.x
          + viewport
            .width
            .saturating_sub(layout.get_text_width(&hint, Some(&params)))
            / 2,
        y: viewport.y.saturating_add(viewport.height.saturating_sub(1)),
        text: format!(
          "f%<fg:rgb(85,87,83)>{}</fg>",
          hint.strip_prefix("f%").unwrap_or(&hint)
        ),
        params: Some(params),
        ..Default::default()
      },
    );

    if self.edit_mode.is_none() {
      return None;
    }
    self.render_add_dialog(objects, render, canvas, layout, i18n, text_input)
  }

  fn render_add_dialog(
    &mut self,
    objects: &mut UiObjectPool,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    text_input: &TextInputService,
  ) -> Option<(u16, u16)> {
    let viewport = layout.developer_viewport_rect();
    let hint = i18n.get_runtime_text(NS, "fonts_settings.hint");
    let lines: Vec<_> = hint.lines().collect();
    let actions = format!(
      "{}  {}",
      i18n.get_runtime_text(NS, "fonts_settings.action.cancel"),
      i18n.get_runtime_text(NS, "fonts_settings.action.confirm")
    );
    let params = RichTextParams::from_action_map(&Self::action_map(), "fonts_settings.");
    let placeholder = i18n.get_runtime_text(NS, "fonts_settings.placeholder");
    let max_line = lines
      .iter()
      .map(|line| layout.get_text_width(line, None))
      .max()
      .unwrap_or(1)
      .max(layout.get_text_width(&actions, Some(&params)))
      .max(layout.get_text_width(&placeholder, None));
    let width = max_line
      .saturating_add(4)
      .min(viewport.width.saturating_sub(4))
      .max(8);
    let height = (lines.len() as u16)
      .saturating_add(6)
      .min(viewport.height.saturating_sub(2));
    let rect = Rect {
      x: viewport.x + viewport.width.saturating_sub(width) / 2,
      y: viewport.y + viewport.height.saturating_sub(height) / 2,
      width,
      height,
    };
    render.draw_host_border_rect(
      canvas,
      rect.x,
      rect.y,
      rect.width,
      rect.height,
      &BorderStyle::Line,
      Some(TextColor::Terminal(
        crate::host_engine::services::TerminalColor::BrightBlue,
      )),
      None,
      None,
      None,
    );
    for (index, line) in lines
      .iter()
      .take(rect.height.saturating_sub(5) as usize)
      .enumerate()
    {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: rect.x.saturating_add(2),
          y: rect.y.saturating_add(1 + index as u16),
          text: (*line).to_string(),
          max_width: Some(rect.width.saturating_sub(4)),
          ..Default::default()
        },
      );
    }
    let separator_style = "f%<fg:bright_blue>";
    let separator = format!(
      "{separator_style}├{}┤</fg>",
      "─".repeat(rect.width.saturating_sub(2) as usize)
    );
    let first_separator_y = rect.y.saturating_add(1 + lines.len() as u16);
    for y in [first_separator_y, first_separator_y.saturating_add(2)] {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: rect.x,
          y,
          text: separator.clone(),
          max_width: Some(rect.width),
          max_height: Some(1),
          ..Default::default()
        },
      );
    }
    let input_y = first_separator_y.saturating_add(1);
    let cursor = text_input.render_host(
      objects,
      self.input,
      &TextInputRenderParams {
        rect: Rect {
          x: rect.x + 1,
          y: input_y,
          width: rect.width.saturating_sub(2),
          height: 1,
        },
        placeholder,
        placeholder_fg: Some(TextColor::Rgb {
          r: 85,
          g: 87,
          b: 83,
        }),
        ..Default::default()
      },
      canvas,
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: rect.x
          + rect
            .width
            .saturating_sub(layout.get_text_width(&actions, Some(&params)))
            / 2,
        y: rect.y.saturating_add(rect.height.saturating_sub(2)),
        text: format!("f%<fg:rgb(85,87,83)>{actions}</fg>"),
        params: Some(params),
        ..Default::default()
      },
    );
    cursor
  }

  fn hint(&self, i18n: &I18nService) -> String {
    let select = if self.locked {
      "fonts_settings.action.move"
    } else {
      "fonts_settings.action.select"
    };
    let lock = if self.locked {
      "fonts_settings.action.unlock"
    } else {
      "fonts_settings.action.lock"
    };
    [
      "fonts_settings.action.scroll",
      select,
      "fonts_settings.action.back",
      lock,
      "fonts_settings.action.add",
      "fonts_settings.action.modify",
      "fonts_settings.action.del",
      "fonts_settings.action.export_preview",
    ]
    .into_iter()
    .map(|key| i18n.get_runtime_text(NS, key))
    .collect::<Vec<_>>()
    .join("  ")
  }
}
