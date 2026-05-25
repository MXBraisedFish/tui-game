use super::package_id::PackageId;
use anyhow::{Result, bail};
use std::collections::{HashMap, HashSet};

pub trait Package {
    fn id(&self) -> &PackageId;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
    fn is_valid(&self) -> bool;
}

#[derive(Clone)]
pub struct PackageRegistry<T: Package> {
    packages: Vec<T>,
    by_uid: HashMap<String, usize>,
    source_order: Vec<String>,
    display_order: Vec<String>,
    enabled: HashSet<String>,
    missing: HashSet<String>,
}

impl<T: Package> PackageRegistry<T> {
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
            by_uid: HashMap::new(),
            source_order: Vec::new(),
            display_order: Vec::new(),
            enabled: HashSet::new(),
            missing: HashSet::new(),
        }
    }

    pub fn insert(&mut self, package: T) -> Result<()> {
        let uid = package.id().uid.clone();
        if self.by_uid.contains_key(&uid) {
            bail!("package uid already exists: {uid}");
        }

        let index = self.packages.len();
        self.packages.push(package);
        self.by_uid.insert(uid.clone(), index);
        self.source_order.push(uid.clone());
        if !self
            .display_order
            .iter()
            .any(|existing_uid| existing_uid == &uid)
        {
            self.display_order.push(uid.clone());
        }
        self.enabled.insert(uid.clone());
        self.missing.remove(&uid);
        Ok(())
    }

    pub fn remove(&mut self, uid: &str) -> Option<T> {
        let index = self.by_uid.remove(uid)?;
        let package = self.packages.remove(index);
        self.source_order.retain(|existing_uid| existing_uid != uid);
        self.display_order
            .retain(|existing_uid| existing_uid != uid);
        self.enabled.remove(uid);
        self.missing.remove(uid);
        self.rebuild_uid_index();
        Some(package)
    }

    pub fn get(&self, uid: &str) -> Option<&T> {
        self.by_uid
            .get(uid)
            .and_then(|index| self.packages.get(*index))
    }

    pub fn get_mut(&mut self, uid: &str) -> Option<&mut T> {
        self.by_uid
            .get(uid)
            .copied()
            .and_then(|index| self.packages.get_mut(index))
    }

    pub fn all(&self) -> &[T] {
        &self.packages
    }

    pub fn enabled_packages(&self) -> Vec<&T> {
        self.packages
            .iter()
            .filter(|package| {
                let uid = package.id().uid.as_str();
                package.is_valid() && self.enabled.contains(uid) && !self.missing.contains(uid)
            })
            .collect()
    }

    pub fn displayed_packages(&self) -> Vec<&T> {
        let mut packages = Vec::new();
        let mut seen = HashSet::new();

        for uid in &self.display_order {
            if let Some(package) = self.enabled_existing_package(uid) {
                packages.push(package);
                seen.insert(uid.as_str());
            }
        }

        for uid in &self.source_order {
            if seen.contains(uid.as_str()) {
                continue;
            }
            if let Some(package) = self.enabled_existing_package(uid) {
                packages.push(package);
            }
        }

        packages
    }

    pub fn set_enabled(&mut self, uid: &str, enabled: bool) {
        if enabled {
            self.enabled.insert(uid.to_string());
        } else {
            self.enabled.remove(uid);
        }
    }

    pub fn is_enabled(&self, uid: &str) -> bool {
        self.enabled.contains(uid) && !self.missing.contains(uid)
    }

    pub fn mark_missing(&mut self, uid: &str) {
        self.missing.insert(uid.to_string());
    }

    pub fn unmark_missing(&mut self, uid: &str) {
        self.missing.remove(uid);
    }

    pub fn set_display_order(&mut self, order: &[String]) {
        self.display_order.clear();
        let mut seen = HashSet::new();

        for uid in order {
            if self.by_uid.contains_key(uid) && seen.insert(uid.clone()) {
                self.display_order.push(uid.clone());
            }
        }

        for uid in &self.source_order {
            if self.by_uid.contains_key(uid) && seen.insert(uid.clone()) {
                self.display_order.push(uid.clone());
            }
        }
    }

    pub fn len(&self) -> usize {
        self.packages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.packages.iter()
    }

    fn enabled_existing_package(&self, uid: &str) -> Option<&T> {
        if !self.enabled.contains(uid) || self.missing.contains(uid) {
            return None;
        }
        self.get(uid).filter(|package| package.is_valid())
    }

    fn rebuild_uid_index(&mut self) {
        self.by_uid.clear();
        for (index, package) in self.packages.iter().enumerate() {
            self.by_uid.insert(package.id().uid.clone(), index);
        }
    }
}

impl<T: Package> Default for PackageRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_engine::package::package_id::{PackageKind, PackageSource};

    #[derive(Clone, Debug)]
    struct TestPackage {
        id: PackageId,
        name: String,
        description: String,
        version: String,
        valid: bool,
    }

    impl TestPackage {
        fn new(uid: &str) -> Self {
            Self {
                id: PackageId::new(PackageSource::ThirdParty, PackageKind::Game, uid),
                name: format!("name-{uid}"),
                description: format!("description-{uid}"),
                version: "1.0.0".to_string(),
                valid: true,
            }
        }

        fn invalid(uid: &str) -> Self {
            Self {
                valid: false,
                ..Self::new(uid)
            }
        }
    }

    impl Package for TestPackage {
        fn id(&self) -> &PackageId {
            &self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn version(&self) -> &str {
            &self.version
        }

        fn is_valid(&self) -> bool {
            self.valid
        }
    }

    #[test]
    fn insert_and_get_basic_package() {
        let mut registry = PackageRegistry::new();
        registry.insert(TestPackage::new("alpha")).unwrap();

        let package = registry.get("alpha").unwrap();
        assert_eq!(package.name(), "name-alpha");
        assert_eq!(registry.len(), 1);
        assert!(registry.insert(TestPackage::new("alpha")).is_err());
    }

    #[test]
    fn enabled_packages_filter_disabled_missing_and_invalid_packages() {
        let mut registry = PackageRegistry::new();
        registry.insert(TestPackage::new("alpha")).unwrap();
        registry.insert(TestPackage::new("beta")).unwrap();
        registry.insert(TestPackage::invalid("gamma")).unwrap();
        registry.insert(TestPackage::new("delta")).unwrap();

        registry.set_enabled("beta", false);
        registry.mark_missing("delta");

        let enabled_uids = registry
            .enabled_packages()
            .into_iter()
            .map(|package| package.id().uid.as_str())
            .collect::<Vec<_>>();

        assert_eq!(enabled_uids, vec!["alpha"]);
        assert!(!registry.is_enabled("beta"));
        assert!(!registry.is_enabled("delta"));
    }

    #[test]
    fn missing_marker_can_be_removed() {
        let mut registry = PackageRegistry::new();
        registry.insert(TestPackage::new("alpha")).unwrap();
        registry.mark_missing("alpha");
        assert!(registry.enabled_packages().is_empty());

        registry.unmark_missing("alpha");
        assert_eq!(registry.enabled_packages().len(), 1);
    }

    #[test]
    fn display_order_controls_displayed_packages() {
        let mut registry = PackageRegistry::new();
        registry.insert(TestPackage::new("alpha")).unwrap();
        registry.insert(TestPackage::new("beta")).unwrap();
        registry.insert(TestPackage::new("gamma")).unwrap();
        registry.set_enabled("beta", false);
        registry.set_display_order(&["gamma".to_string(), "alpha".to_string()]);

        let displayed_uids = registry
            .displayed_packages()
            .into_iter()
            .map(|package| package.id().uid.as_str())
            .collect::<Vec<_>>();

        assert_eq!(displayed_uids, vec!["gamma", "alpha"]);
    }

    #[test]
    fn remove_package_updates_indexes_and_orders() {
        let mut registry = PackageRegistry::new();
        registry.insert(TestPackage::new("alpha")).unwrap();
        registry.insert(TestPackage::new("beta")).unwrap();
        registry.insert(TestPackage::new("gamma")).unwrap();

        let removed = registry.remove("beta").unwrap();
        assert_eq!(removed.id().uid, "beta");
        assert!(registry.get("beta").is_none());
        assert_eq!(registry.get("gamma").unwrap().name(), "name-gamma");

        let source_order_uids = registry
            .iter()
            .map(|package| package.id().uid.as_str())
            .collect::<Vec<_>>();
        assert_eq!(source_order_uids, vec!["alpha", "gamma"]);
    }
}
