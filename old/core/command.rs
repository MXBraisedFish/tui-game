// 定义游戏运行时可向宿主（终端应用）发送的命令枚举，用于游戏逻辑与宿主之间的双向通信。属于 core 模块的基础消息类型

// 命令枚举
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeCommand {
    ExitGame,
    SkipEventQueue,
    ClearEventQueue,
    RenderNow,
    SaveBestScore,
    SaveGame,
    ShowToast { message: String },
}
