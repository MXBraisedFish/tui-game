#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeAction {
  RequestStop,
  CloseOverlay,
}
