use std::time::Duration;

use crate::host_engine::services::{
  CanvasService, DrawTextParams, I18nService, LayoutService, ProgressBarFillOrigin, ProgressBarId,
  ProgressBarOptions, ProgressBarSegmentStyle, ProgressBarService, Rect, RenderService,
  RuntimeObjectPool, RuntimeObjectPoolOwner, TerminalColor, TextColor, TextStyle, UiObjectPool,
  UiObjectPoolOwner, TimeService, TimerId,
};

pub struct LanguageLoadingUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  bar: ProgressBarId,
  animation_timer: TimerId,
}

impl LanguageLoadingUi {
  pub fn init(progress_bar: &ProgressBarService, time: &TimeService) -> Self {
    let mut objects = UiObjectPool::new();
    let mut runtime_objects = RuntimeObjectPool::new();
    let bar = progress_bar
      .create(&mut objects, block_options())
      .expect("valid language loading progress bar options");
    let animation_timer = time.create_count_up(&mut runtime_objects);
    let _ = time.start(&mut runtime_objects, animation_timer);
    Self {
      objects,
      runtime_objects,
      bar,
      animation_timer,
    }
  }

  pub fn restart_animation(&mut self, time: &TimeService) {
    let _ = time.reset(&mut self.runtime_objects, self.animation_timer);
    let _ = time.start(&mut self.runtime_objects, self.animation_timer);
  }

  pub fn update(&mut self, time: &TimeService, dt: Duration) {
    time.update(&mut self.runtime_objects, dt);
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
    time: &TimeService,
  ) {
    let size = layout.physical_size();
    let elapsed = time
      .elapsed(&self.runtime_objects, self.animation_timer)
      .unwrap_or(Duration::ZERO);
    let tip = format!(
      "{}{}",
      i18n.get_runtime_text("language_loading", "language_loading.tip"),
      ".".repeat((elapsed.as_millis() / 500 % 3 + 1) as usize),
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

impl RuntimeObjectPoolOwner for LanguageLoadingUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
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
