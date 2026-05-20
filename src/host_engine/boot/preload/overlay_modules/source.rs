//! Saver/老板覆盖层包来源。

use crate::host_engine::boot::environment::data_dirs;
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
            Self::Office => data_dirs::root_dir().join("scripts").join(kind.as_str()),
            Self::ThirdParty => data_dirs::root_dir()
                .join("data")
                .join("mod")
                .join(kind.as_str()),
        }
    }
}
