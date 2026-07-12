use std::fs;

use crate::host_engine::services::{
  ActionMapEntry, BorderStyle, CanvasService, CodeHighlightService, DrawTextParams, HitAreaEvent,
  HitAreaId, HitAreaOptions, HitAreaService, I18nService, KeyState, LayoutService,
  MarkdownRenderParams, MarkdownService, MarkdownViewId, MarkdownViewOptions, MouseButton,
  Overflow, Rect, RenderService, RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner,
  ScrollBoxId, ScrollBoxOptions, ScrollBoxService, ScrollbarLayout, ScrollbarPolicy,
  ScrollbarVisibility, StorageService, TextColor, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

const NS: &str = "security_details";
const SCROLL_STEP: i32 = 3;

pub struct SecurityDetailsUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  back_area: HitAreaId,
  scroll_box: ScrollBoxId,
  markdown: MarkdownViewId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecurityDetailsCommand {
  Back,
  Scroll(i32),
}

impl UiObjectPoolOwner for SecurityDetailsUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }
  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for SecurityDetailsUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }
  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

impl SecurityDetailsUi {
  pub fn init(
    hit_area: &HitAreaService,
    scroll_box: &ScrollBoxService,
    markdown: &MarkdownService,
    storage: &StorageService,
    i18n: &I18nService,
  ) -> Self {
    let mut objects = UiObjectPool::new();
    let markdown_text = load_markdown(storage, i18n.current_language_code());
    Self {
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      scroll_box: scroll_box
        .create(
          &mut objects,
          ScrollBoxOptions {
            overflow_y: Overflow::Auto,
            overflow_x: Overflow::Hidden,
            scrollbar_layout: ScrollbarLayout::Inside,
            scrollbar: ScrollbarPolicy {
              vertical: ScrollbarVisibility::Auto,
              horizontal: ScrollbarVisibility::Never,
            },
            wheel_step: SCROLL_STEP as u16,
            ..Default::default()
          },
        )
        .expect("security details scroll box options are valid"),
      markdown: markdown
        .create(&mut objects, MarkdownViewOptions::new(markdown_text))
        .expect("security details markdown options are valid"),
      objects,
      runtime_objects: RuntimeObjectPool::new(),
    }
  }

  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      action("security_details.scroll_up", "w", "Scroll markdown up"),
      action("security_details.scroll_down", "s", "Scroll markdown down"),
      action("security_details.back", "esc", "Back to security settings"),
    ]
  }

  pub fn handle_event(&self, event: &UiEvent) -> Option<SecurityDetailsCommand> {
    match event {
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => Some(SecurityDetailsCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "security_details.scroll_up" => Some(SecurityDetailsCommand::Scroll(-SCROLL_STEP)),
        "security_details.scroll_down" => Some(SecurityDetailsCommand::Scroll(SCROLL_STEP)),
        "security_details.back" => Some(SecurityDetailsCommand::Back),
        _ => None,
      },
      _ => None,
    }
  }

  pub fn scroll(&mut self, amount: i32, scroll_box: &ScrollBoxService, layout: &LayoutService) {
    let _ = scroll_box.scroll_by(&mut self.objects, self.scroll_box, 0, amount, layout);
  }

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
    scroll_box: &ScrollBoxService,
    markdown: &MarkdownService,
    code_highlight: &CodeHighlightService,
  ) {
    let viewport = layout.developer_viewport_rect();
    let title = i18n.get_runtime_text(NS, "security_details.title");
    let hint = format!(
      "f%<fg:rgb(85,87,83)>{}  {}</fg>",
      i18n.get_runtime_text(NS, "security_details.action.scroll"),
      i18n.get_runtime_text(NS, "security_details.action.back"),
    );
    let key_params = RichTextParams::from_action_map(&Self::action_map(), "security_details.");
    let title_x = viewport.x.saturating_add(layout.resolve_x(
      LayoutService::ALIGN_CENTER,
      layout.get_text_width(&title, None),
      0,
    ));
    let hint_x = viewport.x.saturating_add(layout.resolve_x(
      LayoutService::ALIGN_CENTER,
      layout.get_text_width(&hint, Some(&key_params)),
      0,
    ));
    let hint_y = viewport.y.saturating_add(viewport.height.saturating_sub(1));
    let frame = Rect {
      x: viewport.x,
      y: viewport.y.saturating_add(1),
      width: viewport.width,
      height: viewport.height.saturating_sub(2),
    };
    let local = Rect {
      x: frame.x.saturating_sub(viewport.x).saturating_add(1),
      y: frame.y.saturating_sub(viewport.y).saturating_add(1),
      width: frame.width.saturating_sub(2),
      height: frame.height.saturating_sub(2),
    };
    let content_width = local.width.saturating_sub(4).max(1);
    let measured = markdown
      .measure(&self.objects, self.markdown, content_width, code_highlight)
      .unwrap_or_default();
    let _ = scroll_box.set_rect(&mut self.objects, self.scroll_box, local, layout);
    let _ = scroll_box.set_content_size(
      &mut self.objects,
      self.scroll_box,
      local.width,
      measured.height.saturating_add(2).max(local.height),
      layout,
    );

    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: title_x,
        y: viewport.y,
        text: format!("f%<fg:rgb(190,130,180)>{title}</fg>"),
        bold: true,
        ..Default::default()
      },
    );
    render.draw_host_border_rect(
      canvas,
      frame.x,
      frame.y,
      frame.width,
      frame.height,
      &BorderStyle::Line,
      Some(TextColor::Rgb {
        r: 220,
        g: 223,
        b: 218,
      }),
      None,
      None,
      None,
    );
    let _ = markdown.render_in_scroll_box(
      &mut self.objects,
      self.markdown,
      self.scroll_box,
      MarkdownRenderParams {
        x: 2,
        y: 1,
        width: content_width,
        max_height: None,
      },
      canvas,
      code_highlight,
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: hint_x,
        y: hint_y,
        text: hint,
        params: Some(key_params),
        ..Default::default()
      },
    );
    hit_area.render_host(&mut self.objects, self.back_area, viewport, canvas);
  }
}

fn load_markdown(storage: &StorageService, language: &str) -> String {
  let relative = format!("assets/markdown/{language}/security_details.md");
  let path = storage.path(&relative);
  fs::read_to_string(path)
    .or_else(|_| fs::read_to_string(storage.path("assets/markdown/en_us/security_details.md")))
    .unwrap_or_default()
}

fn action(name: &str, key: &str, description: &str) -> ActionMapEntry {
  ActionMapEntry {
    action: name.to_string(),
    description: description.to_string(),
    keys: vec![vec![key.to_string()]],
  }
}
