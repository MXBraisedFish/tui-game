// startup 模块入口，声明 cli 和 environment 子模块，并提供一个统一的 prepare_environment() 入口函数，按顺序编排整个启动流程

// 命令行参数处理
pub mod cli;

// 环境准备：清理旧数据、创建目录、初始化默认文件
pub mod environment;

use anyhow::Result; // 错误处理，prepare_environment 返回 Result
use std::io; // 标准输出句柄，用于 panic hook 中恢复终端

use crossterm::cursor::Show; // 显示光标，panic 时恢复
use crossterm::execute; // 执行终端指令
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode}; // 退出 alternate screen 和 raw mode

use crate::app::i18n; // 初始化国际化

// 安装程序崩溃处理钩子。panic 发生时自动恢复终端（关闭 raw mode、显示光标、退出 alternate screen），记录错误日志，然后调用原有的 panic handler
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

// 按顺序编排启动流程：CLI 处理 → panic hook → 清理旧数据 → 初始化国际化 → 创建目录结构。返回 Ok(true) 表示 CLI 已处理应退出，Ok(false) 表示继续启动
pub fn prepare_environment() -> Result<bool> {
    if cli::handle_cli_passthrough()? {
        return Ok(true);
    }

    install_panic_hook();
    environment::cleanup_legacy_runtime_data()?;
    i18n::init("us_en")?;
    environment::initialize_runtime_layout()?;

    Ok(false)
}
