pub fn execute() {
  // 获取路径并清理
    let cache_dir = std::path::Path::new("data/cache");
    let log_dir = std::path::Path::new("data/log");

    if cache_dir.is_dir() {
        std::fs::remove_dir_all(cache_dir).unwrap_or_default();
        std::fs::create_dir_all(cache_dir).unwrap_or_default();
    }

    if log_dir.is_dir() {
        std::fs::remove_dir_all(log_dir).unwrap_or_default();
        std::fs::create_dir_all(log_dir).unwrap_or_default();
    }

    println!("{}", CACHE_CLEARED);
}