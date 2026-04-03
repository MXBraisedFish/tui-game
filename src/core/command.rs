/// Lua 游戏运行时向宿主提交的高层意图。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeCommand {
    ExitGame,
    SaveRequest,
    ClearSave,
    RefreshBestScore,
    ShowToast { message: String },
}
