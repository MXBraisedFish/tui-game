//! Compiled English strings used when deployed translations are unavailable.

use std::collections::HashMap;

include!(concat!(env!("OUT_DIR"), "/embedded_en_us.rs"));

/// Fill missing runtime translation keys from the compiled English namespace.
pub fn fill_embedded_namespace(ns: &str, map: &mut HashMap<String, String>) -> bool {
  fill_namespace(ns, map)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn embedded_english_covers_runtime_namespaces_and_rejects_unknown_ones() {
    let mut map = HashMap::new();
    assert!(fill_embedded_namespace("exit_warning", &mut map));
    assert!(!map.is_empty());
    let mut unknown = HashMap::new();
    assert!(!fill_embedded_namespace("no_such_namespace", &mut unknown));
    assert!(unknown.is_empty());
  }
}
