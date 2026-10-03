# Lua Runtime 协议

本文档描述当前已实现的第一阶段 Lua Runtime。每个游戏和屏保拥有独立的 Lua 5.4 VM；同一时间最多存在一个游戏 Session 和一个屏保 Session，均由 Runtime 主线程驱动。

## 入口与生命周期

游戏包的 `game.json` 或屏保包的 `screensaver.json` 中，`entry` 相对于包内 `scripts/` 目录；省略 `.lua` 后缀时由宿主补齐。入口必须是 UTF-8 文本，最大 1 MiB，规范路径不得逃出 `scripts/`。

游戏和屏保均必须实现：

```lua
function Init(ctx) end
function HandleEvent(event) end
function Update(dt) end
function UpdateFrame(dt, alpha) end
function Render() end
```

游戏可以额外实现：

```lua
function SaveGame() return nil end
function SaveBest() return nil end
```

回调发现阶段若必需字段缺失或不是函数，Session 不会启动。

## 初始化上下文

```lua
ctx = {
  package_id = "example_game",
  package_type = "game", -- 或 "screensaver"
  base = { width = 120, height = 40 },
  start_mode = "new", -- 或 "continue"
  best_data = nil,
  continue_data = nil,
  api_version = 1,
}
```

`best_data` 和 `continue_data` 不存在时，对应字段不会出现在 `ctx` 中。Lua 不会获得物理终端宽高；尺寸查询仅限当前 Base 画布和脚本持有的切片图层。

`draw` 当前只提供 `width` 和 `height`，尚未提供绘制函数。

## 事件

所有事件均具有统一外层：

```lua
{
  type = "action",
  sequence = 1,
  frame = 100,
  data = {}
}
```

支持的类型：

- `key`：`data.key` 和 `data.state`，用 ime.receive_key_event() 开启；默认关闭。
- `action`：`data.action` 和 `data.state`（`pressed`、`held`、`released`）。
- `mouse`：`data.kind`、`button`、`scroll`、`x`、`y`。
- `resize`：`data.width`、`height`。
- `focus`：`data.gained`。
- `screensaver_started`、`screensaver_stopped`：空 `data`。

Lua 可通过 ime 开启规范化 key 状态事件；不接收原始终端对象或输入法提交文字。动作名来自当前用户映射，全部命中动作依次发送。宿主层优先，各层 priority 降序、实际命中组合键优先，再按声明顺序。held 只在 pressed 后下一宿主帧发送一次，持续移动在 Update 中使用脚本保存的按住状态。

Runtime 每帧最多向 Lua 分发 128 个事件，剩余事件保留到下一帧，且不会在回调中递归分发新事件。

## 更新与屏保

- `Update` 固定为 60 Hz。
- 真实帧增量最大按 250 ms 计算，每个呈现帧最多追赶 8 次固定更新。
- 每帧依次执行事件、`Update`、`UpdateFrame(real_dt, alpha)`、活跃画面的 `Render`。
- `game.target_fps` 只控制呈现频率。
- 屏保激活期间游戏继续执行 `Update` 和 `UpdateFrame`，但跳过游戏 `Render`。
- 屏保激活期间，动作和鼠标事件不会进入任何 Lua Session；`resize` 与 `focus` 同时投递给游戏和屏保。

## 沙箱

可用基础函数：

`assert`、`error`、`pcall`、`xpcall`、`ipairs`、`pairs`、`next`、`select`、`tonumber`、`tostring`、`type`、`rawequal`、`rawlen`、`_VERSION`。

可用标准库：`math`、`string`、`utf8`、`table`。

以下函数存在，但调用会返回明确的“暂未支持”错误：

`print`、`warn`、`collectgarbage`、`math.random`、`math.randomseed`、`getmetatable`、`setmetatable`。

以下内容不存在：

`load`、`loadfile`、`loadstring`、`dofile`、`require`、`rawget`、`rawset`、`os`、`io`、`debug`、`package`、`coroutine`。

## 资源限制与故障

- 每个 VM 最大 32 MiB。
- 每 1000 条 Lua 指令检查一次预算。
- 入口、`Init`、保存回调：100 ms、100 万指令。
- `HandleEvent`、`Update`、`UpdateFrame`、`Render`：8 ms、20 万指令。
- 每次回调后执行一次增量 GC；停止 Session 时执行最终 GC。

游戏故障会销毁当前游戏 Session、恢复宿主帧率并返回游戏列表。屏保故障只移除屏保 Session 和显示层，不影响仍在运行的游戏。Lua 错误不会触发宿主 Panic。

## 保存值

`SaveGame` 和 `SaveBest` 的返回值只进行 JSON 兼容性验证，本阶段不会写入磁盘。允许 `nil`、布尔、有限数值、UTF-8 字符串、连续数组和字符串键对象。

拒绝函数、线程、userdata、循环引用、混合键表、非字符串对象键、非有限数值；最大深度 32，序列化结果最大 1 MiB。

## 本阶段明确不支持

`require`、辅助 Lua 文件、绘制函数、存储 API、资源 API、音频、动画、模块加载和 Lua 异步 API。
