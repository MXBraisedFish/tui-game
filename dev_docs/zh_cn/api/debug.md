# debug 库

`debug` 提供调试日志、断言和受保护调用。

---

# 目录

## 常量

| 常量 | 说明 | 定位 |
| --- | --- | --- |
| `VERSION` | Lua 与 TUI GAME API 版本字符串 | [VERSION](#version) |
| `TRACE` | 追踪日志等级 | [TRACE](#trace) |
| `DEBUG` | 调试日志等级 | [DEBUG](#debug) |
| `INFO` | 信息日志等级 | [INFO](#info) |
| `WARN` | 警告日志等级 | [WARN](#warn) |
| `ERROR` | 错误日志等级 | [ERROR](#error) |
| `FATAL` | 致命日志等级 | [FATAL](#fatal) |

## 方法

| 方法 | 说明 | 定位 |
| --- | --- | --- |
| `print` | 输出可设置等级和标题的日志 | [print](#print) |
| `info` | 输出信息日志 | [info](#info) |
| `warn` | 输出警告日志 | [warn](#warn) |
| `error` | 输出错误日志 | [error](#error) |
| `assert` | 检查值是否为真值 | [assert](#assert) |
| `pcall` | 受保护地调用函数并返回多个结果 | [pcall](#pcall) |
| `xpcall` | 受保护调用并处理错误值 | [xpcall](#xpcall) |

---

# 常量

## `VERSION`

返回运行时和 TUI GAME API 的版本标识。

### 调用

```lua
debug.VERSION
```

### 可用于

- 任意

### 示例

```lua
debug.print(debug.VERSION)
```

**输出：**

```lua
```

### 等值

```text
"Lua 5.4 / TUI GAME API 1"
```

---

## `TRACE`

追踪日志等级字符串。

### 调用

```lua
debug.TRACE
```

### 可用于

- `debug.print` 的选填参数 `level`

### 示例

```lua
debug.print("trace message", {level = debug.TRACE})
```

**输出：**

```lua
```

### 等值

```text
"trace"
```

---

## `DEBUG`

调试日志等级字符串。

### 调用

```lua
debug.DEBUG
```

### 可用于

- `debug.print` 的选填参数 `level`

### 示例

```lua
debug.print("debug message", {level = debug.DEBUG})
```

**输出：**

```lua
```

### 等值

```text
"debug"
```

---

## `INFO`

信息日志等级字符串。

### 调用

```lua
debug.INFO
```

### 可用于

- `debug.print` 的选填参数 `level`

### 示例

```lua
debug.print("info message", {level = debug.INFO})
```

**输出：**

```lua
```

### 等值

```text
"info"
```

---

## `WARN`

警告日志等级字符串。

### 调用

```lua
debug.WARN
```

### 可用于

- `debug.print` 的选填参数 `level`

### 示例

```lua
debug.print("warning message", {level = debug.WARN})
```

**输出：**

```lua
```

### 等值

```text
"warn"
```

---

## `ERROR`

错误日志等级字符串。

### 调用

```lua
debug.ERROR
```

### 可用于

- `debug.print` 的选填参数 `level`

### 示例

```lua
debug.print("error message", {level = debug.ERROR})
```

**输出：**

```lua
```

### 等值

```text
"error"
```

---

## `FATAL`

致命日志等级字符串。

### 调用

```lua
debug.FATAL
```

### 可用于

- `debug.print` 的选填参数 `level`

### 示例

```lua
debug.print("fatal message", {level = debug.FATAL})
```

**输出：**

```lua
```

### 等值

```text
"fatal"
```

---

# 方法

## `print`

向调试日志输出一条消息。该方法需要启用调试模式。

### 限制

- 需开启调试模式

### 调用

```lua
debug.print
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `message` | any | 要输出的内容，会转换为文本 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `title` | string / nil | `nil` | 日志标题 |
| `level` | string / nil | `nil` | `trace`、`debug`、`info`、`warn`、`error` 或 `fatal` |
| `time` | boolean | `false` | 是否显示时间 |
| `type_head` | boolean | `false` | 是否显示会话类型 |

## 返回值

无返回值。

### 示例

```lua
debug.print("A message")
debug.print("A warning", {level = debug.WARN})
debug.print("A titled message", {title = "Game", time = true, type_head = true})
```

**输出：**

```lua
```

---

## `info`

向调试日志输出一条信息。该方法需要启用调试模式。

### 限制

- 需开启调试模式

### 调用

```lua
debug.info
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `message` | any | 要输出的内容，会转换为文本 |

## 返回值

无返回值。

### 示例

```lua
debug.info("Saved")
```

**输出：**

```lua
```

---

## `warn`

向调试日志输出一条警告。该方法需要启用调试模式。

### 限制

- 需开启调试模式

### 调用

```lua
debug.warn
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `message` | any | 要输出的内容，会转换为文本 |

## 返回值

无返回值。

### 示例

```lua
debug.warn("Low health")
```

**输出：**

```lua
```

---

## `error`

向调试日志输出一条错误。该方法需要启用调试模式。

### 限制

- 需开启调试模式

### 调用

```lua
debug.error
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `message` | any | 要输出的内容，会转换为文本 |

## 返回值

无返回值。

### 示例

```lua
debug.error("Could not load save")
```

**输出：**

```lua
```

---

## `assert`

若值为 `nil` 或 `false`，抛出错误；否则返回传入的原值。可用它检查游戏条件。

### 调用

```lua
debug.assert
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `value` | any | 要检查的值；显式传入 `nil` 会触发断言错误 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `message` | any | `"assertion failed"` | 断言失败时显示的消息 |

## 返回值

断言成功时返回原始 `value`；值为 `nil` 或 `false` 时抛出错误。

### 示例

```lua
local score = debug.assert(10, {message = "score is required"})
```

**输出：**

```lua
```

---

## `pcall`

受保护地调用函数。调用结果以 Lua 多返回值形式返回，成功标记后保留函数的返回值和 nil 位置。

### 调用

```lua
debug.pcall
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `func` | function | 要调用的函数 |

## 返回值

函数成功时返回 `true` 和函数的全部返回值；失败时返回 `false` 和错误值。致命资源错误会继续向外传播。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `ok` | boolean | 调用是否成功 |
| `...` | any... | 成功时为函数返回值；失败时只有错误值 |

### 示例

```lua
local ok, sum, label = debug.pcall(function(a, b)
  return a + b, "sum", nil
end, 10, 20)
```

**输出：**

```lua
```

### 额外说明

- `func` 后依次传入要交给它的值；这些值是调用数据，表和 nil 都会原样保留，不作为选项表解析。

---

## `xpcall`

受保护地调用函数；发生普通 Lua 错误时，先将错误传给处理函数，再返回处理后的错误值。

### 调用

```lua
debug.xpcall
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `func` | function | 要调用的函数 |
| `error_callback` | function | 处理错误值的函数 |

## 返回值

函数成功时返回 `true` 和函数的全部返回值；失败时返回 `false` 和错误处理函数的结果。致命资源错误不会被捕获。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `ok` | boolean | 调用是否成功 |
| `...` | any... | 成功时为函数返回值；失败时为错误处理函数的结果 |

### 示例

```lua
local ok, value = debug.xpcall(function(text)
  return string.upper(text)
end, function(err)
  return "failed: " .. tostring(err)
end, "hello")
```

**输出：**

```lua
```

### 额外说明

- `error_callback` 后依次传入要交给 `func` 的值；这些值是调用数据，表和 nil 都会原样保留，不作为选项表解析。
