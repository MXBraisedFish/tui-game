# Lifecycle 库

## 基本库说明

`Lifecycle` 提供脚本的生命周期回调。

---

## 目录

### 回调

| 回调名        | 说明                                           | 索引                        |
| ------------- | ---------------------------------------------- | --------------------------- |
| `Init`        | 初始化游戏或屏保                               | [Init](#Init)               |
| `HandleEvent` | 事件处理                                       | [HandleEvent](#HandleEvent) |
| `Update`      | 物理帧更新，固定步长调用                       | [Update](#Update)           |
| `UpdateFrame` | 帧更新，随着系统帧调用                         | [UpdateFrame](#UpdateFrame) |
| `Render`      | 绘制当前画面                                   | [Render](#Render)           |
| `SaveGame`    | 保存游戏数据，供玩家"继续游戏"后传递初始化数据 | [SaveGame](#SaveGame)       |
| `SaveBest`    | 保存最佳记录数据，用于游戏列表展示             | [SaveBest](#SaveBest)       |

---

## 回调

## `Init`

初始化游戏或屏保。

### 调用

```lua
function Init(ctx)
end
```

### 参数

`ctx` 为初始化上下文表。

| 字段            | 类型    | 说明                 |
| --------------- | ------- | -------------------- |
| `package_id`    | string  | 模组包 ID            |
| `package_type`  | string  | 模组包类型           |
| `base`          | table   | 基础切片图层状态信息 |
| `base.width`    | integer | 基础切片图层宽度     |
| `base.height`   | integer | 基础切片图层高度     |
| `api_version`   | integer | API 版本             |
| `start_mdoe`    | string  | 游戏启动模式         |
| `continue_data` | any     | 继续游戏数据         |
| `best_data`     | table   | 最佳记录数据         |

**额外补充**

- 字段 `package_type` 为 "game" 或 "screensaver"。
- 字段 `api_version` 当前为 1。
- 字段 `start_mode` 为 "new" 或 "continue"。
- 字段 `continue_data` 仅在玩家"继续游戏"时提供，其内容来自此前 `SaveGame` 保存的数据。
- 字段 `best_data` 的内容来自此前 `SaveBest` 保存的数据。

### 返回

无。

### 示例

```lua
function Init(ctx)
  debug.print { message = serialization.json_encode(ctx) }
  -- 初始化逻辑
end
```

输出：

> X 为占位符

```json
{
  "package_id": "test",
  "package_type": "game",
  "base": {
    "width": X,
    "height": X
  },
  "api_version": 1,
  "start_mode": "new",
  "best_data": {
    "best_string": "string",
    "score": X
  }
}
```

---

## `HandleEvent`

事件处理。

### 调用

```lua
function HandleEvent(event)
end
```

### 参数

`event` 为事件上下文表。

| 字段       | 类型    | 说明         |
| ---------- | ------- | ------------ |
| `type`     | string  | 事件类型     |
| `sequence` | integer | 事件全局序号 |
| `frame`    | integer | 系统帧序号   |
| `data`     | table   | 事件数据     |

**额外补充**

- 字段 `sequence` 事件全局序号，部分事件会被系统全局处理，脚本收到的序号不保证连续。
- 字段 `frame` 为运行时系统帧序号，不代表该脚本自身处理事件的次数。
- 字段 `type` 和字段 `data` 的具体结构请查看⌊[事件结构](../EVENT.md)⌉文档。

### 返回

无。

### 示例

```lua
function HandleEvent(event)
  if event.type == "action" then
    debug.print { message = serialization.json_encode(event) }
    -- 事件处理逻辑
  end
end
```

输出：

> X 为占位符

```json
{
  "type": "type",
  "frame": X,
  "sequence": X,
  "data": {
    X
  }
}
```

---

## `Update`

物理帧更新，固定步长调用。

### 调用

```lua
function Update(dt)
end
```

### 参数

`dt` 为物理帧时间差，单位 `秒`。

| 类型   | 说明         |
| ------ | ------------ |
| float | 物理帧时间差 |

**额外补充**

- 物理帧更新间隔固定为 1/60 秒。
- 系统会根据实际帧间隔来计算 `Update` 调用次数，每帧最多调用 8 次。

### 返回

无。

### 示例

```lua
function Update(dt)
  debug.print { message = serialization.json_encode(dt) }
  -- 物理帧更新逻辑
end
```

输出：

```text
0.016666667
0.016666667
0.016666667
...
```

---

## `UpdateFrame`

帧更新，随着系统帧调用。

### 调用

```lua
function UpdateFrame(dt, alpha)
end
```

### 参数

`dt` 为帧更新时间差，单位 `秒`。

| 类型   | 说明         |
| ------ | ------------ |
| float | 物理帧时间差 |

`alpha` 当前显示帧在最近两次次固定 `Update` 前后两个状态之间的差值比例，百分比。

| 类型   | 说明     |
| ------ | -------- |
| float | 差值比例 |

**额外补充**

- 帧更新随系统帧调用，实际调用频率受游戏帧率设置、系统帧设置和玩家设备性能影响。

### 返回

无。

### 示例

```lua
function Update(dt)
  debug.print { message = tostring(dt) }
  debug.print { message = tostring(alpha) }
  debug.print { message = "" }
  -- 帧更新逻辑
end
```

输出：

> 数字均为示例，不代表实际运行值。

```text
0.0170336
0.02201597955968041

0.0501579
0.03148991937020162

0.0169854
0.05061389898772202

...
```

---

## `Render`

绘制当前画面。

### 调用

```lua
function Render()
end
```

### 参数

无。

### 返回

无。

### 示例

```lua
function Render()
  draw.text { x = 0, y = 1, text = "Hello Tui Game", fg = color.BRIGHT_BLUE, bold = true }
end
```

输出：

![lifecycle.Render示例](../image/lifecycle_Render_example.png)

---

## `SaveGame`

保存游戏数据，供玩家"继续游戏"后传递初始化数据。

> 仅游戏脚本可用。
> 仅 `package.json` 中 `game.save` 为 `true` 时可用，且必须实现。
> 通过 `game.save_game` 调用。

### 调用

```lua
function SaveGame()
  return value
end
```

### 参数

无。

### 返回

需要保存的游戏数据。

| 类型 | 说明     |
| ---- | -------- |
| any  | 游戏数据 |

**额外补充**

- 整个返回值必须可序列化。
- 仅返回一个值。

### 示例

```lua
function Init(ctx)
  debug.print { message = serialization.json_encode(ctx) }
  -- 初始化逻辑
end

function SaveGame()
  -- 保存前的数据处理
  local coin = 10
  return coin
end
```

输出：

> X 为占位符

```json
{
  "package_id": "test",
  "package_type": "game",
  "base": {
    "height": X,
    "width": X
  },
  "api_version": 1,
  "start_mode": "continue",
  "continue_data": 10
}
```

---

## `SaveBest`

保存最佳记录数据，用于游戏列表展示，并在后续包含在传递的初始化数据中。

> 仅游戏脚本可用。
> 仅 `package.json` 中 `game.best.enabled` 为 `true` 时可用，且必须实现。
> 通过 `game.save_best` 调用。

### 调用

```lua
function SaveBest()
  return {
    best_string = string,
    any...
  }
end
```

### 参数

无。

### 返回

返回需要保存的最佳记录数据表。

| 类型  | 说明         |
| ----- | ------------ |
| table | 最佳记录数据 |

**额外补充**

- 返回值表必须包含 `best_string` 字段，类型为 `string`，用于游戏列表展示。
- 整个返回值必须可序列化。

### 示例

```lua
function Init(ctx)
  debug.print { message = serialization.json_encode(ctx) }
  -- 初始化逻辑
end

function SaveBest()
  -- 保存前的数据处理
  local coin = 10
  return {
    best_string = "Coin" .. coin,
    coin = coin
  }
end
```

输出：

> X 为占位符

```json
{
  "package_id": "test",
  "package_type": "game",
  "base": {
    "height": X,
    "width": X
  },
  "api_version": 1,
  "start_mode": "new",
  "best_data": {
    "best_string": "Coin 10",
    "coin": 10
  }
}
```

![lifecycle.BestScore示例](../image/lifecycle_BestScore_example.png)
