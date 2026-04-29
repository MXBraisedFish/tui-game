pub fn execute() {
  // 获取路径并输出
    let exe_path = std::env::current_exe().unwrap_or_default();
    let root_dir = exe.parent().unwrap_or(std::path::Path::new("."));
    println!("{}", root_dir.display());
}