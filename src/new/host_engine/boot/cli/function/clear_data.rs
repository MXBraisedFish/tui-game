pub fn execute() {
  // 清理所有数据
    let data_root = std::path::Path::new("data");

    if data_root.is_dir() {
        std::fs::remove_dir_all(data_root)?;
        std::fs::create_dir_all(data_root)?;
        // 重新创建子目录结构
        environment::data_dirs::ensure()?;
    }

    println!("{}", DATA_CLEARED);
}