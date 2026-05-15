//! Saver/老板覆盖层包来源。

use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum OverlayKind {
    Saver,
    Boss,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum OverlaySource {
    Office,
    ThirdParty,
}

impl OverlayKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Saver => "saver",
            Self::Boss => "boss",
        }
    }

    pub fn name_field(self) -> &'static str {
        match self {
            Self::Saver => "saver_name",
            Self::Boss => "boss_name",
        }
    }
}

impl OverlaySource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Office => "office",
            Self::ThirdParty => "third_party",
        }
    }

    pub fn uid_prefix(self, kind: OverlayKind) -> &'static str {
        match (self, kind) {
            (Self::Office, OverlayKind::Saver) => "saver_",
            (Self::ThirdParty, OverlayKind::Saver) => "mod_saver_",
            (Self::Office, OverlayKind::Boss) => "boss_",
            (Self::ThirdParty, OverlayKind::Boss) => "mod_boss_",
        }
    }

    pub fn root_dir(self, kind: OverlayKind) -> PathBuf {
        match self {
            Self::Office => root_dir().join("scripts").join(kind.as_str()),
            Self::ThirdParty => root_dir().join("data").join("mod").join(kind.as_str()),
        }
    }
}

fn root_dir() -> PathBuf {
    std::env::current_dir()
        .ok()
        .filter(|path| path.join("assets").exists() || path.join("Cargo.toml").exists())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(PathBuf::from))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}
