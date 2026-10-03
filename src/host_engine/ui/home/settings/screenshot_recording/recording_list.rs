//! Recording list page state, user commands, and terminal-cell presentation.

use crate::host_engine::services::ActionMapEntry;

use super::media_list::{MediaListCommand, MediaListSpec, MediaListUi, actions};

/// The shared type used for recording list command.
pub type RecordingListCommand = MediaListCommand;
/// The shared type used for recording list ui.
pub type RecordingListUi = MediaListUi<RecordingListSpec>;

/// The recording list spec representation used by this module.
pub struct RecordingListSpec;

impl MediaListSpec for RecordingListSpec {
  const NS: &'static str = "recording_list";
  const SUPPORTS_DURATION: bool = true;

  fn action_map() -> Vec<ActionMapEntry> {
    actions(&[
      ("recording_list.scroll_up", "w"),
      ("recording_list.scroll_down", "s"),
      ("recording_list.scroll_left", "a"),
      ("recording_list.del", "d"),
      ("recording_list.scroll_right", "d"),
      ("recording_list.focus_up", "up"),
      ("recording_list.focus_down", "down"),
      ("recording_list.back", "esc"),
      ("recording_list.search", "c"),
      ("recording_list.order", "z"),
      ("recording_list.sort", "x"),
      ("recording_list.modify", "f"),
      ("recording_list.warning_yes", "y"),
      ("recording_list.warning_no", "n"),
      ("recording_list.switch", "tab"),
      ("recording_list.play_pause", "space"),
      ("recording_list.skip_forward", "right"),
      ("recording_list.rewind", "left"),
      ("recording_list.volume_down", "-"),
      ("recording_list.volume_up", "="),
      ("recording_list.zoom", "z"),
      ("recording_list.export", "1"),
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
      "action.play",
      "action.skip",
      "warning.sound",
      "action.switch",
      "action.zoom.in",
      "action.export",
    ]
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::translate_action_map;

  #[test]
  fn info_hints_use_existing_translation_keys_in_display_order() {
    assert_eq!(
      RecordingListSpec::right_hint_keys(),
      &[
        "action.scroll.info",
        "action.back",
        "action.play",
        "action.skip",
        "warning.sound",
        "action.switch",
        "action.zoom.in",
        "action.export",
      ]
    );
    assert!(
      RecordingListSpec::action_map()
        .iter()
        .any(|entry| entry.action == "recording_list.export" && entry.keys == [["1"]])
    );
    let actions = RecordingListSpec::action_map();
    assert!(
      actions
        .iter()
        .any(|entry| entry.action == "recording_list.volume_down" && entry.keys == [["-"]])
    );
    assert!(
      actions
        .iter()
        .any(|entry| entry.action == "recording_list.volume_up" && entry.keys == [["="]])
    );
    let delete = actions
      .iter()
      .position(|entry| entry.action == "recording_list.del")
      .unwrap();
    let scroll_right = actions
      .iter()
      .position(|entry| entry.action == "recording_list.scroll_right")
      .unwrap();
    assert!(delete < scroll_right);
  }

  #[test]
  fn every_recording_list_action_uses_valid_input_tokens() {
    translate_action_map(&RecordingListSpec::action_map())
      .expect("recording list action map should be translatable");
  }
}
