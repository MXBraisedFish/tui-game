# 文档信息

1. 更新日期：2026年4月9日
2. API 版本：**7**
3. 本文档定义了脚本与宿主之间的交互接口规范，所有实现须遵循其中约定的函数签名、参数类型及行为准则，以确保兼容性与正确性。

# 文档导航

- [README](../../README-i18n/README-zh-cn.md)
- [MOD 制作规范及教程](./MOD.md)
- [富文本指令](./RICH_TEXT.md)

# 目录

- [语义歧义消除](#语义歧义消除)
- [声明式 API](#声明式-api)
  - [API 列表](#api-列表)
  - [执行流程](#执行流程)
  - [使用示例](#使用示例)
  - [数据格式](#数据格式)
  - [事件类型](#事件类型)
  - [注意事项](#注意事项)
- [直用式 API](#直用式-api)
  - [系统请求](#系统请求)
  - [内容绘制](#内容绘制)
  - [内容尺寸计算](#内容尺寸计算)
  - [布局定位计算](#布局定位计算)
  - [数据读取](#数据读取)
  - [数据写入](#数据写入)
  - [辅助脚本加载](#辅助脚本加载)
  - [时间处理](#时间处理)
  - [随机数](#随机数)
  - [调试信息](#调试信息)
- [附录](#附录)
  - [特定参数](#特定参数)
    - [锚点 anchor](#锚点-anchor)
    - [颜色 color](#颜色-color)
  - [调试输出目录](#调试输出目录)
- [快速查询](#快速查询)
- [异常说明](#异常说明)
  - [通用异常](@通用异常)
  - [声明式 API 异常](#声明式-api-异常)
  - [直用式 API 异常](#直用式-api-异常)
    - [内容绘制 异常](#内容绘制-异常)
    - [内容尺寸计算 异常](#内容尺寸计算-异常)
    - [布局定位计算 异常](#布局定位计算-异常)
    - [系统请求 异常](#系统请求-异常)
    - [数据读取 异常](#数据读取-异常)
    - [数据写入 异常](#数据写入-异常)
    - [辅助脚本加载 异常](#辅助脚本加载-异常)
    - [时间处理 异常](#时间处理-异常)
    - [随机数 API 异常](#随机数-api-异常)
    - [调试信息 异常](#调试信息-异常)

---

# 语义歧义消除

| 名称                | 方向        | 位置                  | 适用场景            |
| ------------------- | ----------- | --------------------- | ------------------- |
| **声明式 API 参数** | 宿主 → 脚本 | 通过函数参数传入      | 声明式 API 的参数   |
| **传递值**          | 脚本 → 宿主 | 通过 `return` 返回    | 声明式 API 的传递值 |
| **直用式 API 参数** | 脚本 → 宿主 | 通过函数参数传入      | 直用式 API 的参数   |
| **返回值**          | 宿主 → 脚本 | 直用式 API 的调用结果 | 直用式 API 的返回值 |

---

# 声明式 API

<div style="color: red;"><b>该部分包含的部分 API 必须在入口脚本中完整实现，否则脚本将无法被宿主接收或运行。</b></div>

**声明式 API 要求您在脚本中重写以下函数，并按照规范接收参数并传递(return)对应的值。**

## API 列表

以下是调整列顺序后的表格：

| 重写需求                                           | 函数名                       | 作用说明                 | 参数名                                                                                     | 参数说明                                                                   | 传递值类型                                   | 传递值说明                                                                                     | 宿主调用时机                                                       |
| -------------------------------------------------- | ---------------------------- | ------------------------ | ------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- | -------------------------------------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| <font color="red">必须重写</font>                  | `init_game(state)`           | 游戏脚本的初始化         | `state` - <font color="#92cddc">table</font> \| <font color="#92cddc">nil</font>           | 继续游戏时传入上次保存的 `state`；新游戏时传入 `nil`。                     | `state` - <font color="#92cddc">table</font> | 传递初始化后的游戏状态。宿主会将其作为当前帧数据保存，并用于后续 `handle_event` 和 `render`。  | 游戏首次启动时调用一次。                                           |
| <font color="red">必须重写</font>                  | `handle_event(state, event)` | 游戏事件逻辑处理         | `state` - <font color="#92cddc">table</font>, `event` - <font color="#92cddc">table</font> | `state`：宿主临时存储的游戏上一帧数据；<br>`event`：宿主解析后的事件信息。 | `state` - <font color="#92cddc">table</font> | 传递更新后的游戏状态。宿主会用其替换当前帧数据。                                               | 游戏运行时，每帧对事件队列中的每个事件依次调用。                   |
| <font color="red">必须重写</font>                  | `render(state)`              | 游戏画面绘制             | `state` - <font color="#92cddc">table</font>                                               | 宿主临时存储的游戏当前帧数据。                                             | <font color="#7f7f7f">无</font>              | <font color="#7f7f7f">无传递值</font>                                                          | 游戏运行时，每帧在所有事件处理完成后调用一次，脚本也可以手动调用。 |
| <font color="red">必须重写</font>                  | `exit_game(state)`           | 游戏退出前的最后一次处理 | `state` - <font color="#92cddc">table</font>                                               | 宿主临时存储的游戏当前帧数据。                                             | `state` - <font color="#92cddc">table</font> | 传递修改后的 `state`，供后续 `save_best_score` 使用。宿主不会保存此返回值。                    | 脚本调用 `request_exit()` 后，宿主在退出前调用一次。               |
| 当 `game.json` 中 `best_none` 为 `true` 时必须重写 | `save_best_score(state)`     | 向宿主传递游戏最佳记录   | `state` - <font color="#92cddc">table</font>                                               | 宿主临时存储的游戏当前帧数据（通常来自 `exit_game` 的返回值）。            | `best` - <font color="#92cddc">table</font>  | 传递包含最佳记录文本及变量表的 `best` 表，结构见下文。                                         | 宿主在 `exit_game` 之后自动调用（若启用），脚本也可手动调用。      |
| 当 `game.json` 中 `save` 为 `true` 时必须重写      | `save_game(state)`           | 保存游戏存档             | `state` - <font color="#92cddc">table</font>                                               | 宿主临时存储的游戏当前帧数据。                                             | `state` - <font color="#92cddc">table</font> | 传递用于长期存储的 `state`。**注意**：此传递值仅用于存档，当前游戏会继续使用传入的原 `state`。 | 由脚本手动调用，宿主不会自动调用。                                 |

## 执行流程

宿主与脚本运行链如下图所示：

![执行流程](./image/program_flowchart.png)

## 使用示例

```lua
function init_game(state)
    local new_state = state or {}
    -- 初始化逻辑
    return new_state
end

function handle_event(state, event)
    -- 事件处理逻辑
    -- ... 根据 event 更新 state ...
    return state
end

function render(state)
    -- 画面绘制逻辑
    -- ... 使用绘制 API 渲染当前状态 ...
end

function exit_game(state)
    -- 游戏退出前最后一次修改 state
    state.final_score = 1000
    return state
end

function save_best_score(state)
    local best = {
        best_string = "最高分：{score}",
        score = state.final_score
    }
    return best
end

function save_game(state)
    -- 存档逻辑：可在此深拷贝或修改 state 用于存储
    local saved_state = { ... }
    return saved_state
end
```

## 数据格式

> 注：
>
> - `#` 表示自定义或可变内容.
> - `[]` 表示该字段可重复出现或扩展。

### `state` 数据格式

```lua
{
  [#key] = "#value"
}
```

- `state` 可以是任意可序列化的数据结构。
- 宿主仅负责存储与传递 `state`，不解析其内部内容。

### `event` 数据格式

```lua
{
  type = "#type",
  [#name] = "#value"
}
```

- `event` 的数据结构由宿主定义并传递。
- `type` 字段决定了事件的类型，具体取值及对应的扩展字段见下文「事件类型」章节。

### `best` 数据格式

```lua
{
  best_string = "#string",
  [#key] = "#value"
}
```

- `best_string`：必填字段，用于传递最佳记录的显示文本。
- `["#key"]`：可选字段，作为 `best_string` 中对应变量的引用值，支持动态替换文本中的变量占位符。

## 事件类型

宿主会根据运行时环境产生以下类型的事件，脚本应据此进行相应逻辑处理。

### 1. `action`

```lua
{
  type = "action",
  name = "#registered_key"
}
```

**作用**：  
宿主根据 `game.json` 中的 `actions` 配置，将物理按键映射为语义化动作事件。适用于自定义动作按键的处理。

### 2. `key`

```lua
{
  type = "key",
  name = "#enter"
}
```

**作用**：  
宿主输出原始按键事件。适用于处理未在 `actions` 中注册的按键。

### 3. `resize`

```lua
{
  type = "resize",
  width = int,
  height = int
}
```

**作用**：  
通知脚本终端显示区域的宽度或高度发生变化。用于实现响应式界面布局。

### 4. `tick`

```lua
{
  type = "tick",
  dt_ms = int
}
```

**作用**：  
通知脚本时间推进，`dt_ms` 表示距离上一个 `tick` 事件的时间差（毫秒）。

---

## 注意事项

### 一、实现要求

1. **必须实现的函数**：`init_game`、`handle_event`、`render`、`exit_game` 四个 API **缺一不可**。
2. **按需实现的函数**：
   - 当 `game.json` 中 `best_none` 为 `true` 时，**必须实现** `save_best_score`。
   - 当 `game.json` 中 `save` 为 `true` 时，**必须实现** `save_game`。

### 二、返回值规范

| 函数                         | 传递值要求                                                                          |
| ---------------------------- | ----------------------------------------------------------------------------------- |
| `init_game(state)`           | **必须传递** `state` 表                                                             |
| `handle_event(state, event)` | **必须传递** `state` 表                                                             |
| `exit_game(state)`           | **必须传递** `state` 表                                                             |
| `render(state)`              | 无传递值                                                                            |
| `save_best_score(state)`     | **必须传递** `best` 表（结构见「数据格式」章节）                                    |
| `save_game(state)`           | **必须传递** `state` 表（该返回值仅用于长期存储，当前游戏继续使用传入的原 `state`） |

### 三、宿主职责与限制

1. 宿主仅负责**事件的交流**与 **`state` 的存储/恢复**，**不对事件或 `state` 进行任何业务逻辑处理**。所有游戏逻辑（状态更新、事件响应、画面绘制等）均需由脚本自身实现。
2. `save_game` 传递的 `state` 仅用于**持久化存档**，当前游戏的运行仍使用传入的原始 `state`。

### 四、事件队列规则

1. 每帧处理的事件队列数量上限为 **256** 个。超出该数量的事件将推迟至**下一帧**继续处理（该限制不适用于 `tick` 事件）。
2. 每帧的事件队列末尾**必定包含**一个 `tick` 事件。

---

# 直用式 API

**直用式 API 要求您在脚本中直接调用以下函数，无需重写，并按照规范传入参数及接收返回值。**

<font color="red"><b>脚本中必须至少存在一条可执行路径能够调用 request_exit()，否则游戏将无法正常退出。</b></font>

> 注：
>
> - `[` 表示参数可选，如需跳过参数，填写 `nil` 占位。
> - `*` 表示特定参数。
> - 多返回值以多个独立值返回，而非表（table）。

---

## 系统请求

> 注：`request_skip_event_queue` 和 `request_clear_event_queue` 不会影响队尾的 `tick` 事件。`tick` 事件在每个帧循环中**必定**会被传入，详情流程见 声明式 API - [执行流程](#执行流程)。

| 可达性                                | 函数名                        | 作用                                                       | 参数                            | 返回值                                                                                                                                        |
| ------------------------------------- | ----------------------------- | ---------------------------------------------------------- | ------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| <font color="#7f7f7f">无要求</font>   | `get_launch_mode()`           | 获取本次游戏的启动模式。                                   | <font color="#7f7f7f">无</font> | `status` - <font color="#92cddc">"new"</font> \| <font color="#92cddc">"continue"</font>：`"new"` 表示新游戏，`"continue"` 表示继续已有存档。 |
| <font color="#7f7f7f">无要求</font>   | `get_best_score()`            | 获取游戏存储的最佳记录数据。                               | <font color="#7f7f7f">无</font> | `data` - <font color="#92cddc">table</font> | <font color="#92cddc">nil</font>：存储的最佳记录数据，宿主返回脚本所传递的 best 参数，若不存在返回 nil。                                             |
| <font color="red">至少一条可达</font> | `request_exit()`              | 向宿主发送退出游戏请求。                                   | <font color="#7f7f7f">无</font> | <font color="#7f7f7f">无</font>                                                                                                               |
| <font color="#7f7f7f">无要求</font>   | `request_skip_event_queue()`  | 向宿主发送跳过尚未处理的事件队列请求。                     | <font color="#7f7f7f">无</font> | <font color="#7f7f7f">无</font>                                                                                                               |
| <font color="#7f7f7f">无要求</font>   | `request_clear_event_queue()` | 向宿主发送清空尚未处理的事件队列请求。                     | <font color="#7f7f7f">无</font> | <font color="#7f7f7f">无</font>                                                                                                               |
| <font color="#7f7f7f">无要求</font>   | `request_render()`            | 请求宿主调用 `render(state)` 以重绘当前界面。              | <font color="#7f7f7f">无</font> | <font color="#7f7f7f">无</font>                                                                                                               |
| <font color="#7f7f7f">无要求</font>   | `request_save_best_score()`   | 请求宿主调用 `save_best_score(state)` 以保存当前最佳记录。 | <font color="#7f7f7f">无</font> | <font color="#7f7f7f">无</font>                                                                                                               |
| <font color="#7f7f7f">无要求</font>   | `request_save_game()`         | 请求宿主调用 `save_game(state)` 以保存当前游戏存档。       | <font color="#7f7f7f">无</font> | <font color="#7f7f7f">无</font>                                                                                                               |

---

## 内容绘制

> 注：
> 1. `color` 类型见 附录-[颜色 color](#颜色-color)
> 2. 所有绘制操作的基准点均为**内容的左上角**，即绘制内容将从指定的 (x, y) 坐标处开始向右、向下延伸。

| 函数名                                                     | 作用                                     | 参数                                                                                                                                                                                                                                                                                                                                                                                                                                         | 返回值                          |
| ---------------------------------------------------------- | ---------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------- |
| `canvas_clear()`                                           | 清空当前帧的画布。                       | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                                                                                                                              | <font color="#7f7f7f">无</font> |
| `canvas_eraser(x, y, width, height)`                                           | 清空画布指定区域。                       | `x` - <font color="#92cddc">int</font>：横轴坐标。<br>`y` - <font color="#92cddc">int</font>：纵轴坐标。<br>`width` - <font color="#92cddc">int</font>：区域宽度。<br>`height` - <font color="#92cddc">int</font>：区域高度。 | <font color="#7f7f7f">无</font> |
| `canvas_draw_text(x, y, text, *[fg, *[bg)`                 | 在画布指定位置绘制内容串。           | `x` - <font color="#92cddc">int</font>：横轴坐标。<br>`y` - <font color="#92cddc">int</font>：纵轴坐标。<br>`text` - <font color="#92cddc">string</font>：要绘制的字符串。<br>`*[fg` - <font color="#92cddc">color</font>：可选，字符颜色。<br>`*[bg` - <font color="#92cddc">color</font>：可选，背景颜色。                                                                                                                                 | <font color="#7f7f7f">无</font> |
| `canvas_fill_rect(x, y, width, height, [char, *[fg, *[bg)` | 从指定位置绘制矩形，并使用指定字符填充。 | `x` - <font color="#92cddc">int</font>：横轴坐标。<br>`y` - <font color="#92cddc">int</font>：纵轴坐标。<br>`width` - <font color="#92cddc">int</font>：矩形宽度。<br>`height` - <font color="#92cddc">int</font>：矩形高度。<br>`[char` - <font color="#92cddc">string</font>：可选，用于填充的单个字符。<br>`*[fg` - <font color="#92cddc">color</font>：可选，字符颜色。<br>`*[bg` - <font color="#92cddc">color</font>：可选，背景颜色。 | <font color="#7f7f7f">无</font> |
| `canvas_border_rect(x, y, width, height, [char_list, *[fg, *[bg)` | 从指定位置绘制矩形边框，并使用指定字符作为边框。 | `x` - <font color="#92cddc">int</font>：横轴坐标。<br>`y` - <font color="#92cddc">int</font>：纵轴坐标。<br>`width` - <font color="#92cddc">int</font>：矩形宽度。<br>`height` - <font color="#92cddc">int</font>：矩形高度。<br>`[char_list` - <font color="#92cddc">table</font>：可选，边框字符配置表，结构见下方。<br>`*[fg` - <font color="#92cddc">color</font>：可选，字符颜色。<br>`*[bg` - <font color="#92cddc">color</font>：可选，背景颜色。 | <font color="#7f7f7f">无</font> |

**`[char_list` 格式**

```lua
{
  top = char,           -- 上边框
  top_right = char,     -- 右上角
  right = char,         -- 右边框
  bottom_right = char,  -- 右下角
  bottom = char,        -- 下边框
  bottom_left = char,   -- 左下角
  left = char,          -- 左边框
  top_left = char       -- 左上角
}
```

> 若 `char_list` 未提供或字段缺失，对应位置的边框将不绘制（留空）。

---

## 内容尺寸计算

> 注：宽度、高度返回值均以**终端字符数**为单位（宽度为列数，高度为行数）。

| 函数名                  | 作用                                 | 参数                                                           | 返回值                                                                                                                    |
| ----------------------- | ------------------------------------ | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `get_text_size(text)`   | 计算字符串在终端中所占的宽度和高度。 | `text` - <font color="#92cddc">string</font>：要测量的字符串。 | `width` - <font color="#92cddc">int</font>：文字所占宽度。<br>`height` - <font color="#92cddc">int</font>：文字所占高度。 |
| `get_text_width(text)`  | 计算字符串在终端中所占的宽度。       | `text` - <font color="#92cddc">string</font>：要测量的字符串。 | `width` - <font color="#92cddc">int</font>：文字所占宽度。                                                                |
| `get_text_height(text)` | 计算字符串在终端中所占的高度。       | `text` - <font color="#92cddc">string</font>：要测量的字符串。 | `height` - <font color="#92cddc">int</font>：文字所占高度。                                                               |
| `get_terminal_size()`   | 获取当前终端的宽度和高度。           | <font color="#7f7f7f">无</font>                                | `width` - <font color="#92cddc">int</font>：终端宽度。<br>`height` - <font color="#92cddc">int</font>：终端高度。         |

---

## 布局定位计算

> 注：
>
> 1. 宽度、高度返回值均以**终端字符数**为单位（宽度为列数，高度为行数）。
> 2. `x_anchor` 和 `y_anchor` 类型见 附录-[锚点 anchor](#锚点-anchor)

| 函数名                                                                    | 作用                                                         | 参数                                                                                                                                                                                                                                                                                                                                                                                                | 返回值                                                                                                         |
| ------------------------------------------------------------------------- | ------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `resolve_x(*x_anchor, cw, [offset_x)`                                     | 根据水平锚点、内容宽度和偏移量，计算起始 X 坐标。            | `*x_anchor` - <font color="#92cddc">x_anchor</font>：水平锚点。<br>`cw` - <font color="#92cddc">int</font>：内容宽度。<br>`[offset_x` - <font color="#92cddc">int</font>：可选，水平偏移量。                                                                                                                                                                                                        | `x` - <font color="#92cddc">int</font>：起始 X 坐标。                                                          |
| `resolve_y(*y_anchor, ch, [offset_y)`                                     | 根据垂直锚点、内容高度和偏移量，计算起始 Y 坐标。            | `*y_anchor` - <font color="#92cddc">y_anchor</font>：垂直锚点。<br>`ch` - <font color="#92cddc">int</font>：内容高度。<br>`[offset_y` - <font color="#92cddc">int</font>：可选，垂直偏移量。                                                                                                                                                                                                        | `y` - <font color="#92cddc">int</font>：起始 Y 坐标。                                                          |
| `resolve_rect(*x_anchor, *y_anchor, width, height, [offset_x, [offset_y)` | 根据水平和垂直锚点、宽高及偏移量，计算矩形的起始 X、Y 坐标。 | `*x_anchor` - <font color="#92cddc">x_anchor</font>：水平锚点。<br>`*y_anchor` - <font color="#92cddc">y_anchor</font>：垂直锚点。<br>`width` - <font color="#92cddc">int</font>：矩形宽度。<br>`height` - <font color="#92cddc">int</font>：矩形高度。<br>`[offset_x` - <font color="#92cddc">int</font>：可选，水平偏移量。<br>`[offset_y` - <font color="#92cddc">int</font>：可选，垂直偏移量。 | `x` - <font color="#92cddc">int</font>：起始 X 坐标。<br>`y` - <font color="#92cddc">int</font>：起始 Y 坐标。 |

---

## 数据读取

> 注：
>
> 1. 本章节中所有 `path` 参数均相对于游戏资源包中的 `assets/` 目录。
> 2. 请注意返回值的数据类型，避免解析错误。

| 函数名             | 作用                                                   | 参数                                                                    | 返回值                                                                                                             |
| ------------------ | ------------------------------------------------------ | ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `translate(key)`   | 读取当前游戏资源包中指定语言键对应的本地化字符串。     | `key` - <font color="#92cddc">string</font>：语言键。                   | `value` - <font color="#92cddc">string</font>：对应的本地化字符串。                                                |
| `read_bytes(path)` | 读取资源包中指定路径的二进制文件。                     | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">string</font>：文件的二进制数据（以 Lua 字符串形式返回，并非二进制/十六进制类型）。 |
| `read_text(path)`  | 读取资源包中指定路径的 `.txt` 文本文件。               | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">string</font>：文件文本内容。                                                       |
| `read_json(path)`  | 读取资源包中指定路径的 `.json` 文件，并解析为 Lua 表。 | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">table</font>：解析后的 Lua 表。                                                     |
| `read_xml(path)`   | 读取资源包中指定路径的 `.xml` 文件，并解析为 Lua 表。  | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">table</font>：解析后的 Lua 表。                                                     |
| `read_yaml(path)`  | 读取资源包中指定路径的 `.yaml` 文件，并解析为 Lua 表。 | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">table</font>：解析后的 Lua 表。                                                     |
| `read_toml(path)`  | 读取资源包中指定路径的 `.toml` 文件，并解析为 Lua 表。 | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">table</font>：解析后的 Lua 表。                                                     |
| `read_csv(path)`   | 读取资源包中指定路径的 `.csv` 文件，并解析为 Lua 表。  | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">table</font>：解析后的 Lua 表。                                                     |

---

## 数据写入

> 注：
>
> 1. 本章节中所有 `path` 参数均相对于游戏资源包中的 `assets/` 目录。
> 2. 所有 `write_*` 函数均为高风险直写操作。仅当 `game.json` 中 `write` 字段为 `true` 且用户授予模组完全信任权限时，直写操作才会被执行；否则所有直写请求将被宿主忽略。
> 3. 无论直写操作是否执行，每次调用都会在调试报告中记录，供用户安全检查。
> 4. 所有 `write_*` 函数的 `content` 参数均为 `string` 类型。

<font color="red"><b>直写操作不可撤回！</b></font>
<font color="red"><b>直写操作不可撤回！</b></font>
<font color="red"><b>直写操作不可撤回！</b></font>

<font color="red"><b>直写操作为高风险操作，请最大程度避免使用！</b></font>

| 风险等级                        | 函数名                       | 作用                             | 参数                                                                                                                                              | 返回值                                                                             |
| ------------------------------- | ---------------------------- | -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| <font color="red">高风险</font> | `write_bytes(path, content)` | 写入二进制文件到资源包指定路径。 | `path` - <font color="#92cddc">string</font>：文件路径。<br>`content` - <font color="#92cddc">string</font>：要写入的二进制数据（以字符串形式）。 | `bool` - <font color="#92cddc">boolean</font>：`true` 写入成功，`false` 写入失败。 |
| <font color="red">高风险</font> | `write_text(path, content)`  | 写入文本文件到资源包指定路径。   | `path` - <font color="#92cddc">string</font>：文件路径。<br>`content` - <font color="#92cddc">string</font>：要写入的文本内容。                   | `bool` - <font color="#92cddc">boolean</font>：`true` 写入成功，`false` 写入失败。 |
| <font color="red">高风险</font> | `write_json(path, content)`  | 写入 JSON 文件到资源包指定路径。 | `path` - <font color="#92cddc">string</font>：文件路径。<br>`content` - <font color="#92cddc">string</font>：要写入的 JSON 字符串。               | `bool` - <font color="#92cddc">boolean</font>：`true` 写入成功，`false` 写入失败。 |
| <font color="red">高风险</font> | `write_xml(path, content)`   | 写入 XML 文件到资源包指定路径。  | `path` - <font color="#92cddc">string</font>：文件路径。<br>`content` - <font color="#92cddc">string</font>：要写入的 XML 字符串。                | `bool` - <font color="#92cddc">boolean</font>：`true` 写入成功，`false` 写入失败。 |
| <font color="red">高风险</font> | `write_yaml(path, content)`  | 写入 YAML 文件到资源包指定路径。 | `path` - <font color="#92cddc">string</font>：文件路径。<br>`content` - <font color="#92cddc">string</font>：要写入的 YAML 字符串。               | `bool` - <font color="#92cddc">boolean</font>：`true` 写入成功，`false` 写入失败。 |
| <font color="red">高风险</font> | `write_toml(path, content)`  | 写入 TOML 文件到资源包指定路径。 | `path` - <font color="#92cddc">string</font>：文件路径。<br>`content` - <font color="#92cddc">string</font>：要写入的 TOML 字符串。               | `bool` - <font color="#92cddc">boolean</font>：`true` 写入成功，`false` 写入失败。 |
| <font color="red">高风险</font> | `write_csv(path, content)`   | 写入 CSV 文件到资源包指定路径。  | `path` - <font color="#92cddc">string</font>：文件路径。<br>`content` - <font color="#92cddc">string</font>：要写入的 CSV 字符串。                | `bool` - <font color="#92cddc">boolean</font>：`true` 写入成功，`false` 写入失败。 |

---

## 辅助脚本加载

> 注：辅助脚本的具体编写与使用请参考 `MOD.md`。

| 函数名                | 作用                                                                      | 参数                                                                          | 返回值                                                                           |
| --------------------- | ------------------------------------------------------------------------- | ----------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| `load_function(path)` | 加载指定路径的 Lua 辅助脚本，返回脚本中定义的所有变量和函数（以表形式）。 | `path` - <font color="#92cddc">string</font>：相对于 `function/` 目录的路径。 | `functions` - <font color="#92cddc">table</font>：包含脚本中所有变量和函数的表。 |

---

## 时间处理

> 注：
>
> 1. 所有时间值单位均为毫秒（ms）。
> 2. 计时器创建后处于 `init` 状态，需调用 `timer_start` 或 `timer_restart` 才会启动。计时器结束后状态变为 `completed`，不会自动删除，需手动调用 `timer_kill` 清理。
> 3. `timer_reset` 将计时器重置为 `init` 状态（已过时间归零）；`timer_restart` 相当于 reset + start。
> 4. 查询 `init` 状态的计时器，`elapsed` 返回 0；查询 `completed` 状态的计时器，`remaining` 返回 0。
> 5. 除 `is_timer_exists` 外，参数使用不存在的计时器 ID 会抛出异常。
> 6. 所有脚本创建的计时器会在游戏退出后被删除。

| 是否启动计时器                  | 函数名                          | 作用                                                        | 参数                                                                                                                                       | 返回值                                                                                                                                                                                                                                                                                  |
| ------------------------------- | ------------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| <font color="#7f7f7f">否</font> | `now()`                         | 获取从游戏启动到当前时刻经过的总时长（毫秒）。              | <font color="#7f7f7f">无</font>                                                                                                            | `time` - <font color="#92cddc">int</font>：已运行总时长（毫秒）。                                                                                                                                                                                                                       |
| <font color="#7f7f7f">否</font> | `timer_create(delay_ms, [note)` | 创建一个持续 `delay_ms` 毫秒的计时器（初始状态为 `init`）。 | `delay_ms` - <font color="#92cddc">int</font>：计时时长（毫秒）。<br>`[note` - <font color="#92cddc">string</font>：可选，计时器备注信息。 | `id` - <font color="#92cddc">string</font>：计时器唯一标识 ID。                                                                                                                                                                                                                         |
| <font color="red">是</font>     | `timer_start(id)`               | 启动指定 ID 的计时器。                                      | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                    | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                         |
| <font color="#7f7f7f">否</font> | `timer_pause(id)`               | 暂停指定 ID 的计时器。                                      | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                    | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                         |
| <font color="#7f7f7f">否</font> | `timer_resume(id)`              | 恢复暂停的计时器，从暂停点继续计时。                        | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                    | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                         |
| <font color="#7f7f7f">否</font> | `timer_reset(id)`               | 重置指定 ID 的计时器（状态变为 `init`，已过时间归零）。     | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                    | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                         |
| <font color="red">是</font>     | `timer_restart(id)`             | 重置并立即启动指定 ID 的计时器。                            | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                    | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                         |
| <font color="#7f7f7f">否</font> | `timer_kill(id)`                | 删除指定 ID 的计时器。                                      | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                    | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                         |
| <font color="#7f7f7f">否</font> | `set_timer_note(id, note)`      | 修改指定 ID 计时器的备注信息。                              | `id` - <font color="#92cddc">string</font>：计时器 ID。<br>`note` - <font color="#92cddc">string</font>：新的备注信息。                    | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                         |
| <font color="#7f7f7f">否</font> | `get_timer_list()`              | 获取所有计时器的信息列表。                                  | <font color="#7f7f7f">无</font>                                                                                                            | `timers` - <font color="#92cddc">table</font>：计时器信息表，结构见下文。                                                                                                                                                                                                               |
| <font color="#7f7f7f">否</font> | `get_timer_info(id)`            | 获取指定 ID 计时器的详细信息。                              | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                    | `timer` - <font color="#92cddc">table</font> 计时器信息表，结构见下文。                                                                                                                                                                                                                 |
| <font color="#7f7f7f">否</font> | `get_timer_status(id)`          | 获取指定 ID 计时器的当前状态。                              | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                    | `status` - <font color="#92cddc">"init"</font> \| <font color="#92cddc">"running"</font> \| <font color="#92cddc">"pause"</font> \| <font color="#92cddc">"completed"</font>：<br>`"init"` 初始状态，未启动；<br>`"running"` 正在运行；<br>`"pause"` 已暂停；<br>`"completed"` 已结束。 |
| <font color="#7f7f7f">否</font> | `get_timer_elapsed(id)`         | 获取指定 ID 计时器的已过时间（毫秒）。                      | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                    | `time` - <font color="#92cddc">int</font>：已过时间（毫秒）。                                                                                                                                                                                                                           |
| <font color="#7f7f7f">否</font> | `get_timer_remaining(id)`       | 获取指定 ID 计时器的剩余时间（毫秒）。                      | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                    | `time` - <font color="#92cddc">int</font>：剩余时间（毫秒）。                                                                                                                                                                                                                           |
| <font color="#7f7f7f">否</font> | `get_timer_duration(id)`        | 获取指定 ID 计时器的总时长（毫秒）。                        | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                    | `time` - <font color="#92cddc">int</font>：总时长（毫秒）。                                                                                                                                                                                                                             |
| <font color="#7f7f7f">否</font> | `get_timer_completed(id)`       | 检查指定 ID 的计时器是否已结束。                            | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                    | `bool` - <font color="#92cddc">boolean</font>：`true` 已结束，`false` 未结束 。                                                                                                                                                                                                         |
| <font color="#7f7f7f">否</font> | `is_timer_exists(id)`           | 检查指定 ID 的计时器是否存在。                              | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                    | `bool` - <font color="#92cddc">boolean</font>：`true` 存在，`false` 不存在。                                                                                                                                                                                                            |

### `timers` 数据格式

```lua
{
  {
    id = "string",      -- 计时器 ID
    note = "string",    -- 备注信息
    status = "init | running | pause | completed",  -- 状态
    elapsed = int,      -- 已过时间（毫秒）
    remaining = int,    -- 剩余时间（毫秒）
    duration = int      -- 总时长（毫秒）
  },
  ...
}
```

- 若无计时器，`timers` 为空表 `{}`。

### `timer` 数据格式

```lua
{
  id = "string",      -- 计时器 ID
  note = "string",    -- 备注信息
  status = "init | running | pause | completed",  -- 状态
  elapsed = int,      -- 已过时间（毫秒）
  remaining = int,    -- 剩余时间（毫秒）
  duration = int      -- 总时长（毫秒）
}
```

---

## 随机数

> 注：
>
> 1. 为保证随机数的安全性与可复现性，建议使用下文提供的 API 生成随机数。
> 2. 若未使用 `random_create` 或 `random_float_create` 构建生成器，则默认使用宿主提供的随机数生成器，该随机数生成器结果不可复现。
> 3. 参数使用不存在的随机数生成器 ID 会抛出异常。
> 4. `random` 系列函数只能使用整数类型生成器，`random_float` 只能使用浮点数类型生成器，类型不匹配时会抛出异常。
> 5. 所有脚本构建的随机数生成器会在游戏退出后被删除。

| 函数名                             | 作用                                         | 参数                                                                                                                                                                                                 | 返回值                                                                                          |
| ---------------------------------- | -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `random([id)`                      | 生成区间 $[0, 2^{31}-1]$ 内的随机整数。      | `[id` - <font color="#92cddc">string</font>：可选，随机数生成器 ID。                                                                                                                                 | `number` - <font color="#92cddc">int</font>：随机整数。                                         |
| `random(max, [id)`                 | 生成区间 $[0, max]$ 内的随机整数。           | `max` - <font color="#92cddc">int</font>：区间上限（包含）。<br>`[id` - <font color="#92cddc">string</font>：可选，随机数生成器 ID。                                                                 | `number` - <font color="#92cddc">int</font>：随机整数。                                         |
| `random(min, max, [id)`            | 生成区间 $[min, max]$ 内的随机整数。         | `min` - <font color="#92cddc">int</font>：区间下限（包含）。<br>`max` - <font color="#92cddc">int</font>：区间上限（包含）。<br>`[id` - <font color="#92cddc">string</font>：可选，随机数生成器 ID。 | `number` - <font color="#92cddc">int</font>：随机整数。                                         |
| `random_float([id)`                | 生成区间 $[0, 1)$ 内的随机浮点数。           | `[id` - <font color="#92cddc">string</font>：可选，随机数生成器 ID。                                                                                                                                 | `number` - <font color="#92cddc">double</font>：随机浮点数。                                    |
| `random_create(seed, [note)`       | 创建一个整数随机数生成器。                   | `seed` - <font color="#92cddc">string</font>：随机种子。<br>`[note` - <font color="#92cddc">string</font>：可选，备注信息。                                                                          | `id` - <font color="#92cddc">string</font>：生成器 ID。                                         |
| `random_float_create(seed, [note)` | 创建一个浮点数随机数生成器。                 | `seed` - <font color="#92cddc">string</font>：随机种子。<br>`[note` - <font color="#92cddc">string</font>：可选，备注信息。                                                                          | `id` - <font color="#92cddc">string</font>：生成器 ID。                                         |
| `random_reset_step(id)`            | 重置指定随机数生成器的步进数（步进数归零）。 | `id` - <font color="#92cddc">string</font>：生成器 ID。                                                                                                                                              | <font color="#7f7f7f">无</font>                                                                 |
| `random_kill(id)`                  | 删除指定随机数生成器。                       | `id` - <font color="#92cddc">string</font>：生成器 ID。                                                                                                                                              | <font color="#7f7f7f">无</font>                                                                 |
| `set_random_note(id, note)`        | 修改指定随机数生成器的备注信息。             | `id` - <font color="#92cddc">string</font>：生成器 ID。<br>`note` - <font color="#92cddc">string</font>：新的备注信息。                                                                              | <font color="#7f7f7f">无</font>                                                                 |
| `get_random_list()`                | 获取所有已创建的随机数生成器信息列表。       | <font color="#7f7f7f">无</font>                                                                                                                                                                      | `randoms` - <font color="#92cddc">table</font>：信息列表，结构见下文。                          |
| `get_random_info(id)`              | 获取指定随机数生成器的详细信息。             | `id` - <font color="#92cddc">string</font>：生成器 ID。                                                                                                                                              | `random` - <font color="#92cddc">table</font>：信息表，结构见下文。                             |
| `get_random_step(id)`              | 获取指定随机数生成器的当前步进数。           | `id` - <font color="#92cddc">string</font>：生成器 ID。                                                                                                                                              | `step` - <font color="#92cddc">int</font>：步进数。                                             |
| `get_random_seed(id)`              | 获取指定随机数生成器的种子。                 | `id` - <font color="#92cddc">string</font>：生成器 ID。                                                                                                                                              | `seed` - <font color="#92cddc">string</font>：种子字符串。                                      |
| `get_random_type(id)`              | 获取指定随机数生成器的类型（整数或浮点数）。 | `id` - <font color="#92cddc">string</font>：生成器 ID。                                                                                                                                              | `type` - <font color="#92cddc">"int"</font> \| <font color="#92cddc">"float"</font>：类型标识。 |

### `randoms` 数据格式

```lua
{
  {
    id = "string",   -- 生成器 ID
    note = "string", -- 备注信息
    seed = "string", -- 种子
    step = int,      -- 当前步进数
    type = "int | float"  -- 类型
  },
  ...
}
```

- 若无随机数生成器，`randoms` 为空表 `{}`。

### `random` 数据格式

```lua
{
  id = "string",   -- 生成器 ID
  note = "string", -- 备注信息
  seed = "string", -- 种子
  step = int,      -- 当前步进数
  type = "int | float"  -- 类型
}
```

---

## 调试信息

> 注：
>
> 1. 该部分 API 仅在游戏开启调试模式（debug 模式）时可用，否则调用将被宿主忽略。
> 2. `info` 数据格式中 `key` 数据类型含义为填写语言键
> 3. `info` 数据格式中 `image` 数据类型含义为相对于assets/的图片路径
> 4. `任意` 类型会被强制转换为 `string` 类型打印
> 5. 详细调试输出见 附录-[调试输出目录](#调试输出目录)

| 函数名                        | 作用                                         | 参数                                                                                                                            | 返回值                                                                  |
| ----------------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| `debug_log(message)`          | 在日志文件中写入一条调试信息。               | `message` - <font color="#92cddc">任意</font>：要写入的信息。                                                                   | <font color="#7f7f7f">无</font>                                         |
| `debug_warn(message)`         | 在日志文件中写入一条警告信息。               | `message` - <font color="#92cddc">任意</font>：要写入的警告信息。                                                               | <font color="#7f7f7f">无</font>                                         |
| `debug_error(message)`        | 在日志文件中写入一条错误信息。               | `message` - <font color="#92cddc">任意</font>：要写入的错误信息。                                                               | <font color="#7f7f7f">无</font>                                         |
| `debug_print(title, message)` | 在日志文件中写入一条带自定义标题的调试信息。 | `title` - <font color="#92cddc">string</font>：日志标题。 <br>`message` - <font color="#92cddc">任意</font>：要写入的错误信息。 | <font color="#7f7f7f">无</font>                                         |
| `clear_debug_log()`           | 清空日志文件。                               | <font color="#7f7f7f">无</font>                                                                                                 | <font color="#7f7f7f">无</font>                                         |
| `get_game_uid()`              | 获取当前模组在宿主中的唯一标识符（UID）。    | <font color="#7f7f7f">无</font>                                                                                                 | `uid` - <font color="#92cddc">string</font>：模组 UID。                 |
| `get_game_info()`             | 获取当前模组的完整元信息。                   | <font color="#7f7f7f">无</font>                                                                                                 | `info` - <font color="#92cddc">table</font>：模组元信息表，结构见下文。 |

### 日志输出格式

- `debug_log(message)` 输出格式：`[日志] message`
- `debug_warn(message)` 输出格式：`[警告] message`
- `debug_error(message)` 输出格式：`[错误] message`
- `debug_print(title, message)` 输出格式：`[title] message`

### `info` 数据格式

```lua
{
  uid = string,                    -- 游戏在宿主的唯一标识 ID
  package = string,                -- 包名
  introduction = string | key,     -- 包简介
  author = string | key,           -- 作者
  name = string | key,             -- 游戏显示名称
  description = string | key,      -- 游戏简短描述
  detail = string | key,           -- 游戏详细描述
  icon = Array | string | image,   -- 图标
  banner = Array | string | image, -- 横幅
  api = Array | int,               -- 支持的API版本
  entry = path,                    -- 入口脚本路径
  save = boolean,                  -- 是否支持保存
  best_none = string | key | null, -- 最佳记录字段配置
  min_width = int,                 -- 最小宽度（字符数）
  min_height = int,                -- 最小高度（字符数）
  write = boolean,                 -- 是否允许写入文件
  actions = table,                 -- 按键事件注册表
  runtime = {
    target_fps = int               -- 目标帧率
  }
}
```

---

# 附录

## 特定参数

### 锚点 `anchor`

> 以下常量的值与含义对应。

#### 水平锚点 `x_anchor`

| 常量            | 值  | 作用         |
| --------------- | --- | ------------ |
| `ANCHOR_LEFT`   | `0` | 水平左对齐   |
| `ANCHOR_CENTER` | `1` | 水平居中对齐 |
| `ANCHOR_RIGHT`  | `2` | 水平右对齐   |

#### 垂直锚点 `y_anchor`

| 常量            | 值  | 作用         |
| --------------- | --- | ------------ |
| `ANCHOR_TOP`    | `0` | 垂直顶部对齐 |
| `ANCHOR_MIDDLE` | `1` | 垂直居中对齐 |
| `ANCHOR_BOTTOM` | `2` | 垂直底部对齐 |

---

### 颜色 `color`

#### 预定义颜色名称

> 注：以下颜色值为逻辑名称，实际显示效果取决于终端的颜色映射。

| 颜色值          | 映射的终端颜色 |
| --------------- | -------------- |
| `black`         | Black          |
| `white`         | White          |
| `red`           | Red            |
| `light_red`     | Red            |
| `dark_red`      | DarkRed        |
| `yellow`        | Yellow         |
| `light_yellow`  | Yellow         |
| `dark_yellow`   | DarkYellow     |
| `orange`        | DarkYellow     |
| `green`         | Green          |
| `light_green`   | Green          |
| `blue`          | Blue           |
| `light_blue`    | Blue           |
| `cyan`          | Cyan           |
| `light_cyan`    | Cyan           |
| `magenta`       | Magenta        |
| `light_magenta` | Magenta        |
| `grey`          | Grey           |
| `gray`          | Grey           |
| `dark_grey`     | DarkGrey       |
| `dark_gray`     | DarkGrey       |

#### 自定义颜色格式

| 格式         | 示例              | 注意事项                                                                   |
| ------------ | ----------------- | -------------------------------------------------------------------------- |
| `rgb(r,g,b)` | `rgb(255,128,64)` | 标准 RGB 颜色，括号内为 0–255 的整数值。**请勿在字母与括号之间添加空格**。 |
| `#rrggbb`    | `#ff8040`         | 十六进制颜色表示（6 位）。**不支持 `#rgb` 缩写格式**。                     |

---

## 调试输出目录

> 注：以下日志文件均位于宿主运行目录下的 `./tui-game-data/log/` 中。

### 游戏日志 `[uid].txt`

该文件以模组的唯一标识符 `uid` 命名，记录与该模组相关的运行时信息。

> 类型：脚本异常

**包含内容：**

- 调试信息 API 的输出（`debug_log`、`debug_warning`、`debug_error`）
- 脚本运行时的异常信息（如语法错误、运行时错误）
- 计时器操作中传入不存在的 ID 所抛出的异常
- 随机数生成器操作中传入不存在的 ID 所抛出的异常
- 随机数生成器类型不匹配所抛出的异常

> 注：该日志仅在模组开启 Debug 模式时记录调试信息 API 的输出，但异常信息不受 Debug 模式影响。

### 官方日志 `tui_log.txt`

> 类型：宿主异常

该文件记录宿主自身的运行状态以及所有模组相关的全局操作。

**包含内容：**

- 脚本启动异常（如入口脚本加载失败、声明式 API 缺失）
- 直写操作请求（无论是否执行，均记录）
- 宿主自身的异常信息

---

# 快速查询

## 声明式 API

| 重写要求                              | 函数                         | 作用         | 参数                                                                                       | 传递值                                       | 主条目定位                |
| ------------------------------------- | ---------------------------- | ------------ | ------------------------------------------------------------------------------------------ | -------------------------------------------- | ------------------------- |
| <font color="red">必须重写</font>     | `init_game(state)`           | 初始化游戏   | `state` - <font color="#92cddc">table</font> \| <font color="#92cddc">nil</font>           | `state` - <font color="#92cddc">table</font> | [声明式 API](#声明式-api) |
| <font color="red">必须重写</font>     | `handle_event(state, event)` | 处理事件     | `state` - <font color="#92cddc">table</font>, `event` - <font color="#92cddc">table</font> | `state` - <font color="#92cddc">table</font> | [声明式 API](#声明式-api) |
| <font color="red">必须重写</font>     | `render(state)`              | 绘制画面     | `state` - <font color="#92cddc">table</font>                                               | <font color="#7f7f7f">无</font>              | [声明式 API](#声明式-api) |
| <font color="red">必须重写</font>     | `exit_game(state)`           | 退出前处理   | `state` - <font color="#92cddc">table</font>                                               | `state` - <font color="#92cddc">table</font> | [声明式 API](#声明式-api) |
| <font color="#fac08f">条件重写</font> | `save_best_score(state)`     | 传递最佳记录 | `state` - <font color="#92cddc">table</font>                                               | `best` - <font color="#92cddc">table</font>  | [声明式 API](#声明式-api) |
| <font color="#fac08f">条件重写</font> | `save_game(state)`           | 保存存档     | `state` - <font color="#92cddc">table</font>                                               | `state` - <font color="#92cddc">table</font> | [声明式 API](#声明式-api) |

## 直用式 API

| 风险等级                            | 函数                                                                      | 作用               | 参数                                                                                                                                                                                                                                                                                                             | 返回值                                                                                   | 主条目定位                    |
| ----------------------------------- | ------------------------------------------------------------------------- | ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ----------------------------- |
| <font color="#7f7f7f">无风险</font> | `get_launch_mode()`                                                       | 获取启动模式       | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | `status` - <font color="#92cddc">"new"</font> \| <font color="#92cddc">"continue"</font> | [系统请求](#系统请求)         |
| <font color="#7f7f7f">无风险</font> | `get_best_score()`                                                        | 读取最佳记录       | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | `data` - <font color="#92cddc">table</font>                                              | [系统请求](#系统请求)         |
| <font color="#7f7f7f">无风险</font> | `request_exit()`                                                          | 请求退出           | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | <font color="#7f7f7f">无</font>                                                          | [系统请求](#系统请求)         |
| <font color="#7f7f7f">无风险</font> | `request_skip_event_queue()`                                              | 请求跳过事件队列   | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | <font color="#7f7f7f">无</font>                                                          | [系统请求](#系统请求)         |
| <font color="#7f7f7f">无风险</font> | `request_clear_event_queue()`                                             | 请求清空事件队列   | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | <font color="#7f7f7f">无</font>                                                          | [系统请求](#系统请求)         |
| <font color="#7f7f7f">无风险</font> | `request_render()`                                                        | 请求重绘           | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | <font color="#7f7f7f">无</font>                                                          | [系统请求](#系统请求)         |
| <font color="#7f7f7f">无风险</font> | `request_save_best_score()`                                               | 保存最佳记录       | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | <font color="#7f7f7f">无</font>                                                          | [系统请求](#系统请求)         |
| <font color="#7f7f7f">无风险</font> | `request_save_game()`                                                     | 保存游戏存档       | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | <font color="#7f7f7f">无</font>                                                          | [系统请求](#系统请求)         |
| <font color="#7f7f7f">无风险</font> | `canvas_clear()`                                                          | 清空画布           | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | <font color="#7f7f7f">无</font>                                                          | [内容绘制](#内容绘制)         |
| <font color="#7f7f7f">无风险</font> | `canvas_draw_text(x, y, text, *[fg, *[bg)`                                | 绘制文本           | `x` - <font color="#92cddc">int</font>, `y` - <font color="#92cddc">int</font>, `text` - <font color="#92cddc">string</font>, `*[fg` - <font color="#92cddc">color</font>, `*[bg` - <font color="#92cddc">color</font>                                                                                           | <font color="#7f7f7f">无</font>                                                          | [内容绘制](#内容绘制)         |
| <font color="#7f7f7f">无风险</font> | `canvas_fill_rect(x, y, width, height, [char, *[fg, *[bg)`                | 填充矩形           | `x` - <font color="#92cddc">int</font>, `y` - <font color="#92cddc">int</font>, `width` - <font color="#92cddc">int</font>, `height` - <font color="#92cddc">int</font>, `[char` - <font color="#92cddc">string</font>, `*[fg` - <font color="#92cddc">color</font>, `*[bg` - <font color="#92cddc">color</font> | <font color="#7f7f7f">无</font>                                                          | [内容绘制](#内容绘制)         |
| <font color="#7f7f7f">无风险</font> | `canvas_border_rect(x, y, width, height, [char_list, *[fg, *[bg)`                | 绘制矩形边框           | `x` - <font color="#92cddc">int</font>, `y` - <font color="#92cddc">int</font>, `width` - <font color="#92cddc">int</font>, `height` - <font color="#92cddc">int</font>, `[char_list` - <font color="#92cddc">table</font>, `*[fg` - <font color="#92cddc">color</font>, `*[bg` - <font color="#92cddc">color</font> | <font color="#7f7f7f">无</font>                                                          | [内容绘制](#内容绘制)         |
| <font color="#7f7f7f">无风险</font> | `get_text_size(text)`                                                     | 测量文本尺寸       | `text` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                     | `width` - <font color="#92cddc">int</font>, `height` - <font color="#92cddc">int</font>  | [内容尺寸计算](#内容尺寸计算) |
| <font color="#7f7f7f">无风险</font> | `get_text_width(text)`                                                    | 测量文本宽度       | `text` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                     | `width` - <font color="#92cddc">int</font>                                               | [内容尺寸计算](#内容尺寸计算) |
| <font color="#7f7f7f">无风险</font> | `get_text_height(text)`                                                   | 测量文本高度       | `text` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                     | `height` - <font color="#92cddc">int</font>                                              | [内容尺寸计算](#内容尺寸计算) |
| <font color="#7f7f7f">无风险</font> | `get_terminal_size()`                                                     | 获取终端尺寸       | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | `width` - <font color="#92cddc">int</font>, `height` - <font color="#92cddc">int</font>  | [内容尺寸计算](#内容尺寸计算) |
| <font color="#7f7f7f">无风险</font> | `resolve_x(*x_anchor, cw, [offset)`                                       | 计算X坐标          | `*x_anchor` - <font color="#92cddc">x_anchor</font>, `cw` - <font color="#92cddc">int</font>, `[offset` - <font color="#92cddc">int</font>                                                                                                                                                                       | `x` - <font color="#92cddc">int</font>                                                   | [布局定位计算](#布局定位计算) |
| <font color="#7f7f7f">无风险</font> | `resolve_y(*y_anchor, ch, [offset)`                                       | 计算Y坐标          | `*y_anchor` - <font color="#92cddc">y_anchor</font>, `ch` - <font color="#92cddc">int</font>, `[offset` - <font color="#92cddc">int</font>                                                                                                                                                                       | `y` - <font color="#92cddc">int</font>                                                   | [布局定位计算](#布局定位计算) |
| <font color="#7f7f7f">无风险</font> | `resolve_rect(*x_anchor, *y_anchor, width, height, [offset_x, [offset_y)` | 计算矩形位置       | `*x_anchor` - <font color="#92cddc">x_anchor</font>, `*y_anchor` - <font color="#92cddc">y_anchor</font>, `width` - <font color="#92cddc">int</font>, `height` - <font color="#92cddc">int</font>, `[offset_x` - <font color="#92cddc">int</font>, `[offset_y` - <font color="#92cddc">int</font>                | `x` - <font color="#92cddc">int</font>, `y` - <font color="#92cddc">int</font>           | [布局定位计算](#布局定位计算) |
| <font color="#7f7f7f">无风险</font> | `translate(key)`                                                          | 翻译文本           | `key` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                      | `value` - <font color="#92cddc">string</font>                                            | [数据读取](#数据读取)         |
| <font color="#7f7f7f">无风险</font> | `read_bytes(path)`                                                        | 读取二进制         | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">string</font>                                             | [数据读取](#数据读取)         |
| <font color="#7f7f7f">无风险</font> | `read_text(path)`                                                         | 读取文本           | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">string</font>                                             | [数据读取](#数据读取)         |
| <font color="#7f7f7f">无风险</font> | `read_json(path)`                                                         | 读取JSON           | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">table</font>                                              | [数据读取](#数据读取)         |
| <font color="#7f7f7f">无风险</font> | `read_xml(path)`                                                          | 读取XML            | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">table</font>                                              | [数据读取](#数据读取)         |
| <font color="#7f7f7f">无风险</font> | `read_yaml(path)`                                                         | 读取YAML           | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">table</font>                                              | [数据读取](#数据读取)         |
| <font color="#7f7f7f">无风险</font> | `read_toml(path)`                                                         | 读取TOML           | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">table</font>                                              | [数据读取](#数据读取)         |
| <font color="#7f7f7f">无风险</font> | `read_csv(path)`                                                          | 读取CSV            | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">table</font>                                              | [数据读取](#数据读取)         |
| <font color="red">高风险</font>     | `write_bytes(path, content)`                                              | 写入二进制         | `path` - <font color="#92cddc">string</font>, `content` - <font color="#92cddc">string</font>                                                                                                                                                                                                                    | `bool` - <font color="#92cddc">boolean</font>                                            | [数据写入](#数据写入)         |
| <font color="red">高风险</font>     | `write_text(path, content)`                                               | 写入文本           | `path` - <font color="#92cddc">string</font>, `content` - <font color="#92cddc">string</font>                                                                                                                                                                                                                    | `bool` - <font color="#92cddc">boolean</font>                                            | [数据写入](#数据写入)         |
| <font color="red">高风险</font>     | `write_json(path, content)`                                               | 写入JSON           | `path` - <font color="#92cddc">string</font>, `content` - <font color="#92cddc">string</font>                                                                                                                                                                                                                    | `bool` - <font color="#92cddc">boolean</font>                                            | [数据写入](#数据写入)         |
| <font color="red">高风险</font>     | `write_xml(path, content)`                                                | 写入XML            | `path` - <font color="#92cddc">string</font>, `content` - <font color="#92cddc">string</font>                                                                                                                                                                                                                    | `bool` - <font color="#92cddc">boolean</font>                                            | [数据写入](#数据写入)         |
| <font color="red">高风险</font>     | `write_yaml(path, content)`                                               | 写入YAML           | `path` - <font color="#92cddc">string</font>, `content` - <font color="#92cddc">string</font>                                                                                                                                                                                                                    | `bool` - <font color="#92cddc">boolean</font>                                            | [数据写入](#数据写入)         |
| <font color="red">高风险</font>     | `write_toml(path, content)`                                               | 写入TOML           | `path` - <font color="#92cddc">string</font>, `content` - <font color="#92cddc">string</font>                                                                                                                                                                                                                    | `bool` - <font color="#92cddc">boolean</font>                                            | [数据写入](#数据写入)         |
| <font color="red">高风险</font>     | `write_csv(path, content)`                                                | 写入CSV            | `path` - <font color="#92cddc">string</font>, `content` - <font color="#92cddc">string</font>                                                                                                                                                                                                                    | `bool` - <font color="#92cddc">boolean</font>                                            | [数据写入](#数据写入)         |
| <font color="#7f7f7f">无风险</font> | `load_function(path)`                                                     | 加载辅助脚本       | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                     | `functions` - <font color="#92cddc">table</font>                                         | [辅助脚本加载](#辅助脚本加载) |
| <font color="#7f7f7f">无风险</font> | `now()`                                                                   | 获取运行时长       | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | `time` - <font color="#92cddc">int</font>                                                | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `timer_create(delay_ms, [note)`                                           | 创建计时器         | `delay_ms` - <font color="#92cddc">int</font>, `[note` - <font color="#92cddc">string</font>                                                                                                                                                                                                                     | `id` - <font color="#92cddc">string</font>                                               | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `timer_start(id)`                                                         | 启动计时器         | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                          | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `timer_pause(id)`                                                         | 暂停计时器         | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                          | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `timer_resume(id)`                                                        | 恢复计时器         | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                          | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `timer_reset(id)`                                                         | 重置计时器         | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                          | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `timer_restart(id)`                                                       | 重启计时器         | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                          | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `timer_kill(id)`                                                          | 删除计时器         | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                          | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `set_timer_note(id, note)`                                                | 设置计时器备注     | `id` - <font color="#92cddc">string</font>, `note` - <font color="#92cddc">string</font>                                                                                                                                                                                                                         | <font color="#7f7f7f">无</font>                                                          | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `get_timer_list()`                                                        | 获取所有计时器     | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | `timers` - <font color="#92cddc">table</font>                                            | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `get_timer_info(id)`                                                      | 获取计时器信息     | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | `timer` - <font color="#92cddc">table</font>                                             | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `get_timer_status(id)`                                                    | 获取计时器状态     | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | `status` - <font color="#92cddc">string</font>                                           | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `get_timer_elapsed(id)`                                                   | 获取已过时间       | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | `time` - <font color="#92cddc">int</font>                                                | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `get_timer_remaining(id)`                                                 | 获取剩余时间       | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | `time` - <font color="#92cddc">int</font>                                                | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `get_timer_duration(id)`                                                  | 获取总时长         | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | `time` - <font color="#92cddc">int</font>                                                | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `get_timer_completed(id)`                                                 | 检查是否结束       | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | `bool` - <font color="#92cddc">boolean</font>                                            | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `is_timer_exists(id)`                                                     | 检查计时器存在     | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | `bool` - <font color="#92cddc">boolean</font>                                            | [时间处理](#时间处理)         |
| <font color="#7f7f7f">无风险</font> | `random([id)`                                                             | 随机整数[0,2^31-1] | `[id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                      | `number` - <font color="#92cddc">int</font>                                              | [随机数](#随机数)             |
| <font color="#7f7f7f">无风险</font> | `random(max, [id)`                                                        | 随机整数[0,max]    | `max` - <font color="#92cddc">int</font>, `[id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                            | `number` - <font color="#92cddc">int</font>                                              | [随机数](#随机数)             |
| <font color="#7f7f7f">无风险</font> | `random(min, max, [id)`                                                   | 随机整数[min,max]  | `min` - <font color="#92cddc">int</font>, `max` - <font color="#92cddc">int</font>, `[id` - <font color="#92cddc">string</font>                                                                                                                                                                                  | `number` - <font color="#92cddc">int</font>                                              | [随机数](#随机数)             |
| <font color="#7f7f7f">无风险</font> | `random_float([id)`                                                       | 随机浮点数[0,1)    | `[id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                      | `number` - <font color="#92cddc">double</font>                                           | [随机数](#随机数)             |
| <font color="#7f7f7f">无风险</font> | `random_create(seed, [note)`                                              | 创建整数生成器     | `seed` - <font color="#92cddc">string</font>, `[note` - <font color="#92cddc">string</font>                                                                                                                                                                                                                      | `id` - <font color="#92cddc">string</font>                                               | [随机数](#随机数)             |
| <font color="#7f7f7f">无风险</font> | `random_float_create(seed, [note)`                                        | 创建浮点生成器     | `seed` - <font color="#92cddc">string</font>, `[note` - <font color="#92cddc">string</font>                                                                                                                                                                                                                      | `id` - <font color="#92cddc">string</font>                                               | [随机数](#随机数)             |
| <font color="#7f7f7f">无风险</font> | `random_reset_step(id)`                                                   | 重置步进数         | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                          | [随机数](#随机数)             |
| <font color="#7f7f7f">无风险</font> | `random_kill(id)`                                                         | 删除生成器         | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                          | [随机数](#随机数)             |
| <font color="#7f7f7f">无风险</font> | `set_random_note(id, note)`                                               | 设置生成器备注     | `id` - <font color="#92cddc">string</font>, `note` - <font color="#92cddc">string</font>                                                                                                                                                                                                                         | <font color="#7f7f7f">无</font>                                                          | [随机数](#随机数)             |
| <font color="#7f7f7f">无风险</font> | `get_random_list()`                                                       | 获取所有生成器     | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | `randoms` - <font color="#92cddc">table</font>                                           | [随机数](#随机数)             |
| <font color="#7f7f7f">无风险</font> | `get_random_info(id)`                                                     | 获取生成器信息     | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | `random` - <font color="#92cddc">table</font>                                            | [随机数](#随机数)             |
| <font color="#7f7f7f">无风险</font> | `get_random_step(id)`                                                     | 获取步进数         | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | `step` - <font color="#92cddc">int</font>                                                | [随机数](#随机数)             |
| <font color="#7f7f7f">无风险</font> | `get_random_seed(id)`                                                     | 获取种子           | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | `seed` - <font color="#92cddc">string</font>                                             | [随机数](#随机数)             |
| <font color="#7f7f7f">无风险</font> | `get_random_type(id)`                                                     | 获取生成器类型     | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                       | `type` - <font color="#92cddc">"int"</font> \| <font color="#92cddc">"float"</font>      | [随机数](#随机数)             |
| <font color="#7f7f7f">无风险</font> | `debug_log(message)`                                                      | 调试日志           | `message` - <font color="#92cddc">任意</font>                                                                                                                                                                                                                                                                    | <font color="#7f7f7f">无</font>                                                          | [调试信息](#调试信息)         |
| <font color="#7f7f7f">无风险</font> | `debug_warn(message)`                                                     | 警告日志           | `message` - <font color="#92cddc">任意</font>                                                                                                                                                                                                                                                                    | <font color="#7f7f7f">无</font>                                                          | [调试信息](#调试信息)         |
| <font color="#7f7f7f">无风险</font> | `debug_error(message)`                                                    | 错误日志           | `message` - <font color="#92cddc">任意</font>                                                                                                                                                                                                                                                                    | <font color="#7f7f7f">无</font>                                                          | [调试信息](#调试信息)         |
| <font color="#7f7f7f">无风险</font> | `debug_print(title, message)`                                             | 自定义日志         | `title` - <font color="#92cddc">string</font><br>`message` - <font color="#92cddc">任意</font>                                                                                                                                                                                                                   | <font color="#7f7f7f">无</font>                                                          | [调试信息](#调试信息)         |
| <font color="#7f7f7f">无风险</font> | `clear_debug_log()`                                                       | 清空日志           | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | <font color="#7f7f7f">无</font>                                                          | [调试信息](#调试信息)         |
| <font color="#7f7f7f">无风险</font> | `get_game_uid()`                                                          | 获取模组UID        | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | `uid` - <font color="#92cddc">string</font>                                              | [调试信息](#调试信息)         |
| <font color="#7f7f7f">无风险</font> | `get_game_info()`                                                         | 获取模组元信息     | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                  | `info` - <font color="#92cddc">table</font>                                              | [调试信息](#调试信息)         |

---

# 异常说明

> 注：本章节列举了脚本调用 API 时可能抛出的常见异常。除非另有说明，异常将导致脚本终止执行，并在游戏日志中记录。

## 通用异常

| 适用函数       | 异常条件                   | 异常信息                                                                           | 异常类型                             |
| -------------- | -------------------------- | ---------------------------------------------------------------------------------- | ------------------------------------ |
| 所有声明式 API | 脚本函数参数数量与约定不符 | 函数签名无效：`{function_name}` 应接收 `{expected}` 个参数，实际收到 `{actual}` 个 | <font color="orange">宿主异常</font> |
| 所有直用式 API | 脚本函数参数数量与约定不符 | 函数 `{function_name}` 参数数量错误：期望 `{expected}` 个，实际收到 `{actual}` 个  | <font color="purple">脚本异常</font> |
| 所有直用式 API | 脚本函数参数类型与约定不符 | 参数 `{param_name}` 类型错误：应为 `{expected_type}`，实际为 `{actual_type}`       | <font color="purple">脚本异常</font> |
| 所有 API       | 脚本执行时抛出未捕获异常   | `{error_message}`                                                                  | <font color="purple">脚本异常</font> |
| 所有 API       | API 版本不符合宿主要求     | API 版本不兼容：宿主版本为 `{host_version}`，模组支持的版本为 `{mod_version}`      | <font color="orange">宿主异常</font> |
| 所有 API       | 未按照路径找到入口脚本     | 入口脚本未找到：`{path}`                                                           | <font color="orange">宿主异常</font> |

---

## 声明式 API 异常

| 适用函数                                              | 异常条件                                                                 | 异常信息                                                                      | 异常类型                             |
| ----------------------------------------------------- | ------------------------------------------------------------------------ | ----------------------------------------------------------------------------- | ------------------------------------ |
| `init_game`, `handle_event`, `exit_game`, `save_game` | 应返回 `state` 但未返回或返回类型错误                                    | 返回值类型错误：`{function_name}` 应返回 table 类型，实际返回 `{actual_type}` | <font color="orange">宿主异常</font> |
| `init_game`, `handle_event`, `render`, `exit_game`    | 必须重写的 API 未实现                                                    | 缺少必须重写的声明式 API：`{function_name}`                                   | <font color="orange">宿主异常</font> |
| `save_best_score`, `save_game`                        | 条件满足但 API 未实现                                                    | 缺少按需重写的声明式 API：`{function_name}`（已在 game.json 中启用）          | <font color="orange">宿主异常</font> |
| `init_game`                                           | 返回的 `state` 包含不可序列化内容（函数、userdata、循环引用等）          | `state` 无法序列化：`{reason}`                                                | <font color="orange">宿主异常</font> |
| `save_game`                                           | 返回的 `state` 包含不可序列化内容                                        | 存档数据 `state` 无法序列化：`{reason}`                                       | <font color="orange">宿主异常</font> |
| `save_best_score`                                     | 应返回 `best` 但未返回或返回类型错误                                     | 返回值类型错误：`save_best_score` 应返回 table 类型，实际返回 `{actual_type}` | <font color="orange">宿主异常</font> |
| `save_best_score`                                     | 返回的 `best` 表缺少 `best_string` 字段                                  | `best` 表中缺少必填字段 `best_string`                                         | <font color="orange">宿主异常</font> |
| `save_best_score`                                     | `best_string` 字段类型不是字符串                                         | `best_string` 字段类型错误：应为字符串，实际为 `{actual_type}`                | <font color="orange">宿主异常</font> |
| `save_best_score`                                     | `best_string` 中包含 `{变量名}` 占位符，但 `best` 表中未提供对应的变量值 | `best_string` 中的变量 `{var_name}` 未在 `best` 表中定义                      | <font color="orange">宿主异常</font> |
| `handle_event`                                        | 脚本在单次调用中产生过多新事件导致队列溢出                               | 事件队列溢出：`handle_event` 单次调用产生事件过多                             | <font color="orange">宿主异常</font> |
| `exit_game`                                           | 退出流程中再次调用 `request_exit`                                        | 退出流程已在进行中，忽略本次退出请求                                          | <font color="orange">宿主异常</font> |

---

## 直用式 API 异常

### 内容绘制 异常

| 适用函数                               | 异常条件                                             | 异常信息                                                                   | 异常类型                             |
| -------------------------------------- | ---------------------------------------------------- | -------------------------------------------------------------------------- | ------------------------------------ |
| `canvas_draw_text`, `canvas_fill_rect` | 坐标或尺寸超出画布边界（终端范围）                   | 绘制区域超出画布边界：x=`{x}`, y=`{y}`, width=`{width}`, height=`{height}` | <font color="purple">脚本异常</font> |
| `canvas_draw_text`, `canvas_fill_rect` | 颜色值无效（不在预定义列表中，或自定义颜色格式错误） | 无效的颜色值：`{color}`                                                    | <font color="purple">脚本异常</font> |
| `canvas_fill_rect`                     | 矩形宽度或高度为负数或0                              | 矩形尺寸无效：宽度和高度必须为正整数，实际宽度=`{width}`, 高度=`{height}`  | <font color="purple">脚本异常</font> |
| `canvas_fill_rect`                     | `[char` 参数长度不为 1                               | 填充字符必须为单个字符，实际长度为 `{len}`                                 | <font color="purple">脚本异常</font> |
| 所有内容绘制 API                       | 画布未初始化                                         | 画布上下文不可用，请检查宿主环境                                           | <font color="orange">宿主异常</font> |

---

### 内容尺寸计算 异常

| 适用函数            | 异常条件         | 异常信息                         | 异常类型                             |
| ------------------- | ---------------- | -------------------------------- | ------------------------------------ |
| `get_terminal_size` | 无法获取终端尺寸 | 无法获取终端尺寸，请检查宿主环境 | <font color="orange">宿主异常</font> |

---

### 布局定位计算 异常

| 适用函数                                 | 异常条件                                      | 异常信息                                     | 异常类型                             |
| ---------------------------------------- | --------------------------------------------- | -------------------------------------------- | ------------------------------------ |
| `resolve_x`, `resolve_y`, `resolve_rect` | 锚点值无效（不是 `0`、`1`、`2` 或对应的常量） | 无效的锚点值：`{anchor}`                     | <font color="purple">脚本异常</font> |
| `resolve_x`, `resolve_rect`              | 内容宽度 `cw` 或矩形宽度 `width` 为负数或0    | 宽度无效：宽度必须为正整数，实际为 `{value}` | <font color="purple">脚本异常</font> |
| `resolve_y`, `resolve_rect`              | 内容高度 `ch` 或矩形高度 `height` 为负数或0   | 高度无效：高度必须为正整数，实际为 `{value}` | <font color="purple">脚本异常</font> |

---

### 系统请求 异常

| 适用函数            | 异常条件                     | 异常信息                             | 异常类型                             |
| ------------------- | ---------------------------- | ------------------------------------ | ------------------------------------ |
| `request_exit`      | 退出流程已在进行中，再次调用 | 退出流程已在进行中，忽略本次退出请求 | <font color="orange">脚本异常</font> |
| `get_launch_mode`   | 无法确定启动模式             | 无法获取启动模式：`{reason}`         | <font color="orange">宿主异常</font> |
| `clear_event_queue` | 事件队列不可用               | 事件队列不可用，清空操作失败         | <font color="orange">宿主异常</font> |

---

### 数据读取 异常

> 注：`translate` 在未找到值时优先返回 [missing-i18n-key:`key`] 提示字符串，在无法返回值时抛出异常

| 适用函数                                                                                 | 异常条件                           | 异常信息                                           | 异常类型                             |
| ---------------------------------------------------------------------------------------- | ---------------------------------- | -------------------------------------------------- | ------------------------------------ |
| `read_bytes`, `read_text`, `read_json`, `read_xml`, `read_yaml`, `read_toml`, `read_csv` | 文件不存在                         | 文件不存在：`{path}`                               | <font color="purple">脚本异常</font> |
| `read_bytes`, `read_text`, `read_json`, `read_xml`, `read_yaml`, `read_toml`, `read_csv` | 路径非法（包含 `..` 或绝对路径等） | 路径非法：`{path}`，只允许相对路径且不能包含 `..`  | <font color="purple">脚本异常</font> |
| `read_bytes`, `read_text`, `read_json`, `read_xml`, `read_yaml`, `read_toml`, `read_csv` | 读取权限不足                       | 无法读取文件：`{path}`，权限不足                   | <font color="orange">宿主异常</font> |
| `read_json`                                                                              | JSON 格式解析失败                  | JSON 解析失败：`{path}`，错误信息：`{parse_error}` | <font color="purple">脚本异常</font> |
| `read_xml`                                                                               | XML 格式解析失败                   | XML 解析失败：`{path}`，错误信息：`{parse_error}`  | <font color="purple">脚本异常</font> |
| `read_yaml`                                                                              | YAML 格式解析失败                  | YAML 解析失败：`{path}`，错误信息：`{parse_error}` | <font color="purple">脚本异常</font> |
| `read_toml`                                                                              | TOML 格式解析失败                  | TOML 解析失败：`{path}`，错误信息：`{parse_error}` | <font color="purple">脚本异常</font> |
| `read_csv`                                                                               | CSV 格式解析失败                   | CSV 解析失败：`{path}`，错误信息：`{parse_error}`  | <font color="purple">脚本异常</font> |
| `translate`                                                                              | 语言键不存在                       | 语言键不存在：`{key}`                              | <font color="purple">脚本异常</font> |
| `translate`                                                                              | 语言文件加载失败                   | 语言文件加载失败：`{reason}`                       | <font color="orange">宿主异常</font> |

---

### 数据写入 异常

| 适用函数                                                                                        | 异常条件                             | 异常信息                                          | 异常类型                             |
| ----------------------------------------------------------------------------------------------- | ------------------------------------ | ------------------------------------------------- | ------------------------------------ |
| `write_bytes`, `write_text`, `write_json`, `write_xml`, `write_yaml`, `write_toml`, `write_csv` | 路径非法（包含 `..` 或绝对路径等）   | 路径非法：`{path}`，只允许相对路径且不能包含 `..` | <font color="purple">脚本异常</font> |
| `write_bytes`, `write_text`, `write_json`, `write_xml`, `write_yaml`, `write_toml`, `write_csv` | 写入权限不足（用户未授权或文件只读） | 写入权限不足：`{path}`                            | <font color="orange">宿主异常</font> |
| `write_bytes`, `write_text`, `write_json`, `write_xml`, `write_yaml`, `write_toml`, `write_csv` | 磁盘空间不足                         | 磁盘空间不足，写入失败：`{path}`                  | <font color="orange">宿主异常</font> |
| `write_json`                                                                                    | 写入的 JSON 字符串格式无效           | 无效的 JSON 格式：`{content}`                     | <font color="purple">脚本异常</font> |
| `write_xml`                                                                                     | 写入的 XML 字符串格式无效            | 无效的 XML 格式：`{content}`                      | <font color="purple">脚本异常</font> |
| `write_yaml`                                                                                    | 写入的 YAML 字符串格式无效           | 无效的 YAML 格式：`{content}`                     | <font color="purple">脚本异常</font> |
| `write_toml`                                                                                    | 写入的 TOML 字符串格式无效           | 无效的 TOML 格式：`{content}`                     | <font color="purple">脚本异常</font> |
| `write_csv`                                                                                     | 写入的 CSV 字符串格式无效            | 无效的 CSV 格式：`{content}`                      | <font color="purple">脚本异常</font> |

---

### 辅助脚本加载 异常

| 适用函数              | 异常条件                                                | 异常信息                                          | 异常类型                             |
| --------------------- | ------------------------------------------------------- | ------------------------------------------------- | ------------------------------------ |
| `load_function(path)` | 路径非法（包含 `..` 或绝对路径等）                      | 路径非法：`{path}`，只允许相对路径且不能包含 `..` | <font color="purple">脚本异常</font> |
| `load_function(path)` | 文件不存在                                              | 文件不存在：`{path}`                              | <font color="purple">脚本异常</font> |
| `load_function(path)` | 脚本语法错误（Lua 解析失败）                            | 脚本语法错误：`{path}`，错误信息：`{parse_error}` | <font color="purple">脚本异常</font> |
| `load_function(path)` | 脚本运行时错误（如加载时执行了非法操作）                | 脚本加载错误：`{path}`，错误信息：`{error}`       | <font color="purple">脚本异常</font> |
| `load_function(path)` | 加载的文件不是有效的 Lua 脚本，或返回的表不包含任何函数 | 加载的内容无效：`{path}`，期望返回包含函数的表    | <font color="purple">脚本异常</font> |
| `load_function(path)` | 读取权限不足                                            | 无法读取文件：`{path}`，权限不足                  | <font color="orange">宿主异常</font> |

---

### 时间处理 异常

| 适用函数                                                                                                                                                                                                                                    | 异常条件               | 异常信息（中文）                                           | 异常类型                             |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------- | ---------------------------------------------------------- | ------------------------------------ |
| `timer_start`, `timer_pause`, `timer_resume`, `timer_reset`, `timer_restart`, `timer_kill`, `set_timer_note`, `get_timer_info`, `get_timer_status`, `get_timer_elapsed`, `get_timer_remaining`, `get_timer_duration`, `get_timer_completed` | 传入的计时器 ID 不存在 | 计时器不存在：`{id}`                                       | <font color="purple">脚本异常</font> |
| `timer_create`                                                                                                                                                                                                                              | `delay_ms` 为负数或0   | 计时时长无效：`delay_ms` 必须为正整数，实际为 `{delay_ms}` | <font color="purple">脚本异常</font> |
| `now`                                                                                                                                                                                                                                       | 宿主内部时间计数器异常 | 无法获取当前时间                                           | <font color="orange">宿主异常</font> |

---

### 随机数 API 异常

| 适用函数                                                                                                                                                    | 异常条件                                    | 异常信息（中文）                                                       | 异常类型                             |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------ |
| `random`, `random_float`, `random_reset_step`, `random_kill`, `set_random_note`, `get_random_info`, `get_random_step`, `get_random_seed`, `get_random_type` | 传入的随机数生成器 ID 不存在                | 随机数生成器不存在：`{id}`                                             | <font color="purple">脚本异常</font> |
| `random`, `random_float`                                                                                                                                    | 传入的 `[id` 对应的生成器类型与函数要求不符 | 随机数生成器类型不匹配：期望 `{expected_type}`，实际为 `{actual_type}` | <font color="purple">脚本异常</font> |
| `random(max, [id)`                                                                                                                                          | `max` 小于 `0`                              | 随机数范围无效：最大值 `{max}` 不能小于 0                              | <font color="purple">脚本异常</font> |
| `random(min, max, [id)`                                                                                                                                     | `min` 大于 `max`                            | 随机数范围无效：最小值 `{min}` 不能大于最大值 `{max}`                  | <font color="purple">脚本异常</font> |
| `random_create`, `random_float_create`                                                                                                                      | `seed` 为空字符串                           | 随机种子不能为空                                                       | <font color="purple">脚本异常</font> |
| 所有随机数 API                                                                                                                                              | 默认生成器或宿主随机数服务不可用            | 随机数生成器不可用                                                     | <font color="orange">宿主异常</font> |

---

### 调试信息 异常

| 适用函数                                                                   | 异常条件         | 异常信息                     | 异常类型                             |
| -------------------------------------------------------------------------- | ---------------- | ---------------------------- | ------------------------------------ |
| `debug_log`, `debug_warn`, `debug_error`, `debug_print`, `clear_debug_log` | 日志文件写入失败 | 写入日志文件失败：`{reason}` | <font color="orange">宿主异常</font> |
