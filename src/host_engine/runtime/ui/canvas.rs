//! Rust UI canvas abstraction.
//!
//! Current backend strategy is transitional:
//! - `Canvas::Bridge` writes to the existing `HostLuaBridge` canvas and therefore
//!   requires Lua runtime initialization before use.
//! - `Canvas::Detached` owns an independent `CanvasState` for component tests and
//!   future direct Rust rendering.
//!
//! Keeping the bridge here avoids changing the active Lua UI runtime while the Rust
//! UI framework is being introduced.

use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::border_chars::BorderChars;
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::canvas_state::CanvasState;
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_operation;
use crate::host_engine::boot::preload::lua_runtime::api::drawing_support::drawing_parser::{
    ALIGN_LEFT, BorderRectArgs, DrawRichTextArgs, DrawTextArgs, EraserArgs, FillRectArgs,
    WrapOptions,
};
use crate::host_engine::boot::preload::lua_runtime::{HostLuaBridge, LuaRuntimeContext};

use super::ui_manager::UiResult;

pub enum Canvas {
    /// Writes to the existing Lua-rendering canvas. Only valid after Lua runtime boot.
    Bridge(HostLuaBridge),
    /// Independent canvas state for tests and future non-Lua rendering.
    Detached(CanvasState),
}

impl Canvas {
    pub fn from_bridge(host_bridge: HostLuaBridge) -> Self {
        Self::Bridge(host_bridge)
    }

    pub fn new(width: u16, height: u16) -> Self {
        Self::Detached(CanvasState::new(width, height))
    }

    pub fn clear(&mut self) -> UiResult<()> {
        self.with_canvas_state(|canvas_state| canvas_state.clear())
    }

    pub fn eraser(&mut self, x: u16, y: u16, width: u16, height: u16) -> UiResult<()> {
        self.with_canvas_state(|canvas_state| {
            drawing_operation::erase_rect(
                canvas_state,
                EraserArgs {
                    x,
                    y,
                    width,
                    height,
                },
            );
        })
    }

    pub fn draw_text(&mut self, x: u16, y: u16, text: impl Into<String>) -> UiResult<()> {
        self.draw_text_styled(x, y, text, None, None, Vec::new())
    }

    pub fn draw_text_styled(
        &mut self,
        x: u16,
        y: u16,
        text: impl Into<String>,
        fg: Option<String>,
        bg: Option<String>,
        styles: Vec<i64>,
    ) -> UiResult<()> {
        self.with_canvas_state(|canvas_state| {
            drawing_operation::draw_text(
                canvas_state,
                DrawTextArgs {
                    x,
                    y,
                    text: text.into(),
                    fg,
                    bg,
                    styles,
                    align: ALIGN_LEFT,
                    wrap_options: WrapOptions::default(),
                },
            );
        })
    }

    pub fn draw_rich_text(&mut self, x: u16, y: u16, rich_text: impl Into<String>) -> UiResult<()> {
        self.draw_rich_text_styled(x, y, rich_text, None, None, Vec::new())
    }

    pub fn draw_rich_text_styled(
        &mut self,
        x: u16,
        y: u16,
        rich_text: impl Into<String>,
        fg: Option<String>,
        bg: Option<String>,
        styles: Vec<i64>,
    ) -> UiResult<()> {
        let args = DrawRichTextArgs {
            x,
            y,
            rich_text: rich_text.into(),
            fg,
            bg,
            styles,
            align: ALIGN_LEFT,
            wrap_options: WrapOptions::default(),
        };
        match self {
            Self::Bridge(host_bridge) => {
                let runtime_context = host_bridge.runtime_context();
                host_bridge.with_canvas_state(|canvas_state| {
                    let _ = drawing_operation::draw_rich_text(canvas_state, args, &runtime_context);
                })?;
            }
            Self::Detached(canvas_state) => {
                drawing_operation::draw_rich_text(
                    canvas_state,
                    args,
                    &LuaRuntimeContext::default(),
                )?;
            }
        }
        Ok(())
    }

    pub fn fill_rect(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        fill_char: char,
        fg: Option<String>,
        bg: Option<String>,
    ) -> UiResult<()> {
        self.with_canvas_state(|canvas_state| {
            drawing_operation::fill_rect(
                canvas_state,
                FillRectArgs {
                    x,
                    y,
                    width,
                    height,
                    fill_char,
                    fg,
                    bg,
                },
            );
        })
    }

    pub fn border_rect(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        border_chars: BorderChars,
        fg: Option<String>,
        bg: Option<String>,
    ) -> UiResult<()> {
        self.with_canvas_state(|canvas_state| {
            drawing_operation::border_rect(
                canvas_state,
                BorderRectArgs {
                    x,
                    y,
                    width,
                    height,
                    border_chars,
                    fg,
                    bg,
                },
            );
        })
    }

    pub fn canvas_state(&self) -> CanvasState {
        match self {
            Self::Bridge(host_bridge) => host_bridge.canvas_state(),
            Self::Detached(canvas_state) => canvas_state.clone(),
        }
    }

    fn with_canvas_state(&mut self, operation: impl FnOnce(&mut CanvasState)) -> UiResult<()> {
        match self {
            Self::Bridge(host_bridge) => host_bridge.with_canvas_state(operation)?,
            Self::Detached(canvas_state) => operation(canvas_state),
        }
        Ok(())
    }
}
