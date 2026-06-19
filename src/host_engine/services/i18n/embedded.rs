use std::collections::HashMap;

// 编译时由 build.rs 生成
include!(concat!(env!("OUT_DIR"), "/embedded_en_us.rs"));

/// 将嵌入的 en_us 运行时文本填充到指定 namespace。
/// 返回 true 表示该 namespace 存在嵌入数据。
pub fn fill_embedded_namespace(ns: &str, map: &mut HashMap<String, String>) -> bool {
  fill_namespace(ns, map)
}
