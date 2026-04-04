use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;

use crate::core::stats;
use crate::game::action::ActionBinding;
use crate::game::package::{GamePackageSource, discover_packages};
use crate::mods;
use crate::utils::path_utils;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameSourceKind {
    Official,
    Mod,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageDescriptor {
    pub root_dir: PathBuf,
    pub namespace: String,
    pub package_name: String,
    pub author: String,
    pub version: String,
    pub source: GameSourceKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub detail: String,
    pub best_none: Option<String>,
    pub save: bool,
    pub min_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
    pub actions: BTreeMap<String, ActionBinding>,
    pub entry_path: PathBuf,
    pub source: GameSourceKind,
    pub package: Option<PackageDescriptor>,
}

impl GameDescriptor {
    pub fn package_info(&self) -> Option<&PackageDescriptor> {
        self.package.as_ref()
    }

    pub fn is_mod_game(&self) -> bool {
        matches!(self.source, GameSourceKind::Mod)
    }
}

#[derive(Clone, Debug, Default)]
pub struct GameRegistry {
    games: Vec<GameDescriptor>,
}

impl GameRegistry {
    pub fn empty() -> Self {
        Self { games: Vec::new() }
    }

    pub fn from_games(games: Vec<GameDescriptor>) -> Self {
        Self { games }
    }

    pub fn scan_all() -> Result<Self> {
        let mut games = Vec::new();
        games.extend(scan_manifest_games(GamePackageSource::Official)?);
        games.extend(scan_manifest_games(GamePackageSource::Mod)?);

        let mut dedup = Vec::new();
        for game in games {
            if !dedup
                .iter()
                .any(|existing: &GameDescriptor| existing.id == game.id)
            {
                dedup.push(game);
            }
        }

        let valid_ids = dedup.iter().map(|game| game.id.clone()).collect::<Vec<_>>();
        let _ = stats::prune_runtime_scores(valid_ids);

        Ok(Self { games: dedup })
    }

    pub fn games(&self) -> &[GameDescriptor] {
        &self.games
    }

    pub fn into_games(self) -> Vec<GameDescriptor> {
        self.games
    }

    pub fn find(&self, id: &str) -> Option<&GameDescriptor> {
        self.games.iter().find(|game| game.id == id)
    }
}

fn scan_manifest_games(source: GamePackageSource) -> Result<Vec<GameDescriptor>> {
    let base_dir = match source {
        GamePackageSource::Official => path_utils::official_games_dir()?,
        GamePackageSource::Mod => mods::mod_data_dir()?,
    };

    let mut games = Vec::new();
    for package in discover_packages(&base_dir, source.clone())? {
        let package_descriptor = PackageDescriptor {
            root_dir: package.root_dir.clone(),
            namespace: package.package.namespace.clone(),
            package_name: package.package.package_name.clone(),
            author: package.package.author.clone(),
            version: package.package.version.clone(),
            source: match source {
                GamePackageSource::Official => GameSourceKind::Official,
                GamePackageSource::Mod => GameSourceKind::Mod,
            },
        };

        for game in package.games {
            games.push(GameDescriptor {
                id: game.id,
                name: game.name,
                description: game.description,
                detail: game.detail,
                best_none: game.best_none,
                save: game.save,
                min_width: game.min_width,
                min_height: game.min_height,
                max_width: game.max_width,
                max_height: game.max_height,
                actions: game.actions,
                entry_path: package.root_dir.join(&game.entry),
                source: package_descriptor.source.clone(),
                package: Some(package_descriptor.clone()),
            });
        }
    }
    Ok(games)
}
