//! Rust UI pages.

pub mod common;
pub mod game_list;
pub mod home;
pub mod keybind_system;
pub mod mod_hub;
pub mod mod_package_list;
pub mod setting;
pub mod setting_display;
pub mod setting_keybind;
pub mod setting_language;
pub mod setting_memory;
pub mod setting_security;
pub mod storage_details;
pub mod warnings;

pub use game_list::GameListPage;
pub use home::HomePage;
pub use keybind_system::KeybindSystemPage;
pub use mod_hub::ModHubPage;
pub use mod_package_list::{ModBossListPage, ModGameListPage, ModScreensaverListPage};
pub use setting::SettingPage;
pub use setting_display::SettingDisplayPage;
pub use setting_keybind::SettingKeybindPage;
pub use setting_language::SettingLanguagePage;
pub use setting_memory::SettingMemoryPage;
pub use setting_security::SettingSecurityPage;
pub use storage_details::StorageDetailsPage;
pub use warnings::{
    WarningClearCachePage, WarningClearDataPage, WarningModPage, WarningNeededSizePage,
    WarningSecurityPage,
};
