# 事件协议规范

## 前言

为保证程序与脚本之间能够以非阻断的方式持续循环运行，程序会将输入、状态变化和异步操作结果以事件的形式传递给脚本处理。未注册专用回调时，事件由 `HandleEvent` 接收。

这些事件遵循统一的结构，即事件协议。本文档详细说明各类事件的结构、发送条件和使用示例。

---

## 目录

| 章节 | 说明 | 索引 |
| --- | --- | --- |
| 通用结构 | 所有事件共用的父结构 | [通用结构](#通用结构) |
| 类型事件结构 | 各类事件的数据结构 | [类型事件结构](#类型事件结构) |
| `action` | 游戏动作触发事件 | [action](#action) |
| `key` | 原始按键触发事件 | [key](#key) |
| `mouse` | 使用鼠标触发的事件 | [mouse](#mouse) |
| `resize` | Base 画布尺寸变化事件 | [resize](#resize) |
| `focus` | 终端焦点变化事件 | [focus](#focus) |
| `overlay_started` | 覆盖屏开始接管事件 | [overlay_started](#overlay_started) |
| `overlay_stopped` | 覆盖屏结束接管事件 | [overlay_stopped](#overlay_stopped) |
| `timer` | 计时器触发或结束事件 | [timer](#timer) |
| `animation` | 动画播放状态变化或触发标记的事件 | [animation](#animation) |
| `file` | 异步文件操作结果事件 | [file](#file) |
| `image` | 图片转换结果事件 | [image](#image) |
| `network` | 网络请求结果事件 | [network](#network) |
| `i18n` | 国际化语言加载结果事件 | [i18n](#i18n) |
| `audio` | 音频对象状态变化事件 | [audio](#audio) |
| `hit_area` | 点击区域交互事件 | [hit_area](#hit_area) |
| `hyperlink` | 超链接点击事件 | [hyperlink](#hyperlink) |
| `markdown` | Markdown 文本中的链接点击事件 | [markdown](#markdown) |
| `text_input` | 文本输入组件的状态变化或交互事件 | [text_input](#text_input) |
| `scroll_box` | 滚动框位置变化事件 | [scroll_box](#scroll_box) |

---

## 通用结构

所有事件共用同一个外层父结构，用于表示事件的基本信息。

### 结构

```lua
{
  type = ...,     -- string
  sequence = ..., -- integer
  frame = ...,    -- integer
  data = ...,     -- table
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `type` | string | 事件类型，用于判断 `data` 的结构 |
| `sequence` | integer | 宿主运行期间全局单调递增的事件序号 |
| `frame` | integer | 事件进入 Lua 事件队列时的宿主帧序号 |
| `data` | table | 事件数据，无额外数据时为空表 |

### 额外说明

- 相邻事件的 `sequence` 不一定连续，事件经过会话过滤后可能出现跳号。
- `frame` 不等同于游戏自行维护的帧号，也不表示脚本处理事件时的帧号。
- 父结构的四个字段始终存在，未出现的可选数据字段为 `nil`；脚本不应依赖表中字段的遍历顺序。
- 未注册对象或异步请求回调时，事件交给 `HandleEvent(event)`；注册回调后，完整事件只交给该回调，不会重复交给 `HandleEvent`。
- 回调在宿主主线程执行，使用与 `HandleEvent` 相同的时间和指令预算。处理事件时产生的新事件追加至队尾，最早在下一宿主帧投递，不会递归调用脚本。
- 游戏和屏保会话各自维护独立队列，单帧最多处理 `128` 条，待处理上限为 `1024` 条。队列溢出只使对应会话故障。
- 对象 ID、请求 ID 是当前会话内的不透明整数，不是宿主内部 ID。会话停止或重启后，旧 ID、待处理事件和回调失效，旧请求的迟到结果不会投递给新会话。
- 带有 `error` 字段的事件共用以下错误表，`code` 用于判断错误类型，`message` 为不含绝对路径、系统调用栈、宿主 ID 或敏感请求内容的说明：

```lua
{
  code = ...,    -- string
  message = ..., -- string
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `code` | string | 稳定错误码 |
| `message` | string | 经过净化的通用错误说明 |

- 错误码包含以下固定值：

```lua
"invalid_request"
"permission_denied"
"not_found"
"too_large"
"invalid_utf8"
"cancelled"
"timeout"
"io"
"network"
"unsupported"
"decode"
"backend_unavailable"
"internal"
```

---

## 类型事件结构

> 父结构中的字段 `sequence` 和字段 `frame` 在后续“结构”说明中会被省略，但在实际使用时依然存在。示例输出使用 `X` 表示这两个字段的实际值。

## `action`

游戏动作触发事件。

### 结构

```lua
{
  type = "action",
  data = {
    action = ...,  -- string
    state = ...,   -- string
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `action` | string | 游戏包注册的动作名称 |
| `state` | string | 动作触发状态 |

### 发送条件

玩家按下、持续按住或松开按键，且输入命中游戏包的动作映射时，发送给游戏会话。宿主先处理全局按键，剩余输入才转换为动作事件；覆盖屏接管交互期间不发送。

### 示例

```lua
function HandleEvent(event)
  if event.type == "action" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "action",
  frame = X,
  sequence = X,
  data =
  {
    action = "jump",
    state = "pressed",
  },
}

{
  type = "action",
  frame = X,
  sequence = X,
  data =
  {
    action = "jump",
    state = "held",
  },
}

{
  type = "action",
  frame = X,
  sequence = X,
  data =
  {
    action = "jump",
    state = "released",
  },
}
```

### 额外补充

- 字段 `state` 包含以下固定值：

```lua
"pressed"  -- 当前帧按下按键
"held"     -- 当前帧持续按住按键
"released" -- 当前帧松开按键
```

- 覆盖屏出现后，尚未投递的交互事件会被清理，避免宿主输入继续传递给游戏。

---

## `key`

原始按键触发事件。

### 结构

```lua
{
  type = "key",
  data = {
    key = ...,    -- string
    state = ...,  -- string
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `key` | string | 按键名称 |
| `state` | string | 按键触发状态 |

### 发送条件

玩家按下、持续按住或松开按键后发送。

### 示例

```lua
function HandleEvent(event)
  if event.type == "key" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "key",
  frame = X,
  sequence = X,
  data =
  {
    key = "space",
    state = "pressed",
  },
}

{
  type = "key",
  frame = X,
  sequence = X,
  data =
  {
    key = "space",
    state = "held",
  },
}

{
  type = "key",
  frame = X,
  sequence = X,
  data =
  {
    key = "space",
    state = "released",
  },
}
```

### 额外补充

- 字段 `state` 包含以下固定值：

```lua
"pressed"  -- 当前帧按下按键
"held"     -- 当前帧持续按住按键
"released" -- 当前帧松开按键
```

---

## `mouse`

使用鼠标触发的事件。

### 结构

```lua
{
  type = "mouse",
  data = {
    kind = ...,    -- string
    button = ...,  -- string / nil
    scroll = ...,  -- string / nil
    x = ...,       -- integer
    y = ...,       -- integer
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `kind` | string | 鼠标事件类型 |
| `button` | string / nil | 有对应鼠标键时出现，表示触发事件的鼠标键 |
| `scroll` | string / nil | 滚动事件时出现，表示滚动方向 |
| `x` | integer | Base 画布内的水平单元格坐标 |
| `y` | integer | Base 画布内的垂直单元格坐标 |

### 发送条件

鼠标在游戏 Base 可视区域内，且终端拥有焦点时发送给游戏会话。覆盖屏接管交互期间不发送，屏保不接收鼠标事件。

### 示例

```lua
function HandleEvent(event)
  if event.type == "mouse" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "mouse",
  frame = X,
  sequence = X,
  data =
  {
    kind = "pressed",
    button = "left",
    x = 12,
    y = 4,
  },
}
```

### 额外补充

- 字段 `kind` 包含以下固定值：

```lua
"pressed"  -- 按下鼠标键
"released" -- 松开鼠标键
"moved"    -- 移动鼠标
"dragged"  -- 拖动鼠标
"held"     -- 持续按住鼠标键
"scrolled" -- 滚动鼠标滚轮
```

- 字段 `button` 包含以下固定值：

```lua
"left"   -- 鼠标左键
"middle" -- 鼠标中键
"right"  -- 鼠标右键
```

- 字段 `scroll` 包含以下固定值：

```lua
"up"    -- 向上滚动
"down"  -- 向下滚动
"left"  -- 向左滚动
"right" -- 向右滚动
```

- 坐标以 Base 画布左上角为原点，从 `0` 开始。
- 游戏包中的鼠标声明用于说明所需能力，不作为事件权限开关。
- 尚未处理的 `moved` 或 `held` 事件中，具有相同 `kind` 和 `button` 的事件只保留最新坐标；鼠标按键、拖动和滚轮事件不会合并。

---

## `resize`

Base 画布尺寸变化事件。

### 结构

```lua
{
  type = "resize",
  data = {
    width = ...,   -- integer
    height = ...,  -- integer
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `width` | integer | 当前会话的 Base 画布宽度，单位为单元格 |
| `height` | integer | 当前会话的 Base 画布高度，单位为单元格 |

### 发送条件

终端尺寸变化并引起当前会话的 Base 画布尺寸更新时，发送给存活的游戏和屏保会话。覆盖屏接管交互期间仍会发送。

### 示例

```lua
function HandleEvent(event)
  if event.type == "resize" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "resize",
  frame = X,
  sequence = X,
  data =
  {
    width = 120,
    height = 40,
  },
}
```

### 额外补充

- 宿主会分别换算游戏和屏保的 Base 画布尺寸，事件中的宽高不直接代表物理终端尺寸。
- 尚未处理的多个 `resize` 事件只保留最新尺寸。
- 即使覆盖屏已经消费本帧输入，尺寸变化事件仍可送达脚本。

---

## `focus`

终端焦点变化事件。

### 结构

```lua
{
  type = "focus",
  data = {
    gained = ...,  -- boolean
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `gained` | boolean | 是否获得终端焦点 |

### 发送条件

终端获得或失去焦点时，发送给存活的游戏和屏保会话。覆盖屏接管交互期间仍会发送。

### 示例

```lua
function HandleEvent(event)
  if event.type == "focus" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "focus",
  frame = X,
  sequence = X,
  data =
  {
    gained = false,
  },
}
```

### 额外补充

- 字段 `gained` 为 `true` 时表示获得焦点，为 `false` 时表示失去焦点。
- 失去焦点不会额外生成所有动作的 `released` 事件。游戏应在 `gained == false` 时清理自行保存的输入状态。
- 焦点事件不会合并。

---

## `overlay_started`

覆盖屏开始接管事件。

### 结构

```lua
{
  type = "overlay_started",
  data = {},
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `data` | table | 固定为空表，不包含额外字段 |

### 发送条件

覆盖屏栈从空变为非空时，发送给游戏会话。

### 示例

```lua
function HandleEvent(event)
  if event.type == "overlay_started" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "overlay_started",
  frame = X,
  sequence = X,
  data = {},
}
```

### 额外补充

- `data` 固定为空表。
- 覆盖屏包括截屏模式、尺寸提醒和屏保等使用同一覆盖屏栈的界面。
- 第一个覆盖屏之上再加入其他覆盖屏时，不会重复发送此事件。
- 覆盖屏存在期间，游戏仍可更新并接收非交互事件，但不会接收动作、鼠标或组件交互事件。
- 此事件描述整个覆盖阶段，不代表某一种具体覆盖屏，也不会与其他事件合并。

---

## `overlay_stopped`

覆盖屏结束接管事件。

### 结构

```lua
{
  type = "overlay_stopped",
  data = {},
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `data` | table | 固定为空表，不包含额外字段 |

### 发送条件

覆盖屏栈从非空变为空，即最后一个覆盖屏退出时，发送给游戏会话。

### 示例

```lua
function HandleEvent(event)
  if event.type == "overlay_stopped" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "overlay_stopped",
  frame = X,
  sequence = X,
  data = {},
}
```

### 额外补充

- `data` 固定为空表。
- 移除顶层覆盖屏后，如果栈内仍存在其他覆盖屏，不会发送此事件。
- 此事件与 `overlay_started` 对应，描述整个覆盖阶段的结束，不代表某一种具体覆盖屏，也不会与其他事件合并。

---

## `timer`

计时器触发或结束事件。

### 结构

```lua
{
  type = "timer",
  data = {
    id = ...,              -- integer
    timer_kind = ...,      -- string
    kind = ...,            -- string
    executed_count = ...,  -- integer / nil
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `id` | integer | 当前会话内的计时器 ID |
| `timer_kind` | string | 计时器类型 |
| `kind` | string | 计时器事件类型 |
| `executed_count` | integer / nil | 重复计时器事件中出现，表示已经完成的触发次数 |

### 发送条件

计时器触发、计时结束或休眠请求完成时，发送给创建该计时器或请求的游戏、屏保会话。

### 示例

```lua
function HandleEvent(event)
  if event.type == "timer" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "timer",
  frame = X,
  sequence = X,
  data =
  {
    id = 1,
    timer_kind = "repeat",
    kind = "tick",
    executed_count = 3,
  },
}
```

### 额外补充

- 字段 `timer_kind` 包含以下固定值：

```lua
"timer"  -- 计时器
"delay"  -- 延迟计时器
"repeat" -- 重复计时器
"sleep"  -- 休眠请求
```

- 字段 `kind` 包含以下固定值：

```lua
"tick"     -- 重复计时器触发一次
"finished" -- 计时或休眠结束
```

- `timer`、`delay`、`sleep` 只产生一次 `finished` 事件。
- `repeat` 每次触发时产生 `tick`，结束时产生 `finished`；这两种事件都会携带 `executed_count`。
- 一次性计时器的终态回调在投递后回收，重复计时器的回调保留至结束或取消。
- 计时器事件不会合并。

---

## `animation`

动画播放状态变化或触发标记的事件。

### 结构

```lua
{
  type = "animation",
  data = {
    id = ...,         -- integer
    kind = ...,       -- string
    name = ...,       -- string / nil
    completed = ...,  -- integer / nil
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `id` | integer | 当前会话内的动画 ID |
| `kind` | string | 动画事件类型 |
| `name` | string / nil | 标记事件中出现，表示当前触发的标记名称 |
| `completed` | integer / nil | 循环事件中出现，表示已经完成的循环次数 |

### 发送条件

动画开始播放、触发标记、完成循环、结束或取消时，发送给创建该动画的游戏、屏保会话。

### 示例

```lua
function HandleEvent(event)
  if event.type == "animation" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "animation",
  frame = X,
  sequence = X,
  data =
  {
    id = 2,
    kind = "marker",
    name = "impact",
  },
}
```

### 额外补充

- 字段 `kind` 包含以下固定值：

```lua
"started"   -- 动画开始
"marker"    -- 触发动画标记
"loop"      -- 完成一次循环
"finished"  -- 动画结束
"cancelled" -- 动画取消
```

- `name` 仅在 `kind == "marker"` 时出现。
- `completed` 仅在 `kind == "loop"` 时出现。
- 动画标记事件不会合并。

---

## `file`

异步文件操作结果事件。

### 结构

```lua
{
  type = "file",
  data = {
    request_id = ...,  -- integer
    kind = ...,        -- string
    path = ...,        -- string
    tip = ...,         -- string / nil
    ok = ...,          -- boolean
    text = ...,        -- string / nil
    bytes = ...,       -- string / nil
    entries = ...,     -- table / nil
    error = ...,       -- table / nil
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `request_id` | integer | 当前会话内的请求 ID |
| `kind` | string | 文件操作类型 |
| `path` | string | 调用方可见的虚拟相对路径 |
| `tip` | string / nil | 请求传入 event_tip 时出现，原样返回调用方的事件标记 |
| `ok` | boolean | 操作是否成功完成 |
| `text` | string / nil | 读取文本成功时出现，表示解码后的文本 |
| `bytes` | string / nil | 读取二进制内容成功时出现，类型为 Lua 二进制字符串 |
| `entries` | table / nil | 列出文件成功时出现，表示文件条目数组 |
| `error` | table / nil | 操作失败时出现，包含通用错误码和错误说明 |

### 发送条件

异步文件请求成功完成或失败时，发送给登记该请求的会话。屏保只能接收自身只读文件请求的结果，不接收写入和目录操作请求的结果。

### 示例

```lua
function HandleEvent(event)
  if event.type == "file" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "file",
  frame = X,
  sequence = X,
  data =
  {
    request_id = 4,
    kind = "read_text",
    path = "config/state.txt",
    tip = "load_state",
    ok = true,
    text = "content",
  },
}

{
  type = "file",
  frame = X,
  sequence = X,
  data =
  {
    request_id = 5,
    kind = "create_dir",
    path = "save/slot-a",
    tip = "create_slot",
    ok = true,
  },
}

{
  type = "file",
  frame = X,
  sequence = X,
  data =
  {
    request_id = 6,
    kind = "remove",
    path = "save/slot-a",
    tip = "remove_slot",
    ok = true,
  },
}
```

### 额外补充

- 字段 `kind` 包含以下固定值：

```lua
"read_text"   -- 读取文本
"read_bytes"  -- 读取二进制内容
"write_text"  -- 写入文本
"write_bytes" -- 写入二进制内容
"list_dir"    -- 列出文件
"create_dir"  -- 创建目录
"remove"      -- 删除文件或目录
```

- `file.read` 和 `file.write` 默认产生 `read_text`、`write_text`；传入 `byte = true` 时产生 `read_bytes`、`write_bytes`。
- 同步的 `file.exists` 直接返回布尔值，不产生事件。
- `path` 为虚拟相对路径，不是操作系统绝对路径。
- `text` 经过严格解码，换行统一为 `\n`。`text`、`bytes`、`entries` 互斥。
- `write_text`、`write_bytes`、`create_dir` 和 `remove` 成功时不携带正文；除请求标识、操作类型、路径和可选标记外，只携带 `ok = true`。
- 创建目录与删除操作仅允许关闭安全模式的游戏发起。
- `entries` 只包含文件，目录仅在递归扫描时使用，不作为条目返回。每个条目的结构如下：

```lua
{
  path = ...,      -- string
  file_type = ..., -- string
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `path` | string | 相对于安全文件根目录的虚拟文件路径，例如 `src/main.rs` |
| `file_type` | string | 不含点号的扩展名，例如 `rs` |

- 失败时 `ok == false`，并携带 `error`；异步文件操作的终态事件不会合并。

---

## `image`

图片转换结果事件。

### 结构

```lua
{
  type = "image",
  data = {
    request_id = ...,  -- integer
    kind = ...,        -- string
    ok = ...,          -- boolean
    output = ...,      -- string / nil
    error = ...,       -- table / nil
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `request_id` | integer | 当前会话内的请求 ID |
| `kind` | string | 图片操作类型 |
| `ok` | boolean | 转换是否成功 |
| `output` | string / nil | 转换成功时出现，表示转换结果的虚拟标识或路径 |
| `error` | table / nil | 转换失败时出现，包含通用错误码和错误说明 |

### 发送条件

图片转换任务成功完成或失败时，发送给登记该请求的游戏、屏保会话。

### 示例

```lua
function HandleEvent(event)
  if event.type == "image" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "image",
  frame = X,
  sequence = X,
  data =
  {
    request_id = 5,
    kind = "convert",
    ok = true,
    output = "converted-image-id",
  },
}
```

### 额外补充

- 字段 `kind` 固定为 `"convert"`。
- 转换成功时携带 `output`；转换失败时携带 `error`，两者不会同时出现。
- 图片转换的终态事件不会合并。

---

## `network`

网络请求结果事件。

### 结构

```lua
{
  type = "network",
  data = {
    request_id = ...,  -- integer
    kind = ...,        -- string
    url = ...,         -- string
    ok = ...,          -- boolean
    final_url = ...,   -- string / nil
    status = ...,      -- integer / nil
    headers = ...,     -- table / nil
    text = ...,        -- string / nil
    bytes = ...,       -- string / nil
    error = ...,       -- table / nil
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `request_id` | integer | 当前会话内的请求 ID |
| `kind` | string | 网络请求类型 |
| `url` | string | 原始规范化 URL |
| `ok` | boolean | 请求是否正常完成 |
| `final_url` | string / nil | 请求成功时出现，表示重定向后的最终 URL |
| `status` | integer / nil | 请求成功时出现，表示 HTTP 状态码 |
| `headers` | table / nil | 请求成功时出现，表示字符串键和值组成的响应头表 |
| `text` | string / nil | 文本响应模式成功时出现，表示严格 UTF-8 响应正文 |
| `bytes` | string / nil | 二进制响应模式成功时出现，类型为 Lua 二进制字符串 |
| `error` | table / nil | 请求失败时出现，包含通用错误码和错误说明 |

### 发送条件

GET 或 POST 请求产生最终结果时，发送给登记该请求的游戏、屏保会话。每个请求只产生一个终态结果。

### 示例

```lua
function HandleEvent(event)
  if event.type == "network" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "network",
  frame = X,
  sequence = X,
  data =
  {
    request_id = 6,
    kind = "get",
    url = "https://example.com/data",
    ok = true,
    final_url = "https://example.com/data",
    status = 200,
    headers = { ["content-type"] = "application/json" },
    text = "{}",
  },
}
```

### 额外补充

- 字段 `kind` 包含以下固定值：

```lua
"get"  -- GET 请求
"post" -- POST 请求
```

- HTTP `4xx`、`5xx` 表示已收到 HTTP 响应，`ok` 仍为 `true`；校验、传输、超时或取消等失败才令 `ok` 为 `false`。
- `headers` 只保留允许暴露的响应头，键名为小写，重复值用逗号连接。
- `text` 与 `bytes` 互斥，具体取决于请求使用的响应模式。
- 请求取消时，`error.code` 为 `"cancelled"`。
- URL、响应头和错误在进入脚本前会经过安全过滤。
- 网络请求的终态事件不会合并。

---

## `i18n`

国际化语言加载结果事件。

### 结构

```lua
{
  type = "i18n",
  data = {
    kind = ...,                    -- string
    ok = ...,                      -- boolean
    message = ...,                 -- string
    language_code = ...,           -- string
    callback_language_code = ...,  -- string
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `kind` | string | 语言加载事件类型 |
| `ok` | boolean | 本次语言加载是否成功 |
| `message` | string | 经过净化的加载结果说明 |
| `language_code` | string | 成功时为实际加载的主语言或备用语言代码，失败时为请求的主语言代码 |
| `callback_language_code` | string | 本次请求使用的备用语言代码 |

### 发送条件

调用 `i18n.create {}` 或 `i18n.reload {}` 后，包语言文件异步加载结束时，发送给发起请求的游戏、屏保会话。

### 示例

```lua
function HandleEvent(event)
  if event.type == "i18n" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "i18n",
  frame = X,
  sequence = X,
  data =
  {
    kind = "created",
    ok = true,
    message = "i18n instance created",
    language_code = "zh_cn",
    callback_language_code = "en_us",
  },
}
```

### 额外补充

- 字段 `kind` 包含以下固定值：

```lua
"created"  -- create 请求结束
"reloaded" -- reload 请求结束
```

- 事件进入 `HandleEvent` 前，宿主已经提交成功加载的语言数据，可立即调用 `i18n.get_value {}` 获取文本。
- 包语言文件从 `assets/language/<language_code>/*.json` 读取，不递归扫描子目录；每个命名空间 JSON 必须是单层对象，所有值必须是字符串。
- 主语言缺少的命名空间和键由备用语言补齐，已有主语言值不会被覆盖。
- 两种语言都没有某个键时，`i18n.get_value {}` 使用宿主当前语言的 `language_warning.missing` 文本生成缺失提示。
- `message` 不包含绝对路径、系统错误或宿主任务 ID。
- `reload` 失败时保留上一次成功加载的数据。

---

## `audio`

音频对象状态变化事件。

### 结构

```lua
{
  type = "audio",
  data = {
    id = ...,           -- integer
    kind = ...,         -- string
    duration_ms = ...,  -- integer / nil
    position_ms = ...,  -- integer / nil
    error = ...,        -- table / nil
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `id` | integer | 当前会话内的音频对象 ID |
| `kind` | string | 音频事件类型 |
| `duration_ms` | integer / nil | ready、finished 事件中出现，表示音频总时长，单位为毫秒 |
| `position_ms` | integer / nil | started、paused、resumed、finished 事件中出现，表示当前播放位置，单位为毫秒 |
| `error` | table / nil | failed 事件中出现，包含通用错误码和错误说明 |

### 发送条件

音频对象就绪、开始、暂停、恢复、停止、播放结束或失败时，发送给登记该对象的游戏、屏保会话。

### 示例

```lua
function HandleEvent(event)
  if event.type == "audio" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "audio",
  frame = X,
  sequence = X,
  data =
  {
    id = 7,
    kind = "paused",
    position_ms = 530,
  },
}
```

### 额外补充

- 字段 `kind` 包含以下固定值：

```lua
"ready"    -- 音频就绪
"started"  -- 开始播放
"paused"   -- 暂停播放
"resumed"  -- 恢复播放
"stopped"  -- 停止播放
"finished" -- 播放结束
"failed"   -- 音频操作失败
```

- `finished` 事件中的 `position_ms` 等于音频总时长。
- 音频回调是与对象生命周期绑定的持久回调。`finished` 不会自动删除对象或回调，同一对象仍可再次播放。
- 删除对象或停止会话时，才会清理相应回调。
- 全局音频后端故障、录音保存事件和其他对象的状态不会作为该对象的事件发送给脚本。
- 常见音频错误码为 `"decode"` 和 `"backend_unavailable"`。

---

## `hit_area`

点击区域交互事件。

### 结构

```lua
{
  type = "hit_area",
  data = {
    id = ...,      -- integer
    kind = ...,    -- string
    x = ...,       -- integer
    y = ...,       -- integer
    button = ...,  -- string / nil
    dx = ...,      -- integer / nil
    dy = ...,      -- integer / nil
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `id` | integer | 当前会话内的点击区域 ID |
| `kind` | string | 点击区域事件类型 |
| `x` | integer | 事件的水平坐标 |
| `y` | integer | 事件的垂直坐标 |
| `button` | string / nil | 按下、松开、点击或拖动时出现，表示对应鼠标键 |
| `dx` | integer / nil | 拖动时出现，表示本次水平位移 |
| `dy` | integer / nil | 拖动时出现，表示本次垂直位移 |

### 发送条件

玩家在点击区域内悬停、移动、按下、松开、点击或拖动鼠标，或将鼠标移出点击区域时，发送给创建并登记该组件的游戏会话。覆盖屏接管交互期间不发送，屏保不接收此事件。

### 示例

```lua
function HandleEvent(event)
  if event.type == "hit_area" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "hit_area",
  frame = X,
  sequence = X,
  data =
  {
    id = 8,
    kind = "drag",
    x = 30,
    y = 12,
    button = "left",
    dx = 2,
    dy = -1,
  },
}
```

### 额外补充

- 字段 `kind` 包含以下固定值：

```lua
"hover_enter" -- 鼠标进入区域
"hover_move"  -- 鼠标在区域内移动
"hover_leave" -- 鼠标离开区域
"press"       -- 按下鼠标键
"release"     -- 松开鼠标键
"click"       -- 点击区域
"drag"        -- 拖动鼠标
```

- 字段 `button` 的值为 `"left"`、`"middle"` 或 `"right"`，仅在 `press`、`release`、`click`、`drag` 事件中出现。
- `dx`、`dy` 仅在 `kind == "drag"` 时出现。
- 同一对象尚未处理的 `hover_move` 事件只保留最新坐标，拖动事件不会合并。

---

## `hyperlink`

超链接点击事件。

### 结构

```lua
{
  type = "hyperlink",
  data = {
    id = ...,    -- integer
    kind = ...,  -- string
    link = ...,  -- string
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `id` | integer | 当前会话内的超链接对象 ID |
| `kind` | string | 超链接事件类型 |
| `link` | string | 超链接目标 |

### 发送条件

玩家点击超链接时，发送给创建并登记该组件的游戏会话。覆盖屏接管交互期间不发送，屏保不接收此事件。

### 示例

```lua
function HandleEvent(event)
  if event.type == "hyperlink" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "hyperlink",
  frame = X,
  sequence = X,
  data =
  {
    id = 9,
    kind = "clicked",
    link = "https://example.com",
  },
}
```

### 额外补充

- 字段 `kind` 固定为 `"clicked"`。
- `link` 表示被点击的超链接目标。

---

## `markdown`

Markdown 文本中的链接点击事件。

### 结构

```lua
{
  type = "markdown",
  data = {
    id = ...,    -- integer
    kind = ...,  -- string
    href = ...,  -- string
    text = ...,  -- string
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `id` | integer | 当前会话内的 Markdown 对象 ID |
| `kind` | string | Markdown 事件类型 |
| `href` | string | 被点击链接的目标 |
| `text` | string | 被点击链接的显示文本 |

### 发送条件

玩家点击 Markdown 文本中的链接时，发送给创建并登记该组件的游戏会话。覆盖屏接管交互期间不发送，屏保不接收此事件。

### 示例

```lua
function HandleEvent(event)
  if event.type == "markdown" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "markdown",
  frame = X,
  sequence = X,
  data =
  {
    id = 10,
    kind = "link_clicked",
    href = "guide.md",
    text = "Guide",
  },
}
```

### 额外补充

- 字段 `kind` 固定为 `"link_clicked"`。
- `href` 表示链接目标，`text` 表示该链接在 Markdown 中显示的文本。

---

## `text_input`

文本输入组件的状态变化或交互事件。

### 结构

```lua
{
  type = "text_input",
  data = {
    id = ...,     -- integer
    kind = ...,   -- string
    value = ...,  -- string / nil
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `id` | integer | 当前会话内的文本输入对象 ID |
| `kind` | string | 文本输入事件类型 |
| `value` | string / nil | 内容变化、提交或取消时出现，表示当时的文本内容 |

### 发送条件

文本输入组件获得或失去焦点、内容变化、提交、取消，或发生组件内外的按下操作时，发送给创建并登记该组件的游戏会话。覆盖屏接管交互期间不发送，屏保不接收此事件。

### 示例

```lua
function HandleEvent(event)
  if event.type == "text_input" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "text_input",
  frame = X,
  sequence = X,
  data =
  {
    id = 11,
    kind = "changed",
    value = "player",
  },
}
```

### 额外补充

- 字段 `kind` 包含以下固定值：

```lua
"focused"         -- 获得焦点
"blurred"         -- 失去焦点
"changed"         -- 文本内容变化
"submit"          -- 提交文本
"cancel"          -- 取消输入
"pressed"         -- 在组件内按下
"pressed_outside" -- 在组件外按下
```

- 字段 `value` 仅在 `changed`、`submit`、`cancel` 事件中出现。

---

## `scroll_box`

滚动框位置变化事件。

### 结构

```lua
{
  type = "scroll_box",
  data = {
    id = ...,    -- integer
    kind = ...,  -- string
    x = ...,     -- integer
    y = ...,     -- integer
  },
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `id` | integer | 当前会话内的滚动框 ID |
| `kind` | string | 滚动框事件类型 |
| `x` | integer | 当前水平滚动位置 |
| `y` | integer | 当前垂直滚动位置 |

### 发送条件

滚动框的滚动位置变化时，发送给创建并登记该组件的游戏会话。覆盖屏接管交互期间不发送，屏保不接收此事件。

### 示例

```lua
function HandleEvent(event)
  if event.type == "scroll_box" then
    debug.print { message = table.pretty(event) }
  end
end
```

输出：

> X 为占位符

```lua
{
  type = "scroll_box",
  frame = X,
  sequence = X,
  data =
  {
    id = 12,
    kind = "scrolled",
    x = 5,
    y = 20,
  },
}
```

### 额外补充

- 字段 `kind` 固定为 `"scrolled"`。
- `x`、`y` 表示当前滚动位置。
- 同一对象尚未处理的滚动事件只保留最新滚动位置。
