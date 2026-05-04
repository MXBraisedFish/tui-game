//! 弹窗状态机

/// 模态弹窗状态。
///
/// 弹窗覆盖当前页面，必须确认或取消后才能回到原页面。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogState {
    /// Mod 安全模式关闭确认弹窗。
    ///
    /// TODO: 确认后关闭弹窗并关闭当前 Mod 安全模式。
    /// TODO: 取消后关闭弹窗且不修改状态。
    ModSecurityWarning,

    /// 全局安全模式关闭确认弹窗。
    ///
    /// TODO: 确认后关闭弹窗并关闭全局默认安全模式。
    /// TODO: 取消后关闭弹窗且不修改状态。
    SecurityWarning,

    /// 清理缓存确认弹窗。
    ///
    /// TODO: 确认后关闭弹窗并执行缓存清理。
    /// TODO: 取消后关闭弹窗且不修改状态。
    ClearCacheWarning,

    /// 清理全部数据确认弹窗。
    ///
    /// TODO: 确认后关闭弹窗并执行不可逆数据清理。
    /// TODO: 取消后关闭弹窗且不修改状态。
    ClearDataWarning,
}

/// 弹窗上下文数据。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogContext {
    /// 无额外上下文。
    None,

    /// Mod 安全模式弹窗上下文。
    ///
    /// 当前项目中 UID 是完整包 UID，因此这里保存包 UID。
    ModPackage { uid: String },
}
