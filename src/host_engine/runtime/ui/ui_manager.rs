//! Rust UI manager and page trait definitions.

use std::collections::HashMap;
use std::sync::Arc;

use crate::host_engine::boot::preload::init_environment::TerminalSize;
use crate::host_engine::keybind::keybind_manager::KeybindManager;
use crate::host_engine::package::package_manager::PackageManager;
use crate::host_engine::runtime::ui_page::page_key::UiPageKey;
use crate::host_engine::runtime::ui_state::needed_size_state::NeededSizeMode;
use crate::host_engine::storage::profile_store::ProfileStore;
use crate::host_engine::theme::ThemeManager;

use super::{Canvas, HostI18n};

pub type UiResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct UiManager {
    active_page: UiPageKey,
    pages: HashMap<UiPageKey, Box<dyn UiPage>>,
    context: UiContext,
}

pub struct UiContext {
    /// Current terminal size; pages may update layout decisions from this value.
    pub terminal_size: TerminalSize,
    /// Read-only host i18n snapshot for Rust UI rendering.
    pub i18n: Arc<HostI18n>,
    /// Shared theme snapshot. Pages should not mutate theme state directly.
    pub themes: Arc<ThemeManager>,
    /// Shared keybind resolver. Keybind changes must go through KeybindManager APIs.
    pub keybinds: Arc<KeybindManager>,
    /// Shared package registry. Package enable/disable changes must go through PackageManager APIs.
    pub packages: Arc<PackageManager>,
    /// Shared profile store snapshot. Pages may call explicit save methods for simple preferences.
    pub profiles: Arc<ProfileStore>,
    /// Current page key hints resolved from official_ui actions and user keybinds.
    pub action_hints: HashMap<String, String>,
    /// Package name to show in the mod security warning dialog.
    pub mod_warning_package_name: String,
    /// Current mode for the needed-size warning page.
    pub needed_size_mode: NeededSizeMode,
}

pub trait UiPage {
    fn page_key(&self) -> UiPageKey;
    fn handle_event(&mut self, event: &UiEvent, ctx: &mut UiContext) -> UiResult<()>;
    fn render(&self, canvas: &mut Canvas, ctx: &UiContext) -> UiResult<()>;
    fn take_navigation(&mut self) -> Option<UiNavigation> {
        None
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiEvent {
    Action { name: String, status: String },
    Key { name: String, status: String },
    Resize { width: u16, height: u16 },
    Tick { dt_ms: u64 },
    FocusGained,
    FocusLost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNavigation {
    Page(UiPageKey),
    Exit,
}

impl UiEvent {
    pub fn action(name: impl Into<String>) -> Self {
        Self::Action {
            name: name.into(),
            status: "pressed".to_string(),
        }
    }

    pub fn key(name: impl Into<String>) -> Self {
        Self::Key {
            name: name.into(),
            status: "pressed".to_string(),
        }
    }
}

impl UiManager {
    pub fn new(active_page: UiPageKey, context: UiContext) -> Self {
        Self {
            active_page,
            pages: HashMap::new(),
            context,
        }
    }

    pub fn register_page(&mut self, page: Box<dyn UiPage>) {
        self.pages.insert(page.page_key(), page);
    }

    pub fn set_active_page(&mut self, page_key: UiPageKey) {
        self.active_page = page_key;
    }

    pub fn navigate_to(&mut self, page_key: UiPageKey) -> UiResult<()> {
        if !self.pages.contains_key(&page_key) {
            return Err(format!("Rust UI page is not registered: {}", page_key.as_str()).into());
        }
        self.active_page = page_key;
        Ok(())
    }

    pub fn navigate_back(&mut self) -> UiResult<()> {
        self.navigate_to(UiPageKey::Home)
    }

    pub fn active_page(&self) -> UiPageKey {
        self.active_page
    }

    pub fn set_terminal_size(&mut self, terminal_size: TerminalSize) {
        self.context.terminal_size = terminal_size;
    }

    pub fn set_action_hints(&mut self, action_hints: HashMap<String, String>) {
        self.context.action_hints = action_hints;
    }

    pub fn set_mod_warning_package_name(&mut self, name: String) {
        self.context.mod_warning_package_name = name;
    }

    pub fn set_needed_size_mode(&mut self, mode: NeededSizeMode) {
        self.context.needed_size_mode = mode;
    }

    pub fn handle_event(&mut self, event: &UiEvent) -> UiResult<()> {
        if let UiEvent::Resize { width, height } = event {
            self.context.terminal_size = TerminalSize {
                width: *width,
                height: *height,
            };
        }
        if let Some(page) = self.pages.get_mut(&self.active_page) {
            page.handle_event(event, &mut self.context)?;
        }
        Ok(())
    }

    pub fn render(&self, canvas: &mut Canvas) -> UiResult<()> {
        if let Some(page) = self.pages.get(&self.active_page) {
            page.render(canvas, &self.context)?;
        }
        Ok(())
    }

    pub fn take_navigation(&mut self) -> Option<UiNavigation> {
        self.pages
            .get_mut(&self.active_page)
            .and_then(|page| page.take_navigation())
    }
}
