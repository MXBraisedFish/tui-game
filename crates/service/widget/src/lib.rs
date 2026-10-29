//! Owned UI and runtime objects, clipped surfaces, input focus, and pointer routing.
//!
//! # Examples
//!
//! ```rust
//! use tg_service_widget::{SliceOptions, SliceService, UiObjectPool};
//!
//! fn main() {
//!   let slices = SliceService::new();
//!   let mut pool = UiObjectPool::new();
//!
//!   let slice = slices
//!     .create(&mut pool, SliceOptions::default())
//!     .expect("create slice");
//!   assert!(slices.exists(&pool, slice));
//!   assert!(slices.set_position(&mut pool, slice, 3, 2));
//!   let rect = slices.configured_rect(&pool, slice).expect("slice rect");
//!   assert_eq!((rect.x, rect.y), (3, 2));
//!
//!   assert!(slices.remove(&mut pool, slice));
//!   assert!(!slices.exists(&pool, slice));
//!   println!("widget ok: slice created, moved and removed");
//! }
//! ```

pub(crate) mod runtime_object;
pub(crate) mod ui_object;

pub use runtime_object::{RuntimeObjectPool, RuntimeObjectPoolOwner};
pub use ui_object::interactives::hit_area::{
  HitAreaEvent, HitAreaId, HitAreaOptions, HitAreaService,
};
pub use ui_object::interactives::hyperlink::{
  HyperlinkEvent, HyperlinkId, HyperlinkOptions, HyperlinkService, is_valid_link_target,
  open_link_target, resolve_link_target,
};
pub use ui_object::interactives::text_input::{
  TextInputCursorShape, TextInputEvent, TextInputId, TextInputMode, TextInputOptions,
  TextInputRenderParams, TextInputService, VerticalAlign,
};
pub use ui_object::surfaces::markdown_view::{
  MarkdownEvent, MarkdownLayout, MarkdownRenderParams, MarkdownService, MarkdownTheme,
  MarkdownViewId, MarkdownViewOptions,
};
pub use ui_object::surfaces::progress_bar::{
  ProgressBarFillOrigin, ProgressBarId, ProgressBarOptions, ProgressBarSegmentStyle,
  ProgressBarService,
};
pub use ui_object::surfaces::scroll_box::{
  Overflow, ScrollBoxContentView, ScrollBoxEvent, ScrollBoxId, ScrollBoxOptions, ScrollBoxService,
  ScrollBoxWidget, ScrollBoxWidgetLayout, ScrollBoxWidgetOptions, ScrollbarLayout, ScrollbarPolicy,
  ScrollbarSide, ScrollbarStyle, ScrollbarVisibility, clip_rect,
};
pub use ui_object::surfaces::slice::{SliceId, SliceLength, SliceOptions, SliceRect, SliceService};
pub use ui_object::surfaces::surface::SurfaceId;
pub use ui_object::surfaces::table::{
  TableAlign, TableBorderMode, TableBorderStyle, TableCell, TableColumn, TableDrawParams, TableId,
  TableOptions, TableOverflow, TableRow, TableService, TableStyle,
};
pub use ui_object::{PointerScopeId, UiEvent, UiObjectPool, UiObjectPoolOwner};
