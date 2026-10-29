//! Session-owned Markdown documents backed by private embedded scrollboxes.
use super::*;
use tg_service_widget::{
  MarkdownEvent, MarkdownLayout, MarkdownService, MarkdownViewId, MarkdownViewOptions,
  is_valid_link_target,
};
/// A Markdown source, native document and private scrolling surface.
pub(crate) struct LuaMarkdownBinding {
  pub scrollbox: u64,
  pub native: MarkdownViewId,
  pub options: MarkdownViewOptions,
  pub allow_overflow: bool,
  pub layout: std::sync::Arc<MarkdownLayout>,
  pub layouts: HashMap<u16, std::sync::Arc<MarkdownLayout>>,
}
impl LuaObjectPool {
  pub(crate) fn prepare_markdown(
    &mut self,
    id: u64,
    external: tg_service_layout::Size,
  ) -> tg_service_layout::Size {
    let binding = self.markdowns.get_mut(&id).unwrap();
    let scroll = &mut self.scrollboxes.get_mut(&binding.scrollbox).unwrap().widget;
    let mut width = scroll
      .layout_for(tg_service_layout::Size::default())
      .content
      .width;
    let mut extent = external;
    for _ in 0..4 {
      binding.layout = binding
        .layouts
        .entry(width)
        .or_insert_with(|| {
          std::sync::Arc::new(MarkdownService::new().layout(
            &binding.options,
            width,
            binding.allow_overflow,
            &tg_service_code_highlight::CodeHighlightService::new(),
          ))
        })
        .clone();
      extent = tg_service_layout::Size {
        width: external.width.max(binding.layout.size().width),
        height: external.height.max(binding.layout.size().height),
      };
      let next = scroll.layout_for(extent).content.width;
      if next == width || next == 0 {
        break;
      }
      width = next;
    }
    extent
  }
  pub(crate) fn translate_markdown_event(&self, event: MarkdownEvent) -> Option<LuaEventData> {
    let MarkdownEvent::LinkClicked { id, href, text } = event;
    if !is_valid_link_target(&href) {
      return None;
    }
    self
      .markdowns
      .iter()
      .find(|(_, binding)| binding.native == id)
      .map(|(id, _)| {
        LuaEventData::Markdown(crate::LuaMarkdownEvent {
          id: *id,
          href,
          text,
          scroll_offset: None,
        })
      })
  }
  pub(crate) fn delete_markdown(&mut self, id: u64) -> bool {
    let Some(binding) = self.markdowns.remove(&id) else {
      return false;
    };
    self.delete_scrollbox(binding.scrollbox);
    MarkdownService::new().remove(&mut self.ui, binding.native);
    true
  }
}
