//! package.json 图片字段解析

use std::io;
use std::path::Path;

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
    if relative_path.is_empty() || relative_path.contains("..") {
        return None;
    }

    let absolute_path = package_root_dir.join("assets").join(relative_path);
    if !absolute_path.is_file() {
        return None;
    }

    Some(GameImageSpec {
        relative_path: normalize_relative_path(relative_path),
        absolute_path,
        color_mode,
    })
}

fn normalize_relative_path(path: &str) -> String {
    path.replace('\\', "/")
}

pub fn invalid_image_error(path: &Path, error: impl std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("failed to render image {}: {error}", path.display()),
    )
}
