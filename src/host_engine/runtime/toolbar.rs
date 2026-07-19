use std::time::Duration;

use sysinfo::{Networks, System};

use super::*;
use crate::host_engine::services::{
  ProgressBarId, ProgressBarOptions, ProgressBarSegmentStyle, ProgressBarService, Rect,
  TerminalColor, TextStyle, UiObjectPool,
};

const SAMPLE_INTERVAL: Duration = Duration::from_secs(2);
const SMOOTHING: f32 = 0.25;
const EXPORT_PROGRESS_PER_SECOND: f32 = 0.75;
const EXPORT_PROGRESS_HOLD: Duration = Duration::from_millis(400);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum TopToolbarView {
  #[default]
  SystemInfo,
  ImageExport,
  VideoExport,
  Recording,
  Custom,
}

impl TopToolbarView {
  fn next(self) -> Self {
    match self {
      Self::SystemInfo => Self::ImageExport,
      Self::ImageExport => Self::VideoExport,
      Self::VideoExport => Self::Recording,
      Self::Recording => Self::Custom,
      Self::Custom => Self::SystemInfo,
    }
  }
}

#[derive(Default)]
struct SystemSnapshot {
  cpu: f32,
  memory: f32,
  upload: f64,
  download: f64,
  fps: f32,
  initialized: bool,
}

#[derive(Default)]
struct ExportProgressAnimation {
  active: bool,
  finishing: bool,
  queue: usize,
  displayed: f32,
  target: f32,
  hold_elapsed: Duration,
}

impl ExportProgressAnimation {
  fn observe(&mut self, queue: usize, progress: Option<f32>) {
    if let Some(target) = progress {
      let target = target.clamp(0.0, 1.0);
      if !self.active || target < self.target {
        self.displayed = 0.0;
      }
      self.active = true;
      self.finishing = false;
      self.queue = queue.max(1);
      self.target = target;
      self.hold_elapsed = Duration::ZERO;
    } else if self.active {
      self.finishing = true;
      self.target = 1.0;
    }
  }

  fn update(&mut self, dt: Duration) {
    if !self.active {
      return;
    }
    let was_complete = self.displayed >= 1.0;
    self.displayed =
      (self.displayed + dt.as_secs_f32() * EXPORT_PROGRESS_PER_SECOND).min(self.target);
    if self.finishing && was_complete {
      self.hold_elapsed = self.hold_elapsed.saturating_add(dt);
      if self.hold_elapsed >= EXPORT_PROGRESS_HOLD {
        *self = Self::default();
      }
    }
  }

  fn value(&self) -> Option<(usize, f32)> {
    self.active.then_some((self.queue, self.displayed))
  }
}

pub(super) struct TopToolbarRuntime {
  view: TopToolbarView,
  objects: UiObjectPool,
  progress: ProgressBarId,
  system: System,
  networks: Networks,
  sample_elapsed: Duration,
  sampled_frames: u64,
  snapshot: SystemSnapshot,
  image_progress: ExportProgressAnimation,
}

impl TopToolbarRuntime {
  pub(super) fn new(progress_bar: &ProgressBarService) -> Self {
    let mut objects = UiObjectPool::new();
    let mut options = ProgressBarOptions::default();
    options.completed.ch = '─';
    options.preview.ch = '─';
    options.remaining = ProgressBarSegmentStyle {
      ch: '─',
      style: TextStyle {
        foreground: Some(TextColor::Terminal(TerminalColor::BrightBlack)),
        background: Some(TextColor::Transparent),
        ..Default::default()
      },
    };
    let progress = progress_bar
      .create(&mut objects, options)
      .expect("toolbar progress bar style must be valid");
    Self {
      view: TopToolbarView::default(),
      objects,
      progress,
      system: System::new_all(),
      networks: Networks::new_with_refreshed_list(),
      sample_elapsed: Duration::ZERO,
      sampled_frames: 0,
      snapshot: SystemSnapshot::default(),
      image_progress: ExportProgressAnimation::default(),
    }
  }

  pub(super) fn cycle(&mut self) {
    self.view = self.view.next();
  }

  pub(super) fn update(&mut self, dt: Duration) {
    self.image_progress.update(dt);
    self.sampled_frames = self.sampled_frames.saturating_add(1);
    self.sample_elapsed = self.sample_elapsed.saturating_add(dt);
    if self.sample_elapsed < SAMPLE_INTERVAL {
      return;
    }

    let seconds = self.sample_elapsed.as_secs_f64().max(0.001);
    let fps = self.sampled_frames as f32 / seconds as f32;
    self.system.refresh_cpu_usage();
    self.system.refresh_memory();
    self.networks.refresh(true);
    let upload = self
      .networks
      .iter()
      .map(|(_, data)| data.transmitted())
      .sum::<u64>() as f64
      / seconds;
    let download = self
      .networks
      .iter()
      .map(|(_, data)| data.received())
      .sum::<u64>() as f64
      / seconds;
    let memory = if self.system.total_memory() == 0 {
      0.0
    } else {
      self.system.used_memory() as f32 / self.system.total_memory() as f32 * 100.0
    };
    let initialized = self.snapshot.initialized;
    self.snapshot.cpu = smooth(
      self.snapshot.cpu,
      self.system.global_cpu_usage(),
      initialized,
    );
    self.snapshot.memory = smooth(self.snapshot.memory, memory, initialized);
    self.snapshot.upload = smooth64(self.snapshot.upload, upload, initialized);
    self.snapshot.download = smooth64(self.snapshot.download, download, initialized);
    self.snapshot.fps = smooth(self.snapshot.fps, fps, initialized);
    self.snapshot.initialized = true;
    self.sample_elapsed = Duration::ZERO;
    self.sampled_frames = 0;
  }

  pub(super) fn render(
    &mut self,
    services: &mut EngineServices,
    image_queue: usize,
    image_progress: Option<f32>,
    custom_text: Option<&str>,
  ) {
    self.image_progress.observe(image_queue, image_progress);
    let Some(top) = services.host_objects.area_rect(HostAreaKind::TopBar) else {
      return;
    };
    let Some(separator) = services.host_objects.area_rect(HostAreaKind::Separator) else {
      return;
    };
    if top.width == 0 {
      return;
    }

    services.render.draw_host_text(
      &mut services.canvas,
      &DrawTextParams {
        x: separator.x,
        y: separator.y,
        text: "─".repeat(separator.width as usize),
        max_width: Some(separator.width),
        ..Default::default()
      },
    );

    let view = if custom_text.is_some() {
      TopToolbarView::Custom
    } else {
      self.view
    };
    match view {
      TopToolbarView::SystemInfo => self.draw_system_info(services, top),
      TopToolbarView::ImageExport => match self.image_progress.value() {
        Some((queue, progress)) => self.draw_export(services, top, true, queue, Some(progress)),
        None => self.draw_export(services, top, true, 0, None),
      },
      TopToolbarView::VideoExport => self.draw_export(services, top, false, 0, None),
      TopToolbarView::Recording => {
        let text = services
          .i18n
          .get_runtime_text("toolbar", "toolbar.recording.stop");
        self.draw_centered(services, top, text);
      }
      TopToolbarView::Custom => {
        let text = custom_text.map(str::to_owned).unwrap_or_else(|| {
          services
            .storage
            .display_settings_profile()
            .top_toolbar_custom_text
            .clone()
        });
        self.draw_centered(services, top, text);
      }
    }
  }

  fn draw_system_info(&self, services: &mut EngineServices, rect: Rect) {
    let cpu = percent_field(self.snapshot.cpu);
    let memory = percent_field(self.snapshot.memory);
    let upload = rate_field(self.snapshot.upload);
    let download = rate_field(self.snapshot.download);
    let fps = format!("{:<6.1}", self.snapshot.fps.min(9999.9));
    let cpu = toolbar_value(services, "toolbar.system_info.cpu", "{cpu}", &cpu);
    let memory = toolbar_value(services, "toolbar.system_info.mem", "{mem}", &memory);
    let net = services
      .i18n
      .get_runtime_text("toolbar", "toolbar.system_info.net");
    let upload = toolbar_value(
      services,
      "toolbar.system_info.net.upload",
      "{upload}",
      &upload,
    );
    let download = toolbar_value(
      services,
      "toolbar.system_info.net.download",
      "{download}",
      &download,
    );
    let fps = toolbar_value(services, "toolbar.system_info.net.fps", "{fps}", &fps);
    let text = format!("{cpu}  {memory}  {net} {upload} {download}  {fps}");
    self.draw_centered(services, rect, format!("f%{text}"));
  }

  fn draw_export(
    &mut self,
    services: &mut EngineServices,
    rect: Rect,
    image: bool,
    queue: usize,
    progress: Option<f32>,
  ) {
    let prefix = if image { "save_image" } else { "save_video" };
    let Some(progress) = progress else {
      let text = services
        .i18n
        .get_runtime_text("toolbar", &format!("toolbar.{prefix}.no"));
      self.draw_centered(services, rect, text);
      return;
    };
    let queue_text = format!(
      "f%{}",
      services
        .i18n
        .get_runtime_text("toolbar", &format!("toolbar.{prefix}.queue"))
        .replace("{value:done}", &queue.to_string())
    );
    let percent = format!("{:>5.1}%", progress.clamp(0.0, 1.0) * 100.0);
    let queue_width = services.layout.get_text_width(&queue_text, None);
    let percent_width = services.layout.get_text_width(&percent, None);
    let bar_x = rect.x.saturating_add(queue_width).saturating_add(2);
    let bar_width = rect
      .width
      .saturating_sub(queue_width)
      .saturating_sub(percent_width)
      .saturating_sub(4);
    services.render.draw_host_text(
      &mut services.canvas,
      &DrawTextParams {
        x: rect.x.saturating_add(1),
        y: rect.y,
        text: queue_text,
        max_width: Some(queue_width),
        ..Default::default()
      },
    );
    services.render.draw_host_text(
      &mut services.canvas,
      &DrawTextParams {
        x: rect
          .x
          .saturating_add(rect.width.saturating_sub(percent_width + 1)),
        y: rect.y,
        text: percent,
        max_width: Some(percent_width),
        ..Default::default()
      },
    );
    let _ =
      services
        .progress_bar
        .set_progress(&mut self.objects, self.progress, progress, progress);
    let _ = services.progress_bar.render_host(
      &self.objects,
      self.progress,
      Rect {
        x: bar_x,
        y: rect.y,
        width: bar_width,
        height: 1,
      },
      &mut services.canvas,
    );
  }

  fn draw_centered(&self, services: &mut EngineServices, rect: Rect, text: String) {
    let width = services.layout.get_text_width(&text, None).min(rect.width);
    services.render.draw_host_text(
      &mut services.canvas,
      &DrawTextParams {
        x: rect.x.saturating_add(rect.width.saturating_sub(width) / 2),
        y: rect.y,
        text,
        max_width: Some(width),
        ..Default::default()
      },
    );
  }
}

fn smooth(current: f32, sample: f32, initialized: bool) -> f32 {
  if initialized {
    current + (sample - current) * SMOOTHING
  } else {
    sample
  }
}

fn smooth64(current: f64, sample: f64, initialized: bool) -> f64 {
  if initialized {
    current + (sample - current) * SMOOTHING as f64
  } else {
    sample
  }
}

fn byte_rate(mut bytes: f64) -> String {
  let units = ["B/s", "KB/s", "MB/s", "GB/s"];
  let mut unit = 0;
  while bytes >= 1024.0 && unit + 1 < units.len() {
    bytes /= 1024.0;
    unit += 1;
  }
  format!("{bytes:.1}{}", units[unit])
}

fn percent_field(value: f32) -> String {
  format!("{:<6}", format!("{:.1}%", value.clamp(0.0, 100.0)))
}

fn rate_field(bytes: f64) -> String {
  format!("{:<10}", byte_rate(bytes))
}

fn toolbar_value(services: &EngineServices, key: &str, placeholder: &str, value: &str) -> String {
  services
    .i18n
    .get_runtime_text("toolbar", key)
    .replace(placeholder, value)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn toolbar_view_cycles_through_every_view() {
    let mut view = TopToolbarView::SystemInfo;
    for expected in [
      TopToolbarView::ImageExport,
      TopToolbarView::VideoExport,
      TopToolbarView::Recording,
      TopToolbarView::Custom,
      TopToolbarView::SystemInfo,
    ] {
      view = view.next();
      assert_eq!(view, expected);
    }
  }

  #[test]
  fn byte_rate_uses_binary_units() {
    assert_eq!(byte_rate(512.0), "512.0B/s");
    assert_eq!(byte_rate(1536.0), "1.5KB/s");
    assert_eq!(byte_rate(1024.0 * 1024.0), "1.0MB/s");
    assert_eq!(byte_rate(1024.0 * 1024.0 * 1024.0), "1.0GB/s");
  }

  #[test]
  fn toolbar_numeric_fields_are_left_aligned_and_fixed_width() {
    assert_eq!(percent_field(80.7), "80.7% ");
    assert_eq!(percent_field(100.0), "100.0%");
    assert_eq!(rate_field(1536.0), "1.5KB/s   ");
  }

  #[test]
  fn export_progress_advances_gradually_after_task_finishes() {
    let mut progress = ExportProgressAnimation::default();
    progress.observe(1, Some(1.0));
    progress.update(Duration::from_millis(100));
    assert!(progress.value().unwrap().1 < 1.0);
    progress.observe(0, None);
    progress.update(Duration::from_secs(2));
    assert_eq!(progress.value().unwrap().1, 1.0);
  }

  #[test]
  fn smoothing_keeps_only_a_fraction_of_the_new_sample() {
    assert_eq!(smooth(10.0, 30.0, false), 30.0);
    assert_eq!(smooth(10.0, 30.0, true), 15.0);
  }
}
