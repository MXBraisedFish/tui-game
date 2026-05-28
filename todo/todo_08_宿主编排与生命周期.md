# 宿主编排与生命周期

## 当前阶段目标

完善引擎的三阶段生命周期（Boot → Runtime → Shutdown），使启动流程完整、帧循环健壮、关闭流程可靠。本层是引擎的最外层编排，所有服务的初始化和销毁都在这里协调。

---

## 服务项目清单

### 8-1 Boot 启动序列

- **职责**：按正确的顺序初始化引擎的所有子系统，处理初始化失败的情况，最终产出一个就绪的 `BootOutput`（包含已初始化的 EngineServices 和 RuntimeWorld）。
- **当前状态**：基本存在。`boot/mod.rs` 中 `prepare()` 已完成：创建所有服务、扫描包、构建 RuntimeWorld。但初始化流程过于简单，缺少错误处理和依赖顺序保证。
- **待完善**：

  **初始化顺序规范化**：
  1. 解析 CLI 参数（如果需要交互模式之外的命令，先执行并可能直接退出）。
  2. 注册 panic hook（最早注册，确保后续任何步骤 panic 都能恢复终端）。
  3. 初始化日志系统（其他服务的初始化日志可以记录了）。
  4. 环境管理：确定和创建所有目录。
  5. 初始化 StorageService：读取 profiles 和 cache。
  6. 初始化 i18n：加载翻译文件。
  7. 初始化 TerminalService（但不立即 enter 终端模式——仅在 Runtime 阶段 enter）。
  8. 初始化 RenderService（此时可以获取终端尺寸）。
  9. 初始化 InputService。
  10. 初始化 Theme 系统。
  11. 初始化 KeybindingManager（从 profile 加载键位）。
  12. 初始化 ActionService（注册动作映射）。
  13. 初始化 PackageService：扫描包目录。
  14. 包状态 reconcile：与 profile 对齐。
  15. 初始化 LuaService：配置沙箱、注册 API 模块。
  16. 初始化 GameService 和 OverlayService。
  17. 初始化 UiService：注册所有页面。
  18. 初始化 EventBus：注册各服务的事件订阅。
  19. 构建 RuntimeWorld（Clock + 初始状态）。
  20. 产出 BootOutput。

  **启动错误处理**：
  - 非致命错误（如某个包解析失败）：记录日志，继续启动。
  - 致命错误（如无法创建必要目录）：显示错误信息后退出，不启动终端模式。
  - 启动诊断信息汇总：启动完成后输出一份诊断摘要到日志。

  **Loading 屏幕**：
  - 在启动过程中（特别是耗时步骤如包扫描）显示加载进度界面。
  - 加载完成后过渡到首页。

### 8-2 Runtime 运行时循环

- **职责**：驱动引擎的主循环，包含帧调度、事件处理、状态更新、渲染输出。这是引擎的核心运行逻辑。
- **当前状态**：基本可用。`runtime/mod.rs` 的 `run()` 实现了固定 16ms 间隔的帧循环：poll 输入 → 处理特定按键 → update → render → sleep。但当前运行时直接耦合了具体的按键处理逻辑和渲染细节，不够通用。
- **待完善**：

  **帧循环重构**（基于 EventBus 和宿主状态机）：
  ```
  每帧：
    1. FrameScheduler::begin_frame() → 帧号
    2. EngineClock::tick() → dt
    3. InputService::poll() → 原始输入队列
    4. ActionService::resolve() → 语义动作列表
    5. 根据宿主状态机当前状态，将动作路由到对应处理器：
       - Menu 状态：动作交给 UiService
       - InGame 状态：动作交给 GameService
       - Overlay 状态：动作交给 OverlayService
    6. 调用当前状态的 update(dt)：
       - Menu 状态：UiService.update()
       - InGame 状态：GameService.update(dt)
       - Overlay 状态：OverlayService.update(dt)
    7. 处理事件总线队列中的所有事件
    8. 调用当前状态的 render()：
       - 获取当前活动画布（UI 画布 / 游戏画布 / 覆盖层画布）
       - 合成所有可见图层
       - IncrementalRenderer::present() 输出到终端
    9. FrameScheduler::wait_for_target_fps() → 等待以维持目标帧率
    10. 检查退出条件：收到 QuitRequested 事件 → 退出循环
  ```

  **FPS 控制**：
  - `FrameScheduler::wait_for_target_fps()` 当前为空（有 TODO 注释）。
  - 按包声明的 `runtime.target_fps` 动态调整帧率（游戏中高帧率，菜单中低帧率）。
  - FPS 统计：计算实际 FPS 用于性能监控。

  **帧时间预算**：
  - 统计每帧中 poll/update/render/present 各阶段的耗时。
  - 如果某帧超出时间预算，记录警告日志但不阻塞。

### 8-3 Shutdown 关闭序列

- **职责**：按正确顺序关闭引擎，确保所有数据已保存、所有资源已释放、终端状态已恢复。
- **当前状态**：占位。`shutdown/mod.rs` 中 `close()` 方法接受三个参数但全部忽略（`_services`、`_world`、`_exit_state`），仅打印一行文字。
- **待完善**：

  **关闭步骤（逆序）**：
  1. 停止所有运行中的会话（GameService、OverlayService）。
  2. 保存所有待处理的数据（profile、cache、存档等）到 StorageService。
  3. 释放 Lua VM 资源。
  4. 退出终端模式（恢复原始终端设置、显示光标）。
  5. 写入关闭日志。
  6. 根据 ExitState 中的退出码退出进程。

  **ExitState 完善**：
  - 当前 `ExitState` 为空结构体。
  - 添加字段：退出码（`exit_code: i32`）、退出原因（`reason: ExitReason` 枚举：UserQuit / Error / Panic / 命令行子命令完成）。
  - 退出码供外部脚本判断引擎是否正常退出。

  **异常关闭 vs 正常关闭**：
  - 正常关闭：用户按 Esc 退出 → 走完整关闭流程。
  - 异常关闭：panic 触发 → panic hook 先恢复终端再记录 crash log。
  - 意外断电等不可控场景：下次启动时检测未正常关闭的标记，执行数据修复。

### 8-4 EngineClock 引擎时钟

- **职责**：追踪引擎的时间状态：自启动以来的总时间、每帧的 delta time、FPS 计算。
- **当前状态**：基本可用。`EngineClock` 已有 `tick()`、`delta_time()`、`elapsed_since_epoch()`、`fps()`、`smooth_fps()`。
- **待完善**：
  - 与 FrameScheduler 的职责划分：FPS 平滑计算和等待逻辑应归属 FrameScheduler，Clock 只负责时间测量。
  - 时间缩放（time scale）：允许慢动作/快进效果（用于调试或游戏特效）。
  - 暂停时的时间处理：暂停期间 Clock 应停止计时或记录暂停时长以补偿。

### 8-5 FrameScheduler 帧调度器

- **职责**：管理帧计数、帧率控制和帧间等待。确保引擎以稳定帧率运行，不占用 100% CPU。
- **当前状态**：骨架存在。`FrameScheduler` 有 `begin_frame()` 和空的 `wait_for_target_fps()`（有 TODO 注释）。帧率控制逻辑实际在 `runtime/mod.rs` 中通过硬编码的 `thread::sleep(Duration::from_millis(16))` 实现。
- **待完善**：
  - 实现 `wait_for_target_fps(target_fps)`：计算本帧已用时间，sleep 剩余时间。
  - 睡眠策略：短时间自旋等待（spin-wait）+ 长时间系统 sleep 的混合策略，平衡精度和 CPU 占用。
  - 帧率自适应：如果连续多帧超时，降低目标帧率并记录警告。
  - 目标帧率来源：从当前活动包的 `runtime.target_fps` 读取，或使用全局默认值（如 UI 下 30fps，游戏中 60fps）。
  - 帧跳过保护：如果某一帧耗时远超目标帧时间，跳过等待直接开始下一帧。

---

## 旧架构参考

### Boot 流程（`old_src/boot/`）
旧架构的启动流程比新骨架复杂得多：
- **cli**：先解析命令行参数。
- **crash_log + panic_hook**：最早注册。
- **loading_screen**：显示启动进度。
- **environment**：确定路径、创建目录、修复资产。
- **i18n**：加载翻译。
- **lua_runtime**：预加载 Lua 环境和 API 模块。
- **preload 阶段**：在正式进入主循环前完成所有可预加载的工作。

旧架构的 boot 阶段有"预加载"和"主加载"两个子阶段，预加载完成所有不依赖终端模式的工作，主加载在终端模式启用后完成。

### 主循环（`old_src/host_engine/runtime/event_loop.rs`）
旧架构的主循环结构：
- 事件轮询 → 事件分发 → 状态更新 → 渲染 → 帧率控制。
- 通过 `event_dispatch` 模块分发事件。
- 帧率通过包声明的 target_fps 动态调整。

旧架构的主循环与当前骨架的结构相似，但旧架构中事件分发和状态更新已经解耦。新架构需要在此基础上进一步用事件总线替代直接调用。

### Shutdown（旧架构中分散在各处）
旧架构没有集中的 shutdown 模块，清理逻辑分散在：
- 各服务的 Drop 实现。
- TerminalGuard 的 Drop 恢复终端。
- ProfileStore 的 flush 写入。

新架构将清理逻辑集中在 `shutdown/mod.rs` 中显式执行，可读性和可控性更好。

---

## 完成后可验证的可用项

1. 启动引擎后看到加载进度（如果启动耗时足够长），然后进入首页。
2. 日志文件记录完整的启动过程，包含每个服务的初始化状态。
3. 正常退出（按 Esc）后，所有数据已保存，终端恢复正常。
4. 退出码正确反映退出原因（正常退出码 0，异常退出码非 0）。
5. 帧率稳定在目标值附近，CPU 占用不异常高。
6. 游戏中帧率达到包声明的 target_fps，菜单中帧率降低以节省资源。
7. 帧时间超出预算时日志中出现警告但引擎不崩溃。
8. 异常退出后再次启动，引擎检测到异常退出记录并提示用户。
