/// 程序入口。
/// 负责按顺序调用启动、加载、主循环等子系统。
use std::time::Duration;

use anyhow::Result;

use tui_game::app::content_cache;
use tui_game::app::i18n;
use tui_game::app::main_loop;

const RUNTIME_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    if let Err(err) = run() {
        let err_text = format!("{err:#}");
        tui_game::utils::host_log::append_host_error("host.error.raw", &[("err", &err_text)]);
    }
}

fn run() -> Result<()> {
    if tui_game::startup::prepare_environment()? {
        return Ok(());
    }

    let mut session = tui_game::terminal::session::TerminalSession::new()?;
    tui_game::app::loading_screen::render_loading_screen(
        &mut session.terminal,
        &content_cache::LoadingProgress {
            percent: 0,
            message: i18n::t_or("loading.startup.preparing", "Preparing startup..."),
        },
    )?;
    content_cache::reload_with_progress(|progress| {
        let _ = tui_game::app::loading_screen::render_loading_screen(
            &mut session.terminal,
            &content_cache::LoadingProgress {
                percent: progress.percent,
                message: progress.message,
            },
        );
    });

    let runtime_version = tui_game::app::version_check::normalize_tag(RUNTIME_VERSION);
    let update_check_rx = tui_game::app::version_check::spawn_update_check(runtime_version.clone());

    tui_game::app::main_loop::run(session, runtime_version, update_check_rx)
}