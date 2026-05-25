use super::package_id::PackageId;
use anyhow::{Result, bail};
use std::collections::HashSet;

#[derive(Clone, Debug, Default)]
pub struct PackageIdRegistry {
    ids: HashSet<PackageId>,
}

impl PackageIdRegistry {
    pub fn register(&mut self, id: &PackageId) -> Result<()> {
        if let Some(conflict) = self.find_conflict(id) {
            bail!("package uid conflict: {} conflicts with {}", id, conflict);
        }

        self.ids.insert(id.clone());
        Ok(())
    }

    pub fn is_registered(&self, id: &PackageId) -> bool {
        self.ids.contains(id)
    }

    pub fn find_conflict(&self, id: &PackageId) -> Option<&PackageId> {
        self.ids
            .iter()
            .find(|registered_id| registered_id.uid == id.uid)
    }

    pub fn remove(&mut self, id: &PackageId) {
        self.ids.remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::super::package_id::{PackageKind, PackageSource};
    use super::*;

    #[test]
    fn register_accepts_unique_uid() {
        let mut registry = PackageIdRegistry::default();
        let package_id = PackageId::new(PackageSource::Office, PackageKind::Game, "snake");

        registry.register(&package_id).unwrap();

        assert!(registry.is_registered(&package_id));
    }

    #[test]
    fn register_rejects_duplicate_uid_across_kinds() {
        let mut registry = PackageIdRegistry::default();
        let game_id = PackageId::new(PackageSource::Office, PackageKind::Game, "shared_uid");
        let screensaver_id = PackageId::new(
            PackageSource::ThirdParty,
            PackageKind::Screensaver,
            "shared_uid",
        );

        registry.register(&game_id).unwrap();
        let error = registry.register(&screensaver_id).unwrap_err();

        assert!(error.to_string().contains("package uid conflict"));
        assert_eq!(registry.find_conflict(&screensaver_id), Some(&game_id));
    }

    #[test]
    fn remove_unregisters_exact_id() {
        let mut registry = PackageIdRegistry::default();
        let package_id = PackageId::new(PackageSource::ThirdParty, PackageKind::Boss, "boss_uid");

        registry.register(&package_id).unwrap();
        registry.remove(&package_id);

        assert!(!registry.is_registered(&package_id));
    }
}
