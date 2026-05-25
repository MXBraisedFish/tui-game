//! 统一包清单验证。

use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use crate::host_engine::boot::preload::game_modules::GameManifest;
use crate::host_engine::constant::API_VERSION;
use crate::host_engine::package::manifest::RawPackageManifest;
use crate::host_engine::package::package_id::PackageKind;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    fn push_error(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ValidationError {
            field: field.into(),
            message: message.into(),
        });
    }

    fn push_warning(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.warnings.push(ValidationWarning {
            field: field.into(),
            message: message.into(),
        });
    }
}

pub fn validate_package(manifest: &RawPackageManifest, kind: PackageKind) -> ValidationResult {
    validate_package_at(manifest, kind, None)
}

pub fn validate_package_at(
    manifest: &RawPackageManifest,
    kind: PackageKind,
    package_dir: Option<&Path>,
) -> ValidationResult {
    let mut result = ValidationResult::default();

    require_text(&mut result, manifest.package.as_deref(), "package");
    require_text(
        &mut result,
        manifest.package_name.as_deref(),
        "package_name",
    );
    require_text(&mut result, manifest.author.as_deref(), "author");
    require_text(&mut result, manifest.version.as_deref(), "version");
    require_text(
        &mut result,
        manifest.introduction.as_deref(),
        "introduction",
    );

    match kind {
        PackageKind::Game => {
            require_text(&mut result, manifest.game_name.as_deref(), "game_name");
            require_text(&mut result, manifest.description.as_deref(), "description");
            require_text(&mut result, manifest.detail.as_deref(), "detail");
        }
        PackageKind::Screensaver => {
            require_text(&mut result, manifest.entry.as_deref(), "entry");
            require_text(&mut result, manifest.screensaver_name.as_deref(), "screensaver_name");
        }
        PackageKind::Boss => {
            require_text(&mut result, manifest.entry.as_deref(), "entry");
            require_text(&mut result, manifest.boss_name.as_deref(), "boss_name");
        }
        PackageKind::ColorPack | PackageKind::UiPack => {}
    }

    validate_api(&mut result, manifest.api.as_ref());

    if let Some(package_dir) = package_dir {
        validate_entry_exists(&mut result, package_dir, manifest.entry.as_deref());
        validate_image_field(&mut result, package_dir, "icon", manifest.icon.as_ref());
        validate_image_field(&mut result, package_dir, "banner", manifest.banner.as_ref());
    }

    result
}

pub fn validate_game_package_at(
    package_manifest: &RawPackageManifest,
    game_manifest: &GameManifest,
    package_dir: Option<&Path>,
) -> ValidationResult {
    let mut result = ValidationResult::default();

    require_text(&mut result, package_manifest.package.as_deref(), "package");
    require_text(
        &mut result,
        package_manifest.package_name.as_deref(),
        "package_name",
    );
    require_text(
        &mut result,
        package_manifest.introduction.as_deref(),
        "introduction",
    );
    require_text(&mut result, package_manifest.author.as_deref(), "author");
    require_text(
        &mut result,
        package_manifest.game_name.as_deref(),
        "game_name",
    );
    require_text(
        &mut result,
        package_manifest.description.as_deref(),
        "description",
    );
    require_text(&mut result, package_manifest.detail.as_deref(), "detail");
    require_text(&mut result, package_manifest.version.as_deref(), "version");
    validate_api(&mut result, Some(&game_manifest.api));

    if let Some(package_dir) = package_dir {
        validate_entry_exists_from_package_root(
            &mut result,
            package_dir,
            Some(&game_manifest.entry),
        );
        validate_image_field(
            &mut result,
            package_dir,
            "icon",
            package_manifest.icon.as_ref(),
        );
        validate_image_field(
            &mut result,
            package_dir,
            "banner",
            package_manifest.banner.as_ref(),
        );
    }

    result
}

fn require_text(result: &mut ValidationResult, value: Option<&str>, field: &str) {
    match value {
        Some(text) if !text.trim().is_empty() => {}
        _ => result.push_error(field, "required non-empty string field is missing"),
    }
}

fn validate_api(result: &mut ValidationResult, api: Option<&Value>) {
    let Some(api) = api else {
        result.push_error("api", "required api field is missing");
        return;
    };

    match api {
        Value::Number(number) => {
            let Some(version) = number.as_i64() else {
                result.push_error("api", "api must be -1, integer, or [min, max]");
                return;
            };
            if version != -1 && version != i64::from(API_VERSION) {
                result.push_error("api", format!("unsupported api version: {version}"));
            }
        }
        Value::Array(values) if values.len() == 2 => {
            let min_version = values[0].as_i64();
            let max_version = values[1].as_i64();
            match (min_version, max_version) {
                (Some(min_version), Some(max_version))
                    if min_version <= i64::from(API_VERSION)
                        && i64::from(API_VERSION) <= max_version => {}
                (Some(_), Some(_)) => result.push_error("api", "host api version is out of range"),
                _ => result.push_error("api", "api range must contain two integers"),
            }
        }
        _ => result.push_error("api", "api must be -1, integer, or [min, max]"),
    }
}

fn validate_entry_exists(result: &mut ValidationResult, package_dir: &Path, entry: Option<&str>) {
    let Some(entry) = entry else {
        return;
    };
    let Some(clean_path) = normalize_relative_path(entry) else {
        result.push_error("entry", "entry must be a safe relative path");
        return;
    };

    if !package_dir.join("scripts").join(clean_path).is_file() {
        result.push_error("entry", "entry script does not exist");
    }
}

fn validate_entry_exists_from_package_root(
    result: &mut ValidationResult,
    package_dir: &Path,
    entry: Option<&str>,
) {
    let Some(entry) = entry else {
        return;
    };
    let Some(clean_path) = normalize_relative_path(entry) else {
        result.push_error("entry", "entry must be a safe relative path");
        return;
    };

    if !package_dir.join(clean_path).is_file() {
        result.push_error("entry", "entry script does not exist");
    }
}

fn validate_image_field(
    result: &mut ValidationResult,
    package_dir: &Path,
    field: &str,
    value: Option<&Value>,
) {
    let Some(value) = value else {
        result.push_warning(field, "image field is missing; default image will be used");
        return;
    };

    let Value::String(text) = value else {
        if !matches!(value, Value::Array(_)) {
            result.push_warning(
                field,
                "image field is not string or array; default image may be used",
            );
        }
        return;
    };

    let text = text.trim();
    if !(text.starts_with("image:") || text.starts_with("color:image:")) {
        return;
    }

    let image_path = text
        .strip_prefix("color:")
        .unwrap_or(text)
        .strip_prefix("image:")
        .unwrap_or("")
        .trim();
    let Some(clean_path) = normalize_relative_path(image_path) else {
        result.push_warning(field, "image path must be a safe relative path");
        return;
    };
    let extension = clean_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    if !matches!(extension.as_deref(), Some("png" | "jpg" | "jpeg")) {
        result.push_warning(field, "image extension must be png, jpg, or jpeg");
        return;
    }
    if !package_dir.join("assets").join(clean_path).is_file() {
        result.push_warning(
            field,
            "image file does not exist; default image will be used",
        );
    }
}

fn normalize_relative_path(path: &str) -> Option<PathBuf> {
    if path.trim().is_empty() || Path::new(path).is_absolute() {
        return None;
    }

    let mut clean_path = PathBuf::new();
    for component in PathBuf::from(path).components() {
        match component {
            Component::Normal(part) => clean_path.push(part),
            Component::CurDir
            | Component::ParentDir
            | Component::Prefix(_)
            | Component::RootDir => return None,
        }
    }
    Some(clean_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validator_collects_missing_fields() {
        let result = validate_package(&RawPackageManifest::default(), PackageKind::Game);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|error| error.field == "package"));
        assert!(result.errors.iter().any(|error| error.field == "game_name"));
    }

    #[test]
    fn api_minus_one_is_valid() {
        let manifest = RawPackageManifest {
            api: Some(json!(-1)),
            package: Some("demo".to_string()),
            package_name: Some("Demo".to_string()),
            introduction: Some("Intro".to_string()),
            author: Some("Author".to_string()),
            version: Some("1.0.0".to_string()),
            game_name: Some("Game".to_string()),
            description: Some("Desc".to_string()),
            detail: Some("Detail".to_string()),
            ..RawPackageManifest::default()
        };
        assert!(validate_package(&manifest, PackageKind::Game).is_valid());
    }
}
