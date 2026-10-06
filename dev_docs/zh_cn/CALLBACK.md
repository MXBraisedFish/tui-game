# Lua 生命周期回调

本文档说明游戏和屏保 Lua Session 使用的生命周期回调，包括各回调的职责、调用时机、参数格式、返回值和运行限制。

入口生命周期回调由宿主调用。计时器可以通过 timer.create 或 timer.set 指定 callback；指定后，该计时器事件只交给这个函数，未指定时交给 HandleEvent。回调使用相同的事件表和执行限额。其他服务的独立 callback 注册尚未开放。可用库见⌊[LUA_COMPATIBILITY.md](LUA_COMPATIBILITY.md)⌉。

## 1. 回调总览

入口脚本在加载完成后，宿主会从脚本环境中查找以下回调：

| 回调                     | 游戏     | 屏保   | 主要用途                                   |
| ------------------------ | -------- | ------ | ------------------------------------------ |
| `Init(ctx)`              | 必需     | 必需   | 初始化脚本状态并读取宿主提供的启动数据。   |
| `HandleEvent(event)`     | 必需     | 必需   | 接收宿主路由给当前 Session 的事件。        |
| `Update(dt)`             | 必需     | 必需   | 按固定的 60 Hz 步长更新确定性逻辑。        |
| `UpdateFrame(dt, alpha)` | 必需     | 必需   | 每个宿主帧更新一次与真实帧时间有关的逻辑。 |
| `Render()`               | 必需     | 必需   | 为当前可见画面提交绘制命令。               |
| `SaveGame()`             | 条件必需 | 不使用 | 返回“继续游戏”槽位所需的数据。             |
| `SaveBest()`             | 条件必需 | 不使用 | 返回游戏最佳记录及其展示文本。             |

前五个回调缺失或不是函数时，Session 创建失败，游戏不会进入运行状态，屏保也不会覆盖当前画面。

对于游戏：

- `game.json` 中 `save_game = true` 时，`SaveGame` 为必需回调。
- `game.json` 中 `best_score.enable = true` 时，`SaveBest` 为必需回调。
- 对应功能未开启时，保存回调可以省略；即使定义，宿主也不会通过该功能调用它。

屏保不使用 `SaveGame` 和 `SaveBest`。屏保脚本即使声明这两个函数，宿主也不会将其注册为屏保生命周期回调。

## 2. 每帧调用顺序

Session 创建成功后，正常 Runtime 帧中的 Lua 调用顺序为：

```text
投递本帧事件
  ↓
HandleEvent(event) 或计时器 callback(event) × 0..128
  ↓
Update(1 / 60) × 0..8
  ↓
UpdateFrame(real_dt, alpha) × 1
  ↓
Render() × 0..1（仅当前可见 Session）
```

补充规则：

- `Init` 只在创建 Session 时调用一次，并且早于上述循环。
- 事件按照进入当前 Session 队列的顺序投递。
- 回调执行期间产生的新事件只追加到队尾，最早在下一宿主帧投递，不会递归调用 `HandleEvent`。
- 游戏被覆盖屏遮挡时仍会执行 `Update` 和 `UpdateFrame`，但不会执行游戏的 `Render`。
- 屏保成为当前覆盖画面时执行屏保的更新和渲染；若更高优先级覆盖屏遮挡屏保，则屏保继续更新但跳过 `Render`。
- `SaveGame` 和 `SaveBest` 不属于每帧调用链，只会在脚本显式请求保存时按条件调用；退出游戏不会自动调用它们。

## 3. `Init`

初始化当前游戏或屏保 Session。

### 声明

```lua
function Init(ctx)
end
```

### 调用时机

- 包入口脚本执行完毕、全部必需回调发现成功后调用。
- 每次创建新的游戏或屏保 Session 只调用一次。
- `Init` 成功返回后，Session 才进入运行状态。
- `Init` 抛出错误、超出预算或产生致命 API 错误时，Session 启动失败，不会进入正常事件和更新循环。

### 参数

`ctx` 为一个 Lua 表：

```lua
local ctx = {
  package_id = "example_game",
  package_type = "game",
  base = {
    width = 120,
    height = 40,
  },
  start_mode = "new",
  best_data = nil,
  continue_data = nil,
  api_version = 1,
}
```

| 字段            | 类型      | 含义                                                                                     |
| --------------- | --------- | ---------------------------------------------------------------------------------------- |
| `package_id`    | `string`  | 当前包的 `mod_id`。不会暴露宿主内部完整包索引。                                          |
| `package_type`  | `string`  | `"game"` 或 `"screensaver"`。                                                            |
| `base`          | `table`   | Session 启动时的 Base 画布尺寸快照。                                                     |
| `base.width`    | `integer` | Base 画布初始宽度，单位为终端单元格。                                                    |
| `base.height`   | `integer` | Base 画布初始高度，单位为终端单元格。                                                    |
| `start_mode`    | `string`  | `"new"` 表示普通启动，`"continue"` 表示从继续游戏槽位启动。屏保恒为 `"new"`。            |
| `best_data`     | `table`   | 当前游戏已有的最佳记录数据，即上一次 `SaveBest` 返回的完整表。不存在时不包含该字段。     |
| `continue_data` | `any`     | 从“继续游戏”进入时，由该游戏上一次 `SaveGame` 返回的数据还原得到。不存在时不包含该字段。 |
| `api_version`   | `integer` | 当前 TUI GAME Lua API 版本，当前为 `1`。                                                 |

`continue_data` 和 `best_data` 只包含 JSON 兼容数据：`nil`、布尔值、有限数值、UTF-8 字符串、连续数组和字符串键对象。

`ctx.base` 是启动时快照。Base 画布之后发生变化时，应通过 `resize` 事件读取新尺寸，不要依赖该表自动更新。脚本修改 `ctx` 也不会改变宿主状态。

Lua API 不公开物理终端宽高。脚本只能查询当前 Session 的 Base 画布，以及自己创建或持有的切片图层尺寸。

### 返回值

无要求。宿主忽略 `Init` 的返回值。

### 限制

- `game.exit_game()` 不允许在 `Init` 中调用。
- 异步 API 可以在 `Init` 中提交请求，但结果只能在 Session 进入正常 Runtime 后通过事件接收。
- 绘制 API 可以使用，但绘制命令仍由宿主在回调结束后统一处理。

### 示例

```lua
local state = {}

function Init(ctx)
  state.width = ctx.base.width
  state.height = ctx.base.height
  state.score = 0

  if ctx.start_mode == "continue" then
    state.score = ctx.continue_data.score
  end

  if ctx.best_data ~= nil then
    state.best_score = ctx.best_data.score
  end
end
```

## 4. `HandleEvent`

接收宿主投递给当前 Session 的事件。

### 声明

```lua
function HandleEvent(event)
end
```

### 调用时机

- Runtime 每帧在 `Update` 之前投递事件。
- 单个 Session 每个宿主帧最多处理 128 个事件，剩余事件保留到后续帧。
- 没有事件时，本帧不会调用 `HandleEvent`。
- `timer.create`、`timer.set` 可以指定 callback；该计时器的事件只交给 callback，不再交给 `HandleEvent`。未指定 callback 时交给 `HandleEvent`。回调接收相同的事件表，遵守相同的执行限额；查看⌊[timer 库](api/timer.md)⌉。

### 参数

所有事件使用统一信封：

```lua
local event = {
  type = "action",
  sequence = 42,
  frame = 1800,
  data = {
    action = "jump",
    state = "pressed",
  },
}
```

| 字段       | 类型      | 含义                                                                                                                 |
| ---------- | --------- | -------------------------------------------------------------------------------------------------------------------- |
| `type`     | `string`  | 事件类型，决定 `data` 的具体结构。                                                                                   |
| `sequence` | `integer` | 按生成顺序全局递增的事件序号；收尾释放优先交付时，回调观察到的序号可能不连续或不按大小排列。经过目标过滤后可能跳号。 |
| `frame`    | `integer` | 事件进入 Lua Broker 时的宿主帧号。                                                                                   |
| `data`     | `table`   | 当前事件的数据表。                                                                                                   |

全部事件类型、字段和投递条件见 [EVENT.md](EVENT.md)。

### 返回值

无要求。宿主忽略 `HandleEvent` 的返回值。

### 限制

- 每个 Session 的待处理队列上限为 1024 条；合并高频事件后仍溢出会令该 Session 故障。
- 游戏可以接收允许的动作、鼠标、系统、服务和对象事件。
- 屏保不接收键盘、动作、鼠标及交互组件事件。
- 覆盖屏接管输入时，游戏不会收到动作、鼠标和交互组件事件。
- `events.skip_action()` 和 `events.clear_action()` 只影响普通游戏动作的 pressed/held，不影响 key、系统事件、宿主行为或已交付活动输入的 released。

### 示例

```lua
function HandleEvent(event)
  if event.type == "action"
      and event.data.action == "jump"
      and event.data.state == "pressed" then
    -- 处理跳跃
  elseif event.type == "resize" then
    -- 使用 event.data.width 和 event.data.height 更新布局
  end
end
```

## 5. `Update`

使用固定步长更新游戏或屏保的确定性状态。

### 声明

```lua
function Update(dt)
end
```

### 调用时机

- 固定更新频率为 60 Hz。
- 宿主使用累加器决定一个宿主帧需要执行多少次 `Update`。
- 一个宿主帧最多追赶 8 次固定更新，避免长时间卡顿后无限追帧。
- 本帧累计时间不足一个固定步长时，可以一次也不调用。
- 游戏或屏保即使被覆盖屏遮挡，仍会继续执行 `Update`。

### 参数

| 参数 | 类型     | 当前值   | 含义                                                            |
| ---- | -------- | -------- | --------------------------------------------------------------- |
| `dt` | `number` | `1 / 60` | 本次固定更新代表的秒数。固定步长不再通过 `Init(ctx)` 重复提供。 |

### 返回值

无要求。宿主忽略 `Update` 的返回值。

### 使用建议

- 将移动、碰撞、计分、规则判断等需要稳定步长的逻辑放在这里。
- 使用 `value = value + speed * dt` 处理按秒定义的速度。
- 不要在回调中编写用于维持游戏循环的死循环；循环由宿主负责。

### 示例

```lua
function Update(dt)
  player.x = player.x + player.speed * dt
end
```

## 6. `UpdateFrame`

每个宿主帧更新一次与真实帧时间或固定更新插值有关的状态。

### 声明

```lua
function UpdateFrame(dt, alpha)
end
```

### 调用时机

- 当前 Session 存活时，每个正常 Runtime 帧调用一次。
- 在本帧的全部固定 `Update` 之后调用。
- 即使本帧没有执行 `Update`，仍会执行一次 `UpdateFrame`。
- 游戏或屏保被覆盖屏遮挡时仍会继续执行。

### 参数

| 参数    | 类型     | 范围         | 含义                                                                         |
| ------- | -------- | ------------ | ---------------------------------------------------------------------------- |
| `dt`    | `number` | `0.0..=0.25` | 当前宿主帧经过的真实秒数；为避免异常大步长，最大钳制为 250 ms。              |
| `alpha` | `number` | `0.0..=1.0`  | 固定更新累加器剩余时间与 `1 / 60` 的比例，可用于在前后两个固定状态之间插值。 |

当一个宿主帧达到 8 次追赶上限后，宿主会丢弃多余的完整固定步长，只保留不足一个固定步长的余数。

### 返回值

无要求。宿主忽略 `UpdateFrame` 的返回值。

### 使用建议

- 将只需每个显示帧更新一次的过渡状态放在这里。
- `alpha` 适合视觉插值，不应替代 `Update` 中的游戏规则更新。

### 示例

```lua
function UpdateFrame(dt, alpha)
  player.render_x = player.previous_x
    + (player.x - player.previous_x) * alpha
end
```

## 7. `Render`

为当前可见的游戏或屏保画面提交绘制命令。

### 声明

```lua
function Render()
end
```

### 调用时机

- 在本帧事件、`Update` 和 `UpdateFrame` 完成后调用。
- 只有当前可见的 Lua Session 会调用 `Render`。
- 游戏被任意覆盖屏遮挡时跳过游戏 `Render`。
- 屏保是当前覆盖画面时调用屏保 `Render`；被更高优先级覆盖屏遮挡时跳过。
- 正常情况下每个可见宿主帧最多调用一次。

### 参数

无。绘制时直接使用全局 `draw` 库，方法参数见 [draw API](api/draw.md)。

Base 画布的初始宽高来自 `Init(ctx)` 的 `ctx.base.width` 和 `ctx.base.height`；运行期间尺寸变化通过 `resize` 事件更新。宿主不会向 `Render` 传递画布或终端尺寸。

### 返回值

无要求。宿主忽略 `Render` 的返回值。

### 绘制模型与限制

- `draw.text` 等普通绘制 API 并非只能在 `Render` 中调用；它们可在任意生命周期回调中向当前 Session 的虚拟画布命令缓冲提交绘制。
- 宿主在 Lua 回调结束后统一消费并拼合绘制命令。建议主要在 `Render` 中组织绘制，以便代码职责清晰。
- `draw.render()` 只请求宿主重绘，不会立即递归调用 `Render`；它不允许在 `Render` 回调内使用。
- 坐标允许为负数，超出画布的部分由宿主裁剪。
- 每个 Session 每帧最多提交 4096 条绘制命令。
- 每个 Session 每帧绘制文本累计最多 1 MiB。
- 被遮挡的 Session 仍可更新，但其本帧绘制命令不会显示，并会在帧末回收。

### 示例

```lua
function Render()
  draw.fill_rect(0, 0, base_width, base_height, {bg = color.BLACK})

  draw.text(1, 1, "Hello TUI GAME", {fg = color.WHITE})
end
```

## 8. `SaveGame`

返回用于宿主“继续游戏”功能的游戏状态。

### 声明

```lua
function SaveGame()
  return {
    level = 3,
    score = 1200,
    player = { x = 8, y = 5 },
  }
end
```

### 启用条件

仅游戏使用。`game.json` 必须包含：

```json
{
  "save_game": true
}
```

启用后，`SaveGame` 为必需回调。

### 调用时机

- 脚本调用 `game.save_game()` 后，由宿主在当前 Lua 回调返回后调用。
- 退出游戏或关闭宿主不会自动调用；开发者如需保留继续游戏数据，必须在退出前显式调用 `game.save_game()`。
- Session 已经故障时不会自动保存故障状态。

该存储只用于宿主的单个“继续游戏”槽位。所有游戏共用该槽位，后保存的游戏会覆盖之前的继续游戏数据。游戏自己的长期、多槽位存档应在获得权限后使用文件 API。

### 参数

无。

### 返回值

可以返回任意非 `nil`、可序列化为 JSON 的单值或表。宿主只读取第一个返回值，其余返回值会被忽略。

允许的值：

- `boolean`
- 有限的 `integer` 或 `number`
- UTF-8 `string`
- 从索引 `1` 开始且连续的数组表
- 只使用字符串键的对象表
- 上述类型的递归组合

不允许的值：

- `nil`
- `function`、`thread`、`userdata`、轻量 userdata 和错误对象
- `NaN`、正无穷或负无穷
- 非 UTF-8 字符串或键
- 循环引用表
- 稀疏数组
- 同时混用数组索引与字符串键的表
- 使用布尔、函数、表等其他类型作为键的对象

序列化最大深度为 32，编码后的 JSON 最大为 1 MiB。

### 限制

- `game.save_game()` 不允许在 `SaveGame` 内再次调用，避免递归保存。
- `game.exit_game()` 不允许在 `SaveGame` 内调用。
- 返回值不合法时不会写入继续游戏槽位，并会作为当前游戏的 Lua Session 错误抛出；宿主继续运行。

## 9. `SaveBest`

返回当前游戏的最佳记录及游戏列表所需的展示文本。

### 声明

```lua
function SaveBest()
  return {
    best_string = { type = "i18n", key = "score", callback = "f%Best: {value:score}" },
    value = {
      score = tostring(best_score),
      rank = { type = "i18n", key = "rank.gold", callback = "Gold" },
    },
    score = best_score,
  }
end
```

### 启用条件

仅游戏使用。`game.json` 必须启用记录：

```json
{
  "best_score": {
    "enable": true
  }
}
```

启用后，`SaveBest` 为必需回调。

### 调用时机

- 脚本调用 `game.save_best()` 后，由宿主在当前 Lua 回调返回后调用。
- 退出游戏或关闭宿主不会自动调用；开发者如需更新最佳记录，必须在退出前显式调用 `game.save_best()`。
- Session 已经故障时不会自动保存故障状态。

### 参数

无。

### 返回值

必须返回一个可序列化的对象表，不能返回单值，并且必须包含：

```lua
local best_data = {
  best_string = "用于游戏列表展示的文本",
}
```

| 字段          | 类型             | 必填 | 含义                                                                                            |
| ------------- | ---------------- | ---- | ----------------------------------------------------------------------------------------------- |
| `best_string` | `string / table` | 是   | 游戏列表显示的最佳记录文本，或与包清单相同的文本表。                                            |
| `value`       | `table`          | 否   | `{value:名称}` 的替换参数；键为名称，每个值为字符串或与包清单相同的文本表。省略时没有替换参数。 |

文本表可以写成 `{ type = "text", text = "..." }`，或 `{ type = "i18n", key = "...", callback = "..." }`。i18n 的 `key` 和 `callback` 必填，`key` 不能为空；不接受自定义 `path` 或其他未知字段。

`best_string` 和 `value` 中的所有 i18n 表都从当前包的 `assets/language/<语言代码>/package/best_string.json` 查找。先使用玩家当前语言，缺失时再找 `en_us`，仍找不到则显示该表的 `callback`。语言文件是键到字符串的 JSON 对象，例如：

```json
{
  "score": "f%最佳成绩：{value:score}，等级：{value:rank}",
  "rank.gold": "金牌"
}
```

使用 `{value:名称}` 或样式标签时，`best_string` 及其翻译、回退文本需要带 `f%` 前缀。替换值按原样插入，不再次解析其中的占位符或样式标签；未提供的参数保留原占位符。数值请先用 `tostring` 转成字符串；空字符串是有效值。`value` 必须是名称到文本的对象表，不能是数组。

保存的是原始文本表及参数，切换语言或修改语言文件后，游戏列表会重新显示对应文本。旧的纯字符串最佳记录仍可读取。

其余字段由游戏自行定义，宿主会连同 `best_string` 和 `value` 一起保存，并在下一次创建该游戏 Session 时通过 `Init(ctx)` 的 `ctx.best_data` 传回。

宿主只读取第一个返回值。表的类型、深度、大小和循环引用限制与 `SaveGame` 完全相同。

### 限制

- `game.save_best()` 不允许在 `SaveBest` 内再次调用，避免递归保存。
- `game.exit_game()` 不允许在 `SaveBest` 内调用。
- 返回值不合法时不会写入最佳记录，并会作为当前游戏的 Lua Session 错误抛出；宿主继续运行。

## 10. 公共执行限制

### 10.1 时间与指令预算

| 回调          | 慢调用警告 | 硬时间上限 | Lua 指令上限 |
| ------------- | ---------- | ---------- | ------------ |
| `Init`        | 50 ms      | 100 ms     | 1,000,000    |
| `HandleEvent` | 20 ms      | 75 ms      | 200,000      |
| `Update`      | 20 ms      | 75 ms      | 200,000      |
| `UpdateFrame` | 20 ms      | 75 ms      | 200,000      |
| `Render`      | 20 ms      | 75 ms      | 200,000      |
| `SaveGame`    | 50 ms      | 100 ms     | 1,000,000    |
| `SaveBest`    | 50 ms      | 100 ms     | 1,000,000    |

- Hook 每 1,000 条 Lua 指令检查一次预算。
- Rust API 调用消耗的墙钟时间也包含在硬时间上限内。
- 只有成功完成但超过警告时间的回调才记录慢调用警告。
- 慢调用警告只在包开启调试模式时写入对应包日志；同一 Session、同一回调最多每 5 秒记录一次，并统计期间被抑制的次数。
- 超过硬时间或指令上限会使当前 Session 故障。

### 10.2 内存和故障隔离

- 每个 Lua VM 的内存上限为 32 MiB。
- 每次回调完成后，宿主执行一次增量 GC。
- 回调抛出的未捕获错误、预算超限、内存超限、事件队列溢出和致命 API 限制只会终止对应 Session，不应导致宿主 Panic。
- 游戏 Session 故障后停止运行并进入游戏崩溃提示；屏保 Session 故障后关闭屏保并恢复原画面。
- 时间、指令、内存及致命 API 限制不能被 `debug.pcall` 或 `debug.xpcall` 吞掉。

### 10.3 一般约束

- 所有生命周期回调都在 Runtime 主线程串行执行，不会并发调用同一 Session。
- 不要在回调中阻塞等待异步服务结果；当前生产请求结果会在后续帧通过 `HandleEvent` 返回。
- 原始终端事件对象、宿主内部任务 ID、绝对路径和宿主 UI 对象不会传给脚本。
- Session 停止后，宿主会清理其事件、对象和异步任务所有权；旧 Session 的迟到结果不会进入新 Session。

## 11. 最小模板

### 游戏

```lua
local state = {
  elapsed = 0,
}
local base_width = 0
local base_height = 0

function Init(ctx)
  base_width = ctx.base.width
  base_height = ctx.base.height
  if ctx.continue_data ~= nil then
    state = ctx.continue_data
  end
end

function HandleEvent(event)
  if event.type == "resize" then
    base_width = event.data.width
    base_height = event.data.height
  end
end

function Update(dt)
  state.elapsed = state.elapsed + dt
end

function UpdateFrame(dt, alpha)
end

function Render()
  draw.text(1, 1, "Elapsed: " .. tostring(state.elapsed))
end

function SaveGame()
  return state
end

function SaveBest()
  return {
    best_string = "Elapsed: " .. tostring(state.elapsed),
    elapsed = state.elapsed,
  }
end
```

若游戏没有启用存档或记录功能，应删除对应的保存回调。

### 屏保

```lua
local elapsed = 0
local base_width = 0
local base_height = 0

function Init(ctx)
  base_width = ctx.base.width
  base_height = ctx.base.height
end

function HandleEvent(event)
  if event.type == "resize" then
    base_width = event.data.width
    base_height = event.data.height
  end
end

function Update(dt)
  elapsed = elapsed + dt
end

function UpdateFrame(dt, alpha)
end

function Render()
  draw.text(1, 1, "Screensaver " .. tostring(elapsed))
end
```
