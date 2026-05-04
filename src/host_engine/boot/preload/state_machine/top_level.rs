//! 顶层状态机

/// 顶层页面状态。
///
/// 顶层页面之间互斥，同一时间只有一个页面处于活动状态。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TopLevelState {
    /// 首页。应用启动后默认进入此状态。
    ///
    /// TODO: 选择“游戏列表”时切换到 `GameList`。
    /// TODO: 选择“设置”时切换到 `Setting`。
    /// TODO: 选择“关于”时切换到 `About`。
    /// TODO: 选择“退出”时进入关闭流程。
    Home,

    /// 游戏列表页。
    ///
    /// TODO: 返回键切换回 `Home`。
    /// TODO: 选择游戏后由中层状态机进入游戏详情页。
    /// TODO: 启动游戏时退出宿主页面栈，进入游戏运行时。
    GameList,

    /// 设置主页。
    ///
    /// TODO: 返回键切换回 `Home`。
    /// TODO: 选择设置项时切换对应设置中层状态。
    Setting,

    /// 关于页。
    ///
    /// TODO: 返回键切换回 `Home`。
    About,
}
