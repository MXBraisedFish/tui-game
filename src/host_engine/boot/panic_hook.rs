//! Rust panic 钩子安装模块
//!
//! 本模块只负责安装 hook，终端恢复与崩溃日志记录分别由 runtime::terminal
//! 和 boot::crash_log 提供。

type PanicHookResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 安装宿主 panic 钩子。
pub fn install() -> PanicHookResult<()> {
    let previous_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        crate::host_engine::runtime::terminal::force_restore();
        let _ = crate::host_engine::boot::crash_log::record_panic(info);
        previous_hook(info);
    }));

    Ok(())
}
