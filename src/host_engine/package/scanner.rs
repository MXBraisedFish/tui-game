//! 统一包目录发现管道。

use std::fs;
use std::path::PathBuf;

use crate::host_engine::boot::environment::data_dirs;
use crate::host_engine::package::PackageError;
use crate::host_engine::package::package_id::{PackageKind, PackageSource};

type ScannerResult<T> = Result<T, PackageError>;

const SCRIPTS_DIR: &str = "scripts";
const OFFICIAL_DIR: &str = "official";
const DATA_MOD_DIR: &str = "data/mod";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackagePath {
    pub path: PathBuf,
    pub source: PackageSource,
    pub kind: PackageKind,
}

pub struct Scanner;

impl Scanner {
    pub fn scan_directories(kind: PackageKind) -> ScannerResult<Vec<PackagePath>> {
        let mut packages = Vec::new();

        for root_dir in office_roots(kind) {
            scan_root_dir(&mut packages, root_dir, PackageSource::Office, kind)?;
        }

        if let Some(root_dir) = third_party_root(kind) {
            scan_root_dir(&mut packages, root_dir, PackageSource::ThirdParty, kind)?;
        }

        packages.sort_by(|left, right| {
            source_rank(left.source)
                .cmp(&source_rank(right.source))
                .then_with(|| left.path.cmp(&right.path))
        });
        Ok(packages)
    }
}

pub fn root_dir() -> PathBuf {
    data_dirs::root_dir()
}

fn scan_root_dir(
    packages: &mut Vec<PackagePath>,
    root_dir: PathBuf,
    source: PackageSource,
    kind: PackageKind,
) -> ScannerResult<()> {
    if !root_dir.is_dir() {
        return Ok(());
    }

    let mut entries = fs::read_dir(root_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir() && path.join("package.json").is_file())
        .collect::<Vec<_>>();
    entries.sort();

    packages.extend(
        entries
            .into_iter()
            .map(|path| PackagePath { path, source, kind }),
    );
    Ok(())
}

fn office_roots(kind: PackageKind) -> Vec<PathBuf> {
    package_dir_name(kind)
        .map(|name| {
            let root = root_dir();
            vec![
                root.join(SCRIPTS_DIR).join(name),
                root.join(OFFICIAL_DIR).join(name),
            ]
        })
        .unwrap_or_default()
}

fn third_party_root(kind: PackageKind) -> Option<PathBuf> {
    package_dir_name(kind).map(|name| root_dir().join(DATA_MOD_DIR).join(name))
}

fn package_dir_name(kind: PackageKind) -> Option<&'static str> {
    match kind {
        PackageKind::Game => Some("game"),
        PackageKind::Screensaver => Some("screensaver"),
        PackageKind::Boss => Some("boss"),
        PackageKind::ColorPack | PackageKind::UiPack => None,
    }
}

fn source_rank(source: PackageSource) -> u8 {
    match source {
        PackageSource::Office => 0,
        PackageSource::ThirdParty => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_kind_has_no_directories() {
        let packages = Scanner::scan_directories(PackageKind::ColorPack).unwrap();
        assert!(packages.is_empty());
    }
}
