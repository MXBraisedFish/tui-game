use crate::host_engine::services::ActionMapEntry;

use super::media_list::{MediaListCommand, MediaListSpec, MediaListUi, actions};

pub type RecordingListCommand = MediaListCommand;
pub type RecordingListUi = MediaListUi<RecordingListSpec>;

pub struct RecordingListSpec;

impl MediaListSpec for RecordingListSpec {
  const NS: &'static str = "recording_list";
  const SUPPORTS_DURATION: bool = true;

  fn action_map() -> Vec<ActionMapEntry> {
    actions(&[
      ("recording_list.scroll_up", "w"),
      ("recording_list.scroll_down", "s"),
      ("recording_list.scroll_left", "a"),
      ("recording_list.scroll_right", "d"),
      ("recording_list.focus_up", "up"),
      ("recording_list.focus_down", "down"),
      ("recording_list.back", "esc"),
      ("recording_list.search", "c"),
      ("recording_list.order", "z"),
      ("recording_list.sort", "x"),
      ("recording_list.modify", "f"),
      ("recording_list.del", "d"),
      ("recording_list.switch", "tab"),
      ("recording_list.play_pause", "space"),
      ("recording_list.skip_forward", "right"),
      ("recording_list.rewind", "left"),
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
      "action.switch",
      "action.zoom.in",
      "action.export",
    ]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn info_hints_use_existing_translation_keys_in_display_order() {
    assert_eq!(
      RecordingListSpec::right_hint_keys(),
      &[
        "action.scroll.info",
        "action.back",
        "action.play",
        "action.skip",
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
  }
}
