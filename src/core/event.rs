/// 宿主统一输入事件。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputEvent {
    Action(String),
    Key(String),
    Resize { width: u16, height: u16 },
    Tick { dt_ms: u32 },
    Quit,
}
