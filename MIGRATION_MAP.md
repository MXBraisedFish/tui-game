# Migration map

Each row maps a migrated block to the old code it came from. Files are moved with `git mv`
(history is kept; `git log --follow` works). The old module path re-exports the new crate,
so callers are unchanged until the layer that owns them is migrated.

| New location | Old location | Change | Tag |
|---|---|---|---|
| crates/core/package_id/src/lib.rs | src/host_engine/core/package_id.rs (whole file) | moved as-is; behavior tests added before the move; old path now `pub use tg_core_package_id as package_id` | refactor/core-package_id-done |
