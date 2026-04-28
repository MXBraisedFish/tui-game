// 二进制 crate 的入口，按顺序调用启动、加载和主循环子系统。是启动流程的最高级编排

// 错误处理类型，run() 函数返回 Result
use anyhow::Result;

use tui_game::app::content_cache; // 内容缓存模块，用于加载画面进度和缓存预热
use tui_game::app::i18n; // 国际化模块，获取加载画面的本地化文本

const RUNTIME_VERSION: &str = env!("CARGO_PKG_VERSION"); // 从 Cargo.toml 获取的编译时版本号,传递给版本检查

// 程序入口，调用 run() 并处理顶层错误
fn main() {
    if let Err(err) = run() {
        let err_text = format!("{err:#}");
        tui_game::utils::host_log::append_host_error("host.error.raw", &[("err", &err_text)]);
    }
}

// 编排启动流程：准备环境 → 显示加载画面 → 预热缓存 → 启动版本检查 → 进入主循环
fn run() -> Result<()> {
    // 环境准备（失败则退出）
    if tui_game::startup::prepare_environment()? {
        return Ok(());
    }

    // 打开终端会话
    let mut session = tui_game::terminal::session::TerminalSession::new()?;
    // 显示 0% 加载画面
    tui_game::app::loading_screen::render_loading_screen(
        &mut session.terminal,
        &content_cache::LoadingProgress {
            percent: 0,
            message: i18n::t_or("loading.startup.preparing", "Preparing startup..."),
        },
    )?;

    // 预热缓存并更新加载画面
    content_cache::reload_with_progress(|progress| {
        let _ = tui_game::app::loading_screen::render_loading_screen(
            &mut session.terminal,
            &content_cache::LoadingProgress {
                percent: progress.percent,
                message: progress.message,
            },
        );
    });

    // 规范化版本号
    let runtime_version = tui_game::app::version_check::normalize_tag(RUNTIME_VERSION);
    // 启动后台版本检查线程
    let update_check_rx = tui_game::app::version_check::spawn_update_check(runtime_version.clone());

    // 进入 TUI 主循环
    tui_game::app::main_loop::run(session, runtime_version, update_check_rx)
}