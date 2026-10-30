//! Session-owned cell shaders and their frame-local application requests.

use mlua::Function;

use crate::LuaDrawTarget;

/// The composition stage at which a cell shader runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaEffectScope {
  /// Process the developer canvas after its slices have been composed.
  Canvas,
  /// Process one surface before it is composed into its parent.
  Slice,
}

impl LuaEffectScope {
  pub(crate) fn as_str(self) -> &'static str {
    match self {
      Self::Canvas => "canvas",
      Self::Slice => "slice",
    }
  }
}

/// A rectangular cell shader retained by one Lua session.
///
/// # Fields
///
/// * `width`, `height` - The selected terminal-cell dimensions.
/// * `process` - The function transforming each original cell.
/// * `scope` - The composition stage of the shader.
/// * `include_continuation` - Whether continuations select their original wide grapheme.
#[derive(Clone, Debug)]
pub struct LuaCellEffect {
  /// The region width in terminal columns.
  pub width: u16,
  /// The region height in terminal rows.
  pub height: u16,
  /// The function transforming each original cell.
  pub process: Function,
  /// The composition stage of the shader.
  pub scope: LuaEffectScope,
  /// Whether a selected continuation also selects its wide grapheme's start.
  pub include_continuation: bool,
}

/// A shader configuration captured at its ordered frame submission.
///
/// # Fields
///
/// * `effect` - The captured shader configuration.
/// * `target` - The selected surface for slice-scoped processing.
/// * `x`, `y` - The signed region origin in target coordinates.
#[derive(Clone, Debug)]
pub struct LuaEffectRequest {
  /// The shader configuration at submission time.
  pub effect: LuaCellEffect,
  /// The target surface, ignored for canvas scope.
  pub target: LuaDrawTarget,
  /// The signed region origin in the target's columns.
  pub x: i32,
  /// The signed region origin in the target's rows.
  pub y: i32,
}
