//! Screenshot recording page state, user commands, and terminal-cell presentation.

pub mod fonts_settings;
mod media_list;
pub mod recording_list;
pub mod recording_settings;
pub mod screenshot_list;

#[allow(clippy::module_inception)]
pub mod screenshot_recording;
pub mod screenshot_settings;

pub use media_list::{MediaListNotice, MediaRenameError};
pub use recording_list::{RecordingListCommand, RecordingListUi};
pub use recording_settings::{RecordingSettingsCommand, RecordingSettingsUi};
pub use screenshot_list::{ScreenshotListCommand, ScreenshotListUi};
pub use screenshot_recording::{ScreenshotRecordingCommand, ScreenshotRecordingUi};
pub use screenshot_settings::{ScreenshotSettingsCommand, ScreenshotSettingsUi};
