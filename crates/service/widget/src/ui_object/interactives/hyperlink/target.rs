//! Resolve package-scoped link targets and hand them to the system default application.

use std::{ffi::OsString, path::Path};
use tg_core_sandbox_path::{SafeRelativePath, SandboxPathKind, resolve_sandbox_path};
use tg_service_clipboard::ClipboardService;

/// Report whether a target is a portable asset-relative path or an HTTP/HTTPS URL.
///
/// Existence is checked when resolving the target, so links may be configured before assets exist.
///
/// # Examples
///
/// ```rust
/// use tg_service_widget::is_valid_link_target;
/// assert!(is_valid_link_target("./manual.txt"));
/// assert!(is_valid_link_target("https://example.com"));
/// assert!(!is_valid_link_target("../outside"));
/// assert!(!is_valid_link_target("file:///outside"));
/// ```
pub fn is_valid_link_target(target: &str) -> bool {
  if target.is_empty() || target.chars().any(char::is_control) {
    return false;
  }
  if let Ok(url) = url::Url::parse(target) {
    return matches!(url.scheme(), "http" | "https")
      && url.has_host()
      && target
        .split_once(':')
        .is_some_and(|(_, rest)| rest.starts_with("//"));
  }
  SafeRelativePath::parse(target).is_ok()
}

/// Resolve a supported URL or an existing file/directory contained by the package asset root.
///
/// Return `None` for missing paths, unsupported targets, and canonical paths escaping the root.
/// File-system links are checked after canonicalization on every activation.
pub fn resolve_link_target(assets_root: &Path, target: &str) -> Option<OsString> {
  if !is_valid_link_target(target) {
    return None;
  }
  if url::Url::parse(target).is_ok() {
    return Some(target.into());
  }
  let relative = SafeRelativePath::parse(target).ok()?;
  let path = resolve_sandbox_path(assets_root, &relative, SandboxPathKind::Any).ok()?;
  if !path.is_file() && !path.is_dir() {
    return None;
  }
  Some(dunce::simplified(&path).as_os_str().to_owned())
}

/// Open an asset or web link with the system default application and optionally copy its target.
///
/// Copy resolved absolute paths for assets and the original URL for web links. Return whether
/// the launcher accepted the target; clipboard and launcher failures produce no output or UI
/// error request from this service. The external application may outlive the engine.
///
/// # Arguments
///
/// * `assets_root` - The owning package asset directory.
/// * `target` - The HTTP/HTTPS URL or portable asset-relative file/directory path.
/// * `copy` - Whether to copy the resolved target before attempting to open it.
/// * `clipboard` - The platform clipboard supplied by the caller.
pub fn open_link_target(
  assets_root: &Path,
  target: &str,
  copy: bool,
  clipboard: &mut ClipboardService,
) -> bool {
  let Some(resolved) = resolve_link_target(assets_root, target) else {
    return false;
  };
  if copy && let Some(text) = resolved.to_str() {
    let _ = clipboard.write_text(text);
  }
  open_resolved_target(&resolved)
}

#[cfg(not(windows))]
fn open_resolved_target(target: &std::ffi::OsStr) -> bool {
  open::that_detached(target).is_ok()
}

#[cfg(windows)]
fn open_resolved_target(target: &std::ffi::OsStr) -> bool {
  use std::os::windows::ffi::OsStrExt;
  use windows_sys::Win32::{
    Foundation::RPC_E_CHANGED_MODE,
    System::Com::{
      COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
    },
    UI::{
      Shell::{SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, ShellExecuteExW},
      WindowsAndMessaging::SW_SHOWNORMAL,
    },
  };
  let target_wide = target
    .encode_wide()
    .chain(std::iter::once(0))
    .collect::<Vec<_>>();
  // The reserved pointer is null and the flags describe one compatible COM apartment.
  let initialized = unsafe {
    CoInitializeEx(
      std::ptr::null(),
      (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
    )
  };
  if initialized < 0 && initialized != RPC_E_CHANGED_MODE {
    return false;
  }
  // Optional handles and pointers may be null; required members are assigned below.
  let mut info = SHELLEXECUTEINFOW {
    cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
    fMask: SEE_MASK_FLAG_NO_UI | SEE_MASK_NOASYNC,
    lpFile: target_wide.as_ptr(),
    nShow: SW_SHOWNORMAL,
    ..unsafe { std::mem::zeroed() }
  };
  // The target is NUL-terminated and remains alive for the synchronous shell handoff.
  // Omitting NOCLOSEPROCESS leaves external application handles owned by the shell.
  let accepted =
    windows_default_handler_exists(target) && unsafe { ShellExecuteExW(&mut info) } != 0;
  if initialized >= 0 {
    // Balance both S_OK and S_FALSE without uninitializing another COM apartment.
    unsafe { CoUninitialize() };
  }
  accepted
}

#[cfg(windows)]
fn windows_default_handler_exists(target: &std::ffi::OsStr) -> bool {
  use std::{ffi::OsString, os::windows::ffi::OsStrExt};
  use windows_sys::Win32::UI::Shell::{
    ASSOCF_INIT_IGNOREUNKNOWN, ASSOCF_IS_PROTOCOL, ASSOCSTR_PROGID, AssocQueryStringW,
  };
  let path = Path::new(target);
  if path.is_dir() {
    return true;
  }
  let (association, flags) = if let Some(url) = target
    .to_str()
    .and_then(|text| url::Url::parse(text).ok())
    .filter(|url| matches!(url.scheme(), "http" | "https"))
  {
    (
      OsString::from(url.scheme()),
      ASSOCF_INIT_IGNOREUNKNOWN | ASSOCF_IS_PROTOCOL,
    )
  } else if let Some(extension) = path.extension() {
    let mut association = OsString::from(".");
    association.push(extension);
    (association, ASSOCF_INIT_IGNOREUNKNOWN)
  } else {
    return false;
  };
  let association = association
    .encode_wide()
    .chain(std::iter::once(0))
    .collect::<Vec<_>>();
  let mut length = 0;
  // Query only the required string size. IGNOREUNKNOWN excludes the shell's OpenWith fallback.
  let result = unsafe {
    AssocQueryStringW(
      flags,
      ASSOCSTR_PROGID,
      association.as_ptr(),
      std::ptr::null(),
      std::ptr::null_mut(),
      &mut length,
    )
  };
  result >= 0 && length > 1
}
