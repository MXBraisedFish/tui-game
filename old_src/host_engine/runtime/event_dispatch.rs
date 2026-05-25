//! Runtime event routing helpers.

use crate::host_engine::boot::preload::init_environment::HostInputEvent;
use crate::host_engine::keybind::binding::Key;
use crate::host_engine::keybind::keybind_manager::KeybindManager;
use crate::host_engine::package::package_id::PackageKind;

/// Engine-level events after raw terminal input has been normalized.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum EngineEvent {
    Key { key: String, status: String },
    KeyPressed(String),
    KeyReleased(String),
    Resize(u16, u16),
    Tick(u64),
    PackagesRefreshed(PackageKind),
    FocusGained,
    FocusLost,
    Shutdown,
}

impl From<HostInputEvent> for EngineEvent {
    fn from(event: HostInputEvent) -> Self {
        match event {
            HostInputEvent::Key { key, status } => Self::Key { key, status },
            HostInputEvent::Resize(resize) => Self::Resize(resize.width, resize.height),
            HostInputEvent::FocusGained => Self::FocusGained,
            HostInputEvent::FocusLost => Self::FocusLost,
            HostInputEvent::ExitRequested => Self::Shutdown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_engine::boot::preload::init_environment::ResizeEvent;

    #[test]
    fn host_input_events_convert_to_engine_events() {
        assert_eq!(
            EngineEvent::from(HostInputEvent::Resize(ResizeEvent {
                width: 120,
                height: 30
            })),
            EngineEvent::Resize(120, 30)
        );
        assert_eq!(
            EngineEvent::from(HostInputEvent::ExitRequested),
            EngineEvent::Shutdown
        );
    }
}

/// Host-global actions that must be handled before UI/game event delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlobalRuntimeAction {
    ToggleScreensaver,
    ToggleBoss,
    ForceStopGame,
}

/// Event dispatcher facade.
///
/// The current runtime still owns most state in `event_loop.rs`; this dispatcher
/// centralizes key-to-global-action routing so the loop no longer hard-codes F2/F3/F4.
pub struct EventDispatcher<'a> {
    keybinds: &'a KeybindManager,
}

impl<'a> EventDispatcher<'a> {
    pub fn new(keybinds: &'a KeybindManager) -> Self {
        Self { keybinds }
    }

    pub fn global_action_for_key(&self, raw_key: &str) -> Option<GlobalRuntimeAction> {
        let key = Key::from_string(raw_key)?;
        self.keybinds.global_action_for_key(&key)
    }
}
