pub mod cli;
pub mod environment;

use anyhow::Result;
use std::io;

use crossterm::cursor::Show;
use crossterm::execute;
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};

use crate::app::i18n;

/// 安装程序崩溃处理钩子。
/// panic 时自动恢复终端状态并记录错误日志。
pub fn install_panic_hook() {
    let old = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let mut out = io::stdout();
        let _ = execute!(out, Show, LeaveAlternateScreen);
        crate::utils::host_log::append_host_error(
            "host.error.program_crashed",
            &[("panic_info", &panic_info.to_string())],
        );
        old(panic_info);
    }));
}

/// 执行启动前的环境准备。
///
/// 按顺序执行：
/// 1. 处理 CLI 参数（如 `-rv` 输出版本号）
/// 2. 安装 panic hook
/// 3. 清理旧版遗留数据
/// 4. 初始化国际化
/// 5. 创建运行时目录与默认文件
///
/// 返回 `Ok(true)` 表示 CLI 已处理完毕，调用方应退出程序。
/// 返回 `Ok(false)` 表示环境准备完成，继续正常启动。
pub fn prepare_environment() -> Result<bool> {
    if cli::handle_cli_passthrough()? {
        return Ok(true);
    }

    install_panic_hook();
    environment::cleanup_legacy_runtime_data()?;
    i18n::init("us-en")?;
    environment::initialize_runtime_layout()?;

    Ok(false)
}