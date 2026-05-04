//! 增量渲染模块

pub mod color_style;
pub mod diff;
pub mod frame_cache;
pub mod terminal_output;

use crate::host_engine::boot::preload::lua_runtime::HostLuaBridge;

use self::diff::diff_frames;
use self::frame_cache::FrameCache;
use self::terminal_output::{clear_screen, write_changes};

type IncrementalRendererResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 运行阶段渲染状态。
#[derive(Default)]
pub struct IncrementalRendererState {
    previous_frame: FrameCache,
    force_full_redraw: bool,
}

impl IncrementalRendererState {
    /// 创建增量渲染状态。首次渲染总是全量输出。
    pub fn new() -> Self {
        Self {
            previous_frame: FrameCache::default(),
            force_full_redraw: true,
        }
    }

    /// 请求下一帧执行全量刷新。
    pub fn request_full_redraw(&mut self) {
        self.force_full_redraw = true;
    }
}

/// 将当前画布按差量刷新到终端。
pub fn render_canvas(
    host_bridge: &HostLuaBridge,
    renderer_state: &mut IncrementalRendererState,
) -> IncrementalRendererResult<()> {
    let current_frame = FrameCache::from_canvas_state(&host_bridge.canvas_state());

    if renderer_state.force_full_redraw
        || renderer_state.previous_frame.width() != current_frame.width()
        || renderer_state.previous_frame.height() != current_frame.height()
    {
        clear_screen()?;
        renderer_state.previous_frame = FrameCache::default();
        renderer_state.force_full_redraw = false;
    }

    let changes = diff_frames(&renderer_state.previous_frame, &current_frame);
    write_changes(&changes)?;
    renderer_state.previous_frame = current_frame;
    Ok(())
}
