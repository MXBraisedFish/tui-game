//! Lua image library bindings with validated arguments and session-owned host access.

use super::*;
use crate::MAX_LUA_IMAGE_TASKS_PER_SESSION;
use crate::path::{SafeRelativePath, SandboxPathError, SandboxPathKind, resolve_sandbox_path};
use tg_service_image::{ImageConvertMode, ImageConvertParams};

/// Build and register the Lua image API in the supplied VM and host context.
///
/// # Errors
///
/// Propagate Lua allocation, table construction, or function registration errors while installing
/// this library.
///
/// # Panics
///
/// Panic if an internal invariant is violated: `mode has a default`.
pub(super) fn image(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  let load_state = state;
  source.raw_set(
    "load",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "image.load";
      let parameters = args::positional(
        lua,
        method,
        values,
        &["path"],
        &[
          "block_width",
          "block_height",
          "crop_x",
          "crop_y",
          "crop_width",
          "crop_height",
          "scale",
          "cache",
          "mode",
          "background",
        ],
      )?;
      let table = parameters.options();
      let relative = image_path(&parameters, method)?;
      let assets_root = load_state.borrow().context.assets_root.clone();
      let resolved = resolve_image_path(&assets_root, &relative)
        .map_err(|error| args::message(method, format!("invalid image path: {error}")))?;

      let output_width = optional_positive_u32(table, method, "block_width")?;
      let output_height = optional_positive_u32(table, method, "block_height")?;
      let crop_x = optional_crop_offset(table, method, "crop_x")?;
      let crop_y = optional_crop_offset(table, method, "crop_y")?;
      let crop_width = optional_positive_u32(table, method, "crop_width")?;
      let crop_height = optional_positive_u32(table, method, "crop_height")?;
      let scale = match table.get::<Value>("scale")? {
        Value::Nil => 1.0,
        value => args::number(value, method, "scale")?,
      };
      if !scale.is_finite() || scale <= 0.0 {
        return Err(args::message(
          method,
          "scale must be a finite positive number",
        ));
      }
      let cache = args::optional_bool(table, method, "cache", true)?;
      let mode = match args::optional_string(table, method, "mode", Some("half_block"))?
        .expect("mode has a default")
        .as_str()
      {
        "half_block" => ImageConvertMode::HalfBlock,
        "mix_block" => ImageConvertMode::MixBlock,
        _ => {
          return Err(args::message(
            method,
            "mode must be 'half_block' or 'mix_block'",
          ));
        }
      };
      let background = parse_background_color(table, method)?;

      let mut api = load_state.borrow_mut();
      if api.pending_image_request_ids.len() >= MAX_LUA_IMAGE_TASKS_PER_SESSION {
        return Err(args::message(
          method,
          format!(
            "at most {MAX_LUA_IMAGE_TASKS_PER_SESSION} image requests may be pending per session"
          ),
        ));
      }
      let request_id = api.next_image_request_id;
      api.next_image_request_id = api.next_image_request_id.wrapping_add(1).max(1);
      api.pending_image_request_ids.insert(request_id);
      push_host_command(
        &mut api,
        LuaHostCommand::ImageRequest {
          request_id,
          params: ImageConvertParams {
            image_path: resolved.to_string_lossy().into_owned(),
            mode,
            background,
            output_width,
            output_height,
            crop_x,
            crop_y,
            crop_width,
            crop_height,
            square_crop: false,
            scale,
            cache,
          },
        },
      );
      Ok(request_id)
    })?,
  )?;
  readonly::proxy(lua, source)
}

fn image_path(parameters: &args::PositionalArgs, method: &str) -> mlua::Result<SafeRelativePath> {
  let path = args::string(parameters.required(0, method, "path")?, method, "path")?;
  SafeRelativePath::parse(&path)
    .map_err(|error| args::message(method, format!("unsafe asset path: {error}")))
}

fn optional_positive_u32(table: &Table, method: &str, name: &str) -> mlua::Result<Option<u32>> {
  args::optional_integer(table, method, name, None)?
    .map(|value| {
      u32::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| args::message(method, format!("{name} must be in 1..=4294967295")))
    })
    .transpose()
}

fn optional_crop_offset(table: &Table, method: &str, name: &str) -> mlua::Result<i32> {
  let value = args::optional_integer(table, method, name, Some(0))?.unwrap_or_default();
  i32::try_from(value)
    .ok()
    .filter(|value| *value >= 0)
    .ok_or_else(|| {
      args::message(
        method,
        format!("{name} must be a non-negative 32-bit integer"),
      )
    })
}

fn parse_background_color(table: &Table, method: &str) -> mlua::Result<[u8; 3]> {
  let value = args::optional_string(table, method, "background", Some("#000000"))?
    .expect("background has a default");
  let bytes = value.as_bytes();
  if bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(u8::is_ascii_hexdigit) {
    let red = u8::from_str_radix(std::str::from_utf8(&bytes[1..3]).unwrap(), 16);
    let green = u8::from_str_radix(std::str::from_utf8(&bytes[3..5]).unwrap(), 16);
    let blue = u8::from_str_radix(std::str::from_utf8(&bytes[5..7]).unwrap(), 16);
    if let (Ok(red), Ok(green), Ok(blue)) = (red, green, blue) {
      return Ok([red, green, blue]);
    }
  }
  if let Some(channels) = value
    .strip_prefix("rgb(")
    .and_then(|value| value.strip_suffix(')'))
  {
    let mut values = channels.split(',');
    let channels = [values.next(), values.next(), values.next()];
    if values.next().is_none()
      && let [Some(red), Some(green), Some(blue)] = channels
      && let (Some(red), Some(green), Some(blue)) = (
        parse_rgb_channel(red),
        parse_rgb_channel(green),
        parse_rgb_channel(blue),
      )
    {
      return Ok([red, green, blue]);
    }
  }
  Err(args::message(
    method,
    "background must be an exact #rrggbb or rgb(r,g,b) string",
  ))
}

fn parse_rgb_channel(value: &str) -> Option<u8> {
  if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
    return None;
  }
  value.parse::<u8>().ok()
}

fn resolve_image_path(root: &Path, relative: &SafeRelativePath) -> Result<PathBuf, String> {
  if let Some(extension) = relative.extension() {
    if !is_supported_image_extension(extension) {
      return Err("only png, jpg, or jpeg extensions are supported".to_string());
    }
    return resolve_sandbox_path(root, relative, SandboxPathKind::File)
      .map_err(|error| error.to_string());
  }

  for extension in ["png", "jpg", "jpeg"] {
    let mut candidate = relative.clone();
    candidate.set_extension(extension);
    match resolve_sandbox_path(root, &candidate, SandboxPathKind::File) {
      Ok(path) => return Ok(path),
      Err(SandboxPathError::NotFound) => {}
      Err(error) => return Err(error.to_string()),
    }
  }
  Err("no matching png, jpg, or jpeg file was found".to_string())
}

fn is_supported_image_extension(extension: &str) -> bool {
  matches!(
    extension.to_ascii_lowercase().as_str(),
    "png" | "jpg" | "jpeg"
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use std::sync::atomic::{AtomicU64, Ordering};

  static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(1);

  #[cfg(unix)]
  fn create_file_link(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
  }

  #[cfg(windows)]
  fn create_file_link(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
  }

  #[test]
  fn image_extension_candidates_cannot_escape_assets_through_symlinks() {
    let root = std::env::temp_dir().join(format!(
      "tg_lua_image_sandbox_{}_{}",
      std::process::id(),
      NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    let assets = root.join("assets");
    let outside = root.join("outside");
    fs::create_dir_all(&assets).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let outside_image = outside.join("secret.png");
    let symlink_image = assets.join("outside.png");
    fs::write(&outside_image, b"image").unwrap();
    if create_file_link(&outside_image, &symlink_image).is_err() {
      let _ = fs::remove_dir_all(&root);
      return;
    }

    let canonical_assets = assets.canonicalize().unwrap();
    let extensionless = SafeRelativePath::parse("outside").unwrap();
    let explicit = SafeRelativePath::parse("outside.png").unwrap();
    assert!(
      resolve_image_path(&canonical_assets, &extensionless)
        .unwrap_err()
        .contains("escapes its safe root")
    );
    assert!(
      resolve_image_path(&canonical_assets, &explicit)
        .unwrap_err()
        .contains("escapes its safe root")
    );

    let _ = fs::remove_dir_all(root);
  }
}
