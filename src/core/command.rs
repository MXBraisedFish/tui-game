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
