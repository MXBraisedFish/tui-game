#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeState {
  Running,
  Stopping,
}
