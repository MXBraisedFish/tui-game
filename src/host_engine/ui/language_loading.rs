use std::time::Duration;

use crate::host_engine::services::{
  CanvasService, DrawTextParams, I18nService, LayoutService, ProgressBarFillOrigin, ProgressBarId,
  ProgressBarOptions, ProgressBarSegmentStyle, ProgressBarService, Rect, RenderService,
  TerminalColor, TextColor, TextStyle, UiObjectPool, UiObjectPoolOwner,
};

pub struct LanguageLoadingUi {
  objects: UiObjectPool,
  bar: ProgressBarId,
  elapsed: Duration,
}

impl LanguageLoadingUi {
  pub fn init(progress_bar: &ProgressBarService) -> Self {
    let mut objects = UiObjectPool::new();
    let bar = progress_bar
      .create(&mut objects, block_options())
      .expect("valid language loading progress bar options");
    Self {
      objects,
      bar,
      elapsed: Duration::ZERO,
    }
  }

  pub fn update(&mut self, dt: Duration) {
    self.elapsed += dt;
  }

  pub fn set_progress(&mut self, progress_bar: &ProgressBarService, completed: f32, preview: f32) {
    let _ = progress_bar.set_progress(&mut self.objects, self.bar, completed, preview);
  }

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    progress_bar: &ProgressBarService,
  ) {
    let size = layout.physical_size();
    let tip = format!(
      "{}{}",
      i18n.get_runtime_text("language_loading", "language_loading.tip"),
      ".".repeat((self.elapsed.as_millis() / 500 % 3 + 1) as usize),
    );
    let tip_w = layout.get_text_width(&tip, None);
    let start_y = size.height.saturating_sub(3) / 2;
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: layout.resolve_host_x(LayoutService::ALIGN_CENTER, tip_w, 0),
        y: start_y,
        text: tip,
        ..Default::default()
      },
    );

    let bar_w = size.width.saturating_sub(24);
    if bar_w > 0 {
      let _ = progress_bar.render_host(
        &self.objects,
        self.bar,
        Rect {
          x: 12,
          y: start_y.saturating_add(2),
          width: bar_w,
          height: 1,
        },
        canvas,
      );
    }
  }
}

impl UiObjectPoolOwner for LanguageLoadingUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

fn block_options() -> ProgressBarOptions {
  ProgressBarOptions {
    completed: segment(TerminalColor::Green),
    preview: segment(TerminalColor::BrightBlue),
    remaining: segment(TerminalColor::White),
    origin: ProgressBarFillOrigin::Left,
  }
}

fn segment(color: TerminalColor) -> ProgressBarSegmentStyle {
  ProgressBarSegmentStyle {
    ch: '█',
    style: TextStyle {
      foreground: Some(TextColor::Terminal(color)),
      background: Some(TextColor::Transparent),
      ..Default::default()
    },
  }
}
