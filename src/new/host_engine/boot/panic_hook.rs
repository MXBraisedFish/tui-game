pub fn install() {
    let default_handler = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // 1. 恢复终端（全部忽略错误）
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stdout(), Show, LeaveAlternateScreen);

        // 2. 提取崩溃消息
        let message = panic_info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| panic_info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| format!("{:?}", panic_info));

        // 3. 尝试写日志，写不了就打印
        let _ = host_log::error("host.error.panic", &message);
        eprintln!("[FATAL] {}", message);

        // 4. 调用默认 handler
        default_handler(panic_info);
    }));
}