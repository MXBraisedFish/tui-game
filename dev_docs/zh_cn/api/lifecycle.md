# lifecycle 库

在脚本中定义这些函数，就能在初始化、处理输入、更新画面和保存数据时执行自己的逻辑。

---

# 目录

## 方法

| 方法        | 说明                                           | 定位                        |
| ------------- | ---------------------------------------------- | --------------------------- |
| `Init`        | 初始化游戏或屏保                               | [Init](#init)               |
| `HandleEvent` | 事件处理                                       | [HandleEvent](#handleevent) |
| `Update`      | 物理帧更新，固定步长调用                       | [Update](#update)           |
| `UpdateFrame` | 帧更新，随着系统帧调用                         | [UpdateFrame](#updateframe) |
| `Render`      | 绘制当前画面                                   | [Render](#render)           |
| `SaveGame`    | 保存游戏数据，供玩家"继续游戏"后传递初始化数据 | [SaveGame](#savegame)       |
| `SaveBest`    | 保存最佳记录数据，用于游戏列表展示             | [SaveBest](#savebest)       |

---

# 方法

## `Init`

初始化游戏或屏保。

### 调用

```lua
Init
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `ctx` | table | 初始化信息 |

## 返回值

无。

### 示例

```lua
function Init(ctx)
  debug.print(serialization.json_encode(ctx))
  -- 初始化逻辑
end
```

**输出：**

```lua
```

## 额外说明

`ctx` 为初始化上下文表。

| 字段            | 类型    | 说明                 |
| --------------- | ------- | -------------------- |
| `package_id`    | string  | 模组包 ID            |
| `package_type`  | string  | 模组包类型           |
| `base`          | table   | 基础切片图层状态信息 |
| `base.width`    | integer | 基础切片图层宽度     |
| `base.height`   | integer | 基础切片图层高度     |
| `api_version`   | integer | API 版本             |
| `start_mode`    | string  | 游戏启动模式         |
| `continue_data` | any     | 继续游戏数据         |
| `best_data`     | table   | 最佳记录数据         |


- 字段 `package_type` 为 "game" 或 "screensaver"。
- 字段 `api_version` 当前为 1。
- 字段 `start_mode` 为 "new" 或 "continue"。
- 字段 `continue_data` 仅在玩家"继续游戏"时提供，其内容来自此前 `SaveGame` 保存的数据。
- 字段 `best_data` 的内容来自此前 `SaveBest` 保存的数据。

---

## `HandleEvent`

事件处理。

### 调用

```lua
HandleEvent
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `event` | table | 本次收到的事件 |

## 返回值

无。

### 示例

```lua
function HandleEvent(event)
  if event.type == "action" then
    debug.print(serialization.json_encode(event))
    -- 事件处理逻辑
  end
end
```

**输出：**

```lua
```

## 额外说明

`event` 为事件上下文表。

| 字段       | 类型    | 说明         |
| ---------- | ------- | ------------ |
| `type`     | string  | 事件类型     |
| `sequence` | integer | 事件全局序号 |
| `frame`    | integer | 系统帧序号   |
| `data`     | table   | 事件数据     |


- 字段 `sequence` 事件全局序号，部分事件会被系统全局处理，脚本收到的序号不保证连续。
- 字段 `frame` 为运行时系统帧序号，不代表该脚本自身处理事件的次数。
- 字段 `type` 和字段 `data` 的具体结构查看⌊[事件结构](../EVENT.md)⌉。

---

## `Update`

物理帧更新，固定步长调用。

### 调用

```lua
Update
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `dt` | number | 固定更新的时间间隔，单位为秒 |

## 返回值

无。

### 示例

```lua
function Update(dt)
  debug.print(serialization.json_encode(dt))
  -- 物理帧更新逻辑
end
```

**输出：**

```lua
```

## 额外说明

`dt` 为物理帧时间差，单位 `秒`。

| 类型   | 说明         |
| ------ | ------------ |
| float | 物理帧时间差 |


- 物理帧更新间隔固定为 1/60 秒。
- 系统会根据实际帧间隔来计算 `Update` 调用次数，每帧最多调用 8 次。

---

## `UpdateFrame`

帧更新，随着系统帧调用。

### 调用

```lua
UpdateFrame
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `dt` | number | 本次画面更新的时间间隔，单位为秒 |
| `alpha` | number | 用于插值的比例，范围为 0～1 |

## 返回值

无。

### 示例

```lua
function UpdateFrame(dt, alpha)
  debug.print(tostring(dt))
  debug.print(tostring(alpha))
  debug.print("")
  -- 帧更新逻辑
end
```

**输出：**

```lua
```

## 额外说明

`dt` 为画面更新时间差，单位为秒。

`alpha` 表示当前显示帧在最近两次固定 `Update` 状态之间的插值比例，范围为 `0` 到 `1`。

| 类型   | 说明     |
| ------ | -------- |
| float | 插值比例 |


- 帧更新随系统帧调用，实际调用频率受游戏帧率设置、系统帧设置和玩家设备性能影响。

---

## `Render`

绘制当前画面。

### 调用

```lua
Render
```

## 参数

无。

## 返回值

无。

### 示例

```lua
function Render()
  draw.text(0, 1, "Hello Tui Game", {fg = color.BRIGHT_BLUE, bold = true})
end
```

**输出：**

```lua
```

---

## `SaveGame`

保存游戏数据，供玩家"继续游戏"后传递初始化数据。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
SaveGame
```

## 参数

无。

## 返回值

需要保存的游戏数据。

| 类型 | 说明     |
| ---- | -------- |
| any  | 游戏数据 |

### 示例

```lua
function Init(ctx)
  debug.print(serialization.json_encode(ctx))
  -- 初始化逻辑
end

function SaveGame()
  -- 保存前的数据处理
  local coin = 10
  return coin
end
```

**输出：**

```lua
```

## 额外说明

- 仅当 `game.json` 中 `save_game` 为 `true` 时可用，且必须实现。通过 `game.save_game` 读取。
- 返回值必须可序列化，且只能返回一个值。

---

## `SaveBest`

保存最佳记录数据，用于游戏列表展示，并在后续包含在传递的初始化数据中。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
SaveBest
```

## 参数

无。

## 返回值

返回需要保存的最佳记录数据表。

| 类型  | 说明         |
| ----- | ------------ |
| table | 最佳记录数据 |

### 示例

```lua
function Init(ctx)
  debug.print(serialization.json_encode(ctx))
  -- 初始化逻辑
end

function SaveBest()
  -- 保存前的数据处理
  local coin = 10
  return {
    best_string = { type = "i18n", key = "coin", callback = "f%Coin: {value:coin}" },
    value = { coin = tostring(coin) },
    coin = coin
  }
end
```

**输出：**

```lua
```

## 额外说明

- 返回值表必须包含 `best_string`，可以是字符串，也可以是 `{ type = "text", text = "..." }` 或 `{ type = "i18n", key = "...", callback = "..." }` 文本表；i18n 的 `key` 和 `callback` 必填，`key` 不能为空，不能添加未知字段。
- 可选的 `value` 是名称到文本的对象表，每个值同样可以是字符串或文本表。例如 `value = { coin = tostring(coin) }` 对应 `f%Coin: {value:coin}`；数值需先转成字符串，不能传数组。
- 所有 i18n 文本都查找当前包的 `assets/language/<语言代码>/package/best_string.json`，依次使用当前语言、`en_us`、各文本表的 `callback`。
- 使用占位符或样式时，展示文本、翻译或回退文本需带 `f%` 前缀；替换值按原样插入，未提供的参数保留原占位符。
- 原始文本表与参数一起保存并通过 `ctx.best_data` 传回。切换语言、热加载语言文件后，游戏列表重新显示对应文本；纯字符串旧存档仍可读取。
- 仅当 `game.json` 中 `best_score.enable` 为 `true` 时可用，且必须实现。通过 `game.save_best` 读取。
- 返回值必须可序列化。
