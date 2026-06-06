// 第一层
// Boot -> 启动服务/CLI指令处理
// Init -> 初始化服务
// Runtime -> 运行时逻辑
// Shutdown -> 关闭服务/清理资源
// Stopped -> 停止服务/退出程序
// 要同步更新Panic的枚举

// 第二层(Runtime)
// Host -> 宿主环境
// Game -> 游戏环境
// Overlay -> 叠加层

// 第三层(Host)
// UiTree -> UI树（这部分是树形结构图）

// 额外服务
// context -> 上下文管理