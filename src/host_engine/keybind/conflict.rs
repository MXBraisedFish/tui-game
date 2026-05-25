//! 键位冲突检测。

use std::collections::HashMap;

use crate::host_engine::package::package_id::PackageId;

use super::binding::Key;

/// 单个物理键被多个动作声明时的冲突。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyConflict {
    pub key: Key,
    pub claimants: Vec<(PackageId, String)>,
    pub resolved_to: Option<PackageId>,
}

/// 检测所有包动作绑定中的重复物理键。
pub fn detect_conflicts(
    bindings: &HashMap<PackageId, HashMap<String, Vec<Key>>>,
) -> Vec<KeyConflict> {
    let mut by_key: HashMap<Key, Vec<(PackageId, String)>> = HashMap::new();

    for (package_id, actions) in bindings {
        for (action, keys) in actions {
            for key in keys {
                by_key
                    .entry(key.clone())
                    .or_default()
                    .push((package_id.clone(), action.clone()));
            }
        }
    }

    let mut conflicts = by_key
        .into_iter()
        .filter_map(|(key, claimants)| {
            if claimants.len() < 2 {
                return None;
            }
            let resolved_to = claimants.first().map(|(package_id, _)| package_id.clone());
            Some(KeyConflict {
                key,
                claimants,
                resolved_to,
            })
        })
        .collect::<Vec<_>>();
    conflicts.sort_by(|left, right| left.key.cmp(&right.key));
    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_engine::package::package_id::{PackageKind, PackageSource};

    #[test]
    fn detects_key_claimed_by_multiple_actions() {
        let first = PackageId::new(PackageSource::ThirdParty, PackageKind::Game, "mod_game_one");
        let second = PackageId::new(PackageSource::ThirdParty, PackageKind::Game, "mod_game_two");
        let mut bindings = HashMap::new();
        bindings.insert(
            first.clone(),
            HashMap::from([("jump".to_string(), vec![Key::Space])]),
        );
        bindings.insert(
            second.clone(),
            HashMap::from([("confirm".to_string(), vec![Key::Space])]),
        );

        let conflicts = detect_conflicts(&bindings);

        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].key, Key::Space);
        assert_eq!(conflicts[0].claimants.len(), 2);
        assert!(conflicts[0].resolved_to.is_some());
    }

    #[test]
    fn ignores_unique_keys() {
        let package_id = PackageId::new(PackageSource::ThirdParty, PackageKind::Game, "mod_game");
        let bindings = HashMap::from([(
            package_id,
            HashMap::from([
                ("jump".to_string(), vec![Key::Space]),
                ("back".to_string(), vec![Key::Esc]),
            ]),
        )]);

        assert!(detect_conflicts(&bindings).is_empty());
    }
}
