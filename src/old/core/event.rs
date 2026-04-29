// 定义宿主向游戏发送的统一输入事件类型，作为终端事件（crossterm）、全局热键（rdev）、定时器等外部输入的抽象层

// 输入事件类型枚举
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputEvent {
    Action(String),
    Key(String),
    Resize { width: u16, height: u16 },
    Tick { dt_ms: u32 },
    Quit,
}
