# Lua 事件协议

本文档描述 Rust 宿主当前能够投递给游戏或屏保 Lua Session 的全部事件。事件只用于宿主向脚本通知状态变化；原始终端按键、宿主 UI 事件、内部任务 ID、绝对路径和内部错误信息不会暴露给 Lua。

## 1. 通用结构

所有事件都使用同一个信封：

```lua
{
  type = "action",
  sequence = 42,
  frame = 1800,
  data = {
    action = "jump",
    state = "pressed",
  },
} 
```

| 字段 | 类型 | 必定存在 | 作用 |
|---|---|---:|---|
| `type` | `string` | 是 | 事件类型。根据它判断 `data` 的结构。 |
| `sequence` | `integer` | 是 | Runtime 全局单调递增的事件序号。事件经过 Session 过滤后可能出现跳号。 |
| `frame` | `integer` | 是 | 事件进入 Lua Broker 时的宿主帧号，不等同于游戏自行维护的帧号。 |
| `data` | `table` | 是 | 事件数据。没有额外数据的生命周期事件也会得到空表。 |

未列出的可选字段为 `nil`。脚本不应依赖 Lua 表的字段遍历顺序。

### 1.1 投递方式

- 没有为对象或异步请求注册回调时，事件交给 `HandleEvent(event)`。
- 注册了回调时，完整事件信封只交给该回调，不再重复交给 `HandleEvent`。
- 回调在 Runtime 主线程执行，并使用与 `HandleEvent` 相同的时间和指令预算。
- 事件处理期间产生的新事件追加至队尾，最早在下一宿主帧投递，不会递归调用 Lua。
- 每个游戏和屏保 Session 各有独立队列；单帧最多处理 128 条，待处理上限为 1024 条。
- 队列溢出只会使对应 Session 故障，不应导致宿主崩溃或影响另一个 Session。
- 对象 ID、请求 ID 均为 Session 内局部、不透明的整数。Session 重启后旧 ID 不再有效。

### 1.2 Session 接收范围

| 分类 | 游戏 | 屏保 | 条件 |
|---|---:|---:|---|
| `action`、`mouse` | 是 | 否 | 仅没有覆盖屏接管交互时投递。 |
| `resize`、`focus` | 是 | 是 | Session 存活时均可投递，包括覆盖屏期间。 |
| `overlay_started`、`overlay_stopped` | 是 | 否 | 只通知游戏 Session。 |
| `timer`、`animation` | 是 | 是 | 只能收到本 Session 所创建对象的事件。 |
| `file` | 是 | 只读 | 只能收到本 Session 登记的请求结果；屏保不接收写入和目录请求。 |
| `i18n`、`image`、`network`、`audio` | 是 | 是 | 只能收到本 Session 登记的请求或对象事件；API 权限仍可能拒绝创建请求。 |
| 交互组件事件 | 是 | 否 | 只能收到本 Session 所创建组件的事件。 |

任意覆盖屏处于栈内时，游戏仍可更新，并继续接收非交互事件，但不会接收动作、鼠标或组件交互事件。屏保本身也不接收键盘、鼠标和交互组件事件。

## 2. 系统与输入事件

### 2.1 `action`

游戏动作状态发生变化时发送。宿主先处理全局按键，剩余输入才按游戏包的用户动作映射转换为此事件。Lua 永远不会收到原始 `TerminalKeyEvent`。

```lua
{
  type = "action",
  data = {
    action = "move_left",
    state = "pressed",
  },
}
```

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `action` | `string` | 始终 | 游戏包注册的动作 ID。 |
| `state` | `string` | 始终 | `pressed`、`held` 或 `released`。 |

覆盖屏出现后，尚未投递的交互事件会被清理，避免截屏模式、尺寸提示等宿主输入穿透给游戏。

### 2.2 `mouse`

鼠标位于游戏 Base 可视区域且终端拥有焦点时发送。坐标以 Base 左上角为原点，从 0 开始。游戏包中的鼠标声明用于能力说明，不作为事件权限开关。

```lua
{
  type = "mouse",
  data = {
    kind = "pressed",
    button = "left",
    scroll = nil,
    x = 12,
    y = 4,
  },
}
```

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `kind` | `string` | 始终 | `pressed`、`released`、`moved`、`dragged`、`held`、`scrolled`。 |
| `button` | `string \| nil` | 有对应鼠标键时 | `left`、`middle` 或 `right`。 |
| `scroll` | `string \| nil` | 滚动事件时 | `up`、`down`、`left` 或 `right`。 |
| `x` | `integer` | 始终 | Base 内水平单元格坐标。 |
| `y` | `integer` | 始终 | Base 内垂直单元格坐标。 |

### 2.3 `resize`

终端尺寸变化并引起当前 Session 的 Base 画布尺寸更新时，发送给当前存活的游戏和屏保 Session。宿主会针对不同 Session 分别换算尺寸，不会把物理终端宽高直接暴露给 Lua。

```lua
{
  type = "resize",
  data = { width = 120, height = 40 },
}
```

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `width` | `integer` | 始终 | 当前 Session 的 Base 画布宽度，单位为单元格。 |
| `height` | `integer` | 始终 | 当前 Session 的 Base 画布高度，单位为单元格。 |

`resize` 使用独立观察通道，即使覆盖屏消费了本帧输入，该事件仍可送达 Lua。

### 2.4 `focus`

终端获得或失去焦点时发送给当前存活的游戏和屏保 Session。

```lua
{
  type = "focus",
  data = { gained = false },
}
```

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `gained` | `boolean` | 始终 | `true` 表示获得焦点，`false` 表示失去焦点。 |

失去焦点不会额外伪造所有动作的 `released`。游戏应在 `gained == false` 时清理自行保存的输入状态。

## 3. 覆盖屏生命周期事件

覆盖屏包括截屏模式、尺寸提醒、屏保以及以后加入同一覆盖屏栈的界面。

### 3.1 `overlay_started`

覆盖屏栈从空变为非空时发送给游戏：

```lua
{
  type = "overlay_started",
  data = {},
}
```

该事件表示宿主覆盖屏开始接管可视或交互区域。第一个覆盖屏之上再加入其他覆盖屏时不会重复发送。

### 3.2 `overlay_stopped`

覆盖屏栈从非空变为空时发送给游戏：

```lua
{
  type = "overlay_stopped",
  data = {},
}
```

移除顶层覆盖屏但栈内仍存在其他覆盖屏时不会发送。只有最后一个覆盖屏退出后才发送。因此这两个事件描述的是整个覆盖阶段，不代表某一种具体覆盖屏。

## 4. 计时器与动画事件

### 4.1 `timer`

只发送给创建计时器或休眠请求的 Session。

```lua
{
  type = "timer",
  data = {
    id = 1,
    timer_kind = "repeat",
    kind = "tick",
    executed_count = 3,
  },
}
```

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `id` | `integer` | 始终 | Session 内计时器 ID。 |
| `timer_kind` | `string` | 始终 | `timer`、`delay`、`repeat` 或 `sleep`。 |
| `kind` | `string` | 始终 | `tick` 或 `finished`。 |
| `executed_count` | `integer \| nil` | `repeat` 事件 | 已完成的触发次数；重复计时器的 `tick` 和最终 `finished` 都会携带。 |

- `timer`、`delay`、`sleep` 只产生一次 `finished`。
- `repeat` 每次触发产生 `tick`，结束时产生 `finished`。
- 一次性计时器的终态回调在投递后回收；重复对象的回调保留至结束或取消。

### 4.2 `animation`

只发送给创建动画的 Session。

```lua
{
  type = "animation",
  data = {
    id = 2,
    kind = "marker",
    name = "impact",
  },
}
```

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `id` | `integer` | 始终 | Session 内动画 ID。 |
| `kind` | `string` | 始终 | `started`、`marker`、`loop`、`finished` 或 `cancelled`。 |
| `name` | `string \| nil` | `kind == "marker"` | 当前触发的标记名称。 |
| `completed` | `integer \| nil` | `kind == "loop"` | 已完成的循环次数。 |

## 5. 异步服务结果

异步服务只会把结果交给登记该任务的 Session。Lua 看到的是 Session 内 `request_id`，不是宿主任务 ID。

### 5.1 通用错误表

失败结果使用净化后的错误：

```lua
error = {
  code = "timeout",
  message = "request timed out",
}
```

| 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `code` | `string` | 始终 | 稳定错误码，适合程序分支判断。 |
| `message` | `string` | 始终 | 面向开发者的通用描述，不含绝对路径、系统调用栈、宿主 ID 或敏感请求内容。 |

可能出现的错误码：

- `invalid_request`
- `permission_denied`
- `not_found`
- `too_large`
- `invalid_utf8`
- `cancelled`
- `timeout`
- `io`
- `network`
- `unsupported`
- `decode`
- `backend_unavailable`
- `internal`

### 5.2 `file`

异步文件请求完成或失败时发送。`file.read` 和 `file.write` 默认产生 `read_text`、`write_text`；传入 `byte = true` 时产生 `read_bytes`、`write_bytes`。其他公开文件 API 会产生 `list_dir`、`create_dir` 和 `remove`。同步的 `file.exists` 直接返回布尔值，不产生事件。

```lua
{
  type = "file",
  data = {
    request_id = 4,
    kind = "read_text",
    path = "config/state.txt",
    tip = "load_state",
    ok = true,
    text = "content",
  },
}
```

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `request_id` | `integer` | 始终 | Session 内请求 ID。 |
| `kind` | `string` | 始终 | `read_text`、`read_bytes`、`write_text`、`write_bytes`、`list_dir`、`create_dir` 或 `remove`。 |
| `path` | `string` | 始终 | 调用方可见的虚拟相对路径，不是操作系统绝对路径。 |
| `tip` | `string \| nil` | 请求传入 `event_tip` 时 | 调用方自定义的事件标记，原样返回以便区分请求。 |
| `ok` | `boolean` | 始终 | 操作是否成功完成。 |
| `text` | `string \| nil` | `read_text` 成功 | 经过严格解码且换行统一为 `\n` 的文本。 |
| `bytes` | `string \| nil` | `read_bytes` 成功 | Lua 二进制字符串。 |
| `entries` | `table \| nil` | `list_dir` 成功 | 文件条目数组。目录仅在递归扫描时使用，不作为条目返回。 |
| `error` | `table \| nil` | `ok == false` | 通用错误表。 |

`entries` 的每个元素为：

```lua
{
  path = "src/main.rs",
  file_type = "rs",
}
```

| 条目字段 | 类型 | 作用 |
|---|---|---|
| `path` | `string` | 相对于安全文件根目录的虚拟文件路径。 |
| `file_type` | `string` | 不含点号的扩展名，例如 `rs`。 |

`write_text`、`write_bytes`、`create_dir` 和 `remove` 成功时只携带 `ok = true`，不携带正文。`text`、`bytes`、`entries` 互斥。屏保只允许收到自身只读文件请求的结果；创建目录与删除操作仅允许游戏会话发起，并仍受包内 `assets/` 路径限制。

创建目录成功事件示例：

```lua
{
  type = "file",
  data = {
    request_id = 5,
    kind = "create_dir",
    path = "save/slot-a",
    tip = "create_slot",
    ok = true,
  },
}
```

删除成功事件示例：

```lua
{
  type = "file",
  data = {
    request_id = 6,
    kind = "remove",
    path = "save/slot-a",
    tip = "remove_slot",
    ok = true,
  },
}
```

### 5.3 `image`

图片转换任务结束时发送。

```lua
{
  type = "image",
  data = {
    request_id = 5,
    kind = "convert",
    ok = true,
    output = "converted-image-id",
  },
}
```

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `request_id` | `integer` | 始终 | Session 内请求 ID。 |
| `kind` | `string` | 始终 | 固定为 `convert`。 |
| `ok` | `boolean` | 始终 | 转换是否成功。 |
| `output` | `string \| nil` | `ok == true` | 转换结果的虚拟标识或路径。 |
| `error` | `table \| nil` | `ok == false` | 通用错误表。 |

### 5.4 `network`

GET 或 POST 请求产生唯一终态结果。HTTP 4xx/5xx 是成功收到的 HTTP 响应，因此 `ok` 仍为 `true`；只有校验、传输、超时、取消等失败才令 `ok` 为 `false`。

```lua
{
  type = "network",
  data = {
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

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `request_id` | `integer` | 始终 | Session 内请求 ID。 |
| `kind` | `string` | 始终 | `get` 或 `post`。 |
| `url` | `string` | 始终 | 原始规范化 URL。 |
| `ok` | `boolean` | 始终 | 请求是否正常完成。 |
| `final_url` | `string \| nil` | `ok == true` | 完成重定向后的最终 URL。 |
| `status` | `integer \| nil` | `ok == true` | HTTP 状态码。 |
| `headers` | `table<string, string> \| nil` | `ok == true` | 白名单过滤后的响应头；键名为小写，重复值用逗号连接。 |
| `text` | `string \| nil` | 文本响应模式成功 | 严格 UTF-8 响应正文。 |
| `bytes` | `string \| nil` | 二进制响应模式成功 | Lua 二进制字符串。 |
| `error` | `table \| nil` | `ok == false` | 通用错误表；取消会使用 `cancelled`。 |

`text` 与 `bytes` 互斥。URL、响应头和错误在进入 Lua 前会经过安全过滤。

### 5.5 `i18n`

调用 `i18n.create{}` 或 `i18n.reload{}` 后，在包语言文件异步加载结束时发送。该事件只投递给发起请求的游戏或屏保 Session。事件进入 `HandleEvent` 前，宿主已经提交成功加载的语言数据，因此可在处理该事件时立即调用 `i18n.get_value{}`。

```lua
{
  type = "i18n",
  data = {
    kind = "created",
    ok = true,
    message = "i18n instance created",
    language_code = "zh_cn",
    callback_language_code = "en_us",
  },
}
```

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `kind` | `string` | 始终 | `created` 表示 `create` 请求结束，`reloaded` 表示 `reload` 请求结束。 |
| `ok` | `boolean` | 始终 | 本次语言加载是否成功。 |
| `message` | `string` | 始终 | 已净化的加载结果说明，不包含绝对路径、系统错误或宿主任务 ID。 |
| `language_code` | `string` | 始终 | 成功时为实际加载的主语言代码；主语言不存在而使用备用语言时为备用语言代码。失败时为本次请求的主语言代码。 |
| `callback_language_code` | `string` | 始终 | 本次请求使用的备用语言代码。 |

- 包语言文件只从 `assets/language/<language_code>/*.json` 读取，不递归扫描子目录。
- 每个命名空间 JSON 必须是单层对象，并且所有值都必须是字符串。
- 主语言成功加载后，缺失的命名空间和键由备用语言补齐；已有主语言值不会被覆盖。
- 两种语言都没有某个键时，`i18n.get_value{}` 使用宿主当前语言的 `language_warning.missing` 文本生成最终缺失提示。
- 加载失败不会把内部文件路径或解析细节暴露给事件。`reload` 失败时继续保留上一次成功加载的数据。

## 6. 音频事件

### 6.1 `audio`

只为已经登记到该 Session 的音频对象发送。全局音频后端故障、录音保存事件和其他对象的状态不会泄漏给脚本。

```lua
{
  type = "audio",
  data = {
    id = 7,
    kind = "paused",
    position_ms = 530,
  },
}
```

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `id` | `integer` | 始终 | Session 内音频对象 ID。 |
| `kind` | `string` | 始终 | `ready`、`started`、`paused`、`resumed`、`stopped`、`finished` 或 `failed`。 |
| `duration_ms` | `integer \| nil` | `ready`、`finished` | 音频总时长，单位毫秒。 |
| `position_ms` | `integer \| nil` | `started`、`paused`、`resumed`、`finished` | 当前播放位置，单位毫秒；`finished` 时等于总时长。 |
| `error` | `table \| nil` | `kind == "failed"` | 通用错误表，常见音频错误为 `decode`、`backend_unavailable`。 |

音频回调与对象生命周期绑定，是持久回调。`finished` 不会自动删除对象或回调，因为同一对象仍可再次播放；删除对象或停止 Session 时才清理。

## 7. 交互组件事件

以下事件只发送给游戏 Session，并且必须来自该 Session 自己创建和登记的组件。覆盖屏接管交互期间不会投递。

### 7.1 `hit_area`

```lua
{
  type = "hit_area",
  data = {
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

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `id` | `integer` | 始终 | Session 内点击区域 ID。 |
| `kind` | `string` | 始终 | `hover_enter`、`hover_move`、`hover_leave`、`press`、`release`、`click` 或 `drag`。 |
| `x` | `integer` | 始终 | 事件水平坐标。 |
| `y` | `integer` | 始终 | 事件垂直坐标。 |
| `button` | `string \| nil` | `press`、`release`、`click`、`drag` | `left`、`middle` 或 `right`。 |
| `dx` | `integer \| nil` | `drag` | 本次拖动的水平位移。 |
| `dy` | `integer \| nil` | `drag` | 本次拖动的垂直位移。 |

### 7.2 `hyperlink`

```lua
{
  type = "hyperlink",
  data = { id = 9, kind = "clicked", link = "https://example.com" },
}
```

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `id` | `integer` | 始终 | Session 内超链接对象 ID。 |
| `kind` | `string` | 始终 | 固定为 `clicked`。 |
| `link` | `string` | 始终 | 超链接目标。 |

### 7.3 `markdown`

```lua
{
  type = "markdown",
  data = { id = 10, kind = "link_clicked", href = "guide.md", text = "Guide" },
}
```

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `id` | `integer` | 始终 | Session 内 Markdown 对象 ID。 |
| `kind` | `string` | 始终 | 固定为 `link_clicked`。 |
| `href` | `string` | 始终 | 链接目标。 |
| `text` | `string` | 始终 | 链接显示文本。 |

### 7.4 `text_input`

```lua
{
  type = "text_input",
  data = { id = 11, kind = "changed", value = "player" },
}
```

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `id` | `integer` | 始终 | Session 内文本输入对象 ID。 |
| `kind` | `string` | 始终 | `focused`、`blurred`、`changed`、`submit`、`cancel`、`pressed` 或 `pressed_outside`。 |
| `value` | `string \| nil` | `changed`、`submit`、`cancel` | 当时的文本内容。 |

### 7.5 `scroll_box`

```lua
{
  type = "scroll_box",
  data = { id = 12, kind = "scrolled", x = 5, y = 20 },
}
```

| `data` 字段 | 类型 | 出现条件 | 作用 |
|---|---|---|---|
| `id` | `integer` | 始终 | Session 内滚动框 ID。 |
| `kind` | `string` | 始终 | 固定为 `scrolled`。 |
| `x` | `integer` | 始终 | 当前水平滚动位置。 |
| `y` | `integer` | 始终 | 当前垂直滚动位置。 |

## 8. 合并、过滤与生命周期清理

为避免高频事件塞满队列，尚未处理的以下事件可以被更新值替换：

- `resize`：保留最新尺寸。
- `mouse`：同一 `kind` 和 `button` 的 `moved` 或 `held` 保留最新坐标。
- `hit_area`：同一对象的 `hover_move` 保留最新坐标。
- `scroll_box`：同一对象保留最新滚动位置。

以下事件不会合并：动作按下/按住/释放、鼠标按键、拖动、滚轮、焦点、覆盖屏生命周期、计时器、动画标记以及所有异步终态事件。

Session 停止或代次改变时，宿主会清理它的待处理事件、对象所有权、回调和异步任务映射。旧 Session 的迟到结果会被丢弃，不会投递给之后启动的新 Session。

## 9. 完整 `type` 清单

当前协议中的全部事件类型为：

```text
action
mouse
resize
focus
overlay_started
overlay_stopped
timer
animation
file
image
network
i18n
audio
hit_area
hyperlink
markdown
text_input
scroll_box
```

宿主包扫描、日志、Popup、截屏/录屏/导出队列、普通 `TaskFinished/TaskFailed` 和宿主自身 UI 对象事件均不属于 Lua 事件协议。
