use std::collections::HashMap;

include!(concat!(env!("OUT_DIR"), "/embedded_en_us.rs"));

/// 使用编译时嵌入的 en_us 翻译填充指定命名空间
pub fn fill_embedded_namespace(ns: &str, map: &mut HashMap<String, String>) -> bool {
  fill_namespace(ns, map)
}
