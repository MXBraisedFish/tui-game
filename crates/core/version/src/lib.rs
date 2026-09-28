//! Version constants of the host and of the formats it reads and writes.

/// Host version, taken from the Cargo package version.
pub const HOST_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Package API version supported by this host.
pub const HOST_API_VERSION: u32 = 1;

/// `package.json` manifest version supported by this host.
pub const PACKAGE_MANIFEST_VERSION: u32 = 2;

/// Screenshot/recording manifest version this host writes and reads.
pub const MEDIA_MANIFEST_VERSION: u32 = 1;

/// Format version of the image character-art cache.
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
