# Lua Runtime API

这份文档只说明一件事：宿主现在真实给 Lua 脚本开放了什么能力，以及这些能力该怎么用。

边界先说清：

- Lua 只负责游戏自己的规则、状态和绘制
- Rust 宿主负责输入、主循环、资源读取、存档、最佳记录、日志和终端输出
- Lua 不自己开主循环，也不直接读写电脑上的任意文件

## 脚本必须导出的函数

这三个函数是必需的：

- `init_game()`
- `handle_event(state, event)`
- `render(state)`

`best_score(state)` 不是总是必需：

- 如果 `game.json.best_none` 是字符串：这个游戏有最佳记录，`best_score(state)` 必须存在
- 如果 `game.json.best_none` 是 `null`：这个游戏没有最佳记录，可以不写 `best_score(state)`

`entry` 在 `game.json` 里表示的是**入口脚本路径**，不是函数名。宿主会从这个脚本里固定查找上面这些导出函数。

## 宿主怎样把事件传给 Lua

宿主不会一次把“本帧事件列表”整个传进 Lua。

真实顺序是：

```text
收集本帧累积事件
-> 逐个调用 handle_event(state, event)
-> 再调用一次 handle_event(state, tick_event)
-> 清空画布
-> 调用一次 render(state)
-> 输出当前帧
```

所以从脚本视角看：

- `handle_event` 先更新状态
- `render` 再按这帧的最新状态画面
- `render` 看到的永远是这一帧所有事件都处理完后的结果

## 五类事件

Lua 每次只会收到一个事件 table，不会收到事件数组。

### `action`

示例：

```lua
{ type = "action", name = "move_left" }
```

作用：

- 宿主根据 `game.json.actions` 把物理按键映射成动作语义
- 适合处理移动、保存、重开、确认、退出这类“游戏动作”

### `key`

示例：

```lua
{ type = "key", name = "enter" }
```

作用：

- 这是原始按键
- 适合输入模式、字母输入、数字输入、退格、Tab、Enter 这种更细的输入场景

### `resize`

示例：

```lua
{ type = "resize", width = 120, height = 40 }
```

作用：

- 告诉脚本终端大小刚刚变了
- 适合重新布局、清掉旧尺寸缓存、切到“窗口太小”的提示状态

### `tick`

示例：

```lua
{ type = "tick", dt_ms = 16 }
```

作用：

- 这是时间推进事件
- 现在不是“没事件才补一个 tick”，而是**每一帧固定都会有一次**
- `dt_ms` 表示这一帧真实经过了多少毫秒，不保证永远等于 16

建议：

- 动态游戏优先用 `after_ms(...)`、`deadline_passed(...)`、`remaining_ms(...)`
- 不要把逻辑写死成 “`event.dt_ms == 16` 才正确”

### `quit`

示例：

```lua
{ type = "quit" }
```

作用：

- 宿主准备结束当前游戏前，给脚本一个收尾机会
- 适合自动保存、刷新最佳记录、做最后清理

## 可用函数总表

### 画布绘制

| 函数 | 作用 | 参数说明 |
| --- | --- | --- |
| `canvas_clear()` | 清空当前这一帧的画布。 | 无参数。 |
| `canvas_draw_text(x, y, text, fg, bg)` | 把一段文字画到画布上。 | `x`、`y` 是左上角坐标；`text` 是文字；`fg` 是前景色，可不传；`bg` 是背景色，可不传。 |
| `canvas_fill_rect(x, y, width, height, ch, fg, bg)` | 用同一个字符铺满一块矩形区域。适合画底色、色块、背景条。 | `x`、`y` 是左上角；`width`、`height` 是大小；`ch` 是填充字符；`fg`、`bg` 是颜色，可不传。 |

### 文本与尺寸

这些函数都不是返回表，而是普通返回值。

- `measure_text(text)` 返回 `width, height`
- `get_text_size(text)` 返回 `width, height`
- `get_text_width(text)` 返回 `width`
- `get_terminal_size()` 返回 `width, height`

说明：

- 文本宽高只按字符串本身和 `\n` 来算
- 不会按当前终端宽度自动换行
- 中文、全角字符会按终端显示宽度计算

| 函数 | 作用 | 参数说明 |
| --- | --- | --- |
| `measure_text(text)` | 算一段文字本身会占多宽、多高。 | `text` 是要测量的文字。返回 `width, height`。 |
| `get_text_width(text)` | 只算文字宽度。 | `text` 是要测量的文字。返回 `width`。 |
| `get_text_size(text)` | 和 `measure_text` 一样，也返回宽和高。 | `text` 是要测量的文字。返回 `width, height`。 |
| `get_terminal_size()` | 读取当前终端宽高。 | 无参数。返回 `width, height`。 |

### 布局辅助

这些函数都帮你基于当前终端大小算位置。

- `resolve_x(...)` 返回 `x`
- `resolve_y(...)` 返回 `y`
- `resolve_rect(...)` 返回 `x, y`

| 函数 | 作用 | 参数说明 |
| --- | --- | --- |
| `resolve_x(anchor, content_width, offset)` | 按左对齐、居中、右对齐来算最终的 `x`。 | `anchor` 是水平锚点；`content_width` 是内容宽度；`offset` 是额外偏移，可不传。 |
| `resolve_y(anchor, content_height, offset)` | 按上对齐、垂直居中、下对齐来算最终的 `y`。 | `anchor` 是垂直锚点；`content_height` 是内容高度；`offset` 是额外偏移，可不传。 |
| `resolve_rect(h_anchor, v_anchor, width, height, offset_x, offset_y)` | 一次算出一个矩形左上角坐标。 | `h_anchor` 是水平锚点；`v_anchor` 是垂直锚点；`width`、`height` 是矩形大小；`offset_x`、`offset_y` 是额外偏移，可不传。返回 `x, y`。 |

简单例子：

```lua
local title = "2048"
local w = get_text_width(title)
local x = resolve_x(ANCHOR_CENTER, w)
canvas_draw_text(x, 1, title, "yellow", nil)
```

### 终端状态

这两个函数都只返回布尔值。

- `was_terminal_resized()`：只看，不清标记
- `consume_resize_event()`：看完就清掉标记

| 函数 | 作用 | 参数说明 |
| --- | --- | --- |
| `was_terminal_resized()` | 看看最近是不是变过窗口大小。 | 无参数。返回 `true` 或 `false`。 |
| `consume_resize_event()` | 读取并清掉一次“窗口大小变了”的标记。 | 无参数。返回 `true` 或 `false`。 |

### 运行控制

| 函数 | 作用 | 参数说明 |
| --- | --- | --- |
| `request_exit()` | 告诉宿主“退出当前游戏”。 | 无参数。无返回值。 |
| `request_refresh_best_score()` | 告诉宿主重新调用 `best_score(state)`，再把结果保存成当前游戏的最佳记录。 | 无参数。无返回值。没有最佳记录的游戏调用它时，宿主会忽略。 |
| `get_launch_mode()` | 读取这次启动是“新开一局”还是“继续之前的进度”。 | 无参数。返回 `"new"` 或 `"continue"`。 |
| `clear_input_buffer()` | 清掉终端里还没处理的输入事件。适合切阶段、弹确认框、倒计时切场景时用。 | 无参数。无返回值。 |

### 存储

| 函数 | 作用 | 参数说明 |
| --- | --- | --- |
| `save_data(slot, value)` | 保存当前游戏自己的普通数据。适合保存配置和小型槽位数据。 | `slot` 是槽位名；`value` 是要保存的 Lua 值。 |
| `load_data(slot)` | 读取当前游戏之前保存过的普通数据。 | `slot` 是槽位名。没有时返回 `nil`。 |
| `save_continue(value)` | 保存“继续游戏”快照。 | `value` 是这局的状态数据。 |
| `load_continue()` | 读取“继续游戏”快照。 | 无参数。没有时返回 `nil`。 |
| `load_best_score()` | 读取当前游戏已经保存过的最佳记录。 | 无参数。没有最佳记录或没有数据时返回 `nil`。 |
| `update_game_stats(game_id, score, duration_sec)` | 更新宿主层的通用统计。大多数游戏通常不需要主动调。 | `game_id` 是游戏 ID；`score` 是分数；`duration_sec` 是秒数。 |

### 语言与资源

| 函数 | 作用 | 参数说明 |
| --- | --- | --- |
| `translate(key)` | 读取当前游戏包里的语言键。 | `key` 是语言键。宿主先查当前包当前语言，再查包内 `en_us`，最后才回退原文。 |
| `read_text(path)` | 读取当前游戏包里的文本文件。 | `path` 是包内相对路径，例如 `data/help.txt`。只能读当前包自己的资源。 |
| `read_bytes(path)` | 读取当前游戏包里的二进制文件。 | `path` 是包内相对路径。返回原始字节串。 |
| `read_json(path)` | 读取当前游戏包里的 JSON 文件，并转成 Lua 值。 | `path` 是包内相对路径，例如 `data/word.json`。 |
| `load_helper(path)` | 加载当前游戏包里的辅助 Lua 脚本。 | `path` 是包内相对路径，例如 `helpers/layout.lua`。只能加载当前包自己的 `.lua` 文件，不能 `require` 外部文件。 |

### 时间

| 函数 | 作用 | 参数说明 |
| --- | --- | --- |
| `time_now_ms()` | 读取当前 runtime 从启动到现在过去了多少毫秒。 | 无参数。返回整数毫秒。 |
| `after_ms(delay_ms)` | 基于当前时间生成一个未来的截止时间。适合做停顿和阶段切换。 | `delay_ms` 是多少毫秒后到期。返回 `deadline_ms`。 |
| `deadline_passed(deadline_ms)` | 看一个截止时间是不是已经到了。 | `deadline_ms` 是截止时间。返回 `true` 或 `false`。 |
| `remaining_ms(deadline_ms)` | 看离这个截止时间还剩多少毫秒。 | `deadline_ms` 是截止时间。已经超时会返回 `0`。 |

### 调试

| 函数 | 作用 | 参数说明 |
| --- | --- | --- |
| `debug_log(message)` | 往当前游戏自己的日志文件里追加一行调试信息。 | `message` 是要写入的文字。 |
| `clear_debug_log()` | 清空当前游戏自己的日志文件。 | 无参数。 |

### 随机

| 函数 | 作用 | 参数说明 |
| --- | --- | --- |
| `random()` | 取一个宿主生成的随机整数。 | 无参数。 |
| `random(max)` | 取 `0` 到 `max-1` 之间的随机整数。 | `max` 是上限；`max <= 0` 时返回 `0`。注意：这不是 Lua `math.random(max)` 那套语义。 |
| `random(min, max)` | 取 `min` 到 `max` 之间的随机整数，包含两端。 | `min`、`max` 是范围。传反了也会自动纠正。 |

## 常量

| 常量 | 作用 | 值 |
| --- | --- | --- |
| `ANCHOR_LEFT` | 水平左对齐 | `0` |
| `ANCHOR_CENTER` | 水平居中 | `1` |
| `ANCHOR_RIGHT` | 水平右对齐 | `2` |
| `ANCHOR_TOP` | 垂直顶部对齐 | `0` |
| `ANCHOR_MIDDLE` | 垂直居中 | `1` |
| `ANCHOR_BOTTOM` | 垂直底部对齐 | `2` |

## 数据最终写到哪里

| 接口 | 最终位置 |
| --- | --- |
| `save_data(slot, value)` | `tui-game-data/saves.json` 的 `data[game_id][slot]` |
| `save_continue(value)` | `tui-game-data/saves.json` 的 `continue[game_id]` |
| `load_best_score()` | 从 `tui-game-data/best_scores.json` 读取当前游戏条目 |
| `debug_log(message)` | 写到 `tui-game-data/log/<game_id>.log` |

## 常见示例

### 读取包内语言

```lua
local title = translate("game.wordle.name")
canvas_draw_text(2, 1, title, "yellow", nil)
```

### 读取包内 JSON

```lua
local words = read_json("data/word.json")
local first = words[1]
```

### 保存和读取继续游戏

```lua
save_continue({
  score = state.score,
  board = state.board,
  elapsed = state.elapsed_ms,
})

local saved = load_continue()
if saved then
  state = saved
end
```

### 保存普通数据

```lua
save_data("config", {
  difficulty = 3,
  sound = true,
})
```

### 用时间 API 控制停顿

```lua
if state.deadline_ms == nil then
  state.deadline_ms = after_ms(800)
end

if deadline_passed(state.deadline_ms) then
  state.phase = "next"
  state.deadline_ms = nil
end
```
