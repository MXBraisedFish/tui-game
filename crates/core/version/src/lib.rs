//! Host and persisted-format version constants used to check compatibility.
//!
//! # Examples
//!
//! ```rust
//! use tg_core_version::{HOST_VERSION, PACKAGE_MANIFEST_VERSION};
//!
//! assert!(!HOST_VERSION.is_empty());
//! assert_eq!(PACKAGE_MANIFEST_VERSION, 2);
//! ```

/// The host package version embedded by Cargo.
pub const HOST_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The currently supported host API version.
pub const HOST_API_VERSION: u32 = 1;

/// The supported package manifest schema version.
pub const PACKAGE_MANIFEST_VERSION: u32 = 2;

/// The supported persisted-media manifest version.
pub const MEDIA_MANIFEST_VERSION: u32 = 1;

/// The current disposable image cache format version.
pub const IMAGE_CACHE_FORMAT_VERSION: u8 = 3;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn version_constants_keep_their_values() {
    assert_eq!(HOST_VERSION, "2.0.0");
    assert_eq!(HOST_API_VERSION, 1);
    assert_eq!(PACKAGE_MANIFEST_VERSION, 2);
    assert_eq!(MEDIA_MANIFEST_VERSION, 1);
    assert_eq!(IMAGE_CACHE_FORMAT_VERSION, 3);
  }
}
