//! Screenshot list page state, user commands, and terminal-cell presentation.

use std::path::Path;

use crate::host_engine::services::ActionMapEntry;

use super::media_list::{
  MediaListCommand, MediaListSpec, MediaListUi, ScreenshotPreview, actions, load_screenshot_preview,
};

/// The shared type used for screenshot list command.
pub type ScreenshotListCommand = MediaListCommand;
/// The shared type used for screenshot list ui.
pub type ScreenshotListUi = MediaListUi<ScreenshotListSpec>;

/// The screenshot list spec representation used by this module.
pub struct ScreenshotListSpec;

impl MediaListSpec for ScreenshotListSpec {
  const NS: &'static str = "screenshot_list";
  const SUPPORTS_DURATION: bool = false;

  fn load_preview(path: &Path) -> Option<ScreenshotPreview> {
    load_screenshot_preview(path)
  }

  fn action_map() -> Vec<ActionMapEntry> {
    actions(&[
      ("screenshot_list.scroll_up", "w"),
      ("screenshot_list.scroll_down", "s"),
      ("screenshot_list.scroll_left", "a"),
      // Register D deletion first, then interpret it by panel focus; otherwise key consumption
      // hides right-scroll behavior.
      ("screenshot_list.del", "d"),
      ("screenshot_list.scroll_right", "d"),
      ("screenshot_list.focus_up", "up"),
      ("screenshot_list.focus_down", "down"),
      ("screenshot_list.back", "esc"),
      ("screenshot_list.search", "c"),
      ("screenshot_list.order", "z"),
      ("screenshot_list.sort", "x"),
      ("screenshot_list.modify", "f"),
      ("screenshot_list.warning_yes", "y"),
      ("screenshot_list.warning_no", "n"),
      ("screenshot_list.switch", "tab"),
      ("screenshot_list.copy", "1"),
      ("screenshot_list.copy_rich_text", "2"),
      ("screenshot_list.save_image", "3"),
      ("screenshot_list.all", "4"),
      ("screenshot_list.zoom", "z"),
    ])
  }

  fn left_hint_keys() -> &'static [&'static str] {
    &[
      "action.scroll.list",
      "action.select",
      "action.back",
      "action.list.search",
      "action.list.order",
      "action.list.sort",
      "action.modify",
      "action.del",
      "action.switch",
    ]
  }

  fn right_hint_keys() -> &'static [&'static str] {
    &[
      "action.scroll.info",
      "action.back",
      "action.switch",
      "action.copy",
      "action.copy_rich_text",
      "action.save_image",
      "action.all",
      "action.zoom.in",
    ]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn delete_consumes_the_shared_d_key_before_scroll_right() {
    let actions = ScreenshotListSpec::action_map();
    let delete = actions
      .iter()
      .position(|entry| entry.action == "screenshot_list.del")
      .unwrap();
    let scroll_right = actions
      .iter()
      .position(|entry| entry.action == "screenshot_list.scroll_right")
      .unwrap();
    assert!(delete < scroll_right);
  }
}
