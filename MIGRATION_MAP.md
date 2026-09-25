# Migration map

Each row maps a migrated block to the old code it came from. Files are moved with `git mv`
(history is kept; `git log --follow` works). The old module path re-exports the new crate,
so callers are unchanged until the layer that owns them is migrated.

| New location | Old location | Change | Tag |
|---|---|---|---|
| crates/core/package_id/src/lib.rs | src/host_engine/core/package_id.rs (whole file) | moved as-is; behavior tests added before the move; old path now `pub use tg_core_package_id as package_id` | refactor/core-package_id-done |
| crates/core/fault/src/lib.rs | src/host_engine/core/fault.rs (whole file) | moved; `CapturedPanic`, `is_supervised`, `current_fault_domain`, `capture_panic` widened from pub(crate) to pub (used by core/crash); old path re-exports | refactor/core-fault-done |
| crates/core/geometry/src/lib.rs | src/host_engine/services/layout/types.rs (whole file) | moved as-is; Rect::contains behavior test added before the move (pins saturating right/bottom edge); layout re-exports via `use tg_core_geometry as types` | refactor/core-geometry-done |
| crates/core/version/src/lib.rs | src/host_engine/services/version.rs (whole file) | moved as-is; constants pinned by a test before the move; CARGO_PKG_VERSION now read from workspace.package (still 2.0.0) | refactor/core-version-done |
| crates/core/atomic_fs/src/lib.rs | src/host_engine/services/storage/atomic.rs (whole file) | moved; atomic_write/atomic_replace_with widened pub(crate)->pub; existing 2 tests served as pre-move behavior tests; storage re-exports via `use tg_core_atomic_fs as atomic` | refactor/core-atomic_fs-done |
