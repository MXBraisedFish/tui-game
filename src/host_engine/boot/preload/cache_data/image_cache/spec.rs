//! package.json 图片字段解析

use std::io;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use super::types::{GameImageColorMode, GameImageSpec};

const COLOR_PREFIX: &str = "color:";
const IMAGE_PREFIX: &str = "image:";

/// 从 package.json 的 icon/banner 字段解析 image: 规格。
pub fn parse_image_spec(package_root_dir: &Path, value: &Value) -> Option<GameImageSpec> {
    let raw_value = value.as_str()?.trim();
    let (color_mode, image_value) = if let Some(rest) = raw_value.strip_prefix(COLOR_PREFIX) {
        (GameImageColorMode::Color, rest)
    } else {
        (GameImageColorMode::Grayscale, raw_value)
    };

    let relative_path = image_value.strip_prefix(IMAGE_PREFIX)?.trim();
    if relative_path.is_empty() || Path::new(relative_path).is_absolute() {
        return None;
    }

    let clean_path = normalize_and_validate_image_path(relative_path)?;
    let absolute_path = package_root_dir.join("assets").join(&clean_path);
    if !absolute_path.is_file() {
        return None;
    }

    Some(GameImageSpec {
        relative_path: normalize_relative_path(&clean_path),
        absolute_path,
        color_mode,
    })
}

fn normalize_and_validate_image_path(path: &str) -> Option<PathBuf> {
    let mut clean_path = PathBuf::new();
    for component in PathBuf::from(path).components() {
        match component {
            Component::Normal(part) => clean_path.push(part),
            Component::CurDir
            | Component::ParentDir
            | Component::Prefix(_)
            | Component::RootDir => {
                return None;
            }
        }
    }

    let extension = clean_path.extension()?.to_str()?.to_ascii_lowercase();
    matches!(extension.as_str(), "png" | "jpg" | "jpeg").then_some(clean_path)
}

fn normalize_relative_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn invalid_image_error(path: &Path, error: impl std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("failed to render image {}: {error}", path.display()),
    )
}
