// 帧调度器结构体
pub struct FrameScheduler {
  current_frame: u64, // 当前帧序号
}

// 帧调度器实现块
impl FrameScheduler {
  pub fn new() -> Self {
    Self { current_frame: 0 }
  }

  // 起始帧计算
  pub fn begin_frame(&mut self) -> u64 {
    // 当前帧序号+1
    self.current_frame += 1;

    // 返回当前帧信息
    self.current_frame
  }

  // 帧率限制
  pub fn wait_for_target_fps() {
    // TODO(runtime): move target-FPS sleeping from EngineClock to FrameScheduler.
  }
}
