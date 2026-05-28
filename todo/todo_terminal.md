# 终端接管分析：老版本(old_src) vs 新版本(src)

## 一、老版本终端接管架构总览

老版本采用**多层分层**的终端接管设计，核心分为四大子系统：

### 1. 终端环境初始化 (`boot/preload/init_environment/`)

这是整个终端接管的**基础设施层**，在加载阶段就已启动。

| 模块 | 职责 |
|------|------|
| `raw_mode.rs` | crossterm raw mode 的开关 |
| `alternate_screen.rs` | 进入/退出备用屏幕 |
| `cursor_visibility.rs` | 隐藏/显示光标 |
| `terminal_environment.rs` | 汇总上述三者 + 创建 **ratatui Terminal** 实例。Drop 时自动恢复终端 |
| `terminal_size.rs` | 读取终端尺寸，默认 80x24 |
| `color_support.rs` | 通过 `COLORTERM`/`TERM` 环境变量检测 TrueColor/256色/基本色 |
| `resize_watcher.rs` | 创建 mpsc channel 用于传递 resize 事件 |
| `ctrl_c_handler.rs` | 检测 Ctrl+C 按键组合 |
| `input_event.rs` | 定义 `HostInputEvent` 枚举（Key/Resize/FocusGained/FocusLost/ExitRequested） |
| `key_listener.rs` | **核心**，见下文 |

### 2. 双引擎键盘监听 (`key_listener.rs`) —— 老版本最大的亮点

老版本使用 **crossterm + rdev 双重监听**融合架构：

- **Crossterm 线程**（16ms 轮询）：捕获 Key(仅 Press)、Resize、FocusGained、FocusLost、Ctrl+C→ExitRequested
- **rdev 线程**（全局回调）：捕获**操作系统级别**的 KeyPress 和 KeyRelease，推入延迟队列
- **融合逻辑** (`drain_ready_rdev_keys`)：
  - rdev 事件延迟 20ms 后再处理（让 crossterm 先报告）
  - 若同一按键在 120ms 内已被 crossterm 报告，则**抑制** rdev 的 press 事件（防止重复）
  - rdev 提供 **key release 事件**（crossterm 通常无法可靠捕获）
  - 最终输出：每条按键都带有 `"press"` 或 `"release"` 状态

这解决了 crossterm 在多数终端上**无法获取 Release 事件**的固有问题。

- **语义化按键映射**：无论是 crossterm 的 `KeyCode` 还是 rdev 的 `RdevKey`，都映射为统一的字符串名称（如 `"a"`, `"f1"`, `"left_ctrl"`, `"esc"`, `"space"` 等）
- **白名单机制**：定义了完整的 crossterm 按键白名单（字母、数字、符号、F1-F12、所有导航键、CapsLock/NumLock/ScrollLock/PrintScreen/Pause/Menu）

### 3. 增量渲染器 (`runtime/incremental_renderer/`)

这是老版本渲染的核心，使用了 **ratatui** 作为后端：

- **`FrameCache`**：将上一帧画布保存为 `BTreeMap<y, BTreeMap<x, CanvasCell>>` 快照
- **`diff_frames()`**：逐行逐列比较两帧，将连续相同样式的变化单元格**合并为渲染段**（RenderSegment）
- **`terminal_output.rs`**：
  - 维护终端的**样式状态缓存**（当前 fg/bg/attrs）
  - 只在样式变化时发送 ANSI 转义序列
  - 仅移动光标到变化位置并输出变更文本
- **`color_style.rs`**：支持 `#RRGGBB`、`rgb(r,g,b)`、16 色名称、ANSI 0-15 色号，支持粗体/斜体/下划线/删除线/闪烁/反色/隐藏/暗淡共 8 种样式
- 尺寸变化或显式请求时触发**全量重绘**

### 4. 帧率控制 (`runtime/frame_rate/`)

- **Root UI**：正常 60fps，空闲时降为 24fps
- **游戏**：目标帧率由游戏配置，AFK 时降为 24fps
- **覆盖层**：固定 24fps
- 通过 `mark_input()` 跟踪用户活跃度
- 基于空闲超时自动进入屏保

### 5. 运行时终端接管 (`runtime/terminal.rs`)

- `RuntimeTerminalSession`：包装 `TerminalEnvironment`
- `enter()`：进入 raw mode + alt screen + 隐藏光标 + 创建 ratatui terminal
- `force_restore()`：独立函数，用于 panic hook 和崩溃路径强制恢复终端

### 6. 启动流程 (old main.rs)

```
CLI命令处理 → panic钩子安装 → 环境准备 → i18n加载 → 加载屏幕 →
输入监听启动(initialize()) → 游戏/覆盖层模块扫描 → 持久化数据 →
缓存数据 → Lua运行时 → 关闭加载屏幕 →
终端接管(runtime/terminal::enter()) → 主事件循环 → 关闭流程
```

关键点：**输入监听在加载阶段就启动了，终端接管才在进入事件循环前的一刻进行**，这样加载阶段的进度条等普通终端输出不会被 alt screen 吞掉。

---

## 二、新版本的终端接管现状

新版本只有最基础的骨架：

| 模块 | 有的 | 缺失的 |
|------|------|--------|
| `terminal.rs` | raw mode、alt screen、隐藏光标、Drop 恢复 | **无 ratatui**（直接裸 crossterm）、**无 force_restore**（无法在 panic 时恢复终端） |
| `input.rs` | VecDeque 按键缓冲、Press/Release/Repeat 类型 | **无 rdev 监听**=无可靠 Release 事件、**无 resize 事件**、**无 focus 事件**、**无 Ctrl+C**、**无语义化映射**、**无白名单** |
| `render.rs` | 逐行字符串缓冲、居中绘制 | **无增量 diff 渲染**、**无样式/颜色支持**、每帧全屏清空重绘 |
| `runtime/mod.rs` | 基本帧循环 | **固定 16ms sleep** 无帧率控制、硬编码按键匹配、无事件队列、无 focus/resize 处理、无屏保/覆盖层 |

---

## 三、关键差异对照表

| 能力 | 老版本 | 新版本 |
|------|--------|--------|
| 原始模式 + 备用屏幕 | ✅ | ✅ |
| 光标隐藏/显示 | ✅ | ✅ |
| **ratatui 终端** | ✅ CrosstermBackend | ❌ 裸 crossterm |
| **Key Release 事件** | ✅ (rdev) | ❌ |
| **双引擎按键监听** | ✅ crossterm + rdev | ❌ 仅 crossterm |
| **Resize 事件** | ✅ channel 广播 | ❌ |
| **FocusGained/Lost** | ✅ | ❌ |
| **Ctrl+C → 退出信号** | ✅ | ❌ |
| **语义化按键名** | ✅ 统一字符串 | ❌ 裸 KeyCode 枚举 |
| **增量渲染** | ✅ diff + 段合并 | ❌ 每帧全屏清空重绘 |
| **样式/颜色** | ✅ 全色彩+8种样式 | ❌ 纯文本 |
| **帧率控制** | ✅ 三模式自适应 | ❌ 固定 sleep 16ms |
| **空闲检测/屏保** | ✅ | ❌ |
| **覆盖层会话** | ✅ 屏保/Boss 键 | ❌ |
| **Panic 恢复** | ✅ force_restore() | ❌ |
| **事件队列(256上限)** | ✅ | ❌ |
| **颜色能力检测** | ✅ TrueColor/256/Basic | ❌ |

---

## 四、总结

新版本的终端接管只实现了"进入 raw mode、切 alt screen、读按键、画文本"这四件事的最基本形态，而老版本拥有一套完整的、生产级别的 TUI 终端基础设施。差距主要体现在：

1. **输入系统**：双引擎监听是最大的缺失，没有它就无法获得可靠的 release 事件
2. **渲染系统**：缺少增量渲染意味着每帧都要全量刷新，性能差且会有闪烁
3. **ratatui 集成**：老版本用了 ratatui 的 CrosstermBackend，这是成熟的 TUI 后端
4. **事件体系**：老版本有完整的事件类型（Key/Resize/Focus/Exit），新版本只有 Key
5. **健壮性**：缺少 panic 恢复、颜色能力检测、帧率自适应等
