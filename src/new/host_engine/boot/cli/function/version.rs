pub fn execute() {
    // 获取宿主版本
    let current = HOST_VERSION;

    // 获取线上版本
    let latest = fetch_online_version().unwrap_or_default();

    // 格式化比较
    let is_newer = parse_and_compare(latest, current);

    // 是否需要更新提示
    if is_newer {
        println!("{}", VERSION_UPDATE_AVAILABLE);
    } else {
        println!("{}", VERSION_UP_TO_DATE);
    }
}