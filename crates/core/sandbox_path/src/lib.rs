//! Portable relative paths and filesystem resolution constrained to a safe root.
//!
//! # Examples
//!
//! ```rust
//! use tg_core_sandbox_path::SafeRelativePath;
//!
//! let path = SafeRelativePath::parse("./assets/icon.png").expect("safe relative path");
//! assert_eq!(path.virtual_path(), "assets/icon.png");
//! assert!(SafeRelativePath::parse("../outside").is_err());
//! ```

use std::fmt;
use std::path::{Path, PathBuf};

const MAX_VIRTUAL_PATH_BYTES: usize = 8192;

/// The required target kind and creation/removal rules for safe-root resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxPathKind {
  /// The any setting for sandbox path kind.
  Any,
  /// The file setting for sandbox path kind.
  File,
  /// The directory setting for sandbox path kind.
  Directory,
  /// The writable file setting for sandbox path kind.
  WritableFile,
  /// The writable directory setting for sandbox path kind.
  WritableDirectory,
  /// The removable setting for sandbox path kind.
  Removable,
}

/// A normalized portable path that excludes absolute roots and parent traversal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafeRelativePath {
  relative: PathBuf,
  virtual_path: String,
}

impl SafeRelativePath {
  /// Validate a portable relative path and normalize separators and redundant current-directory
  /// segments.
  ///
  /// # Errors
  ///
  /// Return `Empty`, `TooLong`, `ContainsNul`, `Absolute`, `ParentTraversal`, or `InvalidSegment`
  /// for the corresponding invalid portable path.
  pub fn parse(input: &str) -> Result<Self, SandboxPathError> {
    if input.is_empty() {
      return Err(SandboxPathError::Empty);
    }
    if input.len() > MAX_VIRTUAL_PATH_BYTES {
      return Err(SandboxPathError::TooLong);
    }
    if input.contains('\0') {
      return Err(SandboxPathError::ContainsNul);
    }

    let portable = input.replace('\\', "/");
    if portable.starts_with('/') {
      return Err(SandboxPathError::Absolute);
    }

    let mut segments = Vec::new();
    for segment in portable.split('/') {
      match segment {
        "" | "." => continue,
        ".." => return Err(SandboxPathError::ParentTraversal),
        value if !valid_portable_segment(value) => {
          return Err(SandboxPathError::InvalidSegment);
        }
        value => segments.push(value.to_string()),
      }
    }

    let mut relative = PathBuf::new();
    for segment in &segments {
      relative.push(segment);
    }
    let virtual_path = if segments.is_empty() {
      ".".to_string()
    } else {
      segments.join("/")
    };
    Ok(Self {
      relative,
      virtual_path,
    })
  }

  /// Return the normalized portable path using forward slashes.
  pub fn virtual_path(&self) -> &str {
    &self.virtual_path
  }

  /// Report whether the normalized relative path denotes the safe root itself.
  pub fn is_root(&self) -> bool {
    self.relative.as_os_str().is_empty()
  }

  /// Return the current extension.
  pub fn extension(&self) -> Option<&str> {
    self.relative.extension().and_then(|value| value.to_str())
  }

  /// Update the extension used by this safe relative path.
  pub fn set_extension(&mut self, extension: &str) {
    self.relative.set_extension(extension);
    self.virtual_path = path_to_virtual(&self.relative);
  }

  /// Report whether a path is already valid and in its canonical portable spelling.
  pub fn is_normalized(input: &str) -> bool {
    Self::parse(input).is_ok_and(|path| path.virtual_path == input)
  }
}

/// Failures reported by sandbox path operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxPathError {
  /// The empty failure condition.
  Empty,
  /// The too long failure condition.
  TooLong,
  /// The contains nul failure condition.
  ContainsNul,
  /// The absolute failure condition.
  Absolute,
  /// The parent traversal failure condition.
  ParentTraversal,
  /// The invalid segment failure condition.
  InvalidSegment,
  /// The root unavailable failure condition.
  RootUnavailable,
  /// The not found failure condition.
  NotFound,
  /// The parent unavailable failure condition.
  ParentUnavailable,
  /// The escapes root failure condition.
  EscapesRoot,
  /// The not file failure condition.
  NotFile,
  /// The not directory failure condition.
  NotDirectory,
}

impl fmt::Display for SandboxPathError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(match self {
      Self::Empty => "path cannot be empty",
      Self::TooLong => "path exceeds 8192 bytes",
      Self::ContainsNul => "path contains NUL",
      Self::Absolute => "absolute paths are not allowed",
      Self::ParentTraversal => "parent path '..' is not allowed",
      Self::InvalidSegment => "path contains an invalid segment",
      Self::RootUnavailable => "safe path root is unavailable",
      Self::NotFound => "path was not found",
      Self::ParentUnavailable => "path parent does not exist",
      Self::EscapesRoot => "path escapes its safe root",
      Self::NotFile => "path is not a file",
      Self::NotDirectory => "path is not a directory",
    })
  }
}

/// Resolve a validated relative path under the canonical root and enforce the requested path
/// kind.
///
/// # Arguments
///
/// * `root` - The filesystem root that bounds path resolution.
/// * `relative` - The validated path relative to the safe root.
/// * `kind` - The requested event, object, or path kind.
///
/// # Errors
///
/// Return `RootUnavailable`, `NotFound`, or `ParentUnavailable` when required paths cannot be
/// resolved; return `EscapesRoot` for escapes and `NotFile` or `NotDirectory` for a mismatched
/// target kind.
pub fn resolve_sandbox_path(
  root: &Path,
  relative: &SafeRelativePath,
  kind: SandboxPathKind,
) -> Result<PathBuf, SandboxPathError> {
  let canonical_root = root
    .canonicalize()
    .map_err(|_| SandboxPathError::RootUnavailable)?;
  if !canonical_root.is_dir() {
    return Err(SandboxPathError::RootUnavailable);
  }

  let candidate = canonical_root.join(&relative.relative);
  let resolved = if kind == SandboxPathKind::Removable {
    resolve_removable_candidate(&canonical_root, &candidate)?
  } else if candidate.exists() {
    candidate
      .canonicalize()
      .map_err(|_| SandboxPathError::NotFound)?
  } else if kind == SandboxPathKind::WritableDirectory {
    resolve_missing_candidate(&canonical_root, &candidate)?
  } else if kind == SandboxPathKind::WritableFile {
    let parent = candidate
      .parent()
      .ok_or(SandboxPathError::ParentUnavailable)?
      .canonicalize()
      .map_err(|_| SandboxPathError::ParentUnavailable)?;
    if !parent.starts_with(&canonical_root) {
      return Err(SandboxPathError::EscapesRoot);
    }
    candidate
  } else {
    return Err(SandboxPathError::NotFound);
  };

  if !resolved.starts_with(&canonical_root) {
    return Err(SandboxPathError::EscapesRoot);
  }
  match kind {
    SandboxPathKind::Any => Ok(resolved),
    SandboxPathKind::File if !resolved.is_file() => Err(SandboxPathError::NotFile),
    SandboxPathKind::Directory if !resolved.is_dir() => Err(SandboxPathError::NotDirectory),
    SandboxPathKind::WritableFile if resolved.exists() && !resolved.is_file() => {
      Err(SandboxPathError::NotFile)
    }
    SandboxPathKind::WritableDirectory if resolved.exists() && !resolved.is_dir() => {
      Err(SandboxPathError::NotDirectory)
    }
    SandboxPathKind::Removable => Ok(resolved),
    _ => Ok(resolved),
  }
}

fn resolve_removable_candidate(
  canonical_root: &Path,
  candidate: &Path,
) -> Result<PathBuf, SandboxPathError> {
  if std::fs::symlink_metadata(candidate).is_err() {
    return Err(SandboxPathError::NotFound);
  }
  let name = candidate
    .file_name()
    .ok_or(SandboxPathError::ParentUnavailable)?;
  let parent = candidate
    .parent()
    .ok_or(SandboxPathError::ParentUnavailable)?
    .canonicalize()
    .map_err(|_| SandboxPathError::ParentUnavailable)?;
  if !parent.starts_with(canonical_root) {
    return Err(SandboxPathError::EscapesRoot);
  }
  Ok(parent.join(name))
}

/// Check for a path under the safe root without treating a missing target as an error.
///
/// # Errors
///
/// Return a sandbox error for an unavailable root, an escaping path, or an unresolved parent. A
/// safely missing target returns `Ok(false)`.
pub fn sandbox_path_exists(
  root: &Path,
  relative: &SafeRelativePath,
) -> Result<bool, SandboxPathError> {
  match resolve_sandbox_path(root, relative, SandboxPathKind::Any) {
    Ok(_) => Ok(true),
    Err(SandboxPathError::NotFound) => {
      let canonical_root = root
        .canonicalize()
        .map_err(|_| SandboxPathError::RootUnavailable)?;
      let candidate = canonical_root.join(&relative.relative);
      resolve_missing_candidate(&canonical_root, &candidate)?;
      Ok(false)
    }
    Err(error) => Err(error),
  }
}

fn resolve_missing_candidate(
  canonical_root: &Path,
  candidate: &Path,
) -> Result<PathBuf, SandboxPathError> {
  let mut existing = candidate;
  let mut suffix = Vec::new();
  while !existing.exists() {
    let name = existing
      .file_name()
      .ok_or(SandboxPathError::ParentUnavailable)?;
    suffix.push(name.to_os_string());
    existing = existing
      .parent()
      .ok_or(SandboxPathError::ParentUnavailable)?;
  }
  let mut resolved = existing
    .canonicalize()
    .map_err(|_| SandboxPathError::ParentUnavailable)?;
  if !resolved.starts_with(canonical_root) {
    return Err(SandboxPathError::EscapesRoot);
  }
  for segment in suffix.into_iter().rev() {
    resolved.push(segment);
  }
  if !resolved.starts_with(canonical_root) {
    return Err(SandboxPathError::EscapesRoot);
  }
  Ok(resolved)
}

fn valid_portable_segment(segment: &str) -> bool {
  if segment
    .chars()
    .any(|value| matches!(value, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
    || segment.ends_with([' ', '.'])
  {
    return false;
  }
  let stem = segment
    .split_once('.')
    .map_or(segment, |(stem, _)| stem)
    .to_ascii_uppercase();
  !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
    && !(stem.len() == 4
      && (stem.starts_with("COM") || stem.starts_with("LPT"))
      && stem.as_bytes()[3].is_ascii_digit()
      && stem.as_bytes()[3] != b'0')
}

fn path_to_virtual(path: &Path) -> String {
  let value = path
    .iter()
    .map(|segment| segment.to_string_lossy())
    .collect::<Vec<_>>()
    .join("/");
  if value.is_empty() {
    ".".to_string()
  } else {
    value
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn normalizes_current_directory_segments_and_rejects_parent_traversal() {
    assert_eq!(SafeRelativePath::parse(".").unwrap().virtual_path(), ".");
    assert_eq!(
      SafeRelativePath::parse("./images/./icon.png")
        .unwrap()
        .virtual_path(),
      "images/icon.png"
    );
    assert_eq!(
      SafeRelativePath::parse("images\\.\\icon.png")
        .unwrap()
        .virtual_path(),
      "images/icon.png"
    );
    assert_eq!(
      SafeRelativePath::parse("images/../secret.txt").unwrap_err(),
      SandboxPathError::ParentTraversal
    );
    assert_eq!(
      SafeRelativePath::parse("../secret.txt").unwrap_err(),
      SandboxPathError::ParentTraversal
    );
  }

  #[test]
  fn rejects_absolute_and_non_portable_paths() {
    for path in [
      "/etc/passwd",
      r"C:\Windows\system.ini",
      r"\\server\share\file",
      "file:stream",
      "CON",
      "file. ",
    ] {
      assert!(SafeRelativePath::parse(path).is_err(), "accepted {path:?}");
    }
  }
}
