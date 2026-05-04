//! 中层状态机

/// 游戏列表页内部状态。
///
/// 此状态只在顶层状态为 `GameList` 时有效。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameListState {
    /// 游戏列表默认视图。
    ///
    /// TODO: 选择游戏时切换到 `Game`。
    /// TODO: 启动游戏时进入游戏运行时。
    List,

    /// 单个游戏详情/启动页。
    ///
    /// TODO: 返回键切换回 `List`。
    /// TODO: 选择“启动”时进入游戏运行时。
    Game,
}

/// 设置页内部状态。
///
/// 此状态只在顶层状态为 `Setting` 时有效。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettingState {
    /// 设置主页。
    ///
    /// TODO: 选择语言设置时切换到 `Language`。
    /// TODO: 选择 Mod 设置时切换到 `ModList`。
    /// TODO: 选择按键设置时切换到 `Keybind`。
    /// TODO: 选择安全设置时切换到 `Security`。
    /// TODO: 选择内存管理时切换到 `Memory`。
    Hub,

    /// 语言设置。
    ///
    /// TODO: 返回键切换回 `Hub`。
    Language,

    /// Mod 列表。
    ///
    /// TODO: 返回键切换回 `Hub`。
    /// TODO: 关闭单个 Mod 安全模式时打开 `DialogState::ModSecurityWarning`。
    ModList,

    /// 按键设置。
    ///
    /// TODO: 返回键切换回 `Hub`。
    Keybind,

    /// 安全设置。
    ///
    /// TODO: 返回键切换回 `Hub`。
    /// TODO: 关闭全局安全模式时打开 `DialogState::SecurityWarning`。
    Security,

    /// 内存清理。
    ///
    /// TODO: 返回键切换回 `Hub`。
    /// TODO: 清理缓存时打开 `DialogState::ClearCacheWarning`。
    /// TODO: 清理数据时打开 `DialogState::ClearDataWarning`。
    Memory,
}
