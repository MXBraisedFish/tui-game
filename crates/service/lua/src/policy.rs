//! Session memory limits, callback instruction budgets, and draw-command limits.

use std::time::Duration;

/// The callback category selecting the applicable execution budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaBudgetKind {
  /// The execution budget used for the load callback phase.
  Load,
  /// The execution budget used for the init callback phase.
  Init,
  /// The execution budget used for the handle event callback phase.
  HandleEvent,
  /// The execution budget used for the update callback phase.
  Update,
  /// The execution budget used for the update frame callback phase.
  UpdateFrame,
  /// The execution budget used for the render callback phase.
  Render,
  /// The execution budget used for the save callback phase.
  Save,
}

/// Warning and hard duration thresholds plus the instruction ceiling for one callback class.
///
/// # Fields
///
/// * `warn_duration` - The callback duration at which a warning is recorded.
/// * `hard_duration` - The callback duration at which execution is interrupted.
/// * `max_instructions` - The instruction ceiling enforced by VM hooks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LuaExecutionBudget {
  /// The callback duration at which a warning is recorded.
  pub warn_duration: Duration,
  /// The callback duration at which execution is interrupted.
  pub hard_duration: Duration,
  /// The instruction ceiling enforced by VM hooks.
  pub max_instructions: u64,
}

/// VM memory, source, save, hook, and callback limits shared as configuration between isolated
/// sessions.
///
/// # Fields
///
/// * `memory_limit_bytes` - The memory ceiling for one isolated Lua VM in bytes.
/// * `source_limit_bytes` - The maximum script-source size in bytes.
/// * `save_limit_bytes` - The maximum serialized save-data size in bytes.
/// * `save_max_depth` - The maximum nesting depth accepted in game saves.
/// * `hook_interval` - The number of Lua instructions between supervision hook calls.
#[derive(Clone, Debug)]
pub struct LuaPolicy {
  /// The memory ceiling for one isolated Lua VM in bytes.
  pub memory_limit_bytes: usize,
  /// The maximum script-source size in bytes.
  pub source_limit_bytes: usize,
  /// The maximum serialized save-data size in bytes.
  pub save_limit_bytes: usize,
  /// The maximum nesting depth accepted in game saves.
  pub save_max_depth: usize,
  /// The number of Lua instructions between supervision hook calls.
  pub hook_interval: u32,
  load_budget: LuaExecutionBudget,
  callback_budget: LuaExecutionBudget,
}

impl LuaPolicy {
  /// Create the default Lua memory, source, save, and callback execution limits.
  pub fn balanced() -> Self {
    Self {
      memory_limit_bytes: 32 * 1024 * 1024,
      source_limit_bytes: 1024 * 1024,
      save_limit_bytes: 1024 * 1024,
      save_max_depth: 32,
      hook_interval: 1_000,
      load_budget: LuaExecutionBudget {
        warn_duration: Duration::from_millis(50),
        hard_duration: Duration::from_millis(100),
        max_instructions: 1_000_000,
      },
      callback_budget: LuaExecutionBudget {
        warn_duration: Duration::from_millis(20),
        hard_duration: Duration::from_millis(75),
        max_instructions: 200_000,
      },
    }
  }

  /// Return the execution budget assigned to the requested Lua callback category.
  pub fn budget(&self, kind: LuaBudgetKind) -> LuaExecutionBudget {
    match kind {
      LuaBudgetKind::Load | LuaBudgetKind::Init | LuaBudgetKind::Save => self.load_budget,
      LuaBudgetKind::HandleEvent
      | LuaBudgetKind::Update
      | LuaBudgetKind::UpdateFrame
      | LuaBudgetKind::Render => self.callback_budget,
    }
  }

  /// Return the current validate.
  ///
  /// # Errors
  ///
  /// Return an error for zero limits or hook intervals, non-positive callback budgets, or warning
  /// durations that do not precede hard limits.
  pub(super) fn validate(&self) -> Result<(), String> {
    if self.memory_limit_bytes == 0 {
      return Err("Lua memory limit must be greater than zero".to_string());
    }
    if self.source_limit_bytes == 0 {
      return Err("Lua source limit must be greater than zero".to_string());
    }
    if self.save_limit_bytes == 0 {
      return Err("Lua save limit must be greater than zero".to_string());
    }
    if self.save_max_depth == 0 {
      return Err("Lua save depth must be greater than zero".to_string());
    }
    if self.hook_interval == 0 {
      return Err("Lua hook interval must be greater than zero".to_string());
    }
    for (name, budget) in [
      ("load", self.load_budget),
      ("callback", self.callback_budget),
    ] {
      if budget.warn_duration.is_zero() {
        return Err(format!("{name} warning duration must be greater than zero"));
      }
      if budget.hard_duration.is_zero() {
        return Err(format!("{name} hard duration must be greater than zero"));
      }
      if budget.warn_duration >= budget.hard_duration {
        return Err(format!(
          "{name} warning duration must be lower than its hard duration"
        ));
      }
      if budget.max_instructions == 0 {
        return Err(format!(
          "{name} instruction budget must be greater than zero"
        ));
      }
    }
    Ok(())
  }
}

impl Default for LuaPolicy {
  fn default() -> Self {
    Self::balanced()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn balanced_policy_matches_runtime_limits() {
    let policy = LuaPolicy::balanced();
    assert_eq!(policy.memory_limit_bytes, 32 * 1024 * 1024);
    assert_eq!(policy.hook_interval, 1_000);
    assert_eq!(
      policy.budget(LuaBudgetKind::Render),
      LuaExecutionBudget {
        warn_duration: Duration::from_millis(20),
        hard_duration: Duration::from_millis(75),
        max_instructions: 200_000,
      }
    );
    assert_eq!(
      policy.budget(LuaBudgetKind::Save),
      LuaExecutionBudget {
        warn_duration: Duration::from_millis(50),
        hard_duration: Duration::from_millis(100),
        max_instructions: 1_000_000,
      }
    );
  }

  #[test]
  fn invalid_policy_cannot_disable_runtime_limits() {
    let mut policy = LuaPolicy::balanced();
    policy.hook_interval = 0;
    assert!(policy.validate().is_err());

    let mut policy = LuaPolicy::balanced();
    policy.memory_limit_bytes = 0;
    assert!(policy.validate().is_err());

    let mut policy = LuaPolicy::balanced();
    policy.callback_budget.warn_duration = policy.callback_budget.hard_duration;
    assert!(policy.validate().is_err());
  }
}
