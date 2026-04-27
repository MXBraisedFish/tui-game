/// 定义宿主统一的输入事件类型

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputEvent {
    Action(String),
    Key(String),
    Resize { width: u16, height: u16 },
    Tick { dt_ms: u32 },
    Quit,
}
