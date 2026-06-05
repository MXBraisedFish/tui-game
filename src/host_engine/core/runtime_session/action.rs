#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeAction {
  Cancel,
  RequestStop,
  CloseOverlay,
  // 临时测试动作，后续删除
  PushDebugOverlay,
  PopDebugOverlay,
}
