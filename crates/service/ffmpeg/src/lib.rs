//! FFmpeg discovery and encoder capability checks for video export.
//!
//! # Examples
//!
//! ```rust
//! use std::path::PathBuf;
//! use tg_service_ffmpeg::FfmpegService;
//!
//! let mut ffmpeg = FfmpegService::new(PathBuf::from("."), PathBuf::from("data/cache/ffmpeg"));
//! let found = ffmpeg.refresh();
//! assert_eq!(found, ffmpeg.installation().is_some());
//! ```

use std::{
  collections::HashSet,
  ffi::OsStr,
  path::{Path, PathBuf},
  process::{Command, Stdio},
};

/// A discovered FFmpeg executable and its advertised encoder names.
#[derive(Clone, Debug)]
pub struct FfmpegInstallation {
  executable: PathBuf,
  encoders: HashSet<String>,
}

impl FfmpegInstallation {
  /// Return the current executable.
  pub fn executable(&self) -> &Path {
    &self.executable
  }

  /// Report whether the discovered FFmpeg installation advertises the specified encoder.
  pub fn supports_encoder(&self, encoder: &str) -> bool {
    self.encoders.contains(encoder)
  }
}

/// The public entry point for FFmpeg operations.
pub struct FfmpegService {
  deployment_root: PathBuf,
  managed_directory: PathBuf,
  installation: Option<FfmpegInstallation>,
}

impl FfmpegService {
  /// Create the FFmpeg service and immediately search deployment, managed, and system locations.
  pub fn new(deployment_root: impl Into<PathBuf>, managed_directory: impl Into<PathBuf>) -> Self {
    let mut service = Self {
      deployment_root: deployment_root.into(),
      managed_directory: managed_directory.into(),
      installation: None,
    };
    service.refresh();
    service
  }

  /// Repeat executable and encoder discovery, replacing the cached result and returning success.
  pub fn refresh(&mut self) -> bool {
    self.installation = discover(
      &self.deployment_root,
      &self.managed_directory,
      std::env::current_exe().ok().as_deref(),
      std::env::var_os("PATH").as_deref(),
      Platform::current(),
    );
    self.installation.is_some()
  }

  /// Repeat FFmpeg discovery only when no usable installation is cached.
  pub fn refresh_if_missing(&mut self) -> bool {
    self.installation.is_some() || self.refresh()
  }

  /// Return the cached FFmpeg installation and its advertised encoders, if available.
  pub fn installation(&self) -> Option<&FfmpegInstallation> {
    self.installation.as_ref()
  }
}

fn discover(
  deployment_root: &Path,
  managed_directory: &Path,
  current_executable: Option<&Path>,
  path: Option<&OsStr>,
  platform: Platform,
) -> Option<FfmpegInstallation> {
  build_candidates(
    deployment_root,
    managed_directory,
    current_executable,
    path,
    platform,
  )
  .into_iter()
  .find_map(|candidate| probe(&candidate))
}

fn probe(candidate: &Path) -> Option<FfmpegInstallation> {
  let mut version = Command::new(candidate);
  version
    .args(["-hide_banner", "-version"])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null());
  suppress_command_window(&mut version);
  if !version.status().ok()?.success() {
    return None;
  }

  let mut encoders = Command::new(candidate);
  encoders
    .args(["-hide_banner", "-encoders"])
    .stdin(Stdio::null())
    .stderr(Stdio::null());
  suppress_command_window(&mut encoders);
  let output = encoders.output().ok()?;
  if !output.status.success() {
    return None;
  }

  Some(FfmpegInstallation {
    executable: candidate
      .canonicalize()
      .unwrap_or_else(|_| candidate.to_path_buf()),
    encoders: parse_encoders(&String::from_utf8_lossy(&output.stdout)),
  })
}

fn parse_encoders(output: &str) -> HashSet<String> {
  output
    .lines()
    .filter_map(|line| {
      let mut fields = line.split_whitespace();
      let flags = fields.next()?;
      let name = fields.next()?;
      (flags.len() == 6
        && flags
          .chars()
          .all(|character| character == '.' || character.is_ascii_alphabetic()))
      .then(|| name.to_string())
    })
    .collect()
}

fn build_candidates(
  deployment_root: &Path,
  managed_directory: &Path,
  current_executable: Option<&Path>,
  path: Option<&OsStr>,
  platform: Platform,
) -> Vec<PathBuf> {
  let file_name = platform.file_name();
  let mut candidates = Vec::new();

  if let Some(executable_directory) = current_executable.and_then(Path::parent) {
    push_unique(&mut candidates, executable_directory.join(file_name));
    push_unique(
      &mut candidates,
      executable_directory.join("bin").join(file_name),
    );
    if platform == Platform::MacOs
      && let Some(contents_directory) = executable_directory.parent()
    {
      push_unique(
        &mut candidates,
        contents_directory.join("Resources").join(file_name),
      );
    }
  }

  push_unique(&mut candidates, deployment_root.join(file_name));
  push_unique(&mut candidates, deployment_root.join("bin").join(file_name));
  push_unique(&mut candidates, managed_directory.join(file_name));

  if let Some(path) = path {
    for directory in std::env::split_paths(path) {
      push_unique(&mut candidates, directory.join(file_name));
    }
  }

  for candidate in platform.common_candidates() {
    push_unique(&mut candidates, candidate);
  }

  // Delegate the last lookup to the operating system to retain its executable search rules.

  push_unique(&mut candidates, PathBuf::from(file_name));
  candidates
}

fn push_unique(candidates: &mut Vec<PathBuf>, candidate: PathBuf) {
  if !candidates.iter().any(|existing| existing == &candidate) {
    candidates.push(candidate);
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Platform {
  Windows,
  Linux,
  MacOs,
  Other,
}

impl Platform {
  fn current() -> Self {
    if cfg!(target_os = "windows") {
      Self::Windows
    } else if cfg!(target_os = "linux") {
      Self::Linux
    } else if cfg!(target_os = "macos") {
      Self::MacOs
    } else {
      Self::Other
    }
  }

  fn file_name(self) -> &'static str {
    if self == Self::Windows {
      "ffmpeg.exe"
    } else {
      "ffmpeg"
    }
  }

  fn common_candidates(self) -> Vec<PathBuf> {
    match self {
      Self::Windows => {
        let mut candidates = Vec::new();
        for directory in [
          std::env::var_os("ProgramFiles"),
          std::env::var_os("ProgramFiles(x86)"),
          std::env::var_os("LOCALAPPDATA"),
        ]
        .into_iter()
        .flatten()
        {
          candidates.push(
            PathBuf::from(directory)
              .join("ffmpeg")
              .join("bin")
              .join(self.file_name()),
          );
        }
        candidates
      }
      Self::Linux => [
        "/usr/local/bin/ffmpeg",
        "/usr/bin/ffmpeg",
        "/snap/bin/ffmpeg",
        "/home/linuxbrew/.linuxbrew/bin/ffmpeg",
      ]
      .into_iter()
      .map(PathBuf::from)
      .collect(),
      Self::MacOs => [
        "/opt/homebrew/bin/ffmpeg",
        "/usr/local/bin/ffmpeg",
        "/opt/local/bin/ffmpeg",
        "/usr/bin/ffmpeg",
      ]
      .into_iter()
      .map(PathBuf::from)
      .collect(),
      Self::Other => Vec::new(),
    }
  }
}

#[cfg(windows)]
fn suppress_command_window(command: &mut Command) {
  use std::os::windows::process::CommandExt;

  command.creation_flags(0x0800_0000);
}

#[cfg(not(windows))]
fn suppress_command_window(_command: &mut Command) {}

#[cfg(test)]
mod tests {
  use super::*;
  use std::ffi::OsString;

  #[test]
  fn platform_file_names_are_correct() {
    assert_eq!(Platform::Windows.file_name(), "ffmpeg.exe");
    assert_eq!(Platform::Linux.file_name(), "ffmpeg");
    assert_eq!(Platform::MacOs.file_name(), "ffmpeg");
  }

  #[test]
  fn candidates_cover_bundle_root_cache_and_path_without_working_directory() {
    let separator = if cfg!(windows) { ";" } else { ":" };
    let path = OsString::from(format!("path-a{separator}path-b"));
    let candidates = build_candidates(
      Path::new("deployment"),
      Path::new("cache/ffmpeg"),
      Some(Path::new("application/bin/tg")),
      Some(&path),
      Platform::Linux,
    );

    for expected in [
      PathBuf::from("application/bin/ffmpeg"),
      PathBuf::from("deployment/ffmpeg"),
      PathBuf::from("cache/ffmpeg/ffmpeg"),
      PathBuf::from("path-a/ffmpeg"),
      PathBuf::from("path-b/ffmpeg"),
    ] {
      assert!(
        candidates.contains(&expected),
        "missing {}",
        expected.display()
      );
    }
  }

  #[test]
  fn macos_bundle_resources_are_scanned() {
    let candidates = build_candidates(
      Path::new("deployment"),
      Path::new("cache"),
      Some(Path::new("TuiGame.app/Contents/MacOS/tg")),
      None,
      Platform::MacOs,
    );
    assert!(candidates.contains(&PathBuf::from("TuiGame.app/Contents/Resources/ffmpeg")));
  }

  #[test]
  fn encoder_output_is_parsed_by_exact_name() {
    let encoders = parse_encoders(
      "Encoders:\n V....D libx264 H.264\n V....D h264_nvenc NVIDIA\n A..... aac AAC\n",
    );
    assert!(encoders.contains("libx264"));
    assert!(encoders.contains("h264_nvenc"));
    assert!(encoders.contains("aac"));
    assert!(!encoders.contains("Encoders:"));
  }
}
