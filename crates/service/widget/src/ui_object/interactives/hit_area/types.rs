//! Identifiers, configuration, states, and events shared by this module.

use tg_core_input::MouseButton;

/// The identity of hit area within its owning pool or session.
///
/// # Fields
///
/// * `0` - The wrapped u64 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HitAreaId(
  /// The wrapped u64 value.
  pub u64,
);

/// Configuration values controlling hit area behavior.
///
/// # Fields
///
/// * `hover_move` - The hover move.
/// * `drag` - The drag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HitAreaOptions {
  /// The hover move.
  pub hover_move: bool,
  /// The drag.
  pub drag: bool,
}

/// A hit area event payload queued for its owning consumer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HitAreaEvent {
  /// A hover enter notification delivered to the owning consumer.
  HoverEnter {
    /// The identifier of the owned object.
    id: HitAreaId,
    /// The horizontal coordinate in terminal cells.
    x: u16,
    /// The vertical coordinate in terminal cells.
    y: u16,
  },
  /// A hover move notification delivered to the owning consumer.
  HoverMove {
    /// The identifier of the owned object.
    id: HitAreaId,
    /// The horizontal coordinate in terminal cells.
    x: u16,
    /// The vertical coordinate in terminal cells.
    y: u16,
  },
  /// A hover leave notification delivered to the owning consumer.
  HoverLeave {
    /// The identifier of the owned object.
    id: HitAreaId,
    /// The horizontal coordinate in terminal cells.
    x: u16,
    /// The vertical coordinate in terminal cells.
    y: u16,
  },
  /// A press notification delivered to the owning consumer.
  Press {
    /// The identifier of the owned object.
    id: HitAreaId,
    /// The mouse button to query.
    button: MouseButton,
    /// The horizontal coordinate in terminal cells.
    x: u16,
    /// The vertical coordinate in terminal cells.
    y: u16,
  },
  /// A release notification delivered to the owning consumer.
  Release {
    /// The identifier of the owned object.
    id: HitAreaId,
    /// The mouse button to query.
    button: MouseButton,
    /// The horizontal coordinate in terminal cells.
    x: u16,
    /// The vertical coordinate in terminal cells.
    y: u16,
  },
  /// A click notification delivered to the owning consumer.
  Click {
    /// The identifier of the owned object.
    id: HitAreaId,
    /// The mouse button to query.
    button: MouseButton,
    /// The horizontal coordinate in terminal cells.
    x: u16,
    /// The vertical coordinate in terminal cells.
    y: u16,
  },
  /// A drag notification delivered to the owning consumer.
  Drag {
    /// The identifier of the owned object.
    id: HitAreaId,
    /// The mouse button to query.
    button: MouseButton,
    /// The horizontal coordinate in terminal cells.
    x: u16,
    /// The vertical coordinate in terminal cells.
    y: u16,
    /// The dx.
    dx: i32,
    /// The dy.
    dy: i32,
  },
}
