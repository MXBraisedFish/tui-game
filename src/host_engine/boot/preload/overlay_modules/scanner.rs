//! Screensaver/老板覆盖层包扫描。

use crate::host_engine::boot::environment::data_dirs;
use std::fs::{self, OpenOptions};
use std::io;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use chrono::Local;
use serde_json::{Map, Value};

use crate::host_engine::constant::{
    API_VERSION, DEFAULT_BOSS_BANNER, DEFAULT_PACKAGE_ICON, DEFAULT_SCREENSAVER_BANNER,
};

use super::manifest::{OverlayPackage, OverlayPackageManifest, OverlayRegistry, OverlayScanError};
use super::source::{OverlayKind, OverlaySource};
use super::uid;

type ScannerResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn scan_all() -> ScannerResult<OverlayRegistry> {
    let mut registry = OverlayRegistry::default();

    for kind in [OverlayKind::Screensaver, OverlayKind::Boss] {
        for source in [OverlaySource::Office, OverlaySource::ThirdParty] {
            registry.extend(scan_source(kind, source)?);
        }
    }

    sort_packages(&mut registry.screensavers);
    sort_packages(&mut registry.bosses);
    Ok(registry)
}

fn scan_source(kind: OverlayKind, source: OverlaySource) -> ScannerResult<OverlayRegistry> {
    let root_dir = source.root_dir(kind);
    let mut registry = OverlayRegistry::default();
    if !root_dir.is_dir() {
        return Ok(registry);
    }

    let mut entries = fs::read_dir(&root_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    entries.sort();

    for package_dir in entries {
        match read_overlay_package(kind, source, &package_dir) {
            Ok(package) => match kind {
                OverlayKind::Screensaver => registry.screensavers.push(package),
                OverlayKind::Boss => registry.bosses.push(package),
            },
            Err(error) => registry.errors.push(OverlayScanError {
                kind: kind.as_str().to_string(),
                source: source.as_str().to_string(),
                path: package_dir.display().to_string(),
                error: {
                    let error_text = error.to_string();
                    let _ = append_scan_error_log(kind, &package_dir, error_text.as_str());
                    error_text
                },
            }),
        }
    }

    Ok(registry)
}

fn append_scan_error_log(
    kind: OverlayKind,
    package_dir: &Path,
    error_text: &str,
) -> io::Result<()> {
    let namespace = package_dir
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or("unknown");
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_line = format!(
        "[{timestamp}][{namespace}] [异常] {} package scan failed: {error_text}\n",
        kind.as_str()
    );
    let log_path = data_dirs::root_dir().join("data/log/tui_log.txt");
    if let Some(parent_dir) = log_path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    file.write_all(log_line.as_bytes())
}

fn read_overlay_package(
    kind: OverlayKind,
    source: OverlaySource,
    package_dir: &Path,
) -> ScannerResult<OverlayPackage> {
    let namespace = package_dir
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default()
        .to_string();
    if namespace.trim().is_empty() {
        return Err(
            io::Error::new(io::ErrorKind::InvalidData, "overlay namespace is empty").into(),
        );
    }

    require_dir(&package_dir.join("scripts"))?;
    require_dir(&package_dir.join("scripts/function"))?;
    require_dir(&package_dir.join("assets"))?;
    let manifest = read_manifest(kind, package_dir)?;
    validate_entry_path(package_dir, manifest.entry.as_str())?;
    let uid = generate_uid(kind, source, namespace.as_str(), &manifest);

    Ok(OverlayPackage {
        uid,
        kind,
        source,
        namespace,
        root_dir: package_dir.to_path_buf(),
        manifest,
    })
}

fn validate_entry_path(package_dir: &Path, entry: &str) -> ScannerResult<()> {
    let entry_path = Path::new(entry);
    if entry.trim().is_empty() || entry_path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid entry path: {entry}"),
        )
        .into());
    }
    if entry_path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid entry path: {entry}"),
        )
        .into());
    }
    let full_path = package_dir.join("scripts").join(entry_path);
    if full_path.is_file() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("entry script is missing: {}", full_path.display()),
        )
        .into())
    }
}

fn require_dir(path: &Path) -> ScannerResult<()> {
    if path.is_dir() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("required directory is missing: {}", path.display()),
        )
        .into())
    }
}

fn read_manifest(kind: OverlayKind, package_dir: &Path) -> ScannerResult<OverlayPackageManifest> {
    let value = read_json_object(&package_dir.join("package.json"))?;
    let api = require_value(&value, "package.json", "api")?.clone();
    validate_api_version(&api)?;

    Ok(OverlayPackageManifest {
        api,
        entry: require_string(&value, "package.json", "entry")?,
        package: require_string(&value, "package.json", "package")?,
        package_name: require_string(&value, "package.json", "package_name")?,
        author: require_string(&value, "package.json", "author")?,
        version: require_string(&value, "package.json", "version")?,
        display_name: require_string(&value, "package.json", kind.name_field())?,
        introduction: require_string(&value, "package.json", "introduction")?,
        icon: image_or_default(&value, package_dir, "icon", DEFAULT_PACKAGE_ICON),
        banner: image_or_default(
            &value,
            package_dir,
            "banner",
            match kind {
                OverlayKind::Screensaver => DEFAULT_SCREENSAVER_BANNER,
                OverlayKind::Boss => DEFAULT_BOSS_BANNER,
            },
        ),
    })
}

fn image_or_default(
    object: &Map<String, Value>,
    package_dir: &Path,
    field_name: &str,
    default_lines: &[&str],
) -> Value {
    object
        .get(field_name)
        .filter(|value| is_valid_image_field(package_dir, value))
        .cloned()
        .unwrap_or_else(|| default_lines_value(default_lines))
}

fn default_lines_value(lines: &[&str]) -> Value {
    Value::Array(
        lines
            .iter()
            .map(|line| Value::String((*line).to_string()))
            .collect(),
    )
}

fn is_valid_image_field(package_dir: &Path, value: &Value) -> bool {
    match value {
        Value::Array(values) => {
            !values.is_empty()
                && values
                    .iter()
                    .all(|value| value.as_str().is_some_and(|text| !text.is_empty()))
        }
        Value::String(text) => {
            let text = text.trim();
            if text.is_empty() {
                return false;
            }
            if text.starts_with("image:") || text.starts_with("color:image:") {
                return image_reference_exists(package_dir, text);
            }
            true
        }
        _ => false,
    }
}

fn image_reference_exists(package_dir: &Path, text: &str) -> bool {
    let image_path = text
        .strip_prefix("color:")
        .unwrap_or(text)
        .strip_prefix("image:")
        .unwrap_or("")
        .trim();
    let Some(clean_path) = normalize_image_path(image_path) else {
        return false;
    };
    package_dir.join("assets").join(clean_path).is_file()
}

fn normalize_image_path(path: &str) -> Option<PathBuf> {
    if path.is_empty() || Path::new(path).is_absolute() {
        return None;
    }

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

fn read_json_object(path: &Path) -> ScannerResult<Map<String, Value>> {
    let raw_json = fs::read_to_string(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    let value = serde_json::from_str::<Value>(raw_json.trim_start_matches('\u{feff}'))?;
    value.as_object().cloned().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} must be a JSON object", path.display()),
        )
        .into()
    })
}

fn require_value<'a>(
    object: &'a Map<String, Value>,
    file_name: &str,
    field_name: &str,
) -> ScannerResult<&'a Value> {
    object
        .get(field_name)
        .ok_or_else(|| field_missing_error(file_name, field_name))
}

fn require_string(
    object: &Map<String, Value>,
    file_name: &str,
    field_name: &str,
) -> ScannerResult<String> {
    let value = require_value(object, file_name, field_name)?;
    let text = value
        .as_str()
        .ok_or_else(|| field_type_error(file_name, field_name, "string", value))?;
    if text.trim().is_empty() {
        return Err(field_missing_error(file_name, field_name));
    }
    Ok(text.to_string())
}

fn validate_api_version(api: &Value) -> ScannerResult<()> {
    match api {
        Value::Number(number) => {
            let Some(version) = number.as_i64() else {
                return Err(api_version_type_error(api));
            };
            if version == -1 || version == i64::from(API_VERSION) {
                return Ok(());
            }
            Err(api_version_mismatch_error(api))
        }
        Value::Array(values) if values.len() == 2 => {
            let Some(min_version) = values[0].as_i64() else {
                return Err(api_version_type_error(api));
            };
            let Some(max_version) = values[1].as_i64() else {
                return Err(api_version_type_error(api));
            };
            let host_version = i64::from(API_VERSION);
            if min_version <= host_version && host_version <= max_version {
                return Ok(());
            }
            Err(api_version_mismatch_error(api))
        }
        _ => Err(api_version_type_error(api)),
    }
}

fn generate_uid(
    kind: OverlayKind,
    source: OverlaySource,
    namespace: &str,
    manifest: &OverlayPackageManifest,
) -> String {
    let seed = format!(
        "{}|{}|{}|{}|{}|{}",
        source.as_str(),
        namespace,
        manifest.package,
        manifest.display_name,
        manifest.author,
        manifest.entry
    );
    format!("{}{}", source.uid_prefix(kind), uid::hash_base62_16(&seed))
}

fn sort_packages(packages: &mut [OverlayPackage]) {
    packages.sort_by(|left, right| {
        source_rank(left.source)
            .cmp(&source_rank(right.source))
            .then_with(|| left.namespace.len().cmp(&right.namespace.len()))
            .then_with(|| left.namespace.cmp(&right.namespace))
    });
}

fn source_rank(source: OverlaySource) -> u8 {
    match source {
        OverlaySource::Office => 0,
        OverlaySource::ThirdParty => 1,
    }
}

fn field_missing_error(file_name: &str, field_name: &str) -> Box<dyn std::error::Error> {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{file_name} missing required field: {field_name}"),
    )
    .into()
}

fn api_version_mismatch_error(actual_value: &Value) -> Box<dyn std::error::Error> {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "package.json api version mismatch: expected {}, got {}",
            API_VERSION, actual_value
        ),
    )
    .into()
}

fn api_version_type_error(actual_value: &Value) -> Box<dyn std::error::Error> {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "package.json field api type mismatch: expected -1 | integer | [min, max], got {}",
            json_type_name(actual_value)
        ),
    )
    .into()
}

fn field_type_error(
    file_name: &str,
    field_name: &str,
    expected_type: &str,
    actual_value: &Value,
) -> Box<dyn std::error::Error> {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "{file_name} field {field_name} type mismatch: expected {expected_type}, got {}",
            json_type_name(actual_value)
        ),
    )
    .into()
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
