/// 定义运行时命令枚举，用于游戏与宿主之间的通信

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
