//! 新架构包类型定义。

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::host_engine::boot::preload::game_modules::manifest::{
    GameRuntimeManifest, PackageManifest,
};
use crate::host_engine::boot::preload::game_modules::{
    GameActionBinding, GameManifest, GameModule, GameModuleSource,
};
use crate::host_engine::boot::preload::overlay_modules::{
    OverlayKind, OverlayPackage as LegacyOverlayPackage, OverlayPackageManifest, OverlaySource,
};
use crate::host_engine::package::package_id::{PackageId, PackageKind, PackageSource};
use crate::host_engine::package::registry::Package;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionDefault {
    pub key: Value,
    pub key_name: String,
}

impl From<GameActionBinding> for ActionDefault {
    fn from(binding: GameActionBinding) -> Self {
        Self {
            key: binding.key,
            key_name: binding.key_name,
        }
    }
}

impl From<ActionDefault> for GameActionBinding {
    fn from(binding: ActionDefault) -> Self {
        Self {
            key: binding.key,
            key_name: binding.key_name,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GamePackage {
    pub id: PackageId,
    pub uid: String,
    pub source: GameModuleSource,
    pub source_label: String,
    pub root_dir: PathBuf,
    pub package: String,
    pub package_name: String,
    pub introduction: String,
    pub game_name: String,
    pub author: String,
    pub description: String,
    pub detail: String,
    pub version: String,
    pub api_version: String,
    pub entry: String,
    pub icon: String,
    pub banner: String,
    pub icon_value: Value,
    pub banner_value: Value,
    pub actions: HashMap<String, ActionDefault>,
    pub save: bool,
    pub best_none: bool,
    pub best_none_text: Option<String>,
    pub case_sensitive: bool,
    pub write_permission: bool,
    pub target_fps: u16,
    pub afk_time: u64,
    pub min_width: u16,
    pub min_height: u16,
}

impl GamePackage {
    pub fn to_legacy(&self) -> GameModule {
        GameModule {
            package_id: self.id.clone(),
            uid: self.uid.clone(),
            source: self.source,
            source_label: self.source_label.clone(),
            root_dir: self.root_dir.clone(),
            package: PackageManifest {
                package: self.package.clone(),
                package_name: self.package_name.clone(),
                introduction: self.introduction.clone(),
                author: self.author.clone(),
                game_name: self.game_name.clone(),
                description: self.description.clone(),
                detail: self.detail.clone(),
                version: self.version.clone(),
                icon: self.icon_value.clone(),
                banner: self.banner_value.clone(),
            },
            game: GameManifest {
                api: self.api_value(),
                entry: self.entry.clone(),
                save: self.save,
                best_none: self.best_none_text.clone(),
                min_width: i64::from(self.min_width),
                min_height: i64::from(self.min_height),
                write: self.write_permission,
                case_sensitive: self.case_sensitive,
                actions: self
                    .actions
                    .clone()
                    .into_iter()
                    .map(|(action, binding)| (action, binding.into()))
                    .collect(),
                runtime: GameRuntimeManifest {
                    target_fps: self.target_fps,
                    afk_time: self.afk_time,
                },
            },
        }
    }

    fn api_value(&self) -> Value {
        serde_json::from_str(&self.api_version)
            .unwrap_or_else(|_| Value::String(self.api_version.clone()))
    }
}

impl From<GameModule> for GamePackage {
    fn from(module: GameModule) -> Self {
        let api_version = module.game.api.to_string();
        let icon = value_display_text(&module.package.icon);
        let banner = value_display_text(&module.package.banner);
        let best_none_text = module.game.best_none.clone();
        Self {
            id: module.package_id.clone(),
            uid: module.uid.clone(),
            source: module.source,
            source_label: module.source_label.clone(),
            root_dir: module.root_dir.clone(),
            package: module.package.package.clone(),
            package_name: module.package.package_name.clone(),
            introduction: module.package.introduction.clone(),
            game_name: module.package.game_name.clone(),
            author: module.package.author.clone(),
            description: module.package.description.clone(),
            detail: module.package.detail.clone(),
            version: module.package.version.clone(),
            api_version,
            entry: module.game.entry.clone(),
            icon,
            banner,
            icon_value: module.package.icon.clone(),
            banner_value: module.package.banner.clone(),
            actions: module
                .game
                .actions
                .into_iter()
                .map(|(action, binding)| (action, binding.into()))
                .collect(),
            save: module.game.save,
            best_none: best_none_text.is_some(),
            best_none_text,
            case_sensitive: module.game.case_sensitive,
            write_permission: module.game.write,
            target_fps: module.game.runtime.target_fps,
            afk_time: module.game.runtime.afk_time,
            min_width: clamp_i64_to_u16(module.game.min_width),
            min_height: clamp_i64_to_u16(module.game.min_height),
        }
    }
}

impl Package for GamePackage {
    fn id(&self) -> &PackageId {
        &self.id
    }

    fn name(&self) -> &str {
        &self.package_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn is_valid(&self) -> bool {
        !self.uid.is_empty() && !self.package_name.trim().is_empty()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OverlayPackage {
    pub id: PackageId,
    pub uid: String,
    pub kind: OverlayKind,
    pub source: OverlaySource,
    pub namespace: String,
    pub root_dir: PathBuf,
    pub api_version: String,
    pub entry: String,
    pub package: String,
    pub package_name: String,
    pub display_name: String,
    pub author: String,
    pub version: String,
    pub introduction: String,
    pub icon: String,
    pub banner: String,
    pub icon_value: Value,
    pub banner_value: Value,
}

impl OverlayPackage {
    pub fn to_legacy(&self) -> LegacyOverlayPackage {
        LegacyOverlayPackage {
            package_id: self.id.clone(),
            uid: self.uid.clone(),
            kind: self.kind,
            source: self.source,
            namespace: self.namespace.clone(),
            root_dir: self.root_dir.clone(),
            manifest: OverlayPackageManifest {
                api: self.api_value(),
                entry: self.entry.clone(),
                package: self.package.clone(),
                package_name: self.package_name.clone(),
                author: self.author.clone(),
                version: self.version.clone(),
                display_name: self.display_name.clone(),
                introduction: self.introduction.clone(),
                icon: self.icon_value.clone(),
                banner: self.banner_value.clone(),
            },
        }
    }

    fn api_value(&self) -> Value {
        serde_json::from_str(&self.api_version)
            .unwrap_or_else(|_| Value::String(self.api_version.clone()))
    }
}

impl From<LegacyOverlayPackage> for OverlayPackage {
    fn from(package: LegacyOverlayPackage) -> Self {
        Self {
            id: package.package_id.clone(),
            uid: package.uid.clone(),
            kind: package.kind,
            source: package.source,
            namespace: package.namespace.clone(),
            root_dir: package.root_dir.clone(),
            api_version: package.manifest.api.to_string(),
            entry: package.manifest.entry.clone(),
            package: package.manifest.package.clone(),
            package_name: package.manifest.package_name.clone(),
            display_name: package.manifest.display_name.clone(),
            author: package.manifest.author.clone(),
            version: package.manifest.version.clone(),
            introduction: package.manifest.introduction.clone(),
            icon: value_display_text(&package.manifest.icon),
            banner: value_display_text(&package.manifest.banner),
            icon_value: package.manifest.icon.clone(),
            banner_value: package.manifest.banner.clone(),
        }
    }
}

impl Package for OverlayPackage {
    fn id(&self) -> &PackageId {
        &self.id
    }

    fn name(&self) -> &str {
        &self.display_name
    }

    fn description(&self) -> &str {
        &self.introduction
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn is_valid(&self) -> bool {
        !self.uid.is_empty() && !self.display_name.trim().is_empty()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ColorPack {
    pub id: PackageId,
    pub name: String,
    pub colors: HashMap<String, String>,
}

impl ColorPack {
    pub fn new(uid: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: PackageId::new(PackageSource::Office, PackageKind::ColorPack, uid),
            name: name.into(),
            colors: HashMap::new(),
        }
    }
}

impl Package for ColorPack {
    fn id(&self) -> &PackageId {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        ""
    }

    fn version(&self) -> &str {
        ""
    }

    fn is_valid(&self) -> bool {
        !self.id.uid.is_empty() && !self.name.trim().is_empty()
    }
}

fn value_display_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        _ => value.to_string(),
    }
}

fn clamp_i64_to_u16(value: i64) -> u16 {
    u16::try_from(value.max(0)).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_engine::boot::preload::game_modules::GameModuleSource;

    #[test]
    fn color_pack_implements_package() {
        let color_pack = ColorPack::new("theme", "Theme");
        assert_eq!(color_pack.id().uid, "theme");
        assert_eq!(color_pack.name(), "Theme");
        assert!(color_pack.is_valid());
    }

    #[test]
    fn game_package_round_trips_legacy_identity() {
        let legacy = GameModule {
            package_id: PackageId::new(PackageSource::Office, PackageKind::Game, "game_test"),
            uid: "game_test".to_string(),
            source: GameModuleSource::Office,
            source_label: "game".to_string(),
            root_dir: PathBuf::new(),
            package: PackageManifest {
                package: "demo".to_string(),
                package_name: "Demo".to_string(),
                introduction: "Intro".to_string(),
                author: "Author".to_string(),
                game_name: "Demo Game".to_string(),
                description: "Description".to_string(),
                detail: "Detail".to_string(),
                version: "1.0.0".to_string(),
                icon: Value::String("icon".to_string()),
                banner: Value::String("banner".to_string()),
            },
            game: GameManifest {
                api: Value::from(-1),
                entry: "main.lua".to_string(),
                save: true,
                best_none: Some("---".to_string()),
                min_width: 10,
                min_height: 5,
                write: false,
                case_sensitive: false,
                actions: Default::default(),
                runtime: GameRuntimeManifest {
                    target_fps: 60,
                    afk_time: 0,
                },
            },
        };

        let package = GamePackage::from(legacy.clone());
        let restored = package.to_legacy();
        assert_eq!(restored.package_id, legacy.package_id);
        assert_eq!(restored.package.package_name, legacy.package.package_name);
        assert_eq!(restored.game.entry, legacy.game.entry);
    }
}
