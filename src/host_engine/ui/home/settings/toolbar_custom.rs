use crate::host_engine::services::{
  ActionMapEntry, BorderStyle, CanvasService, DrawTextParams, I18nService, LayoutService, Rect,
  RenderService, RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, TextColor,
  TextInputEvent, TextInputId, TextInputMode, TextInputOptions, TextInputRenderParams,
  TextInputService, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

const NS: &str = "toolbar_custom";

pub struct ToolbarCustomUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  input: TextInputId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolbarCustomCommand {
  Changed(String),
  Submit(String),
}

impl UiObjectPoolOwner for ToolbarCustomUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for ToolbarCustomUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl ToolbarCustomUi {
  pub fn init(text_input: &TextInputService, initial_text: String) -> Self {
    let mut objects = UiObjectPool::new();
    let input = text_input.create(
      &mut objects,
      TextInputOptions {
        initial_text,
        mode: TextInputMode::SingleLine,
        mouse: true,
        ..Default::default()
      },
    );
    Self {
      objects,
      runtime_objects: RuntimeObjectPool::new(),
      input,
    }
  }

  pub fn enter(&mut self, text_input: &mut TextInputService) {
    let _ = text_input.focus(&mut self.objects, self.input);
  }

  pub fn leave(&mut self, text_input: &mut TextInputService) {
    let _ = text_input.blur(&mut self.objects);
  }

  pub fn handle_event(&mut self, event: &UiEvent) -> Option<ToolbarCustomCommand> {
    match event {
      UiEvent::TextInput(TextInputEvent::Changed { id, value }) if *id == self.input => {
        Some(ToolbarCustomCommand::Changed(value.clone()))
      }
      UiEvent::TextInput(TextInputEvent::Submit { id, value }) if *id == self.input => {
        Some(ToolbarCustomCommand::Submit(value.clone()))
      }
      _ => None,
    }
  }

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    text_input: &TextInputService,
  ) -> Option<(u16, u16)> {
    let viewport = layout.developer_viewport_rect();
    let tip_params = RichTextParams::from_action_map(
      &[ActionMapEntry {
        action: "host_key.top_toolbar".to_string(),
        description: "Switch top toolbar view".to_string(),
        keys: vec![vec!["f5".to_string()]],
      }],
      "toolbar_custom.",
    );
    let tip = i18n.get_runtime_text(NS, "toolbar_custom.tip");
    for (line_index, line) in tip.strip_prefix("f%").unwrap_or(&tip).lines().enumerate() {
      let text = format!("f%{line}");
      let width = layout.get_text_width(&text, Some(&tip_params));
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: viewport
            .x
            .saturating_add(viewport.width.saturating_sub(width) / 2),
          y: viewport.y.saturating_add(line_index as u16),
          text,
          params: Some(tip_params.clone()),
          ..Default::default()
        },
      );
    }

    let border_width = viewport.width.saturating_sub(24).max(3);
    let border_x = viewport
      .x
      .saturating_add(viewport.width.saturating_sub(border_width) / 2);
    let border_y = viewport
      .y
      .saturating_add(viewport.height.saturating_sub(3) / 2);
    render.draw_host_border_rect(
      canvas,
      border_x,
      border_y,
      border_width,
      3,
      &BorderStyle::Line,
      None,
      None,
      None,
      None,
    );
    let cursor = text_input.render_host(
      &mut self.objects,
      self.input,
      &TextInputRenderParams {
        rect: Rect {
          x: border_x.saturating_add(1),
          y: border_y.saturating_add(1),
          width: border_width.saturating_sub(2),
          height: 1,
        },
        placeholder: i18n.get_runtime_text(NS, "toolbar_custom.placeholder"),
        fg: Some(TextColor::Rgb {
          r: 255,
          g: 255,
          b: 255,
        }),
        bg: Some(TextColor::Rgb {
          r: 30,
          g: 30,
          b: 30,
        }),
        placeholder_fg: Some(TextColor::Rgb {
          r: 85,
          g: 87,
          b: 83,
        }),
        ..Default::default()
      },
      canvas,
    );

    let hint = format!(
      "f%<fg:rgb(85,87,83)>{}</fg>",
      i18n.get_runtime_text(NS, "toolbar_custom.action.confirm")
    );
    let hint_width = layout.get_text_width(&hint, None);
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: viewport
          .x
          .saturating_add(viewport.width.saturating_sub(hint_width) / 2),
        y: viewport.y.saturating_add(viewport.height.saturating_sub(1)),
        text: hint,
        ..Default::default()
      },
    );
    cursor
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn changed_and_submit_events_return_current_text() {
    let service = TextInputService::new();
    let mut ui = ToolbarCustomUi::init(&service, "old".to_string());

    assert_eq!(
      ui.handle_event(&UiEvent::TextInput(TextInputEvent::Changed {
        id: ui.input,
        value: "live".to_string(),
      })),
      Some(ToolbarCustomCommand::Changed("live".to_string()))
    );
    assert_eq!(
      ui.handle_event(&UiEvent::TextInput(TextInputEvent::Submit {
        id: ui.input,
        value: "saved".to_string(),
      })),
      Some(ToolbarCustomCommand::Submit("saved".to_string()))
    );
  }
}
