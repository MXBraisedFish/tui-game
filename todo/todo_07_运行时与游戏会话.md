# 运行时与游戏会话层

## 当前阶段目标

将当前空白的游戏会话和覆盖层会话填充为完整的生命周期管理，建立事件总线以解耦服务间通信，实现宿主应用级的状态机。本层是"调用式→请求式"转换的核心——服务间不再直接互相调用方法，而是通过事件队列传递消息。

---

## 服务项目清单

### 7-1 GameService 游戏会话

- **职责**：管理单个游戏会话的完整生命周期：启动、运行、暂停、恢复、结束。在会话中驱动 Lua 脚本的 init→handle_event→update→render 循环。
- **当前状态**：骨架存在。仅有 `GameSessionState` 枚举（Inactive/Running/Paused）和 `start()`/`stop()` 方法，`update()` 方法体为空。
- **待完善**：

  **会话生命周期**：
  - `start(package_uid)`：加载包的入口脚本、调用 `init()`、将状态设为 Running。
  - `stop()`：调用脚本的清理逻辑（如果存在）、释放脚本资源、将状态设为 Inactive。
  - `pause()` / `resume()`：暂停和恢复游戏更新循环。
  - 会话上下文：记录当前运行的包 UID、Lua 状态引用、运行时长、帧计数。

  **帧循环集成**：
  - `handle_event(event)`：将引擎事件（按键、resize 等）传递给游戏脚本的 `handle_event` 函数。
  - `update(dt)`：调用脚本的 `update` 函数，传递 delta time。
  - `render()`：调用脚本的 `render` 函数，让脚本通过绘制 API 输出到 Canvas。
  - 帧序：handle_event → update → render（每帧按此顺序调用）。

  **游戏存档**：
  - 脚本请求存档时，GameService 接收存档数据，向 StorageService 发起保存请求。
  - 存档格式：由游戏脚本自行定义，引擎作为不透明数据块存储。
  - 读取存档：启动游戏时检查是否存在存档，如果存在则传递给 `init(save_data)`。

  **最佳成绩**：
  - 如果 `package.json` 中声明了 `game.score.enabled = true`，则启用成绩追踪。
  - 脚本提交成绩时，GameService 比较并更新最佳成绩。
  - 最佳成绩持久化到 ProfileStore。

  **游戏消息**：
  - 游戏可以向引擎发送消息（如"显示提示文字"、"震动屏幕"），引擎决定如何处理（当前阶段可以先忽略或记录日志）。

### 7-2 OverlayService 覆盖层会话

- **职责**：管理屏保和 Boss 键覆盖层的会话生命周期。屏保在用户空闲时自动启动，Boss 键在用户按下特定键时紧急切换到伪装界面。
- **当前状态**：骨架存在。仅有 `OverlayKind` 枚举（Screensaver/Boss）、`OverlaySessionState` 枚举（Inactive/Running）和 `start()`/`stop()`/`is_active()` 方法，`update()` 方法体为空。
- **待完善**：

  **屏保会话**：
  - 自动进入：用户在规定时间内无任何输入后，自动启动屏保。
  - 空闲计时器：跟踪最后一次输入的时间，与配置的屏保空闲时间比较。
  - 任意按键退出：屏保运行中，任意按键按下即退出屏保，回到之前的界面。
  - 屏保优先级：只能在非游戏状态下启动（如果游戏正在运行则屏保不触发）。

  **Boss 键会话**：
  - 热键触发：按下指定快捷键（默认由 KeybindingManager 定义）立即切换到 Boss 界面。
  - Boss 界面是一个模仿工作软件的伪装界面（如终端代码编辑器、数据分析表格等）。
  - 再次按下同一热键或 Esc 退出 Boss 界面，回到之前的界面。
  - Boss 键优先级最高：即使在游戏中也可以触发。

  **覆盖层帧循环**：
  - 覆盖层运行时，GameService 的 update/render 暂停，由覆盖层的 update/render 接管渲染输出。
  - 覆盖层也有完整的 init→update→render 生命周期。

### 7-3 EventBus 事件总线

- **职责**：引擎内部的事件分发系统。服务将事件投递到总线，总线根据事件类型路由到订阅了该事件的服务。这是"请求式"架构的核心基础设施。
- **当前状态**：不存在。当前引擎中服务间通过直接方法调用通信（如 `runtime/mod.rs` 中直接调用 `services.game.start()`），运行时模块完全了解所有服务的内部接口。
- **待完善**：

  **事件类型定义**：
  - 输入事件：`KeyPressed(key)`、`KeyReleased(key)`、`MouseClicked(x, y, button)`
  - 系统事件：`Resized(w, h)`、`LanguageChanged(lang)`、`ThemeChanged(theme)`、`ConfigChanged(key, value)`
  - 服务事件：`GameStarted(uid)`、`GameStopped(uid)`、`OverlayStarted(kind)`、`OverlayStopped(kind)`、`PackageInstalled(uid)`、`PackageRemoved(uid)`
  - 应用事件：`QuitRequested`、`PageChanged(page_key)`

  **事件分发机制**：
  - 订阅：服务在初始化时声明对哪些事件类型感兴趣。
  - 投递：服务将事件推入总线。
  - 路由：总线根据事件类型查找订阅者，按序通知。
  - 队列处理：事件在当前帧收集，在帧末尾统一处理（或在下一帧开始前处理），避免事件处理过程中产生新事件导致的级联问题。

  **与"调用式→请求式"的关系**：
  - 调用式：`services.game.start(uid)` —— 调用者需要知道 GameService 的存在和接口。
  - 请求式：`event_bus.post(GameStartRequest { uid })` —— 调用者只需要知道事件类型，不需要知道谁在处理。
  - 请求式的好处：服务可以独立开发、测试、替换，运行时只是把事件总线串起来。

### 7-4 宿主状态机（Host State Machine）

- **职责**：管理宿主应用的顶层状态（Loading → Menu → InGame → Overlay → Shutdown），定义合法的状态转换，确保引擎在任何时候都处于明确定义的状态。
- **当前状态**：不存在。当前运行时无显式状态机，仅通过 `running: bool` 控制循环。
- **待完善**：

  **状态定义**：
  - `Loading`：启动阶段，加载配置、扫描包、初始化服务。
  - `Menu`：主菜单/UI 导航状态，用户在游戏列表和设置页面之间浏览。
  - `InGame`：游戏运行中，渲染和输入由 GameService 接管。
  - `Overlay`：屏保或 Boss 界面运行中，渲染和输入由 OverlayService 接管。
  - `Shutdown`：关闭阶段，保存状态、释放资源、恢复终端。

  **状态转换规则**：
  - `Loading → Menu`：启动完成。
  - `Menu → InGame`：用户选择了游戏并启动。
  - `InGame → Menu`：游戏结束或用户退出。
  - `Menu → Overlay`：屏保超时或 Boss 键触发。
  - `InGame → Overlay`：Boss 键触发（仅 Boss 键，屏保在游戏中不触发）。
  - `Overlay → Menu`：屏保退出或 Boss 键再次按下。
  - `* → Shutdown`：用户请求退出或系统错误。

  **状态进入/退出回调**：
  - 进入状态时执行初始化逻辑。
  - 退出状态时执行清理逻辑。
  - 状态转换通过 EventBus 广播，其他服务可以监听。

---

## 旧架构参考

### Game Engine（`old_src/host_engine/runtime/game_engine/`）
旧架构的游戏引擎包含：
- **action_map**：游戏动作映射表，来自 `package.json` 的 `game.actions` 定义，将游戏自定义动作映射到实际按键。
- **best_score**：最佳成绩管理，支持读写和比较。
- **script_loader**：游戏脚本加载器，处理入口脚本的加载和初始化。
- **session**：游戏会话，协调脚本生命周期、事件处理、更新和渲染。

旧架构中 GameEngine 直接持有对 PackageRegistry、ProfileStore 等的引用。新架构中 GameService 不持有这些引用，而是通过事件总线或请求方式获取所需数据。

### Overlay（`old_src/host_engine/runtime/overlay/`）
旧架构的覆盖层实现了完整的屏保和 Boss 键功能：
- 空闲计时器独立运行。
- Boss 键热键为全局快捷键，优先级最高。
- 覆盖层有自己的渲染循环，与主渲染切换。

### Event Dispatch（`old_src/host_engine/runtime/event_dispatch.rs`）
旧架构有一个事件分发模块，但实现相对简单。新架构可以在此基础上设计更完善的事件系统。

### State Machine（旧架构中的隐式状态）
旧架构没有显式的状态机，状态通过多个布尔标志和枚举组合隐式表达，这导致了复杂的状态判断逻辑。新架构应该用显式状态机替代，使状态转换清晰可见。

---

## 完成后可验证的可用项

1. 从菜单选择游戏后，游戏正常启动，Lua 脚本开始运行。
2. 游戏中按 Esc 返回菜单，GameService 状态变为 Inactive。
3. 游戏运行中按 Boss 键，Boss 界面弹出覆盖游戏画面，再次按下 Boss 键返回游戏。
4. 在菜单页面闲置超过设定时间后，屏保自动启动。
5. 屏保运行中按任意键，屏保退出回到菜单。
6. 游戏中不触发屏保（仅 Boss 键可覆盖游戏）。
7. 游戏提交的成绩被保存，再次启动游戏时显示最佳成绩。
8. 游戏存档数据在重启引擎后仍可读取。
9. 事件总线中订阅者正确收到事件，未订阅的服务不收到无关事件。
