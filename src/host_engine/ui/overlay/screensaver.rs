use crate::host_engine::services::{
  CanvasService, DrawTextParams, I18nService, LayoutService, PackageListEntry, RenderService,
  RichTextParams, RuntimeObjectPool, RuntimeObjectPoolOwner, UiObjectPool, UiObjectPoolOwner,
};

pub struct ScreensaverOverlayUi {
  objects: UiObjectPool,
  runtime_objects: RuntimeObjectPool,
  name: String,
  params: RichTextParams,
}

impl ScreensaverOverlayUi {
  pub fn init() -> Self {
    Self {
      objects: UiObjectPool::new(),
      runtime_objects: RuntimeObjectPool::new(),
      name: String::new(),
      params: RichTextParams::default(),
    }
  }

  pub fn start(&mut self, entry: &PackageListEntry) {
    self.name = entry.screensaver_name.clone();
    self.params = RichTextParams::from_key_actions(&entry.key_actions);
  }

  pub fn render(
    &mut self,
    render: &mut RenderService,
    canvas: &mut CanvasService,
    layout: &LayoutService,
    _i18n: &I18nService,
  ) {
    let size = layout.physical_size();
    let width = layout.get_text_width(&self.name, Some(&self.params));
    render.draw_host_text(
      canvas,
      &DrawTextParams {
        x: layout.resolve_host_x(LayoutService::ALIGN_CENTER, width, 0),
        y: size.height / 2,
        text: self.name.clone(),
        params: Some(self.params.clone()),
        bold: true,
        ..Default::default()
      },
    );
  }
}

impl UiObjectPoolOwner for ScreensaverOverlayUi {
  fn objects(&self) -> &UiObjectPool {
    &self.objects
  }
  fn objects_mut(&mut self) -> &mut UiObjectPool {
    &mut self.objects
  }
}

impl RuntimeObjectPoolOwner for ScreensaverOverlayUi {
  fn runtime_objects(&self) -> &RuntimeObjectPool {
    &self.runtime_objects
  }
  fn runtime_objects_mut(&mut self) -> &mut RuntimeObjectPool {
    &mut self.runtime_objects
  }
}
