//! 预加载阶段：初始化终端运行环境

pub mod alternate_screen;
pub mod color_support;
pub mod ctrl_c_handler;
pub mod cursor_visibility;
pub mod input_event;
pub mod key_listener;
pub mod raw_mode;
pub mod resize_watcher;
pub mod terminal_environment;
pub mod terminal_size;

use std::sync::mpsc::Receiver;

pub use color_support::ColorSupport;
pub use input_event::HostInputEvent;
pub use resize_watcher::ResizeEvent;
pub use terminal_environment::TerminalEnvironment;
pub use terminal_size::TerminalSize;

use key_listener::KeyListener;

type InitEnvironmentResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 初始化后的宿主终端环境
pub struct InitializedEnvironment {
    pub terminal_environment: TerminalEnvironment,
    pub terminal_size: TerminalSize,
    pub color_support: ColorSupport,
    pub resize_receiver: Receiver<ResizeEvent>,
    pub input_receiver: Receiver<HostInputEvent>,
    key_listener: KeyListener,
}

/// 初始化终端环境和输入监听
pub fn initialize() -> InitEnvironmentResult<InitializedEnvironment> {
    let terminal_environment = TerminalEnvironment::enter()?;
    let terminal_size = terminal_size::current()?;
    let color_support = color_support::detect();
    let resize_watcher = resize_watcher::create();
    let (key_listener, input_receiver) = key_listener::start(resize_watcher.sender())?;

    Ok(InitializedEnvironment {
        terminal_environment,
        terminal_size,
        color_support,
        resize_receiver: resize_watcher.into_receiver(),
        input_receiver,
        key_listener,
    })
}

impl InitializedEnvironment {
    /// 确保监听句柄被视为已使用，避免后续接入运行循环前误删。
    pub fn is_input_listener_running(&self) -> bool {
        self.key_listener.is_running()
    }
}
