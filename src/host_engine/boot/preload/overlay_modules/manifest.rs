//! 屏保/老板覆盖层包清单。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::source::{OverlayKind, OverlaySource};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OverlayPackageManifest {
    pub api: Value,
    pub entry: String,
    pub package_name: String,
    pub author: String,
    pub version: String,
    pub display_name: String,
    pub introduction: String,
    pub icon: Value,
    pub banner: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OverlayPackage {
    pub uid: String,
    pub kind: OverlayKind,
    pub source: OverlaySource,
    pub namespace: String,
    pub root_dir: PathBuf,
    pub manifest: OverlayPackageManifest,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OverlayScanError {
    pub kind: String,
    pub source: String,
    pub path: String,
    pub error: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OverlayRegistry {
    pub screens: Vec<OverlayPackage>,
    pub bosses: Vec<OverlayPackage>,
    pub errors: Vec<OverlayScanError>,
}

impl OverlayRegistry {
    pub fn extend(&mut self, other: Self) {
        self.screens.extend(other.screens);
        self.bosses.extend(other.bosses);
        self.errors.extend(other.errors);
    }

    pub fn default_screen(&self) -> Option<&OverlayPackage> {
        self.screens.first()
    }

    pub fn default_boss(&self) -> Option<&OverlayPackage> {
        self.bosses.first()
    }
}
