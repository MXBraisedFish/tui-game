//! 运行阶段渲染入口

pub use super::incremental_renderer::IncrementalRendererState as RendererState;

use crate::host_engine::boot::preload::lua_runtime::HostLuaBridge;

type RendererResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 将 Lua 虚拟画布刷新到终端。
pub fn render_canvas(
    host_bridge: &HostLuaBridge,
    renderer_state: &mut RendererState,
) -> RendererResult<()> {
    super::incremental_renderer::render_canvas(host_bridge, renderer_state)
}
