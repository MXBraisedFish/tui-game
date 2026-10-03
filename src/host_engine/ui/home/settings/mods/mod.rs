//! Mods page state, user commands, and terminal-cell presentation.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use crate::host_engine::services::{
  ActionMapEntry, CanvasService, DrawTextParams, HitAreaEvent, HitAreaId, HitAreaOptions,
  HitAreaService, I18nService, ImageConvertMode, ImageConvertParams, ImageService, KeyState,
  LayoutService, LogService, MouseButton, PackageImageMode, PackageService, Rect, RenderService,
  RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, ScrollBoxService, StorageService,
  TextInputService, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

pub mod game;
pub mod screensaver;

const MODS_MENU_LEN: usize = 2;

const MENU_KEYS: &[&str] = &["mods.game", "mods.screensaver"];

/// Resolved geometry and positions used to display mods.
pub(crate) struct ModsLayout {
  title_x: u16,
  title_y: u16,
  menu_item_rects: [Rect; MODS_MENU_LEN],
  hint_x: u16,
  hint_y: u16,
}

/// The services and view state needed to draw package list.
///
/// # Fields
///
/// * `render` - The &'a mut render service instance used by this owner.
/// * `canvas` - The &'a mut canvas service instance used by this owner.
/// * `layout` - The &'a layout service instance used by this owner.
/// * `i18n` - The &'a i18n service instance used by this owner.
/// * `hit_area` - The &'a hit area service instance used by this owner.
/// * `text_input` - The &'a text input service instance used by this owner.
/// * `scroll_box` - The &'a scroll box service instance used by this owner.
/// * `package` - The &'a package service instance used by this owner.
/// * `storage` - The &'a storage service instance used by this owner.
/// * `log` - The &'a mut log service instance used by this owner.
/// * `image` - The &'a mut image service instance used by this owner.
/// * `mouse_supported` - The mouse supported.
/// * `truecolor_supported` - The truecolor supported.
pub(crate) struct PackageListRenderContext<'a> {
  /// The &'a mut render service instance used by this owner.
  pub(crate) render: &'a mut RenderService,
  /// The &'a mut canvas service instance used by this owner.
  pub(crate) canvas: &'a mut CanvasService,
  /// The &'a layout service instance used by this owner.
  pub(crate) layout: &'a LayoutService,
  /// The &'a i18n service instance used by this owner.
  pub(crate) i18n: &'a I18nService,
  /// The &'a hit area service instance used by this owner.
  pub(crate) hit_area: &'a HitAreaService,
  /// The &'a text input service instance used by this owner.
  pub(crate) text_input: &'a TextInputService,
  /// The &'a scroll box service instance used by this owner.
  pub(crate) scroll_box: &'a ScrollBoxService,
  /// The &'a package service instance used by this owner.
  pub(crate) package: &'a PackageService,
  /// The &'a storage service instance used by this owner.
  pub(crate) storage: &'a StorageService,
  /// The &'a mut log service instance used by this owner.
  pub(crate) log: &'a mut LogService,
  /// The &'a mut image service instance used by this owner.
  pub(crate) image: &'a mut ImageService,
  /// The mouse supported.
  pub(crate) mouse_supported: bool,
  /// The truecolor supported.
  pub(crate) truecolor_supported: bool,
}

const MAX_PACKAGE_IMAGE_CACHE_ENTRIES: usize = 64;

fn package_image_convert_mode(mode: PackageImageMode) -> ImageConvertMode {
  match mode {
    PackageImageMode::HalfBlock => ImageConvertMode::HalfBlock,
    PackageImageMode::MixBlock => ImageConvertMode::MixBlock,
  }
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct PackageImageCacheKey {
  image_path: String,
  mode: ImageConvertMode,
  background: [u8; 3],
  output_width: Option<u32>,
  output_height: Option<u32>,
  crop_x: i32,
  crop_y: i32,
  crop_width: Option<u32>,
  crop_height: Option<u32>,
  square_crop: bool,
  scale_bits: u64,
}

impl PackageImageCacheKey {
  fn from_params(params: &ImageConvertParams) -> Self {
    Self {
      image_path: params.image_path.clone(),
      mode: params.mode,
      background: params.background,
      output_width: params.output_width,
      output_height: params.output_height,
      crop_x: params.crop_x,
      crop_y: params.crop_y,
      crop_width: params.crop_width,
      crop_height: params.crop_height,
      square_crop: params.square_crop,
      scale_bits: params.scale.to_bits(),
    }
  }
}

/// The package image cache representation used by this module.
#[derive(Default)]
pub(crate) struct PackageImageCache {
  snapshot_revision: Option<u64>,
  rendered: HashMap<PackageImageCacheKey, Option<Arc<str>>>,
  insertion_order: VecDeque<PackageImageCacheKey>,
}

impl PackageImageCache {
  fn get_or_convert(
    &mut self,
    snapshot_revision: u64,
    image_service: &mut ImageService,
    params: ImageConvertParams,
  ) -> Option<Arc<str>> {
    if self.snapshot_revision != Some(snapshot_revision) {
      self.rendered.clear();
      self.insertion_order.clear();
      self.snapshot_revision = Some(snapshot_revision);
    }

    let key = PackageImageCacheKey::from_params(&params);
    if let Some(cached) = self.rendered.get(&key) {
      return cached.clone();
    }

    let rendered = image_service.convert(params).ok().map(Arc::<str>::from);
    if self.rendered.len() == MAX_PACKAGE_IMAGE_CACHE_ENTRIES
      && let Some(oldest) = self.insertion_order.pop_front()
    {
      self.rendered.remove(&oldest);
    }
    self.insertion_order.push_back(key.clone());
    self.rendered.insert(key, rendered.clone());
    rendered
  }
}

/// The package info render area representation used by this module.
///
/// # Fields
///
/// * `rect` - The rectangular region in terminal cells.
/// * `scroll_y` - The content offset from the viewport origin in terminal rows.
#[derive(Clone, Copy)]
pub(crate) struct PackageInfoRenderArea {
  /// The rectangular region in terminal cells.
  pub(crate) rect: Rect,
  /// The content offset from the viewport origin in terminal rows.
  pub(crate) scroll_y: u16,
}

/// The package info text position representation used by this module.
///
/// # Fields
///
/// * `x` - The horizontal coordinate in terminal cells.
/// * `y` - The vertical coordinate in terminal cells.
#[derive(Clone, Copy)]
pub(crate) struct PackageInfoTextPosition {
  /// The horizontal coordinate in terminal cells.
  pub(crate) x: u16,
  /// The vertical coordinate in terminal cells.
  pub(crate) y: u16,
}

/// The state and owned widgets of the mods view.
pub struct ModsUi {
  selected_index: usize,
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  back_area: HitAreaId,
  menu_areas: [HitAreaId; MODS_MENU_LEN],
}

impl UiObjectPoolOwner for ModsUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }

  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for ModsUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }

  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}

/// An application request produced by mods interactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModsCommand {
  /// A request to open game.
  OpenGame,
  /// A request to open screensaver.
  OpenScreensaver,
  /// A request to back.
  Back,
}

impl ModsUi {
  /// Create the mods view and allocate its owned UI objects.
  pub fn init(hit_area: &HitAreaService) -> Self {
    let mut objects = UiObjectPool::new();
    Self {
      selected_index: 0,
      back_area: hit_area.create(&mut objects, HitAreaOptions::default()),
      menu_areas: std::array::from_fn(|_| hit_area.create(&mut objects, HitAreaOptions::default())),
      objects,
      runtime_objects: RuntimeObjectPool::new(),
    }
  }

  /// Return the shortcuts currently enabled by the mods view.
  pub fn action_map() -> Vec<ActionMapEntry> {
    vec![
      ActionMapEntry {
        action: "mods.focus_game".to_string(),
        description: "Focus game pack option".to_string(),
        keys: vec![vec!["1".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "mods.focus_screensaver".to_string(),
        description: "Focus screensaver pack option".to_string(),
        keys: vec![vec!["2".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "mods.focus_up".to_string(),
        description: "Focus previous option".to_string(),
        keys: vec![vec!["up".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "mods.focus_down".to_string(),
        description: "Focus next option".to_string(),
        keys: vec![vec!["down".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "mods.confirm".to_string(),
        description: "Confirm selected option".to_string(),
        keys: vec![vec!["enter".to_string()]],
        priority: 0,
      },
      ActionMapEntry {
        action: "mods.back".to_string(),
        description: "Go back to settings".to_string(),
        keys: vec![vec!["esc".to_string()]],
        priority: 0,
      },
    ]
  }

  /// Interpret a mods UI event and return the requested application command.
  pub fn handle_event(&mut self, event: &UiEvent) -> Option<ModsCommand> {
    match event {
      UiEvent::HitArea(HitAreaEvent::HoverEnter { id, .. }) => {
        self.selected_index = self.menu_areas.iter().position(|area| area == id)?;
        None
      }
      UiEvent::HitArea(HitAreaEvent::Click {
        id,
        button: MouseButton::Left,
        ..
      }) => {
        self.selected_index = self.menu_areas.iter().position(|area| area == id)?;
        Some(self.confirm_selected())
      }
      UiEvent::HitArea(HitAreaEvent::Press {
        button: MouseButton::Right,
        ..
      }) => Some(ModsCommand::Back),
      UiEvent::Action(event) if event.state == KeyState::Pressed => match event.action.as_str() {
        "mods.focus_game" => {
          self.selected_index = 0;
          None
        }
        "mods.focus_screensaver" => {
          self.selected_index = 1;
          None
        }
        "mods.focus_up" => {
          self.focus_previous();
          None
        }
        "mods.focus_down" => {
          self.focus_next();
          None
        }
        "mods.confirm" => Some(self.confirm_selected()),
        "mods.back" => Some(ModsCommand::Back),
        _ => None,
      },
      _ => None,
    }
  }

  /// Advance the mods view's transient state for this host frame.
  pub fn update(&mut self, dt: Duration) -> Option<ModsCommand> {
    let _ = dt;
    None
  }

  /// Draw the mods view and register interaction regions in its assigned surfaces.
  ///
  /// # Arguments
  ///
  /// * `render` - The drawing service used to render terminal cells.
  /// * `canvas` - The clipped canvas used for drawing.
  /// * `layout` - The service resolving terminal sizes and positions.
  /// * `i18n` - The service resolving localized text.
  /// * `hit_area` - The hit area.
  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    i18n: &I18nService,
    hit_area: &HitAreaService,
  ) {
    let positions = self.compute_positions(layout, i18n);
    self.draw_content(render, canvas, &positions, i18n);
    let viewport = layout.developer_viewport_rect();
    hit_area.render_host(&mut self.objects, self.back_area, viewport, canvas);
    for (id, rect) in self.menu_areas.into_iter().zip(positions.menu_item_rects) {
      hit_area.render_host(&mut self.objects, id, rect, canvas);
    }
  }

  /// Resolve the mods view's terminal-cell layout from its available dimensions.
  pub fn compute_positions(&self, layout: &LayoutService, i18n: &I18nService) -> ModsLayout {
    let params = self.build_key_params();
    let viewport = layout.developer_viewport_rect();
    let title = i18n.get_runtime_text("mods", "mods.title");
    let title_w = layout.get_text_width(&title, None);
    let title_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, title_w, 0));
    let title_y = viewport.y.saturating_add(1);
    let menu_items = self.menu_items(i18n);
    let menu_item_widths: [u16; MODS_MENU_LEN] =
      std::array::from_fn(|i| layout.get_text_width(&menu_items[i], None));
    let menu_item_xs: [u16; MODS_MENU_LEN] = std::array::from_fn(|i| {
      viewport.x.saturating_add(layout.resolve_x(
        LayoutService::ALIGN_CENTER,
        menu_item_widths[i],
        0,
      ))
    });
    let menu_height = MODS_MENU_LEN as u16;
    let hint = format!(
      "f%<fg:bright_black>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text("mods", "mods.action.focus"),
      i18n.get_runtime_text("mods", "mods.action.select"),
      i18n.get_runtime_text("mods", "mods.action.confirm"),
      i18n.get_runtime_text("mods", "mods.action.back"),
    );
    let hint_w = layout.get_text_width(&hint, Some(&params));
    let hint_x =
      viewport
        .x
        .saturating_add(layout.resolve_x(LayoutService::ALIGN_CENTER, hint_w, 0));
    let hint_y = viewport
      .y
      .saturating_add(layout.developer_height().saturating_sub(1));
    let available = hint_y.saturating_sub(title_y).saturating_sub(1);
    let menu_y = if available > menu_height {
      title_y
        .saturating_add(1)
        .saturating_add((available - menu_height) / 2)
    } else {
      title_y.saturating_add(1)
    };

    let menu_item_rects: [Rect; MODS_MENU_LEN] = std::array::from_fn(|i| Rect {
      x: menu_item_xs[i],
      y: menu_y.saturating_add(i as u16),
      width: menu_item_widths[i],
      height: 1,
    });

    ModsLayout {
      title_x,
      title_y,
      menu_item_rects,
      hint_x,
      hint_y,
    }
  }

  fn focus_previous(&mut self) {
    if self.selected_index == 0 {
      self.selected_index = MODS_MENU_LEN - 1;
    } else {
      self.selected_index -= 1;
    }
  }

  fn focus_next(&mut self) {
    self.selected_index = (self.selected_index + 1) % MODS_MENU_LEN;
  }

  fn confirm_selected(&self) -> ModsCommand {
    match self.selected_index {
      0 => ModsCommand::OpenGame,
      _ => ModsCommand::OpenScreensaver,
    }
  }

  fn menu_items(&self, i18n: &I18nService) -> [String; MODS_MENU_LEN] {
    std::array::from_fn(|i| {
      let label = i18n.get_runtime_text("mods", MENU_KEYS[i]);
      if i == self.selected_index {
        format!("f%<fg:bright_cyan>❯ {} ❮</fg>", label)
      } else {
        label
      }
    })
  }

  fn build_key_params(&self) -> RichTextParams {
    let base = RichTextParams::from_action_map(&Self::action_map(), "mods.");

    let mut key_actions = base.key_actions;
    let aliases: &[(&str, &str)] = &[
      ("settings.focus_game", "mods.focus_game"),
      ("settings.screensaver", "mods.focus_screensaver"),
      ("settings.focus_up", "mods.focus_up"),
      ("settings.focus_down", "mods.focus_down"),
      ("settings.confirm", "mods.confirm"),
      ("settings.back", "mods.back"),
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
    positions: &ModsLayout,
    i18n: &I18nService,
  ) {
    let title = i18n.get_runtime_text("mods", "mods.title");
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.title_x,
        y: positions.title_y,
        text: format!("f%<fg:bright_magenta>{}</fg>", title),
        bold: true,
        ..Default::default()
      },
    );
    let menu_items = self.menu_items(i18n);
    for (i, item) in menu_items.iter().enumerate() {
      render.draw_host_text(
        canvas,
        &DrawTextParams {
          x: positions.menu_item_rects[i].x,
          y: positions.menu_item_rects[i].y,
          text: item.clone(),
          ..Default::default()
        },
      );
    }
    let params = self.build_key_params();
    let hint = format!(
      "f%<fg:rgb(85,87,83)>{}  {}  {}  {}</fg>",
      i18n.get_runtime_text("mods", "mods.action.focus"),
      i18n.get_runtime_text("mods", "mods.action.select"),
      i18n.get_runtime_text("mods", "mods.action.confirm"),
      i18n.get_runtime_text("mods", "mods.action.back"),
    );
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: positions.hint_x,
        y: positions.hint_y,
        text: hint,
        params: Some(params),
        ..Default::default()
      },
    );
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use image::{Rgb, RgbImage};
  use std::time::{SystemTime, UNIX_EPOCH};

  fn temp_dir(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after Unix epoch")
      .as_nanos();
    std::env::temp_dir().join(format!("tui-game-{label}-{}-{nonce}", std::process::id()))
  }

  fn write_solid_png(path: &std::path::Path, color: Rgb<u8>) {
    RgbImage::from_pixel(8, 8, color)
      .save(path)
      .expect("test image should be saved");
  }

  fn params(image_path: &std::path::Path) -> ImageConvertParams {
    ImageConvertParams {
      image_path: image_path.to_string_lossy().into_owned(),
      output_width: Some(2),
      output_height: Some(2),
      ..Default::default()
    }
  }

  #[test]
  fn package_image_cache_reuses_until_snapshot_changes() {
    let dir = temp_dir("package-image-cache");
    std::fs::create_dir_all(&dir).expect("test directory should be created");
    let image_path = dir.join("icon.png");
    write_solid_png(&image_path, Rgb([255, 0, 0]));

    let mut cache = PackageImageCache::default();
    let mut image_service = ImageService::new(None);
    let original = cache
      .get_or_convert(1, &mut image_service, params(&image_path))
      .expect("first conversion should succeed");

    write_solid_png(&image_path, Rgb([0, 0, 255]));
    let same_revision = cache
      .get_or_convert(1, &mut image_service, params(&image_path))
      .expect("cached conversion should remain available");
    assert_eq!(same_revision, original);

    let refreshed = cache
      .get_or_convert(2, &mut image_service, params(&image_path))
      .expect("conversion after snapshot update should succeed");
    assert_ne!(refreshed, original);

    std::fs::remove_dir_all(dir).expect("test directory should be removed");
  }

  #[test]
  fn package_image_cache_keeps_half_and_mix_block_outputs_separate() {
    let dir = temp_dir("package-image-modes");
    std::fs::create_dir_all(&dir).expect("test directory should be created");
    let image_path = dir.join("icon.png");
    write_solid_png(&image_path, Rgb([100, 140, 180]));

    let mut cache = PackageImageCache::default();
    let mut image_service = ImageService::new(None);
    let half_block = cache
      .get_or_convert(1, &mut image_service, params(&image_path))
      .expect("half-block conversion should succeed");
    let mix_block = cache
      .get_or_convert(
        1,
        &mut image_service,
        ImageConvertParams {
          mode: package_image_convert_mode(PackageImageMode::MixBlock),
          ..params(&image_path)
        },
      )
      .expect("mix-block conversion should succeed");

    assert_ne!(half_block, mix_block);
    assert!(half_block.contains('▅'));
    assert!(mix_block.contains('▀'));
    std::fs::remove_dir_all(dir).expect("test directory should be removed");
  }

  #[test]
  fn package_image_cache_retries_missing_asset_after_snapshot_changes() {
    let dir = temp_dir("package-missing-image");
    std::fs::create_dir_all(&dir).expect("test directory should be created");
    let image_path = dir.join("icon.png");
    let mut cache = PackageImageCache::default();
    let mut image_service = ImageService::new(None);

    assert!(
      cache
        .get_or_convert(1, &mut image_service, params(&image_path))
        .is_none()
    );
    write_solid_png(&image_path, Rgb([80, 120, 160]));
    assert!(
      cache
        .get_or_convert(1, &mut image_service, params(&image_path))
        .is_none()
    );
    assert!(
      cache
        .get_or_convert(2, &mut image_service, params(&image_path))
        .is_some()
    );

    std::fs::remove_dir_all(dir).expect("test directory should be removed");
  }
}
