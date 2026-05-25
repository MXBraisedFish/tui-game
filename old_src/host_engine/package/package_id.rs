use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum PackageSource {
    Office,
    ThirdParty,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum PackageKind {
    Game,
    Screensaver,
    Boss,
    ColorPack,
    UiPack,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PackageId {
    pub source: PackageSource,
    pub kind: PackageKind,
    pub uid: String,
}

impl PackageId {
    pub fn new(source: PackageSource, kind: PackageKind, uid: impl Into<String>) -> Self {
        Self {
            source,
            kind,
            uid: uid.into(),
        }
    }

    pub fn from_string(value: &str) -> Option<Self> {
        let mut parts = value.split(':');
        let source = PackageSource::from_string(parts.next()?)?;
        let kind = PackageKind::from_string(parts.next()?)?;
        let uid = parts.next()?;

        if parts.next().is_some() || uid.is_empty() {
            return None;
        }

        Some(Self::new(source, kind, uid))
    }

    pub fn from_legacy(source: &str, kind: &str, uid: &str) -> Self {
        let package_source = PackageSource::from_legacy(source);
        let package_kind = PackageKind::from_legacy(kind);
        Self::new(package_source, package_kind, uid)
    }

    fn source_text(&self) -> &'static str {
        self.source.as_text()
    }

    fn kind_text(&self) -> &'static str {
        self.kind.as_text()
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}:{}",
            self.source_text(),
            self.kind_text(),
            self.uid
        )
    }
}

impl PackageSource {
    fn as_text(self) -> &'static str {
        match self {
            PackageSource::Office => "official",
            PackageSource::ThirdParty => "mod",
        }
    }

    fn from_string(value: &str) -> Option<Self> {
        match value {
            "official" => Some(PackageSource::Office),
            "mod" => Some(PackageSource::ThirdParty),
            _ => None,
        }
    }

    fn from_legacy(value: &str) -> Self {
        match value {
            "official" | "office" | "game" | "screensaver" | "boss" => PackageSource::Office,
            "mod" | "third_party" | "thirdparty" | "third-party" => PackageSource::ThirdParty,
            _ => PackageSource::ThirdParty,
        }
    }
}

impl PackageKind {
    fn as_text(self) -> &'static str {
        match self {
            PackageKind::Game => "game",
            PackageKind::Screensaver => "screensaver",
            PackageKind::Boss => "boss",
            PackageKind::ColorPack => "color_pack",
            PackageKind::UiPack => "ui_pack",
        }
    }

    fn from_string(value: &str) -> Option<Self> {
        match value {
            "game" => Some(PackageKind::Game),
            "screensaver" => Some(PackageKind::Screensaver),
            "boss" => Some(PackageKind::Boss),
            "color_pack" => Some(PackageKind::ColorPack),
            "ui_pack" => Some(PackageKind::UiPack),
            _ => None,
        }
    }

    fn from_legacy(value: &str) -> Self {
        match value {
            "game" | "mod_game" => PackageKind::Game,
            "screensaver" | "screen" | "mod_screensaver" | "mod_screen" => PackageKind::Screensaver,
            "boss" | "mod_boss" => PackageKind::Boss,
            "color_pack" | "colorpack" | "color" => PackageKind::ColorPack,
            "ui_pack" | "uipack" | "ui" | "texture_pack" | "texturepack" => PackageKind::UiPack,
            _ => PackageKind::Game,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_id_round_trips_string_format() {
        let package_id = PackageId::new(
            PackageSource::ThirdParty,
            PackageKind::Screensaver,
            "author.screensaver_name",
        );

        let encoded = package_id.to_string();
        assert_eq!(encoded, "mod:screensaver:author.screensaver_name");
        assert_eq!(PackageId::from_string(&encoded), Some(package_id));
    }

    #[test]
    fn official_game_uses_expected_display_format() {
        let package_id = PackageId::new(PackageSource::Office, PackageKind::Game, "snake");
        assert_eq!(package_id.to_string(), "official:game:snake");
    }

    #[test]
    fn invalid_package_id_string_returns_none() {
        assert_eq!(PackageId::from_string("official:game"), None);
        assert_eq!(PackageId::from_string("official:game:"), None);
        assert_eq!(PackageId::from_string("office:game:snake"), None);
        assert_eq!(PackageId::from_string("official:bad:snake"), None);
    }

    #[test]
    fn legacy_values_convert_without_panicking() {
        let package_id = PackageId::from_legacy("office", "screen", "legacy_uid");
        assert_eq!(package_id.to_string(), "official:screensaver:legacy_uid");
    }
}
