//! Render pipeline: composes prepared canvas surfaces into a frame and presents it to the terminal.

mod compositor;
mod presenter;

pub use compositor::FrameCompositor;
pub use tg_core_style::{ComposedCell, ComposedFrame};
pub use presenter::FramePresenter;
