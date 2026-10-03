//! Page-stack navigation with retained state for each program page.

/// Retained program pages with the current navigation path.
///
/// # Fields
///
/// * `path` - The ordered path retained by this owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiTreeState {
  /// The ordered path retained by this owner.
  pub path: Vec<UiNodeState>,
}

/// The retained state of UI node.
///
/// # Fields
///
/// * `kind` - The UI node kind carried by this UI node state.
/// * `logic` - The logic.
/// * `render` - The retained rendering snapshot of this program page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNodeState {
  /// The UI node kind carried by this UI node state.
  pub kind: UiNodeKind,
  /// The logic.
  pub logic: UiNodeLogicState,
  /// The retained rendering snapshot of this program page.
  pub render: UiNodeRenderState,
}

/// The page category selected by program navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiNodeKind {
  /// The home setting for UI node kind.
  Home,
  /// A request to settings.
  Settings,
  /// The key bindings setting for UI node kind.
  KeyBindings,
  /// The global key bindings setting for UI node kind.
  GlobalKeyBindings,
  /// The game key bindings setting for UI node kind.
  GameKeyBindings,
  /// The display settings setting for UI node kind.
  DisplaySettings,
  /// The toolbar custom setting for UI node kind.
  ToolbarCustom,
  /// The screensaver list setting for UI node kind.
  ScreensaverList,
  /// The screenshot recording setting for UI node kind.
  ScreenshotRecording,
  /// The screenshot settings setting for UI node kind.
  ScreenshotSettings,
  /// The recording settings setting for UI node kind.
  RecordingSettings,
  /// The screenshot list setting for UI node kind.
  ScreenshotList,
  /// The recording list setting for UI node kind.
  RecordingList,
  /// The security settings setting for UI node kind.
  SecuritySettings,
  /// The language select setting for UI node kind.
  LanguageSelect,
  /// The storage management setting for UI node kind.
  StorageManagement,
  /// The storage management clear setting for UI node kind.
  StorageManagementClear,
  /// The storage management export setting for UI node kind.
  StorageManagementExport,
  /// The storage management view setting for UI node kind.
  StorageManagementView,
  /// The mods setting for UI node kind.
  Mods,
  /// The terminal check setting for UI node kind.
  TerminalCheck,
  /// The game list setting for UI node kind.
  GameList,
  /// The game package setting for UI node kind.
  GamePackage,
  /// The screensaver package setting for UI node kind.
  ScreensaverPackage,
  /// The input demo setting for UI node kind.
  InputDemo,
  /// The exit warning setting for UI node kind.
  ExitWarning,
}

/// The retained interaction state of a program page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNodeLogicState;

/// The page rendering snapshot selected for the current frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNodeRenderState;

impl UiNodeState {
  /// Create retained navigation state for the home program page.
  pub fn home() -> Self {
    Self {
      kind: UiNodeKind::Home,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the settings program page.
  pub fn settings() -> Self {
    Self {
      kind: UiNodeKind::Settings,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the key bindings program page.
  pub fn key_bindings() -> Self {
    Self {
      kind: UiNodeKind::KeyBindings,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the global key bindings program page.
  pub fn global_key_bindings() -> Self {
    Self {
      kind: UiNodeKind::GlobalKeyBindings,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the game key bindings program page.
  pub fn game_key_bindings() -> Self {
    Self {
      kind: UiNodeKind::GameKeyBindings,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the display settings program page.
  pub fn display_settings() -> Self {
    Self {
      kind: UiNodeKind::DisplaySettings,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the toolbar custom program page.
  pub fn toolbar_custom() -> Self {
    Self {
      kind: UiNodeKind::ToolbarCustom,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the screensaver list program page.
  pub fn screensaver_list() -> Self {
    Self {
      kind: UiNodeKind::ScreensaverList,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the screenshot recording program page.
  pub fn screenshot_recording() -> Self {
    Self {
      kind: UiNodeKind::ScreenshotRecording,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the screenshot settings program page.
  pub fn screenshot_settings() -> Self {
    Self {
      kind: UiNodeKind::ScreenshotSettings,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the recording settings program page.
  pub fn recording_settings() -> Self {
    Self {
      kind: UiNodeKind::RecordingSettings,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the screenshot list program page.
  pub fn screenshot_list() -> Self {
    Self {
      kind: UiNodeKind::ScreenshotList,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the recording list program page.
  pub fn recording_list() -> Self {
    Self {
      kind: UiNodeKind::RecordingList,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the security settings program page.
  pub fn security_settings() -> Self {
    Self {
      kind: UiNodeKind::SecuritySettings,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the mods program page.
  pub fn mods() -> Self {
    Self {
      kind: UiNodeKind::Mods,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the terminal check program page.
  pub fn terminal_check() -> Self {
    Self {
      kind: UiNodeKind::TerminalCheck,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the game list program page.
  pub fn game_list() -> Self {
    Self {
      kind: UiNodeKind::GameList,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the language select program page.
  pub fn language_select() -> Self {
    Self {
      kind: UiNodeKind::LanguageSelect,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the storage management program page.
  pub fn storage_management() -> Self {
    Self {
      kind: UiNodeKind::StorageManagement,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the storage management clear program page.
  pub fn storage_management_clear() -> Self {
    Self {
      kind: UiNodeKind::StorageManagementClear,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the storage management export program page.
  pub fn storage_management_export() -> Self {
    Self {
      kind: UiNodeKind::StorageManagementExport,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the storage management view program page.
  pub fn storage_management_view() -> Self {
    Self {
      kind: UiNodeKind::StorageManagementView,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the game package program page.
  pub fn game_package() -> Self {
    Self {
      kind: UiNodeKind::GamePackage,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the screensaver package program page.
  pub fn screensaver_package() -> Self {
    Self {
      kind: UiNodeKind::ScreensaverPackage,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the input demo program page.
  pub fn input_demo() -> Self {
    Self {
      kind: UiNodeKind::InputDemo,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }

  /// Create retained navigation state for the exit warning program page.
  pub fn exit_warning() -> Self {
    Self {
      kind: UiNodeKind::ExitWarning,
      logic: UiNodeLogicState,
      render: UiNodeRenderState,
    }
  }
}

impl UiTreeState {
  /// Create a UI tree state with its initial state.
  pub fn new() -> Self {
    Self {
      path: vec![UiNodeState::home()],
    }
  }

  /// Return the current path.
  pub fn path(&self) -> &[UiNodeState] {
    &self.path
  }

  /// Return the current current.
  pub fn current(&self) -> Option<&UiNodeState> {
    self.path.last()
  }

  /// Select the requested program page while retaining page state in the navigation tree.
  pub fn enter(&mut self, node: UiNodeState) {
    self.path.push(node);
  }

  /// Return to the previous retained program page when navigation is not at its root.
  pub fn back(&mut self) -> Option<UiNodeState> {
    if self.path.len() <= 1 {
      return None;
    }

    self.path.pop()
  }
}
