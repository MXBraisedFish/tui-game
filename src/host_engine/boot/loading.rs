//! 启动阶段加载进度显示线程
//! 使用普通终端输出，不接管 TUI 终端状态

use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::host_engine::boot::i18n;

/// 进度条宽度（字符数）
const PROGRESS_BAR_WIDTH: usize = 20;
/// 最小进度百分比
const MIN_PERCENT: u8 = 0;
/// 最大进度百分比
const MAX_PERCENT: u8 = 100;
/// 加载动画每帧间隔
const SPINNER_FRAME_INTERVAL_MS: u64 = 200;
/// 加载动画帧
const SPINNER_FRAMES: [&str; 8] = ["⠇", "⠋", "⠙", "⠸", "⢰", "⣠", "⣄", "⡆"];

// ========== ANSI 转义序列常量 ==========

/// 重置所有样式
const ANSI_RESET: &str = "\x1b[0m";
/// 白色前景色
const ANSI_WHITE: &str = "\x1b[37m";
/// 灰色前景色
const ANSI_GRAY: &str = "\x1b[90m";
/// 绿色前景色
const ANSI_GREEN: &str = "\x1b[32m";
/// 黄色前景色
const ANSI_YELLOW: &str = "\x1b[33m";
/// 清除当前行
const ANSI_CLEAR_LINE: &str = "\x1b[2K";
/// 光标上移一行
const ANSI_CURSOR_UP_ONE_LINE: &str = "\x1b[1A";

/// 加载模块结果类型
type LoadingResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 加载阶段枚举。第六步资源准备通过这些阶段更新进度。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadingStage {
    /// 初始化环境
    InitEnv,
    /// 读取游戏模块
    ScanGame,
    /// 扫描 UI
    ScanUi,
    /// 读取数据
    ReadData,
    /// 预缓存
    PreCache,
    /// 准备启动
    ReadyLaunch,
    /// 完成
    Complete,
}

/// 加载进度事件
#[derive(Clone, Debug)]
pub struct LoadingProgress {
    /// 当前进度百分比 (0-100)
    pub percent: u8,
    /// 当前加载阶段
    pub stage: LoadingStage,
}

/// 加载线程句柄
/// 用于向加载线程发送进度更新并等待其结束
pub struct LoadingHandle {
    /// 进度消息发送端
    progress_sender: Sender<LoadingMessage>,
    /// 线程句柄
    thread_handle: Option<JoinHandle<()>>,
}

/// 加载线程内部消息类型
#[derive(Clone, Debug)]
enum LoadingMessage {
    /// 进度更新消息
    Progress(LoadingProgress),
    /// 完成加载消息
    Finish,
}

/// 启动加载进度线程
pub fn start() -> LoadingHandle {
    let (progress_sender, progress_receiver) = mpsc::channel();
    let thread_handle = thread::spawn(move || run_loading_thread(progress_receiver));

    LoadingHandle {
        progress_sender,
        thread_handle: Some(thread_handle),
    }
}

impl LoadingHandle {
    /// 更新加载进度
    /// 发送进度消息到加载线程，百分比会自动钳位到 0-100
    pub fn update(&self, stage: LoadingStage, percent: u8) -> LoadingResult<()> {
        let progress = LoadingProgress {
            percent: percent.clamp(MIN_PERCENT, MAX_PERCENT),
            stage,
        };
        self.progress_sender
            .send(LoadingMessage::Progress(progress))?;
        Ok(())
    }

    /// 完成加载并等待线程退出
    /// 发送 100% 完成进度，发送 Finish 信号，等待线程结束
    pub fn finish(mut self) -> LoadingResult<()> {
        let _ = self
            .progress_sender
            .send(LoadingMessage::Progress(LoadingProgress {
                percent: MAX_PERCENT,
                stage: LoadingStage::Complete,
            }));
        let _ = self.progress_sender.send(LoadingMessage::Finish);

        if let Some(thread_handle) = self.thread_handle.take() {
            let _ = thread_handle.join();
        }

        Ok(())
    }
}

/// 加载线程主函数
/// 循环接收消息并更新终端显示
fn run_loading_thread(progress_receiver: Receiver<LoadingMessage>) {
    let mut has_rendered = false;
    let mut latest_progress = LoadingProgress {
        percent: MIN_PERCENT,
        stage: LoadingStage::InitEnv,
    };
    let mut spinner_index = 0usize;

    loop {
        match progress_receiver.recv_timeout(Duration::from_millis(SPINNER_FRAME_INTERVAL_MS)) {
            Ok(LoadingMessage::Progress(progress)) => {
                latest_progress = progress;
            }
            Ok(LoadingMessage::Finish) => {
                if has_rendered {
                    println!();
                }
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if has_rendered {
                    println!();
                }
                break;
            }
        }

        render_progress(latest_progress.clone(), has_rendered, spinner_index);
        spinner_index = (spinner_index + 1) % SPINNER_FRAMES.len();
        has_rendered = true;
    }
}

/// 渲染进度条到终端
/// 使用 ANSI 转义序列实现单行动态更新
fn render_progress(progress: LoadingProgress, has_rendered: bool, spinner_index: usize) {
    let mut stdout = io::stdout();
    // 非首次渲染时，光标上移一行覆盖之前的进度行
    if has_rendered {
        let _ = write!(stdout, "{ANSI_CURSOR_UP_ONE_LINE}");
    }

    let stage_text = stage_text(progress.stage);
    let spinner_frame = SPINNER_FRAMES[spinner_index % SPINNER_FRAMES.len()];
    let progress_bar = format_progress_bar(progress.percent);
    let _ = write!(
        stdout,
        "{ANSI_CLEAR_LINE}{progress_bar} {ANSI_WHITE}{}%{ANSI_RESET}\n{ANSI_CLEAR_LINE}{ANSI_YELLOW}{spinner_frame}{ANSI_RESET} {ANSI_WHITE}{stage_text}{ANSI_RESET}\r",
        progress.percent
    );
    let _ = stdout.flush();
}

/// 格式化进度条字符串
/// 格式：`[####........]`，已完成部分为绿色 `#`，未完成部分为灰色 `.`
fn format_progress_bar(percent: u8) -> String {
    let completed_width = (usize::from(percent) * PROGRESS_BAR_WIDTH) / usize::from(MAX_PERCENT);
    let pending_width = PROGRESS_BAR_WIDTH.saturating_sub(completed_width);

    format!(
        "{ANSI_WHITE}[{ANSI_GREEN}{}{ANSI_GRAY}{}{ANSI_WHITE}]{ANSI_RESET}",
        "#".repeat(completed_width),
        ".".repeat(pending_width)
    )
}

/// 获取加载阶段的本地化文本
fn stage_text(stage: LoadingStage) -> &'static str {
    let loading_text = i18n::text().loading;
    match stage {
        LoadingStage::InitEnv => loading_text.init_env,
        LoadingStage::ScanGame => loading_text.scan_game,
        LoadingStage::ScanUi => loading_text.scan_ui,
        LoadingStage::ReadData => loading_text.read_data,
        LoadingStage::PreCache => loading_text.pre_cache,
        LoadingStage::ReadyLaunch => loading_text.ready_launch,
        LoadingStage::Complete => loading_text.complete,
    }
}
