# API 规范与查询

# 文档信息

1. 更新日期：2026年4月21日
2. API 版本：**7**
3. 本文档定义了脚本与宿主之间的交互接口规范，所有实现须遵循其中约定的函数签名、参数类型及行为准则，以确保兼容性与正确性。

# 文档导航

- [README](../../README-i18n/README-zh-cn.md)
- [模组包制作规范](MOD.md)
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
  - [表处理工具](#表处理工具)
  - [辅助脚本加载](#辅助脚本加载)
  - [时间处理](#时间处理)
  - [随机数](#随机数)
  - [调试信息](#调试信息)
- [附录](#附录)
  - [特定参数](#特定参数)
    - [锚点 anchor](#锚点-anchor)
    - [颜色 color](#颜色-color)
    - [对齐 align](#对齐-align)
    - [文本样式 style](#文本样式-style)
  - [调试输出目录](#调试输出目录)
  - [表转换格式](#表转换格式)
- [快速查询](#快速查询)
- [异常和警告速查表](#异常和警告速查表)

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

| 重写需求                                        | 函数名                          | 作用说明         | 参数名                                                                                           | 参数说明                                           | 传递值类型                                        | 传递值说明                                                       | 宿主调用时机                                |
| ------------------------------------------- | ---------------------------- | ------------ | --------------------------------------------------------------------------------------------- | ---------------------------------------------- | -------------------------------------------- | ----------------------------------------------------------- | ------------------------------------- |
| <font color="red">必须重写</font>               | `init_game(state)`           | 游戏脚本的初始化     | `state` - <font color="#92cddc">table</font> \| <font color="#92cddc">nil</font>              | 继续游戏时传入上次保存的 `state`；新游戏时传入 `nil`。             | `state` - <font color="#92cddc">table</font> | 传递初始化后的游戏状态。宿主会将其作为当前帧数据保存，并用于后续 `handle_event` 和 `render`。 | 游戏首次启动时调用一次。                          |
| <font color="red">必须重写</font>               | `handle_event(state, event)` | 游戏事件逻辑处理     | `state` - <font color="#92cddc">table</font><br> `event` - <font color="#92cddc">table</font> | `state`：宿主临时存储的游戏上一帧数据；<br>`event`：宿主解析后的事件信息。 | `state` - <font color="#92cddc">table</font> | 传递更新后的游戏状态。宿主会用其替换当前帧数据。                                    | 游戏运行时，每帧对事件队列中的每个事件依次调用。              |
| <font color="red">必须重写</font>               | `render(state)`              | 游戏画面绘制       | `state` - <font color="#92cddc">table</font>                                                  | 宿主临时存储的游戏当前帧数据。                                | <font color="#7f7f7f">无</font>               | <font color="#7f7f7f">无传递值</font>                           | 游戏运行时，每帧在所有事件处理完成后调用一次，脚本也可以手动调用。     |
| <font color="red">必须重写</font>               | `exit_game(state)`           | 游戏退出前的最后一次处理 | `state` - <font color="#92cddc">table</font>                                                  | 宿主临时存储的游戏当前帧数据。                                | `state` - <font color="#92cddc">table</font> | 传递修改后的 `state`，可供后续 `save_best_score` 使用。                   | 脚本调用 `request_exit()` 后，宿主在退出前调用一次。   |
| 当 `game.json` 中 `best_none` 不为 `null` 时必须重写 | `save_best_score(state)`     | 向宿主传递游戏最佳记录  | `state` - <font color="#92cddc">table</font>                                                  | 宿主临时存储的游戏当前帧数据。                                | `best` - <font color="#92cddc">table</font>  | 传递包含最佳记录文本及变量表的 `best` 表，结构见下文。                             | 宿主在 `exit_game` 之后自动调用（若需要），脚本也可手动调用。 |
| 当 `game.json` 中 `save` 为 `true` 时必须重写       | `save_game(state)`           | 保存游戏存档       | `state` - <font color="#92cddc">table</font>                                                  | 宿主临时存储的游戏当前帧数据。                                | `state` - <font color="#92cddc">table</font> | 传递用于长期存储的 `state`。**注意**：此传递值仅用于存档，当前游戏会继续使用传入的原 `state`。   | 由脚本手动调用，宿主不会自动调用。                     |

## 执行流程

宿主与脚本运行链如下图所示：

![执行流程|697](./image/program_flowchart.png)

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
    if event.type == "action" then
      if event.name == "quit" then
        request_exit()
      end
    end
	
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

-- game.json
-- "best_none": "最高分：--"
function save_best_score(state)
    local best = {
        best_string = "最高分：{score}",
        score = state.final_score
    }
    return best
end

-- game.json
-- "save": true
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
- `type` 字段决定了事件的类型，具体取值及对应的扩展字段见下文『声明式 API -[事件类型](#事件类型)』。

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
宿主根据 `game.json` 中的 `actions` 配置，将物理按键映射为语义化动作事件。
适用于自定义动作按键的处理。

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
通知脚本终端显示区域的宽度或高度发生变化。
可用于响应式重新布局。

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
   - 当 `game.json` 中 `best_none` 不为 `null` 时，**必须实现** `save_best_score`。
   - 当 `game.json` 中 `save` 为 `true` 时，**必须实现** `save_game`。

### 二、返回值规范

| 函数                           | 传递值要求              |
| ---------------------------- | ------------------ |
| `init_game(state)`           | **必须传递** `state` 表 |
| `handle_event(state, event)` | **必须传递** `state` 表 |
| `exit_game(state)`           | **必须传递** `state` 表 |
| `render(state)`              | 无传递值               |
| `save_best_score(state)`     | **必须传递** `best` 表  |
| `save_game(state)`           | **必须传递** `state` 表 |

### 三、宿主职责与限制

1. 宿主仅负责**事件的交流**与 **`state` 的存储/恢复**，**不对事件或 `state` 进行任何游戏逻辑处理**。所有游戏逻辑（状态更新、事件响应、画面绘制等）均需由脚本自身实现。
2. `save_game` 传递的 `state` 仅用于**持久化存档**，当前游戏的运行会使用传入的原始 `state` 继续游戏帧的循环。

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
> - `*` 表示特定参数，需参考相关提示填写。
> - 多返回值以多个独立值返回，而非表。

---

## 系统请求

> 注：`request_skip_event_queue` 和 `request_clear_event_queue` 不会影响队尾的 `tick` 事件。`tick` 事件在每个帧循环中**必定**会被传入，详细流程见『声明式 API -[执行流程](#执行流程)』。

| 可达性                              | 函数名                           | 作用                                         | 参数                             | 返回值                                                                                                                           |
| -------------------------------- | ----------------------------- | ------------------------------------------ | ------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| <font color="#7f7f7f">无要求</font> | `get_launch_mode()`           | 获取本次游戏的启动模式。                               | <font color="#7f7f7f">无</font> | `status` - <font color="#92cddc">"new"</font> \| <font color="#92cddc">"continue"</font>：`"new"` 表示新游戏，`"continue"` 表示继续已有存档。 |
| <font color="#7f7f7f">无要求</font> | `get_best_score()`            | 获取游戏存储的最佳记录数据。                             | <font color="#7f7f7f">无</font> | `data` - <font color="#92cddc">table</font> \| <font color="#92cddc">nil</font>：存储的最佳记录数据，宿主返回脚本所传递的 best 参数，若不存在返回 nil。      |
| <font color="red">至少一条可达</font>  | `request_exit()`              | 向宿主发送退出游戏请求。                               | <font color="#7f7f7f">无</font> | <font color="#7f7f7f">无</font>                                                                                                |
| <font color="#7f7f7f">无要求</font> | `request_skip_event_queue()`  | 向宿主发送跳过尚未处理的事件队列请求。                        | <font color="#7f7f7f">无</font> | <font color="#7f7f7f">无</font>                                                                                                |
| <font color="#7f7f7f">无要求</font> | `request_clear_event_queue()` | 向宿主发送清空尚未处理的事件队列请求。                        | <font color="#7f7f7f">无</font> | <font color="#7f7f7f">无</font>                                                                                                |
| <font color="#7f7f7f">无要求</font> | `request_render()`            | 请求宿主调用 `render(state)` 以重绘当前界面。            | <font color="#7f7f7f">无</font> | <font color="#7f7f7f">无</font>                                                                                                |
| <font color="#7f7f7f">无要求</font> | `request_save_best_score()`   | 请求宿主调用 `save_best_score(state)` 以保存当前最佳记录。 | <font color="#7f7f7f">无</font> | <font color="#7f7f7f">无</font>                                                                                                |
| <font color="#7f7f7f">无要求</font> | `request_save_game()`         | 请求宿主调用 `save_game(state)` 以保存当前游戏存档。       | <font color="#7f7f7f">无</font> | <font color="#7f7f7f">无</font>                                                                                                |

---

## 内容绘制

> 注：
>
> 1. `color` 类型见『附录-[颜色 color](#颜色-color)』。
> 2. `align` 类型见『附录-[对齐 align](#对齐-align)』。
> 3. `style` 类型见『附录-[文本样式 style](#文本样式-style)』。
> 4. 宽度、高度参数均以**终端字符数**为单位（宽度为列数，高度为行数）。
> 5. 所有绘制操作的基准点均为**内容的左上角**，即绘制内容将从指定的 (x, y) 坐标处开始向右、向下延伸。
> 6. 绘制坐标详细见『模组包制作规范及教程-其它-[绘制坐标](MOD.md#绘制坐标)』。

| 函数名                                                               | 作用                       | 参数                                                                                                                                                                                                                                                                                                                                                                                              | 返回值                            |
| ----------------------------------------------------------------- | ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------ |
| `canvas_clear()`                                                  | 清空当前帧的画布。                | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                                                                                  | <font color="#7f7f7f">无</font> |
| `canvas_eraser(x, y, width, height)`                              | 清空画布指定区域。                | `x` - <font color="#92cddc">int</font>：横轴坐标。<br>`y` - <font color="#92cddc">int</font>：纵轴坐标。<br>`width` - <font color="#92cddc">int</font>：区域宽度。<br>`height` - <font color="#92cddc">int</font>：区域高度。                                                                                                                                                                                           | <font color="#7f7f7f">无</font> |
| `canvas_draw_text(x, y, text, *[fg, *[bg, *[style, *[align)`      | 在画布指定位置绘制字符串。            | `x` - <font color="#92cddc">int</font>：横轴坐标。<br>`y` - <font color="#92cddc">int</font>：纵轴坐标。<br>`text` - <font color="#92cddc">string</font>：要绘制的字符串。<br>`*[fg` - <font color="#92cddc">color</font>：可选，字符颜色。<br>`*[bg` - <font color="#92cddc">color</font>：可选，背景颜色。<br>`*[style` - <font color="#92cddc">style</font>：可选，文本样式。<br>`*[align` - <font color="#92cddc">align</font>：可选，换行内容对齐方式。 | <font color="#7f7f7f">无</font> |
| `canvas_fill_rect(x, y, width, height, [char, *[fg, *[bg)`        | 从指定位置绘制矩形，并使用指定字符填充。     | `x` - <font color="#92cddc">int</font>：横轴坐标。<br>`y` - <font color="#92cddc">int</font>：纵轴坐标。<br>`width` - <font color="#92cddc">int</font>：矩形宽度。<br>`height` - <font color="#92cddc">int</font>：矩形高度。<br>`[char` - <font color="#92cddc">char</font>：可选，用于填充的单个字符。<br>`*[fg` - <font color="#92cddc">color</font>：可选，字符颜色。<br>`*[bg` - <font color="#92cddc">color</font>：可选，背景颜色。              | <font color="#7f7f7f">无</font> |
| `canvas_border_rect(x, y, width, height, [char_list, *[fg, *[bg)` | 从指定位置绘制矩形边框，并使用指定字符作为边框。 | `x` - <font color="#92cddc">int</font>：横轴坐标。<br>`y` - <font color="#92cddc">int</font>：纵轴坐标。<br>`width` - <font color="#92cddc">int</font>：矩形宽度。<br>`height` - <font color="#92cddc">int</font>：矩形高度。<br>`[char_list` - <font color="#92cddc">table</font>：可选，边框字符配置表，结构见下文。<br>`*[fg` - <font color="#92cddc">color</font>：可选，字符颜色。<br>`*[bg` - <font color="#92cddc">color</font>：可选，背景颜色。    | <font color="#7f7f7f">无</font> |

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

> 注：
>
> 1. 宽度、高度返回值均以**终端字符数**为单位（宽度为列数，高度为行数）。
> 2. 所有计算操作的基准点均为**内容的左上角**，即计算内容将从指定的 (x, y) 坐标处开始向右、向下延伸。
> 3. 绘制坐标详细见『模组包制作规范及教程-[绘制坐标](MOD.md#绘制坐标)』。

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
> 1. `x_anchor` 和 `y_anchor` 类型见 附录-[锚点 anchor](#锚点-anchor)
> 2. 宽度、高度参数均以**终端字符数**为单位（宽度为列数，高度为行数）。
> 3. 所有计算操作的基准点均为**内容的左上角**，即计算内容将从指定的 (x, y) 坐标处开始向右、向下延伸。
> 4. 绘制坐标详细见『模组包制作规范及教程-[绘制坐标](MOD.md#绘制坐标)』。

| 函数名                                                                    | 作用                                                         | 参数                                                                                                                                                                                                                                                                                                                                                                                                | 返回值                                                                                                         |
| ------------------------------------------------------------------------- | ------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `resolve_x(*x_anchor, width, [offset_x)`                                  | 根据水平锚点、内容宽度和偏移量，计算起始 X 坐标。            | `*x_anchor` - <font color="#92cddc">x_anchor</font>：水平锚点。<br>`width` - <font color="#92cddc">int</font>：内容宽度。<br>`[offset_x` - <font color="#92cddc">int</font>：可选，水平偏移量。                                                                                                                                                                                                     | `x` - <font color="#92cddc">int</font>：起始 X 坐标。                                                          |
| `resolve_y(*y_anchor, height, [offset_y)`                                 | 根据垂直锚点、内容高度和偏移量，计算起始 Y 坐标。            | `*y_anchor` - <font color="#92cddc">y_anchor</font>：垂直锚点。<br>`height` - <font color="#92cddc">int</font>：内容高度。<br>`[offset_y` - <font color="#92cddc">int</font>：可选，垂直偏移量。                                                                                                                                                                                                    | `y` - <font color="#92cddc">int</font>：起始 Y 坐标。                                                          |
| `resolve_rect(*x_anchor, *y_anchor, width, height, [offset_x, [offset_y)` | 根据水平和垂直锚点、宽高及偏移量，计算矩形的起始 X、Y 坐标。 | `*x_anchor` - <font color="#92cddc">x_anchor</font>：水平锚点。<br>`*y_anchor` - <font color="#92cddc">y_anchor</font>：垂直锚点。<br>`width` - <font color="#92cddc">int</font>：矩形宽度。<br>`height` - <font color="#92cddc">int</font>：矩形高度。<br>`[offset_x` - <font color="#92cddc">int</font>：可选，水平偏移量。<br>`[offset_y` - <font color="#92cddc">int</font>：可选，垂直偏移量。 | `x` - <font color="#92cddc">int</font>：起始 X 坐标。<br>`y` - <font color="#92cddc">int</font>：起始 Y 坐标。 |

---

## 数据读取

> 注：
>
> 1. 本章节中所有 `path` 参数均相对于游戏资源包中的 `assets/` 目录。
> 2. 请注意返回值的数据类型，避免解析错误。

| 函数名               | 作用                                 | 参数                                                              | 返回值                                                      |
| ----------------- | ---------------------------------- | --------------------------------------------------------------- | -------------------------------------------------------- |
| `translate(key)`  | 读取当前游戏资源包中指定语言键对应的本地化字符串。          | `key` - <font color="#92cddc">string</font>：语言键。                | `value` - <font color="#92cddc">string</font>：对应的本地化字符串。 |
| `read_text(path)` | 读取资源包中指定路径的 `.txt` 文本文件。           | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">string</font>：文件文本内容。     |
| `read_json(path)` | 读取资源包中指定路径的 `.json` 文件，并解析为 Lua 表。 | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">table</font>：解析后的 Lua 表。  |
| `read_xml(path)`  | 读取资源包中指定路径的 `.xml` 文件，并解析为 Lua 表。  | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">table</font>：解析后的 Lua 表。  |
| `read_yaml(path)` | 读取资源包中指定路径的 `.yaml` 文件，并解析为 Lua 表。 | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">table</font>：解析后的 Lua 表。  |
| `read_toml(path)` | 读取资源包中指定路径的 `.toml` 文件，并解析为 Lua 表。 | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">table</font>：解析后的 Lua 表。  |
| `read_csv(path)`  | 读取资源包中指定路径的 `.csv` 文件，并解析为 Lua 表。  | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">table</font>：解析后的 Lua 表。  |
| `read_json_string(path)` | 读取资源包中指定路径的 `.json` 文件，直接返回读取内容。 | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">string</font>：读取文件的原文字符串。  |
| `read_xml_string(path)`  | 读取资源包中指定路径的 `.xml` 文件，直接返回读取内容。  | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">string</font>：读取文件的原文字符串。  |
| `read_yaml_string(path)` | 读取资源包中指定路径的 `.yaml` 文件，直接返回读取内容。 | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">string</font>：读取文件的原文字符串。  |
| `read_toml_string(path)` | 读取资源包中指定路径的 `.toml` 文件，直接返回读取内容。 | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">string</font>：读取文件的原文字符串。  |
| `read_csv_string(path)`  | 读取资源包中指定路径的 `.csv` 文件，直接返回读取内容。  | `path` - <font color="#92cddc">string</font>：相对于 `assets/` 的路径。 | `data` - <font color="#92cddc">string</font>：读取文件的原文字符串。  |

---

## 数据写入

> 注：
>
> 1. 本章节中所有 `path` 参数均相对于游戏资源包中的 `assets/` 目录。
> 2. 所有 `write_*` 函数的 `content` 参数均为 `string` 类型。
> 3. 所有 `write_*` 函数仅为语义命名，实际写入并不会做结构检查。
> 4. 所有 `write_*` 函数均为高风险直写操作。仅当 `game.json` 中 `write` 字段为 `true` 且用户授予模组包“完全信任权限”时，直写操作才会被执行；否则所有直写请求将被宿主忽略。
> 5. 无论直写操作是否执行，每次调用都会在 `tui_log.txt` 中记录，供用户安全检查。

<font color="red"><b>直写操作不可撤回！</b></font>
<font color="red"><b>直写操作不可撤回！</b></font>
<font color="red"><b>直写操作不可撤回！</b></font>

<font color="red"><b>直写操作均为高风险操作，请最大程度避免使用！</b></font>

| 风险等级                        | 函数名                      | 作用                             | 参数                                                                                                                                | 返回值                                                                             |
| ------------------------------- | --------------------------- | -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| <font color="red">高风险</font> | `write_text(path, content)` | 写入文本文件到资源包指定路径。   | `path` - <font color="#92cddc">string</font>：文件路径。<br>`content` - <font color="#92cddc">string</font>：要写入的文本内容。     | `bool` - <font color="#92cddc">boolean</font>：`true` 写入成功，`false` 写入失败。 |
| <font color="red">高风险</font> | `write_json(path, content)` | 写入 JSON 文件到资源包指定路径。 | `path` - <font color="#92cddc">string</font>：文件路径。<br>`content` - <font color="#92cddc">string</font>：要写入的 JSON 字符串。 | `bool` - <font color="#92cddc">boolean</font>：`true` 写入成功，`false` 写入失败。 |
| <font color="red">高风险</font> | `write_xml(path, content)`  | 写入 XML 文件到资源包指定路径。  | `path` - <font color="#92cddc">string</font>：文件路径。<br>`content` - <font color="#92cddc">string</font>：要写入的 XML 字符串。  | `bool` - <font color="#92cddc">boolean</font>：`true` 写入成功，`false` 写入失败。 |
| <font color="red">高风险</font> | `write_yaml(path, content)` | 写入 YAML 文件到资源包指定路径。 | `path` - <font color="#92cddc">string</font>：文件路径。<br>`content` - <font color="#92cddc">string</font>：要写入的 YAML 字符串。 | `bool` - <font color="#92cddc">boolean</font>：`true` 写入成功，`false` 写入失败。 |
| <font color="red">高风险</font> | `write_toml(path, content)` | 写入 TOML 文件到资源包指定路径。 | `path` - <font color="#92cddc">string</font>：文件路径。<br>`content` - <font color="#92cddc">string</font>：要写入的 TOML 字符串。 | `bool` - <font color="#92cddc">boolean</font>：`true` 写入成功，`false` 写入失败。 |
| <font color="red">高风险</font> | `write_csv(path, content)`  | 写入 CSV 文件到资源包指定路径。  | `path` - <font color="#92cddc">string</font>：文件路径。<br>`content` - <font color="#92cddc">string</font>：要写入的 CSV 字符串。  | `bool` - <font color="#92cddc">boolean</font>：`true` 写入成功，`false` 写入失败。 |

---

## 表处理工具

> 注：
>
> 1. 该部分 API 用于将表转换为各种数据格式的字符串，或进行表的深拷贝操作。
> 2. 转换结果主要用于调试输出、数据交换或持久化存储。
> 3. 转换函数对表格式有严格要求，详细见『附录-[表转换格式](#表转换格式)』。

| 函数名                    | 作用                     | 参数                                                            | 返回值                                                                                              |
| ---------------------- | ---------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `table_to_json(table)` | 将 Lua 表转换为 JSON 格式字符串。 | `table` - <font color="#92cddc">table</font>：要转换的表。           | `json_str` - <font color="#92cddc">string</font>：JSON 格式字符串。                                     |
| `table_to_yaml(table)` | 将 Lua 表转换为 YAML 格式字符串。 | `table` - <font color="#92cddc">table</font>：要转换的表。           | `yaml_str` - <font color="#92cddc">string</font>：YAML 格式字符串。 |
| `table_to_toml(table)` | 将 Lua 表转换为 TOML 格式字符串。 | `table` - <font color="#92cddc">table</font>：要转换的表。           | `toml_str` - <font color="#92cddc">string</font>：TOML 格式字符串。 |
| `table_to_csv(table)`  | 将 Lua 表转换为 CSV 格式字符串。  | `table` - <font color="#92cddc">table</font>：要转换的表。 | `csv_str` - <font color="#92cddc">string</font>：CSV 格式字符串。   |
| `table_to_xml(table)`  | 将 Lua 表转换为 XML 格式字符串。  | `table` - <font color="#92cddc">table</font>：要转换的表。           | `xml_str` - <font color="#92cddc">string</font>：XML 格式字符串。   |
| `deep_copy(table)`     | 深拷贝一个 Lua 表，返回全新的独立副本。 | `table` - <font color="#92cddc">table</font>：要拷贝的表。           | `new_table` - <font color="#92cddc">table</font>：深拷贝后的新表。                                        |

---

## 辅助脚本加载

> 注：辅助脚本的具体编写与使用请参考『模组包制作规范及教程-其它-[辅助脚本规范](MOD.md#辅助脚本规范)』。

| 函数名                | 作用                                                                      | 参数                                                                          | 返回值                                                                           |
| --------------------- | ------------------------------------------------------------------------- | ----------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| `load_function(path)` | 加载指定路径的 Lua 辅助脚本，返回脚本中定义的所有变量和函数（以表形式）。 | `path` - <font color="#92cddc">string</font>：相对于 `function/` 目录的路径。 | `functions` - <font color="#92cddc">table</font>：包含脚本中所有变量和函数的表。 |

---

## 时间处理

> 注：
>
> 1. 计时器创建后处于 `init` 状态，需调用 `timer_start` 或 `timer_restart` 才会启动。计时器结束后状态变为 `completed`，不会自动删除，需手动调用 `timer_kill` 清理。
> 2. `timer_reset` 将计时器重置为 `init` 状态（已过时间归零）；`timer_restart` 相当于 reset + start。
> 3. 查询 `init` 状态的计时器，`elapsed` 返回 0；查询 `completed` 状态的计时器，`remaining` 返回 0。
> 4. 所有脚本创建的计时器会在游戏退出后被删除。
> 5. 每个游戏运行时最多同时存在 64 个计时器。

| 是否启动计时器                        | 函数名                                                               | 作用                                      | 参数                                                                                                                                                                                                                                                                                                                                                                    | 返回值                                                                                                                                                                                                                                                              |
| ------------------------------ | ----------------------------------------------------------------- | --------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| <font color="#7f7f7f">否</font> | `running_time()`                                                  | 获取从游戏启动到当前时刻经过的总时长。                     | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                                                        | `time` - <font color="#92cddc">int</font>：已运行总时长（毫秒）。                                                                                                                                                                                                            |
| <font color="#7f7f7f">否</font> | `timer_create(delay_ms, [note)`                                   | 创建一个持续 `delay_ms` 毫秒的计时器（初始状态为 `init`）。 | `delay_ms` - <font color="#92cddc">int</font>：计时时长（毫秒）。<br>`[note` - <font color="#92cddc">string</font>：可选，计时器备注信息。                                                                                                                                                                                                                                                  | `id` - <font color="#92cddc">string</font>：计时器唯一标识 ID。                                                                                                                                                                                                           |
| <font color="red">是</font>     | `timer_start(id)`                                                 | 启动指定 ID 的计时器。                           | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                                                                                                                                                                                                                                                    | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                   |
| <font color="#7f7f7f">否</font> | `timer_pause(id)`                                                 | 暂停指定 ID 的计时器。                           | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                                                                                                                                                                                                                                                    | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                   |
| <font color="#7f7f7f">否</font> | `timer_resume(id)`                                                | 恢复暂停的计时器，从暂停点继续计时。                      | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                                                                                                                                                                                                                                                    | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                   |
| <font color="#7f7f7f">否</font> | `timer_reset(id)`                                                 | 重置指定 ID 的计时器。                           | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                                                                                                                                                                                                                                                    | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                   |
| <font color="red">是</font>     | `timer_restart(id)`                                               | 重置并立即启动指定 ID 的计时器。                      | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                                                                                                                                                                                                                                                    | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                   |
| <font color="#7f7f7f">否</font> | `timer_kill(id)`                                                  | 删除指定 ID 的计时器。                           | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                                                                                                                                                                                                                                                    | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                   |
| <font color="#7f7f7f">否</font> | `set_timer_note(id, note)`                                        | 修改指定 ID 计时器的备注信息。                       | `id` - <font color="#92cddc">string</font>：计时器 ID。<br>`note` - <font color="#92cddc">string</font>：新的备注信息。                                                                                                                                                                                                                                                            | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                   |
| <font color="#7f7f7f">否</font> | `get_timer_list()`                                                | 获取所有计时器的信息列表。                           | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                                                        | `timers` - <font color="#92cddc">table</font>：计时器信息表，结构见下文。                                                                                                                                                                                                      |
| <font color="#7f7f7f">否</font> | `get_timer_info(id)`                                              | 获取指定 ID 计时器的详细信息。                       | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                                                                                                                                                                                                                                                    | `timer` - <font color="#92cddc">table</font> 计时器信息表，结构见下文。                                                                                                                                                                                                       |
| <font color="#7f7f7f">否</font> | `get_timer_status(id)`                                            | 获取指定 ID 计时器的当前状态。                       | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                                                                                                                                                                                                                                                    | `status` - <font color="#92cddc">"init"</font> \| <font color="#92cddc">"running"</font> \| <font color="#92cddc">"pause"</font> \| <font color="#92cddc">"completed"</font>：<br>`"init"` 初始状态，未启动；<br>`"running"` 正在运行；<br>`"pause"` 已暂停；<br>`"completed"` 已结束。 |
| <font color="#7f7f7f">否</font> | `get_timer_elapsed(id)`                                           | 获取指定 ID 计时器的已过时间。                       | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                                                                                                                                                                                                                                                    | `time` - <font color="#92cddc">int</font>：已过时间（毫秒）。                                                                                                                                                                                                              |
| <font color="#7f7f7f">否</font> | `get_timer_remaining(id)`                                         | 获取指定 ID 计时器的剩余时间。                       | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                                                                                                                                                                                                                                                    | `time` - <font color="#92cddc">int</font>：剩余时间（毫秒）。                                                                                                                                                                                                              |
| <font color="#7f7f7f">否</font> | `get_timer_duration(id)`                                          | 获取指定 ID 计时器的总时长。                        | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                                                                                                                                                                                                                                                    | `time` - <font color="#92cddc">int</font>：总时长（毫秒）。                                                                                                                                                                                                               |
| <font color="#7f7f7f">否</font> | `is_timer_completed(id)`                                          | 检查指定 ID 的计时器是否已结束。                      | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                                                                                                                                                                                                                                                    | `bool` - <font color="#92cddc">boolean</font>：`true` 已结束，`false` 未结束 。                                                                                                                                                                                           |
| <font color="#7f7f7f">否</font> | `is_timer_exists(id)`                                             | 检查指定 ID 的计时器是否存在。                       | `id` - <font color="#92cddc">string</font>：计时器 ID。                                                                                                                                                                                                                                                                                                                    | `bool` - <font color="#92cddc">boolean</font>：`true` 存在，`false` 不存在。                                                                                                                                                                                             |
| <font color="#7f7f7f">否</font> | `now()`                                                           | 获取当前现实世界的时间戳。                           | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                                                        | `timestamp` - <font color="#92cddc">int</font>：已当前时间戳。                                                                                                                                                                                                           |
| <font color="#7f7f7f">否</font> | `get_current_year()`                                              | 获取当前年份。                                 | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                                                        | `year` - <font color="#92cddc">int</font>：当前年份。                                                                                                                                                                                                                  |
| <font color="#7f7f7f">否</font> | `get_current_month()`                                             | 获取当前月份。                                 | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                                                        | `month` - <font color="#92cddc">int</font>：当前月份（1–12）。                                                                                                                                                                                                           |
| <font color="#7f7f7f">否</font> | `get_current_day()`                                               | 获取当前日期（月中第几天）。                          | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                                                        | `day` - <font color="#92cddc">int</font>：当前日期（1–31）。                                                                                                                                                                                                             |
| <font color="#7f7f7f">否</font> | `get_current_hour()`                                              | 获取当前小时（24 小时制）。                         | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                                                        | `hour` - <font color="#92cddc">int</font>：当前小时（0–23）。                                                                                                                                                                                                            |
| <font color="#7f7f7f">否</font> | `get_current_minute()`                                            | 获取当前分钟。                                 | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                                                        | `minute` - <font color="#92cddc">int</font>：当前分钟（0–59）。                                                                                                                                                                                                          |
| <font color="#7f7f7f">否</font> | `get_current_second()`                                            | 获取当前秒数。                                 | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                                                        | `second` - <font color="#92cddc">int</font>：当前秒数（0–59）。                                                                                                                                                                                                          |
| <font color="#7f7f7f">否</font> | `timestamp_to_date(timestamp, [format)`                           | 将时间戳转换为格式化的日期时间字符串。                     | `timestamp` - <font color="#92cddc">int</font>：时间戳。<br>`[format` - <font color="#92cddc">string</font>：可选，格式化模板，默认 `"{year}-{month}-{day} {hour}:{minute}:{second}"`。                                                                                                                                                                                                 | `date_str` - <font color="#92cddc">string</font>：格式化后的日期字符串。                                                                                                                                                                                                     |
| <font color="#7f7f7f">否</font> | `date_to_timestamp([year, [month, [day, [hour, [minute, [second)` | 将日期字符串解析为时间戳。                           | `[year` - <font color="#92cddc">int</font>: 可选，年，默认为2000年。<br>`[month` - <font color="#92cddc">int</font>: 可选，月，默认为1月。<br>`[day` - <font color="#92cddc">int</font>: 可选，日，默认为1日。<br>`[hour` - <font color="#92cddc">int</font>: 可选，时，默认为0时。<br>`[minute` - <font color="#92cddc">int</font>: 可选，分，默认为0分。<br>`[second` - <font color="#92cddc">int</font>: 可选，秒，默认为0秒。 | `timestamp` - <font color="#92cddc">int</font>：时间戳。                                                                                                                                                                                                              |

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
> 2. 若未使用 `random_create` 或 `random_float_create` 构建的生成器，则默认使用宿主提供的随机数生成器，该随机数生成器结果不可复现。
> 3. 参数使用不存在的随机数生成器 ID 会抛出异常。
> 4. `random` 系列函数只能使用整数类型生成器，`random_float` 只能使用浮点数类型生成器，类型不匹配时会抛出异常。
> 5. 所有脚本构建的随机数生成器会在游戏退出后被删除。

| 函数名                                | 作用                           | 参数                                                                                                                                                                    | 返回值                                                                                       |
| ---------------------------------- | ---------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `random([id)`                      | 生成区间 $[0, 2^{31}-1]$ 内的随机整数。 | `[id` - <font color="#92cddc">string</font>：可选，随机数生成器 ID。                                                                                                             | `number` - <font color="#92cddc">int</font>：随机整数。                                         |
| `random(max, [id)`                 | 生成区间 $[0, max]$ 内的随机整数。      | `max` - <font color="#92cddc">int</font>：区间上限（包含）。<br>`[id` - <font color="#92cddc">string</font>：可选，随机数生成器 ID。                                                       | `number` - <font color="#92cddc">int</font>：随机整数。                                         |
| `random(min, max, [id)`            | 生成区间 $[min, max]$ 内的随机整数。    | `min` - <font color="#92cddc">int</font>：区间下限（包含）。<br>`max` - <font color="#92cddc">int</font>：区间上限（包含）。<br>`[id` - <font color="#92cddc">string</font>：可选，随机数生成器 ID。 | `number` - <font color="#92cddc">int</font>：随机整数。                                         |
| `random_float([id)`                | 生成区间 $[0, 1)$ 内的随机浮点数。       | `[id` - <font color="#92cddc">string</font>：可选，随机数生成器 ID。                                                                                                             | `number` - <font color="#92cddc">double</font>：随机浮点数。                                     |
| `random_create(seed, [note)`       | 创建一个整数随机数生成器。                | `seed` - <font color="#92cddc">string</font>：随机种子。<br>`[note` - <font color="#92cddc">string</font>：可选，备注信息。                                                          | `id` - <font color="#92cddc">string</font>：生成器 ID。                                        |
| `random_float_create(seed, [note)` | 创建一个浮点数随机数生成器。               | `seed` - <font color="#92cddc">string</font>：随机种子。<br>`[note` - <font color="#92cddc">string</font>：可选，备注信息。                                                          | `id` - <font color="#92cddc">string</font>：生成器 ID。                                        |
| `random_reset_step(id)`            | 重置指定随机数生成器的步进数（步进数归零）。       | `id` - <font color="#92cddc">string</font>：生成器 ID。                                                                                                                    | <font color="#7f7f7f">无</font>                                                            |
| `random_kill(id)`                  | 删除指定随机数生成器。                  | `id` - <font color="#92cddc">string</font>：生成器 ID。                                                                                                                    | <font color="#7f7f7f">无</font>                                                            |
| `set_random_note(id, note)`        | 修改指定随机数生成器的备注信息。             | `id` - <font color="#92cddc">string</font>：生成器 ID。<br>`note` - <font color="#92cddc">string</font>：新的备注信息。                                                            | <font color="#7f7f7f">无</font>                                                            |
| `get_random_list()`                | 获取所有已创建的随机数生成器信息列表。          | <font color="#7f7f7f">无</font>                                                                                                                                        | `randoms` - <font color="#92cddc">table</font>：信息列表，结构见下文。                                |
| `get_random_info(id)`              | 获取指定随机数生成器的详细信息。             | `id` - <font color="#92cddc">string</font>：生成器 ID。                                                                                                                    | `random` - <font color="#92cddc">table</font>：信息表，结构见下文。                                  |
| `get_random_step(id)`              | 获取指定随机数生成器的当前步进数。            | `id` - <font color="#92cddc">string</font>：生成器 ID。                                                                                                                    | `step` - <font color="#92cddc">int</font>：步进数。                                            |
| `get_random_seed(id)`              | 获取指定随机数生成器的种子。               | `id` - <font color="#92cddc">string</font>：生成器 ID。                                                                                                                    | `seed` - <font color="#92cddc">string</font>：种子字符串。                                       |
| `get_random_type(id)`              | 获取指定随机数生成器的类型（整数或浮点数）。       | `id` - <font color="#92cddc">string</font>：生成器 ID。                                                                                                                    | `type` - <font color="#92cddc">"int"</font> \| <font color="#92cddc">"float"</font>：类型标识。 |

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
> 2. `info` 数据格式中 `key` 数据类型含义为填写语言键。
> 3. `info` 数据格式中 `image` 数据类型含义为相对于assets/的图片路径。
> 4. `任意` 类型会被强制转换为 `string` 类型打印。
> 5. 详细调试输出见『附录-[调试输出目录](#调试输出目录)』。

| 函数名                           | 作用                      | 参数                                                                                                          | 返回值                                                                |
| ----------------------------- | ----------------------- | ----------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| `debug_log(message)`          | 在日志文件中写入一条调试信息。         | `message` - <font color="#92cddc">任意</font>：要写入的信息。                                                         | <font color="#7f7f7f">无</font>                                     |
| `debug_warn(message)`         | 在日志文件中写入一条警告信息。         | `message` - <font color="#92cddc">任意</font>：要写入的警告信息。                                                       | <font color="#7f7f7f">无</font>                                     |
| `debug_error(message)`        | 在日志文件中写入一条异常信息。         | `message` - <font color="#92cddc">任意</font>：要写入的异常信息。                                                       | <font color="#7f7f7f">无</font>                                     |
| `debug_print(title, message)` | 在日志文件中写入一条带自定义标题的调试信息。  | `title` - <font color="#92cddc">string</font>：日志标题。 <br>`message` - <font color="#92cddc">任意</font>：要写入的信息。 | <font color="#7f7f7f">无</font>                                     |
| `clear_debug_log()`           | 清空游戏日志文件。               | <font color="#7f7f7f">无</font>                                                                              | <font color="#7f7f7f">无</font>                                     |
| `get_game_uid()`              | 获取当前模组包在宿主中的唯一标识符（UID）。 | <font color="#7f7f7f">无</font>                                                                              | `uid` - <font color="#92cddc">string</font>：模组包 UID。               |
| `get_game_info()`             | 获取当前模组包的完整元信息。          | <font color="#7f7f7f">无</font>                                                                              | `info` - <font color="#92cddc">table</font>：模组包元信息表，结构见下文。         |
| `get_key([action)`            | 获取按键动作注册表信息。            | `[action` - <font color="#92cddc">string</font>：可选，动作，不填写时返回所有动作信息。                                         | `action_value` - <font color="#92cddc">table</font>：动作的按键信息，结构见下文。 |

### 日志输出格式

- `debug_log(message)` 输出格式：`[日志] message`
- `debug_warn(message)` 输出格式：`[警告] message`
- `debug_error(message)` 输出格式：`[异常] message`
- `debug_print(title, message)` 输出格式：`[title] message`

### `info` 数据格式

```lua
{
  uid = string,                    -- 游戏在宿主的唯一标识 ID
  package = string,                -- 包名
  mod_name = string | key,         -- 模组包显示名称
  introduction = string | key,     -- 包简介
  author = string | key,           -- 作者
  game_name = string | key,        -- 游戏显示名称
  description = string | key,      -- 游戏简短描述
  detail = string | key,           -- 游戏详细描述
  icon = Array | string | image,   -- 图标
  banner = Array | string | image, -- 横幅
  api = Array | int,               -- 支持的API版本
  entry = path,                    -- 入口脚本路径
  save = boolean,                  -- 是否支持保存
  best_none = string | key | null, -- 最佳记录字段配置
  min_width = int,                 -- 最小宽度（终端字符列数）
  min_height = int,                -- 最小高度（终端字符行数）
  write = boolean,                 -- 是否允许写入文件
  case_sensitive = boolean,        -- 按键是否区分大小写
  actions = table,                 -- 按键动作注册表
  runtime = {
    target_fps = int               -- 目标帧率
  }
}
```

### `action_value` 数据格式

```lua
{
  action = {                  -- 动作
    key = Array | string,     -- 原始物理按键
    key_name = string,        -- 动作含义
    key_user = Array | string -- 用户自定义物理按键
  },
  ...
}
```

- 若无对应的按键语义键，`action_value` 为空表 `{}`。

---

# 附录

## 特定参数

### 锚点 `anchor`

> 注：
>
> 1. 以下常量的值与含义对应。
> 2. 值以变量的形式传递。

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

### 对齐 `align`

> 注：
>
> 1. 以下常量的值与含义对应。
> 2. 值以变量的形式传递。

| 常量           | 值  | 作用               |
| -------------- | --- | ------------------ |
| `nil`          | `0` | 不换行             |
| `ALIGN_LEFT`   | `1` | 相对第一行左对齐   |
| `ALIGN_CENTER` | `2` | 相对第一行居中对齐 |
| `ALIGN__RIGHT` | `3` | 相对第一行右对齐   |

---

### 颜色 `color`

#### 预定义颜色名称

> 注：
>
> 1. 以下颜色值为逻辑名称，实际显示效果取决于终端的颜色映射。
> 2. 值以字符串的形式传递。

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

> 注：值以字符串的形式传递。

| 格式           | 示例                | 注意事项                                          |
| ------------ | ----------------- | --------------------------------------------- |
| `rgb(r,g,b)` | `rgb(255,128,64)` | 标准 RGB 颜色，括号内为 0–255 的整数值。**请勿在字母与括号之间添加空格**。 |
| `#rrggbb`    | `#ff8040`         | 十六进制颜色表示（6 位）。**不支持 `#rgb` 缩写格式**。            |

---

### 文本样式 `style`

> 注：
>
> 1. 以下常量的值与含义对应。
> 2. 值以变量的形式传递。

| 常量           | 值  | 样式               |
| -------------- | --- | ------------------ |
| `BOLD`          | `0` | 加粗             |
| `ITALIC`   | `1` | 斜体   |
| `UNDERLINE` | `2` | 下划线 |
| `STRIKE` | `3` | 删除线   |
| `BLINK` | `4` | 闪烁   |
| `REVERSE` | `5` | 反转   |
| `HIDDEN` | `6` | 隐藏   |
| `DIM` | `7` | 暗淡   |

---

## 调试输出目录

> 注：以下日志文件均位于宿主运行目录下的 `./tui-game-data/log/` 中。

### 游戏日志 `[uid].txt`

> 类型：<font color="purple">脚本警告</font>

**包含内容：**

- 调试信息 API 的输出。
- 部分 API 的警告信息。

> 注：该日志仅在开启 Debug 模式时输出。
> 
### 官方日志 `tui_log.txt`

> 类型：<font color="red">宿主异常</font>，<font color="orange">宿主警告</font>

**包含内容：**

- 所有启动、运行异常。
- 直写操作请求（无论是否执行，均记录）。
- 宿主自身的异常信息。

---

## 表转换格式

### `json`

任意普通对象 / 数组 / 嵌套表

```lua
{
  name = "demo",
  score = 100,
  items = { "a", "b" }
}
```

### `yaml`

任意普通对象 / 数组 / 嵌套表

```lua
{
  name = "demo",
  score = 100
}
```

### `toml`

以对象为主的嵌套表

```lua
{
  app = {
    name = "demo",
    version = "1.0"
  },
  window = {
    width = 80,
    height = 24
  }
}
```

### `csv`

二维数组

```lua
{
  { "name", "score" },
  { "alice", 100 },
  { "bob", 80 }
}
```

### `xml`

对象 + 数组的层级表

```lua
{
  player = {
    name = "hero",
    hp = 100
  },
  items = { "potion", "sword" }
}
```

---

# 快速查询

## 声明式 API

| 重写要求                              | 函数                           | 作用     | 参数                                                                                           | 传递值                                          | 主条目定位               |
| --------------------------------- | ---------------------------- | ------ | -------------------------------------------------------------------------------------------- | -------------------------------------------- | ------------------- |
| <font color="red">必须重写</font>     | `init_game(state)`           | 初始化游戏  | `state` - <font color="#92cddc">table</font> \| <font color="#92cddc">nil</font>             | `state` - <font color="#92cddc">table</font> | [声明式 API](#声明式-api) |
| <font color="red">必须重写</font>     | `handle_event(state, event)` | 处理事件   | `state` - <font color="#92cddc">table</font><br>`event` - <font color="#92cddc">table</font> | `state` - <font color="#92cddc">table</font> | [声明式 API](#声明式-api) |
| <font color="red">必须重写</font>     | `render(state)`              | 绘制画面   | `state` - <font color="#92cddc">table</font>                                                 | <font color="#7f7f7f">无</font>               | [声明式 API](#声明式-api) |
| <font color="red">必须重写</font>     | `exit_game(state)`           | 退出前处理  | `state` - <font color="#92cddc">table</font>                                                 | `state` - <font color="#92cddc">table</font> | [声明式 API](#声明式-api) |
| <font color="#fac08f">条件重写</font> | `save_best_score(state)`     | 传递最佳记录 | `state` - <font color="#92cddc">table</font>                                                 | `best` - <font color="#92cddc">table</font>  | [声明式 API](#声明式-api) |
| <font color="#fac08f">条件重写</font> | `save_game(state)`           | 保存存档   | `state` - <font color="#92cddc">table</font>                                                 | `state` - <font color="#92cddc">table</font> | [声明式 API](#声明式-api) |

## 直用式 API

| 风险等级                             | 函数                                                                        | 作用                  | 参数                                                                                                                                                                                                                                                                                                                               | 返回值                                                                                       | 主条目定位             |
| -------------------------------- | ------------------------------------------------------------------------- | ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ----------------- |
| <font color="#7f7f7f">无风险</font> | `get_launch_mode()`                                                       | 获取启动模式              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `status` - <font color="#92cddc">"new"</font> \| <font color="#92cddc">"continue"</font>  | [系统请求](#系统请求)     |
| <font color="#7f7f7f">无风险</font> | `get_best_score()`                                                        | 读取最佳记录              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `data` - <font color="#92cddc">table</font> \| <font color="#92cddc">nil</font>           | [系统请求](#系统请求)     |
| <font color="#7f7f7f">无风险</font> | `request_exit()`                                                          | 请求退出                | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | <font color="#7f7f7f">无</font>                                                            | [系统请求](#系统请求)     |
| <font color="#7f7f7f">无风险</font> | `request_skip_event_queue()`                                              | 跳过事件队列              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | <font color="#7f7f7f">无</font>                                                            | [系统请求](#系统请求)     |
| <font color="#7f7f7f">无风险</font> | `request_clear_event_queue()`                                             | 清空事件队列              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | <font color="#7f7f7f">无</font>                                                            | [系统请求](#系统请求)     |
| <font color="#7f7f7f">无风险</font> | `request_render()`                                                        | 请求重绘                | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | <font color="#7f7f7f">无</font>                                                            | [系统请求](#系统请求)     |
| <font color="#7f7f7f">无风险</font> | `request_save_best_score()`                                               | 保存最佳记录              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | <font color="#7f7f7f">无</font>                                                            | [系统请求](#系统请求)     |
| <font color="#7f7f7f">无风险</font> | `request_save_game()`                                                     | 保存游戏存档              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | <font color="#7f7f7f">无</font>                                                            | [系统请求](#系统请求)     |
| <font color="#7f7f7f">无风险</font> | `canvas_clear()`                                                          | 清空画布                | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | <font color="#7f7f7f">无</font>                                                            | [内容绘制](#内容绘制)     |
| <font color="#7f7f7f">无风险</font> | `canvas_eraser(x, y, width, height)`                                      | 清空画布区域              | `x` - <font color="#92cddc">int</font><br>`y` - <font color="#92cddc">int</font><br>`width` - <font color="#92cddc">int</font><br>`height` - <font color="#92cddc">int</font>                                                                                                                                                    | <font color="#7f7f7f">无</font>                                                            | [内容绘制](#内容绘制)     |
| <font color="#7f7f7f">无风险</font> | `canvas_draw_text(x, y, text, *[fg, *[bg, *[align)`                       | 绘制文本                | `x` - <font color="#92cddc">int</font><br>`y` - <font color="#92cddc">int</font><br>`text` - <font color="#92cddc">string</font><br>`*[fg` - <font color="#92cddc">color</font><br>`*[bg` - <font color="#92cddc">color</font><br>`*[align` - <font color="#92cddc">align</font>                                                 | <font color="#7f7f7f">无</font>                                                            | [内容绘制](#内容绘制)     |
| <font color="#7f7f7f">无风险</font> | `canvas_fill_rect(x, y, width, height, [char, *[fg, *[bg)`                | 填充矩形                | `x` - <font color="#92cddc">int</font><br>`y` - <font color="#92cddc">int</font><br>`width` - <font color="#92cddc">int</font><br>`height` - <font color="#92cddc">int</font><br>`[char` - <font color="#92cddc">string</font><br>`*[fg` - <font color="#92cddc">color</font><br>`*[bg` - <font color="#92cddc">color</font>     | <font color="#7f7f7f">无</font>                                                            | [内容绘制](#内容绘制)     |
| <font color="#7f7f7f">无风险</font> | `canvas_border_rect(x, y, width, height, [char_list, *[fg, *[bg)`         | 绘制矩形边框              | `x` - <font color="#92cddc">int</font><br>`y` - <font color="#92cddc">int</font><br>`width` - <font color="#92cddc">int</font><br>`height` - <font color="#92cddc">int</font><br>`[char_list` - <font color="#92cddc">table</font><br>`*[fg` - <font color="#92cddc">color</font><br>`*[bg` - <font color="#92cddc">color</font> | <font color="#7f7f7f">无</font>                                                            | [内容绘制](#内容绘制)     |
| <font color="#7f7f7f">无风险</font> | `get_text_size(text)`                                                     | 测量文本尺寸              | `text` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `width` - <font color="#92cddc">int</font><br>`height` - <font color="#92cddc">int</font> | [内容尺寸计算](#内容尺寸计算) |
| <font color="#7f7f7f">无风险</font> | `get_text_width(text)`                                                    | 测量文本宽度              | `text` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `width` - <font color="#92cddc">int</font>                                                | [内容尺寸计算](#内容尺寸计算) |
| <font color="#7f7f7f">无风险</font> | `get_text_height(text)`                                                   | 测量文本高度              | `text` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `height` - <font color="#92cddc">int</font>                                               | [内容尺寸计算](#内容尺寸计算) |
| <font color="#7f7f7f">无风险</font> | `get_terminal_size()`                                                     | 获取终端尺寸              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `width` - <font color="#92cddc">int</font><br>`height` - <font color="#92cddc">int</font> | [内容尺寸计算](#内容尺寸计算) |
| <font color="#7f7f7f">无风险</font> | `resolve_x(*x_anchor, width, [offset_x)`                                  | 计算 X 坐标             | `*x_anchor` - <font color="#92cddc">x_anchor</font><br>`width` - <font color="#92cddc">int</font><br>`[offset_x` - <font color="#92cddc">int</font>                                                                                                                                                                              | `x` - <font color="#92cddc">int</font>                                                    | [布局定位计算](#布局定位计算) |
| <font color="#7f7f7f">无风险</font> | `resolve_y(*y_anchor, height, [offset_y)`                                 | 计算 Y 坐标             | `*y_anchor` - <font color="#92cddc">y_anchor</font><br>`height` - <font color="#92cddc">int</font><br>`[offset_y` - <font color="#92cddc">int</font>                                                                                                                                                                             | `y` - <font color="#92cddc">int</font>                                                    | [布局定位计算](#布局定位计算) |
| <font color="#7f7f7f">无风险</font> | `resolve_rect(*x_anchor, *y_anchor, width, height, [offset_x, [offset_y)` | 计算矩形位置              | `*x_anchor` - <font color="#92cddc">x_anchor</font><br>`*y_anchor` - <font color="#92cddc">y_anchor</font><br>`width` - <font color="#92cddc">int</font><br>`height` - <font color="#92cddc">int</font><br>`[offset_x` - <font color="#92cddc">int</font><br>`[offset_y` - <font color="#92cddc">int</font>                      | `x` - <font color="#92cddc">int</font><br>`y` - <font color="#92cddc">int</font>          | [布局定位计算](#布局定位计算) |
| <font color="#7f7f7f">无风险</font> | `translate(key)`                                                          | 解析文本键               | `key` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                      | `value` - <font color="#92cddc">string</font>                                             | [数据读取](#数据读取)     |
| <font color="#7f7f7f">无风险</font> | `read_text(path)`                                                         | 读取文本文件              | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">string</font>                                              | [数据读取](#数据读取)     |
| <font color="#7f7f7f">无风险</font> | `read_json(path)`                                                         | 读取 JSON 文件，并解析      | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">table</font>                                               | [数据读取](#数据读取)     |
| <font color="#7f7f7f">无风险</font> | `read_xml(path)`                                                          | 读取 XML 文件，并解析       | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">table</font>                                               | [数据读取](#数据读取)     |
| <font color="#7f7f7f">无风险</font> | `read_yaml(path)`                                                         | 读取 YAML 文件，并解析      | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">table</font>                                               | [数据读取](#数据读取)     |
| <font color="#7f7f7f">无风险</font> | `read_toml(path)`                                                         | 读取 TOML 文件，并解析      | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">table</font>                                               | [数据读取](#数据读取)     |
| <font color="#7f7f7f">无风险</font> | `read_csv(path)`                                                          | 读取 CSV 文件，并解析       | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">table</font>                                               | [数据读取](#数据读取)     |
| <font color="#7f7f7f">无风险</font> | `read_json_string(path)`                                                  | 读取 JSON 文件，不解析      | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">string</font>                                              | [数据读取](#数据读取)     |
| <font color="#7f7f7f">无风险</font> | `read_xml_string(path)`                                                   | 读取 XML 文件，不解析       | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">string</font>                                              | [数据读取](#数据读取)     |
| <font color="#7f7f7f">无风险</font> | `read_yaml_string(path)`                                                  | 读取 YAML 文件，不解析      | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">string</font>                                              | [数据读取](#数据读取)     |
| <font color="#7f7f7f">无风险</font> | `read_toml_string(path)`                                                  | 读取 TOML 文件，不解析      | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">string</font>                                              | [数据读取](#数据读取)     |
| <font color="#7f7f7f">无风险</font> | `read_csv_string(path)`                                                   | 读取 CSV 文件，不解析       | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `data` - <font color="#92cddc">string</font>                                              | [数据读取](#数据读取)     |
| <font color="red">高风险</font>     | `write_text(path, content)`                                               | 写入文本文件              | `path` - <font color="#92cddc">string</font><br>`content` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                  | `bool` - <font color="#92cddc">boolean</font>                                             | [数据写入](#数据写入)     |
| <font color="red">高风险</font>     | `write_json(path, content)`                                               | 写入 JSON 文件          | `path` - <font color="#92cddc">string</font><br>`content` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                  | `bool` - <font color="#92cddc">boolean</font>                                             | [数据写入](#数据写入)     |
| <font color="red">高风险</font>     | `write_xml(path, content)`                                                | 写入 XML 文件           | `path` - <font color="#92cddc">string</font><br>`content` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                  | `bool` - <font color="#92cddc">boolean</font>                                             | [数据写入](#数据写入)     |
| <font color="red">高风险</font>     | `write_yaml(path, content)`                                               | 写入 YAML 文件          | `path` - <font color="#92cddc">string</font><br>`content` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                  | `bool` - <font color="#92cddc">boolean</font>                                             | [数据写入](#数据写入)     |
| <font color="red">高风险</font>     | `write_toml(path, content)`                                               | 写入 TOML 文件          | `path` - <font color="#92cddc">string</font><br>`content` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                  | `bool` - <font color="#92cddc">boolean</font>                                             | [数据写入](#数据写入)     |
| <font color="red">高风险</font>     | `write_csv(path, content)`                                                | 写入 CSV 文件           | `path` - <font color="#92cddc">string</font><br>`content` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                  | `bool` - <font color="#92cddc">boolean</font>                                             | [数据写入](#数据写入)     |
| <font color="#7f7f7f">无风险</font> | `table_to_json(table)`                                                    | 表转 JSON             | `table` - <font color="#92cddc">table</font>                                                                                                                                                                                                                                                                                     | `json_str` - <font color="#92cddc">string</font>                                          | [表处理工具](#表处理工具)   |
| <font color="#7f7f7f">无风险</font> | `table_to_yaml(table)`                                                    | 表转 YAML             | `table` - <font color="#92cddc">table</font>                                                                                                                                                                                                                                                                                     | `yaml_str` - <font color="#92cddc">string</font>                                          | [表处理工具](#表处理工具)   |
| <font color="#7f7f7f">无风险</font> | `table_to_toml(table)`                                                    | 表转 TOML             | `table` - <font color="#92cddc">table</font>                                                                                                                                                                                                                                                                                     | `toml_str` - <font color="#92cddc">string</font>                                          | [表处理工具](#表处理工具)   |
| <font color="#7f7f7f">无风险</font> | `table_to_csv(table)`                                                     | 表转 CSV              | `table` - <font color="#92cddc">table</font>                                                                                                                                                                                                                                                                                     | `csv_str` - <font color="#92cddc">string</font>                                           | [表处理工具](#表处理工具)   |
| <font color="#7f7f7f">无风险</font> | `table_to_xml(table)`                                                     | 表转 XML              | `table` - <font color="#92cddc">table</font>                                                                                                                                                                                                                                                                                     | `xml_str` - <font color="#92cddc">string</font>                                           | [表处理工具](#表处理工具)   |
| <font color="#7f7f7f">无风险</font> | `deep_copy(table)`                                                        | 深拷贝表                | `table` - <font color="#92cddc">table</font>                                                                                                                                                                                                                                                                                     | `new_table` - <font color="#92cddc">table</font>                                          | [表处理工具](#表处理工具)   |
| <font color="#7f7f7f">无风险</font> | `load_function(path)`                                                     | 加载辅助脚本              | `path` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                     | `functions` - <font color="#92cddc">table</font>                                          | [辅助脚本加载](#辅助脚本加载) |
| <font color="#7f7f7f">无风险</font> | `running_time()`                                                          | 获取运行时长              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `time` - <font color="#92cddc">int</font>                                                 | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `timer_create(delay_ms, [note)`                                           | 创建计时器               | `delay_ms` - <font color="#92cddc">int</font><br>`[note` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                   | `id` - <font color="#92cddc">string</font>                                                | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `timer_start(id)`                                                         | 启动计时器               | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                            | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `timer_pause(id)`                                                         | 暂停计时器               | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                            | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `timer_resume(id)`                                                        | 恢复计时器               | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                            | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `timer_reset(id)`                                                         | 重置计时器               | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                            | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `timer_restart(id)`                                                       | 重启计时器               | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                            | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `timer_kill(id)`                                                          | 删除计时器               | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                            | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `set_timer_note(id, note)`                                                | 设置计时器备注             | `id` - <font color="#92cddc">string</font><br>`note` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                            | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `get_timer_list()`                                                        | 获取所有计时器             | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `timers` - <font color="#92cddc">table</font>                                             | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `get_timer_info(id)`                                                      | 获取计时器信息             | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | `timer` - <font color="#92cddc">table</font>                                              | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `get_timer_status(id)`                                                    | 获取计时器状态             | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | `status` - <font color="#92cddc">string</font>                                            | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `get_timer_elapsed(id)`                                                   | 获取已过时间              | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | `time` - <font color="#92cddc">int</font>                                                 | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `get_timer_remaining(id)`                                                 | 获取剩余时间              | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | `time` - <font color="#92cddc">int</font>                                                 | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `get_timer_duration(id)`                                                  | 获取总时长               | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | `time` - <font color="#92cddc">int</font>                                                 | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `is_timer_completed(id)`                                                  | 检查是否结束              | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | `bool` - <font color="#92cddc">boolean</font>                                             | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `is_timer_exists(id)`                                                     | 检查计时器存在             | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | `bool` - <font color="#92cddc">boolean</font>                                             | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `now()`                                                                   | 获取当前时间戳             | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `timestamp` - <font color="#92cddc">int</font>                                            | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `get_current_year()`                                                      | 获取当前年份              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `year` - <font color="#92cddc">int</font>                                                 | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `get_current_month()`                                                     | 获取当前月份              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `month` - <font color="#92cddc">int</font>                                                | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `get_current_day()`                                                       | 获取当前日期              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `day` - <font color="#92cddc">int</font>                                                  | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `get_current_hour()`                                                      | 获取当前小时              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `hour` - <font color="#92cddc">int</font>                                                 | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `get_current_minute()`                                                    | 获取当前分钟              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `minute` - <font color="#92cddc">int</font>                                               | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `get_current_second()`                                                    | 获取当前秒数              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `second` - <font color="#92cddc">int</font>                                               | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `timestamp_to_date(timestamp, [format)`                                   | 时间戳转日期              | `timestamp` - <font color="#92cddc">int</font><br>`[format` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                | `date_str` - <font color="#92cddc">string</font>                                          | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `date_to_timestamp([year, [month, [day, [hour, [minute, [second)`         | 日期转时间戳              | `[year` - <font color="#92cddc">int</font><br>`[month` - <font color="#92cddc">int</font><br>`[day` - <font color="#92cddc">int</font><br>`[hour` - <font color="#92cddc">int</font><br>`[minute` - <font color="#92cddc">int</font><br>`[second` - <font color="#92cddc">int</font>                                             | `timestamp` - <font color="#92cddc">int</font>                                            | [时间处理](#时间处理)     |
| <font color="#7f7f7f">无风险</font> | `random([id)`                                                             | 随机整数 $[0,2^{31}-1]$ | `[id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                      | `number` - <font color="#92cddc">int</font>                                               | [随机数](#随机数)       |
| <font color="#7f7f7f">无风险</font> | `random(max, [id)`                                                        | 随机整数 $[0,max]$      | `max` - <font color="#92cddc">int</font><br>`[id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                          | `number` - <font color="#92cddc">int</font>                                               | [随机数](#随机数)       |
| <font color="#7f7f7f">无风险</font> | `random(min, max, [id)`                                                   | 随机整数 $[min,max]$    | `min` - <font color="#92cddc">int</font><br>`max` - <font color="#92cddc">int</font><br>`[id` - <font color="#92cddc">string</font>                                                                                                                                                                                              | `number` - <font color="#92cddc">int</font>                                               | [随机数](#随机数)       |
| <font color="#7f7f7f">无风险</font> | `random_float([id)`                                                       | 随机浮点数 $[0,1)$       | `[id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                      | `number` - <font color="#92cddc">double</font>                                            | [随机数](#随机数)       |
| <font color="#7f7f7f">无风险</font> | `random_create(seed, [note)`                                              | 创建整数生成器             | `seed` - <font color="#92cddc">string</font><br>`[note` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                    | `id` - <font color="#92cddc">string</font>                                                | [随机数](#随机数)       |
| <font color="#7f7f7f">无风险</font> | `random_float_create(seed, [note)`                                        | 创建浮点生成器             | `seed` - <font color="#92cddc">string</font><br>`[note` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                    | `id` - <font color="#92cddc">string</font>                                                | [随机数](#随机数)       |
| <font color="#7f7f7f">无风险</font> | `random_reset_step(id)`                                                   | 重置步进数               | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                            | [随机数](#随机数)       |
| <font color="#7f7f7f">无风险</font> | `random_kill(id)`                                                         | 删除生成器               | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                            | [随机数](#随机数)       |
| <font color="#7f7f7f">无风险</font> | `set_random_note(id, note)`                                               | 设置生成器备注             | `id` - <font color="#92cddc">string</font><br>`note` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                       | <font color="#7f7f7f">无</font>                                                            | [随机数](#随机数)       |
| <font color="#7f7f7f">无风险</font> | `get_random_list()`                                                       | 获取所有生成器             | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `randoms` - <font color="#92cddc">table</font>                                            | [随机数](#随机数)       |
| <font color="#7f7f7f">无风险</font> | `get_random_info(id)`                                                     | 获取生成器信息             | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | `random` - <font color="#92cddc">table</font>                                             | [随机数](#随机数)       |
| <font color="#7f7f7f">无风险</font> | `get_random_step(id)`                                                     | 获取步进数               | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | `step` - <font color="#92cddc">int</font>                                                 | [随机数](#随机数)       |
| <font color="#7f7f7f">无风险</font> | `get_random_seed(id)`                                                     | 获取种子                | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | `seed` - <font color="#92cddc">string</font>                                              | [随机数](#随机数)       |
| <font color="#7f7f7f">无风险</font> | `get_random_type(id)`                                                     | 获取生成器类型             | `id` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                       | `type` - <font color="#92cddc">"int"</font> \| <font color="#92cddc">"float"</font>       | [随机数](#随机数)       |
| <font color="#7f7f7f">无风险</font> | `debug_log(message)`                                                      | 调试日志                | `message` - <font color="#92cddc">任意</font>                                                                                                                                                                                                                                                                                      | <font color="#7f7f7f">无</font>                                                            | [调试信息](#调试信息)     |
| <font color="#7f7f7f">无风险</font> | `debug_warn(message)`                                                     | 警告日志                | `message` - <font color="#92cddc">任意</font>                                                                                                                                                                                                                                                                                      | <font color="#7f7f7f">无</font>                                                            | [调试信息](#调试信息)     |
| <font color="#7f7f7f">无风险</font> | `debug_error(message)`                                                    | 异常日志                | `message` - <font color="#92cddc">任意</font>                                                                                                                                                                                                                                                                                      | <font color="#7f7f7f">无</font>                                                            | [调试信息](#调试信息)     |
| <font color="#7f7f7f">无风险</font> | `debug_print(title, message)`                                             | 自定义日志               | `title` - <font color="#92cddc">string</font><br>`message` - <font color="#92cddc">任意</font>                                                                                                                                                                                                                                     | <font color="#7f7f7f">无</font>                                                            | [调试信息](#调试信息)     |
| <font color="#7f7f7f">无风险</font> | `clear_debug_log()`                                                       | 清空游戏日志              | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | <font color="#7f7f7f">无</font>                                                            | [调试信息](#调试信息)     |
| <font color="#7f7f7f">无风险</font> | `get_game_uid()`                                                          | 获取模组包 UID            | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `uid` - <font color="#92cddc">string</font>                                               | [调试信息](#调试信息)     |
| <font color="#7f7f7f">无风险</font> | `get_game_info()`                                                         | 获取模组包元信息             | <font color="#7f7f7f">无</font>                                                                                                                                                                                                                                                                                                   | `info` - <font color="#92cddc">table</font>                                               | [调试信息](#调试信息)     |
| <font color="#7f7f7f">无风险</font> | `get_key([key)`                                                         | 获取自定义按键信息             | `[key` - <font color="#92cddc">string</font>                                                                                                                                                                                                                                                                                                   | `key_value` - <font color="#92cddc">table</font>                                               | [调试信息](#调试信息)     |

---

# 异常和警告速查表

> 注：
>
> 1. 异常会输出日志，并终止宿主和脚本的运行
> 2. 警告仅输出日志，不会终止宿主和脚本的运行

## 异常

### 宿主内部异常

| 适用函数                           | 触发条件                 | 抛出句式                                                                   | 类型                            |
| ------------------------------ | -------------------- | ---------------------------------------------------------------------- | ----------------------------- |
| <font color="#7f7f7f">无</font> | 宿主内部出现错误             | 程序崩溃：{panic_info}                                                      | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | 异常原始错误               | {err}                                                                  | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | 宿主全局按键监听无法使用           | 全局按键监听失效：{err}                                                         | <font color="red">宿主异常</font> |

### 宿主加载模组包异常

| 适用函数                           | 触发条件                   | 抛出句式                                                                   | 类型                            |
| ------------------------------ | ---------------------- | ---------------------------------------------------------------------- | ----------------------------- |
| <font color="#7f7f7f">无</font> | 宿主无法扫描模组包              | 无法扫描模组包：{err}                                                          | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | 模组包目录名无效               | 模组包目录名“{mod_namespace}”不可用                                             | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | `package.json` 格式无效    | package.json 文件 JSON 格式无效：{path}                                       | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | 宿主读取 `package.json` 失败 | 读取 package.json 文件失败：{path}                                            | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | `game.json` 格式无效       | game.json 文件 JSON 格式无效：{path}                                          | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | 宿主读取 `game.json` 失败    | 读取 game.json 文件失败：{path}                                               | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | 字段为空                   | {file} 文件 {key} 字段不能为空                                                 | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | 字段类型错误                 | {file} 文件 {key} 字段类型错误，应为 {type}，实际为 {actual_type}                     | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | 脚本 API 版本不匹配宿主要求       | “{mod_namespce}”API 版本不符合宿主要求，应为 {api_version}，实际 {actual_api_version} | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | 按键动作注册表出现重复注册键         | 按键动作注册表出现重复注册键，冲突键: {keys}                                             | <font color="red">宿主异常</font> |

### 宿主运行游戏异常

| 适用函数                           | 触发条件                 | 抛出句式                                                                   | 类型                            |
| ------------------------------ | -------------------- | ---------------------------------------------------------------------- |---|
| <font color="#7f7f7f">无</font> | 宿主无法运行游戏               | 无法运行游戏“{game_id}”：{err}                                                | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | 宿主无法继续游戏               | 无法继续游戏“{game_id}”：{err}                                                | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | 宿主清理旧存档失败              | 清理旧存档失败：{err}                                                          | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | 宿主无法读取脚本               | 无法读取脚本：{path}                                                          | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | 宿主无法运行脚本               | 无法运行脚本：{path}                                                          | <font color="red">宿主异常</font> |
| <font color="#7f7f7f">无</font> | 宿主未找到入口脚本              | 未找到入口脚本：{path}                                                         | <font color="red">宿主异常</font> |

### 宿主安全异常

| 适用函数                           | 触发条件                 | 抛出句式                                                                   | 类型                            |
| ------------------------------ | -------------------- | ---------------------------------------------------------------------- |---|
| <font color="#7f7f7f">无</font> | 脚本调用被宿主沙箱禁用的 API         | 模组包中调用被沙箱禁用的 Lua 内置 API，已被拦截：{err}                                      | <font color="red">宿主异常</font> |

### 通用异常

| 适用函数                           | 触发条件                 | 抛出句式                                                                   | 类型                            |
| ------------------------------ | -------------------- | ---------------------------------------------------------------------- |---|
|<font color="green">所有 API</font>|参数数量错误|API 参数数量不匹配：期望接收 {expected} 个参数，实际收到 {actual} 个 | <font color="red">宿主异常</font> |
|<font color="green">所有 API</font>|参数类型错误|API 参数 {arg_name} 类型错误：期望类型为 {expected_type}，实际类型为 {actual_type} | <font color="red">宿主异常</font> |


### 声明式 API 异常

| 适用函数                           | 触发条件                 | 抛出句式                                                                   | 类型                            |
| ------------------------------ | -------------------- | ---------------------------------------------------------------------- |---|
| `init_game`<br>`handle_event`<br>`exit_game`<br>`render` | 入口脚本缺少必须实现的声明式 API   | 入口脚本缺少必须实现的声明式 API：{err}                                               | <font color="red">宿主异常</font> |
| `init_game`<br>`handle_event`<br>`exit_game`<br>`save_best_score`<br>`save_game` | 声明式 API 未按要求传递需要的值   | 声明式 API 未按要求传递需要的值。                                                    | <font color="red">宿主异常</font> |
| `save_best_score` | 条件满足但未实现对应的声明式 API | best_none 字段不为 null，但未实现 save_best_score | <font color="red">宿主异常</font> |
| `save_game` | 条件满足但未实现对应的声明式 API | save 字段为 true，但未实现 save_game | <font color="red">宿主异常</font> |

### 直用式 API 异常

#### 系统请求

| 适用函数                           | 触发条件                 | 抛出句式                                                                   | 类型                            |
| ------------------------------ | -------------------- | ---------------------------------------------------------------------- |---|
|`get_launch_mode`|宿主无法获取当前游戏启动模式|无法获取本次游戏启动模式：{err}|<font color="red">宿主异常</font>|
|`get_best_score`|宿主无法获取当前游戏最佳记录数据|无法获取游戏存储的最佳记录数据：{err}|<font color="red">宿主异常</font>|
|`request_exit`|宿主无法处理退出请求|退出请求无效：{err}|<font color="red">宿主异常</font>|
| `request_skip_event_queue` | 宿主清空事件队列失败             | 清空事件队列失败                                                               | <font color="red">宿主异常</font> |
| `request_clear_event_queue` | 宿主跳过事件队列失败             | 跳过事件队列失败                                                               | <font color="red">宿主异常</font> |
|`request_render`|宿主无法处理重绘请求|重绘请求无效：{err}|<font color="red">宿主异常</font>|
|`request_save_best_score`|宿主无法处理最佳纪录保存请求|最佳记录保存请求无效：{err}|<font color="red">宿主异常</font>|
|`request_save_game`|宿主无法处理游戏存档保存请求|游戏存档保存请求无效：{err}|<font color="red">宿主异常</font>|

#### 内容绘制

| 适用函数                           | 触发条件                 | 抛出句式                                                                   | 类型                            |
| ------------------------------ | -------------------- | ---------------------------------------------------------------------- |---|
| <font color="green">内容绘制 API</font> | 宿主无法处理画布操作 | 画布上下文无效，无法执行绘制操作。 | <font color="red">宿主异常</font> |
| `canvas_eraser`<br>`canvas_draw_text`<br>`canvas_fill_rect`<br>`canvas_border_rect` | 相关参数不符合要求 | 坐标参数无效：必须为大于等于 0 的整数，实际值为 {value} | <font color="red">宿主异常</font> |
| `canvas_eraser`<br>`canvas_fill_rect`<br>`canvas_border_rect` | 相关参数不符合要求 | 宽高参数无效：必须为大于等于 0 的整数，实际宽度为 {width}，高度为 {height} | <font color="red">宿主异常</font> |

#### 内容尺寸计算

| 适用函数 | 触发条件 | 抛出句式 | 类型 |
| --- | --- | --- | --- |
| `get_text_height` | 宿主无法获取终端尺寸 | 无法获取终端尺寸，请检查终端环境是否正常 | <font color="red">宿主异常</font> |

#### 布局定位计算

| 适用函数 | 触发条件 | 抛出句式 | 类型 |
| --- | --- | --- | --- |
| `resolve_rect` | 相关参数不符合要求 | 宽高参数无效：必须为大于等于 0 的整数，实际宽度为 {width}，高度为 {height} | <font color="red">宿主异常</font> |
| `resolve_x` | 相关参数不符合要求 | 宽参数无效：必须为大于等于 0 的整数，实际宽度为 {width} | <font color="red">宿主异常</font> |
| `resolve_y` | 相关参数不符合要求 | 高参数无效：必须为大于等于 0 的整数，实际高度为 {height} | <font color="red">宿主异常</font> |
| `resolve_x`<br>`resolve_y`<br>`resolve_rect` | 相关参数不符合要求 | 锚点参数无效 | <font color="red">宿主异常</font> |

#### 数据读取

| 适用函数 | 触发条件 | 抛出句式 | 类型 |
| --- | --- | --- | --- |
| `read_*` | 路径错误 | 未找到目标文件：{path} | <font color="red">宿主异常</font> |
| `read_*` | 路径错误 | 路径格式错误：应为绝对路径，实际为 {path} | <font color="red">宿主异常</font> |
| `read_*` | 路径错误 | 路径中包含 .. 操作符，已拒绝访问 | <font color="red">宿主异常</font> |
| `read_*` | 文件格式有误 | 文件格式无效，解析失败：{path} | <font color="red">宿主异常</font> |
| `read_*` | 宿主无法读取文件 | 读取文件失败：{err} | <font color="red">宿主异常</font> |
| `translate` | 宿主无法解析语言键 | 无法获取语言键对应的内容：{key} | <font color="red">宿主异常</font> |

#### 数据写入

| 适用函数 | 触发条件 | 抛出句式 | 类型 |
| --- | --- | --- | --- |
| `write_*` | 路径错误 | 路径格式错误：应为绝对路径，实际为 {path} | <font color="red">宿主异常</font> |
| `write_*` | 路径错误 | 路径中包含 .. 操作符，已拒绝访问 | <font color="red">宿主异常</font> |
| `write_*` | 宿主无法写入文件 | 写入文件失败：{err} | <font color="red">宿主异常</font> |

#### 表处理工具

| 适用函数 | 触发条件 | 抛出句式 | 类型 |
| --- | --- | --- | --- |
| `table_*`<br>`deep_copy` | 表结构不符合要求 | 表结构已损坏，无法处理 | <font color="red">宿主异常</font> |
| `table_*` | 表结构不符合要求 | 表结构不符合转换规范，无法处理 | <font color="red">宿主异常</font> |
| `table_*` | 宿主无法处理表转换 | 表转换操作失败：{err} | <font color="red">宿主异常</font> |
| `deep_copy` | 宿主无法深拷贝表 | 深拷贝操作失败：{err} | <font color="red">宿主异常</font> |


#### 辅助脚本加载

| 适用函数 | 触发条件 | 抛出句式 | 类型 |
| --- | --- | --- | --- |
| `load_function` | 路径错误 | 未找到目标文件：{path} | <font color="red">宿主异常</font> |
| `load_function` | 路径错误 | 路径格式错误：应为绝对路径，实际为 `{path}` | <font color="red">宿主异常</font> |
| `load_function` | 路径错误 | 路径中包含 .. 操作符，已拒绝访问 | <font color="red">宿主异常</font> |
| `load_function` | 宿主无法解析辅助脚本 | 加载辅助脚本失败：{err} | <font color="red">宿主异常</font> |

#### 时间处理

| 适用函数 | 触发条件 | 抛出句式 | 类型 |
| --- | --- | --- | --- |
| <font color="green">时间处理 API</font> | 使用的 ID 不存在 | 指定 ID 的计时器不存在：{id} | <font color="red">宿主异常</font> |
| `running_time` | 宿主无法获取游戏运行时长 | 无法获取当前游戏运行时长：{err} | <font color="red">宿主异常</font> |
| `timer_create` | 相关参数不符合要求 | 计时时长必须为正整数 | <font color="red">宿主异常</font> |
| `timer_create` | 计时器创建达到上限 | 计时器已达上限 64 | <font color="red">宿主异常</font> |
| `timer_create` | 宿主无法创建计时器 | 创建计时器失败：{err} | <font color="red">宿主异常</font> |
| `timer_start`<br>`timer_restart` | 宿主无法启动计时器 | 启动指定 ID 计时器失败：{err} | <font color="red">宿主异常</font> |
| `timer_pause` | 宿主无法暂停计时器 | 暂停指定 ID 计时器失败：{err} | <font color="red">宿主异常</font> |
| `timer_resume` | 宿主无法恢复计时器 | 恢复指定 ID 计时器失败：{err} | <font color="red">宿主异常</font> |
| `timer_reset`<br>`timer_restart` | 宿主无法重置计时器 | 重置指定 ID 计时器失败：{err} | <font color="red">宿主异常</font> |
| `timer_kill` | 宿主无法删除计时器 | 删除指定 ID 计时器失败：{err} | <font color="red">宿主异常</font> |
| `get_timer_list` | 宿主无法获取计时器列表 | 无法获取当前计时器列表：{err} | <font color="red">宿主异常</font> |
| `get_timer_info`<br>`get_timer_statue`<br>`get_timer_elapsed`<br>`get_timer_remaining`<br>`get_timer_duration`<br>`is_timer_completed` | 宿主无法获取信息 | 无法获取指定 ID 计时器的信息：{id} | <font color="red">宿主异常</font> |
| `is_timer_exists` | 宿主无法检查计时器 | 无法检查指定 ID 的计时器是否存在：{id} | <font color="red">宿主异常</font> |
| `now` | 宿主无法获取系统时间戳 | 获取系统时间戳失败：{err} | <font color="red">宿主异常</font> |
| `get_current_*` | 宿主无法获取系统时间 | 获取系统时间失败：{err} | <font color="red">宿主异常</font> |
| `timestamp_to_date` | 相关参数不符合要求 | 时间戳必须为大于等于 0 的整数 | <font color="red">宿主异常</font> |
| `timestamp_to_date` | 相关参数不符合要求 | 日期字符串缺少必要参数 | <font color="red">宿主异常</font> |
| `date_to_timestamp` | 宿主无法转换日期 | 日期转换失败：{err} | <font color="red">宿主异常</font> |

#### 随机数

| 适用函数 | 触发条件 | 抛出句式 | 类型 |
| --- | --- | --- | --- |
| <font color="green">随机数 API</font> | 使用的 ID 不存在 | 指定 ID 随机数生成器不存在：{id} | <font color="red">宿主异常</font> |
| `random`<br>`random_float` | 宿主无法生成随机数 | 无法生成随机数：{err} | <font color="red">宿主异常</font> |
| `random`<br>`random_float` | 随机数生成器类型不符合要求 | 指定 ID 随机数生成器类型不匹配：{id} | <font color="red">宿主异常</font> |
| `random` | 相关参数不符合要求 | max参数应为正整数，实际为 {max} | <font color="red">宿主异常</font> |
| `random` | 相关参数不符合要求 | max参数应大于min，实际为min {min}，max {max} | <font color="red">宿主异常</font> |
| `random_create`<br>`random_float_create` | 宿主无法根据种子创建随机数生成器 | 种子无效：{seed} | <font color="red">宿主异常</font> |
| `random_create`<br>`random_float_create` | 宿主无法创建随机书生成器 | 创建随机数生成器失败：{err} | <font color="red">宿主异常</font> |
| `random_reset_step` | 宿主无法重置随机数生成器 | 重置随机数生成器步进数失败：{err} | <font color="red">宿主异常</font> |
| `random_kill` | 宿主无法删除随机数生成器 | 删除随机数生成器失败：{err} | <font color="red">宿主异常</font> |
| `get_random_list` | 宿主无法获取随机数生成器列表 | 无法获取随机数生成器列表：{err} | <font color="red">宿主异常</font> |
| `get_random_info`<br>`get_random_step`<br>`get_random_seed`<br>`get_random_type` | 宿主无法获取信息 | 无法获取指定 ID 随机数生成器的信息：{id} | <font color="red">宿主异常</font> |

#### 调试信息

| 适用函数 | 触发条件 | 抛出句式 | 类型 |
| --- | --- | --- | --- |
| <font color="#7f7f7f">无</font> | 日志写入失败               | 日志写入失败                                                                 | <font color="red">宿主异常</font> |

## 警告

### 宿主加载模组包警告

| 适用函数                           | 触发条件                                 | 抛出句式                                                                 | 类型                               |
| ------------------------------ | ------------------------------------ | -------------------------------------------------------------------- | -------------------------------- |
| <font color="#7f7f7f">无</font> | 目录中存在命名空间相同的模组包                      | 模组包命名空间“{mod_namespace}”全局不唯一                                        | <font color="orange">宿主警告</font> |
| <font color="#7f7f7f">无</font> | FPS 不符合宿主要求值                         | FPS 要求为 30/60/120，实际为 {actual_fps}，已退回 60                            | <font color="orange">宿主警告</font> |
| <font color="#7f7f7f">无</font> | `game.json` 中 `best_none` 字段为 `null` | {mod_uid} 的 best_none 字段为 null，相关请求将被忽略                              | <font color="orange">宿主警告</font> |
| <font color="#7f7f7f">无</font> | `game.json` 中 `save` 字段为 `false`     | {mod_uid} 的 save 字段为 false，相关请求将被忽略                                  | <font color="orange">宿主警告</font> |
| <font color="#7f7f7f">无</font> | 按键动作注册表单语义绑定键数量超过 5 个上限              | 按键动作注册表动作 {action} 绑定键数量超过 5 个上限，当前数量为 {key_count}，超出部分将被忽略，仅前 5 个生效 | <font color="orange">宿主警告</font> |
| <font color="#7f7f7f">无</font> | 按键动作注册表出现非显式绑定按键                     | 按键动作注册表动作出现非显式绑定按键 {key}，可能会导致语义映射失效                                 | <font color="orange">宿主警告</font> |

### 直用式 API 警告

#### 内容绘制

| 适用函数 | 触发条件 | 抛出句式 | 类型 |
| --- | --- | --- | --- |
| <font color="green">内容绘制 API</font> | 绘制的内容部分超出画布 | 绘制内容超出画布边界：画布尺寸为 {w} 列 × {h} 行，绘制起始点为 ({x}, {y})。 | <font color="purple">脚本警告</font> |
| `canvas_fill_rect`<br>`canvas_border_rect` | 填充字符长度不符合要求 | 填充字符长度应为 0 或 1，实际长度为 {length}（字符串内容：{string}），将截取首个字符 "{char}" 作为填充内容 | <font color="purple">脚本警告</font> |

#### 内容尺寸计算

| 适用函数 | 触发条件 | 抛出句式 | 类型 |
| --- | --- | --- | --- |
| `get_text_size`<br>`get_text_width`<br>`get_text_height` | 内容宽度为0 | 计算所得的内容宽度为 0，可能导致显示异常 | <font color="purple">脚本警告</font> |
| `get_text_size`<br>`get_text_width`<br>`get_text_height` | 内容高度为0 | 计算所得的内容高度为 0，可能导致显示异常 | <font color="purple">脚本警告</font> |

#### 数据读取

| 适用函数 | 触发条件 | 抛出句式 | 类型 |
| --- | --- | --- | --- |
| `translate` | 未匹配到宿主使用语言的语言键 | 未找到当前语言对应的语言键，已回退使用 en_us.json | <font color="purple">脚本警告</font> |
| `translate` | 未匹配到 `en_us.json` 文件对应的语言键 | 未在 en_us.json 中找到对应的语言键：[missing-i18n-key: {key}\] | <font color="purple">脚本警告</font> |

#### 数据写入

| 适用函数 | 触发条件 | 抛出句式 | 类型 |
| --- | --- | --- | --- |
| `write_*` | 脚本请求调用 `write_*` 相关直写函数               | {game_uid} 于 {timestamp} 请求调用 {api}，路径：{path}，{status} | <font color="orange">宿主警告</font> |
