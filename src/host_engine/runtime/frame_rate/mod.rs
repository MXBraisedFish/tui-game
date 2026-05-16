//! 运行阶段帧率控制

use std::time::{Duration, Instant};

const ROOT_UI_NORMAL_FPS: u64 = 60;
const LOW_RESOURCE_FPS: u64 = 24;
const ROOT_UI_IDLE_TIMEOUT_SECS: u64 = 60;
const OVERLAY_FPS: u64 = 24;

/// 帧率运行模式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameRateMode {
    RootUi,
    Game { afk_time_secs: u64, target_fps: u64 },
    Overlay,
}

/// 帧率控制器。
#[derive(Clone, Debug)]
pub struct FrameRateController {
    mode: FrameRateMode,
    last_input_at: Instant,
}

impl FrameRateController {
    /// 创建宿主 UI 帧率控制器。
    pub fn root_ui() -> Self {
        Self {
            mode: FrameRateMode::RootUi,
            last_input_at: Instant::now(),
        }
    }

    /// 创建游戏帧率控制器。
    ///
    /// `afk_time_secs == 0` 表示不进入低资源模式。
    pub fn game(afk_time_secs: u64, target_fps: u64) -> Self {
        Self {
            mode: FrameRateMode::Game {
                afk_time_secs,
                target_fps,
            },
            last_input_at: Instant::now(),
        }
    }

    /// 创建覆盖层帧率控制器。
    pub fn overlay() -> Self {
        Self {
            mode: FrameRateMode::Overlay,
            last_input_at: Instant::now(),
        }
    }

    /// 标记用户输入，恢复正常帧率。
    pub fn mark_input(&mut self) {
        self.last_input_at = Instant::now();
    }

    /// 当前帧间隔。
    pub fn frame_interval(&self) -> Duration {
        duration_from_fps(self.current_fps())
    }

    /// 当前目标 FPS。
    pub fn current_fps(&self) -> u64 {
        match self.mode {
            FrameRateMode::RootUi => {
                if self.is_idle_for(ROOT_UI_IDLE_TIMEOUT_SECS) {
                    LOW_RESOURCE_FPS
                } else {
                    ROOT_UI_NORMAL_FPS
                }
            }
            FrameRateMode::Game {
                afk_time_secs,
                target_fps,
            } => {
                if afk_time_secs > 0 && self.is_idle_for(afk_time_secs) {
                    LOW_RESOURCE_FPS
                } else {
                    target_fps.max(1)
                }
            }
            FrameRateMode::Overlay => OVERLAY_FPS,
        }
    }

    fn is_idle_for(&self, idle_timeout_secs: u64) -> bool {
        self.last_input_at.elapsed() >= Duration::from_secs(idle_timeout_secs)
    }
}

fn duration_from_fps(fps: u64) -> Duration {
    Duration::from_secs_f64(1.0 / fps.max(1) as f64)
}
