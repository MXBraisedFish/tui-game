//! Boot loading page state, user commands, and terminal-cell presentation.

use crate::host_engine::services::{
  CanvasService, DrawTextParams, I18nService, LayoutService, RenderService, TextColor, TextMode,
};

const PENDING_COLOR: TextColor = TextColor::Rgb {
  r: 85,
  g: 87,
  b: 83,
};
const COMPLETE_COLOR: TextColor = TextColor::Rgb {
  r: 65,
  g: 220,
  b: 92,
};
const ACTIVE_COLOR: TextColor = TextColor::Rgb {
  r: 230,
  g: 75,
  b: 190,
};

/// The startup stage displayed in the boot progress screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootStage {
  /// The storage stage of the operation.
  Storage,
  /// The terminal stage of the operation.
  Terminal,
  /// The language stage of the operation.
  Language,
  /// The services stage of the operation.
  Services,
  /// The packages stage of the operation.
  Packages,
  /// The listeners stage of the operation.
  Listeners,
  /// The runtime stage of the operation.
  Runtime,
  /// The ready stage of the operation.
  Ready,
}

impl BootStage {
  /// The all used by this module.
  pub const ALL: [Self; 8] = [
    Self::Storage,
    Self::Terminal,
    Self::Language,
    Self::Services,
    Self::Packages,
    Self::Listeners,
    Self::Runtime,
    Self::Ready,
  ];

  fn index(self) -> usize {
    Self::ALL
      .iter()
      .position(|stage| *stage == self)
      .unwrap_or(0)
  }

  fn key(self) -> &'static str {
    match self {
      Self::Storage => "boot_loading.stage.storage",
      Self::Terminal => "boot_loading.stage.terminal",
      Self::Language => "boot_loading.stage.language",
      Self::Services => "boot_loading.stage.services",
      Self::Packages => "boot_loading.stage.packages",
      Self::Listeners => "boot_loading.stage.listeners",
      Self::Runtime => "boot_loading.stage.runtime",
      Self::Ready => "boot_loading.stage.ready",
    }
  }

  fn english(self) -> &'static str {
    match self {
      Self::Storage => "Storage and logs",
      Self::Terminal => "Terminal and display",
      Self::Language => "Language resources",
      Self::Services => "Core services",
      Self::Packages => "Games and screensavers",
      Self::Listeners => "Input and hot reload listeners",
      Self::Runtime => "Runtime and UI",
      Self::Ready => "Ready",
    }
  }
}

/// The current startup stage and its completed fraction.
///
/// # Fields
///
/// * `stage` - The boot stage carried by this boot progress.
/// * `stage_progress` - The stage progress.
/// * `ready` - The ready.
#[derive(Clone, Copy, Debug)]
pub struct BootProgress {
  /// The boot stage carried by this boot progress.
  pub stage: BootStage,
  /// The stage progress.
  pub stage_progress: f32,
  /// The ready.
  pub ready: bool,
}

impl BootProgress {
  /// Create a boot-progress snapshot for the requested stage.
  pub fn at(stage: BootStage) -> Self {
    Self {
      stage,
      stage_progress: 0.0,
      ready: false,
    }
  }

  /// Update boot progress with the package currently being scanned.
  pub fn package(scanned: usize, total: usize) -> Self {
    Self {
      stage: BootStage::Packages,
      stage_progress: if total == 0 {
        1.0
      } else {
        scanned as f32 / total as f32
      },
      ready: false,
    }
  }

  /// Return the current ready.
  pub fn ready() -> Self {
    Self {
      stage: BootStage::Ready,
      stage_progress: 1.0,
      ready: true,
    }
  }

  fn total_progress(self) -> f32 {
    if self.ready {
      return 1.0;
    }
    ((self.stage.index() as f32 + self.stage_progress.clamp(0.0, 1.0))
      / BootStage::ALL.len() as f32)
      .clamp(0.0, 1.0)
  }
}

/// The state and owned widgets of the boot loading view.
pub struct BootLoadingUi;

impl BootLoadingUi {
  /// Draw the boot loading view and register interaction regions in its assigned surfaces.
  ///
  /// # Arguments
  ///
  /// * `render` - The drawing service used to render terminal cells.
  /// * `canvas` - The clipped canvas used for drawing.
  /// * `layout` - The service resolving terminal sizes and positions.
  /// * `i18n` - The service resolving localized text.
  /// * `progress` - The progress.
  pub fn render(
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    progress: BootProgress,
  ) {
    let title = localized(i18n, "boot_loading.title", "TUI GAME is starting");
    draw_centered(render, canvas, layout, 1, title, ACTIVE_COLOR.clone(), true);

    let list_height = BootStage::ALL.len() as u16;
    let list_y = layout
      .physical_height()
      .saturating_sub(list_height.saturating_add(3))
      / 2;
    let current = progress.stage.index();
    for (index, stage) in BootStage::ALL.into_iter().enumerate() {
      let label = localized(i18n, stage.key(), stage.english());
      let (text, color) = if progress.ready || index < current {
        (format!("✅️ {label} ✅️"), COMPLETE_COLOR.clone())
      } else if index == current {
        (format!("⚡️ {label} ⚡️"), ACTIVE_COLOR.clone())
      } else {
        (label, PENDING_COLOR.clone())
      };
      draw_centered(
        render,
        canvas,
        layout,
        list_y.saturating_add(index as u16),
        text,
        color,
        false,
      );
    }

    let bar_width = layout.physical_width().saturating_sub(36);
    if bar_width > 0 {
      let x = layout.resolve_host_x(LayoutService::ALIGN_CENTER, bar_width, 0);
      let y = list_y.saturating_add(list_height).saturating_add(2);
      let filled = ((bar_width as f32) * progress.total_progress()).round() as u16;
      render.draw_host_text(
        canvas,
        &plain_text(
          x,
          y,
          "░".repeat(bar_width as usize),
          PENDING_COLOR.clone(),
          false,
        ),
      );
      if filled > 0 {
        render.draw_host_text(
          canvas,
          &plain_text(
            x,
            y,
            "█".repeat(filled as usize),
            if progress.ready {
              COMPLETE_COLOR.clone()
            } else {
              ACTIVE_COLOR.clone()
            },
            false,
          ),
        );
      }
    }
  }
}

fn localized(i18n: &I18nService, key: &str, english: &str) -> String {
  let value = i18n.get_runtime_text("boot_loading", key);
  if value.contains("Missing i18n Key") || value.contains("缺失") {
    english.to_string()
  } else {
    value
  }
}

fn draw_centered(
  render: &mut RenderService,
  canvas: &mut CanvasService,
  layout: &LayoutService,
  y: u16,
  text: String,
  color: TextColor,
  bold: bool,
) {
  let width = layout.get_text_width(&text, None);
  let x = layout.resolve_host_x(LayoutService::ALIGN_CENTER, width, 0);
  render.draw_host_text(canvas, &plain_text(x, y, text, color, bold));
}

fn plain_text(
  x: u16,
  y: u16,
  text: impl Into<String>,
  color: TextColor,
  bold: bool,
) -> DrawTextParams {
  DrawTextParams {
    x,
    y,
    text: text.into(),
    text_mode: TextMode::Plain,
    fg: Some(color),
    bold,
    ..Default::default()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn package_progress_is_part_of_total_boot_progress() {
    let start = BootProgress::package(0, 10).total_progress();
    let half = BootProgress::package(5, 10).total_progress();
    let end = BootProgress::package(10, 10).total_progress();
    assert!(start < half && half < end);
  }
}
