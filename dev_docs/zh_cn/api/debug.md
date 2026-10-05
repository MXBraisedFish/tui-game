# debug 库

`debug` 提供调试日志、断言和受保护调用。

---

# 目录

## 常量

| 常量      | 说明                                 | 定位                |
| --------- | ------------------------------------ | ------------------- |
| `VERSION` | 返回运行时和 TUI GAME API 的版本标识 | [VERSION](#version) |
| `TRACE`   | 追踪日志等级字符串                   | [TRACE](#trace)     |
| `DEBUG`   | 调试日志等级字符串                   | [DEBUG](#debug)     |
| `INFO`    | 信息日志等级字符串                   | [INFO](#info)       |
| `WARN`    | 警告日志等级字符串                   | [WARN](#warn)       |
| `ERROR`   | 错误日志等级字符串                   | [ERROR](#error)     |
| `FATAL`   | 致命日志等级字符串                   | [FATAL](#fatal)     |

## 方法

| 方法     | 说明                                       | 定位              |
| -------- | ------------------------------------------ | ----------------- |
| `print`  | 向调试日志输出一条消息                     | [print](#print)   |
| `info`   | 向调试日志输出一条信息                     | [info](#info)     |
| `warn`   | 向调试日志输出一条警告                     | [warn](#warn)     |
| `error`  | 向调试日志输出一条错误                     | [error](#error)   |
| `assert` | 断言值                                     | [assert](#assert) |
| `pcall`  | 受保护地调用函数                           | [pcall](#pcall)   |
| `xpcall` | 受保护地调用函数，出现错误执行回调函数处理 | [xpcall](#xpcall) |

---

# 常量

## `VERSION`

返回运行时和 TUI GAME API 的版本标识。

### 调用

```lua
debug.VERSION
```

### 可用于

- 任意。

### 示例

```lua
debug.print(debug.VERSION)
```

**输出：**

```lua
Lua 5.4 / TUI GAME API 1
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

- 参数 `level`

### 示例

```lua
debug.print("trace message", {level = debug.TRACE})
```

**输出：**

```lua
[跟踪] trace message
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

- 参数 `level`

### 示例

```lua
debug.print("debug message", {level = debug.DEBUG})
```

**输出：**

```lua
[调试] debug message
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

- 参数 `level`

### 示例

```lua
debug.print("info message", {level = debug.INFO})
```

**输出：**

```lua
[信息] info message
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

- 参数 `level`

### 示例

```lua
debug.print("warning message", {level = debug.WARN})
```

**输出：**

```lua
[警告] warning message
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

- 参数 `level`

### 示例

```lua
debug.print("error message", {level = debug.ERROR})
```

**输出：**

```lua
[错误] error message
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

- 参数 `level`

### 示例

```lua
debug.print("fatal message", {level = debug.FATAL})
```

**输出：**

```lua
[致命] fatal message
```

### 等值

```text
"fatal"
```

---

# 方法

## `print`

向调试日志输出一条消息。

### 限制

- 需开启调试模式。

### 调用

```lua
debug.print
```

## 参数

### 必填参数

| 参数名    | 类型 | 说明     |
| --------- | ---- | -------- |
| `message` | any  | 输出信息 |

### 选填参数

| 参数名      | 类型         | 默认值  | 说明             |
| ----------- | ------------ | ------- | ---------------- |
| `title`     | string / nil | `nil`   | 日志标题         |
| `level`     | const-debug  | `nil`   | 日志等级         |
| `time`      | boolean      | `false` | 是否显示时间     |
| `type_head` | boolean      | `false` | 是否显示会话类型 |

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
A message
[警告] A warning
[游戏][2026-10-05 00:54:12.209][Game] A titled message
```

## 额外补充

- 必填参数 `message` 不可为 `nil`。

---

## `info`

向调试日志输出一条信息。

### 限制

- 需开启调试模式。

### 调用

```lua
debug.info
```

## 参数

### 必填参数

| 参数名    | 类型 | 说明     |
| --------- | ---- | -------- |
| `message` | any  | 输出信息 |

## 返回值

无返回值。

### 示例

```lua
debug.info("Saved")
```

**输出：**

```lua
[游戏][2026-10-05 01:03:05.690][信息] Saved
```

## 额外补充

- 必填参数 `message` 不可为 `nil`。

---

## `warn`

向调试日志输出一条警告。

### 限制

- 需开启调试模式。

### 调用

```lua
debug.warn
```

## 参数

### 必填参数

| 参数名    | 类型 | 说明     |
| --------- | ---- | -------- |
| `message` | any  | 输出信息 |

## 返回值

无返回值。

### 示例

```lua
debug.warn("Low health")
```

**输出：**

```lua
[游戏][2026-10-05 01:03:25.573][警告] Low health
```

## 额外补充

- 必填参数 `message` 不可为 `nil`。

---

## `error`

向调试日志输出一条错误。

### 限制

- 需开启调试模式。

### 调用

```lua
debug.error
```

## 参数

### 必填参数

| 参数名    | 类型 | 说明     |
| --------- | ---- | -------- |
| `message` | any  | 输出信息 |

## 返回值

无返回值。

### 示例

```lua
debug.error("Could not load save")
```

**输出：**

```lua
[游戏][2026-10-05 01:03:41.814][错误] Could not load save
```

## 额外补充

- 必填参数 `message` 不可为 `nil`。

---

## `assert`

断言值。

### 调用

```lua
debug.assert
```

## 参数

### 必填参数

| 参数名  | 类型 | 说明     |
| ------- | ---- | -------- |
| `value` | any  | 断言的值 |

### 选填参数

| 参数名    | 类型 | 默认值               | 说明                 |
| --------- | ---- | -------------------- | -------------------- |
| `message` | any  | `"assertion failed"` | 断言失败时输出的信息 |

## 返回值

**断言成功时**，返回一个值。

| 类型 | 说明         |
| ---- | ------------ |
| any  | 断言成功的值 |

**断言失败时**，抛出异常。

### 示例

```lua
local score = debug.assert(nil, {message = "score is required"})
```

**输出：**

```lua
[脚本终止运行，抛出错误信息]
```

## 额外补充

- 选填参数 `message` 不可为 `nil`。

---

## `pcall`

受保护地调用函数。

### 调用

```lua
debug.pcall
```

## 参数

### 必填参数

| 参数名 | 类型     | 说明             |
| ------ | -------- | ---------------- |
| `func` | function | 要调用的函数     |
| `...`  | any...   | 传递给函数的变参 |

## 返回值

**函数成功时**，返回任意数量值。

| 值名  | 类型    | 说明                        |
| ----- | ------- | --------------------------- |
| `ok`  | boolean | 调用是否成功（必为 `true`） |
| `...` | any...  | 函数返回值                  |

**函数失败时**，返回两个值。

| 值名    | 类型    | 说明                         |
| ------- | ------- | ---------------------------- |
| `ok`    | boolean | 调用是否成功（必为 `false`） |
| `error` | string  | 错误信息                     |

### 示例

```lua
local ok, error = debug.pcall(function(a, b)
  return a + b
end, 10, "3")

debug.print(ok)
debug.print(error)

local ok, sum, label = debug.pcall(function(a, b)
  return a + b, "sum"
end, 10, "3")

debug.print(ok)
debug.print(sum)
debug.print(label)
```

**输出：**

```lua
false
[打印错误信息，脚本继续运行]
true
13
sum
```

---

## `xpcall`

受保护地调用函数，出现错误执行回调函数处理。

### 调用

```lua
debug.xpcall
```

## 参数

### 必填参数

| 参数名           | 类型     | 说明                 |
| ---------------- | -------- | -------------------- |
| `func`           | function | 要调用的函数         |
| `error_callback` | function | 处理错误值的函数     |
| `...`            | any...   | 传递给调用函数的变参 |

## 返回值

**函数成功时**，返回任意数量值。

| 值名  | 类型    | 说明                        |
| ----- | ------- | --------------------------- |
| `ok`  | boolean | 调用是否成功（必为 `true`） |
| `...` | any...  | 调用函数返回值              |

**函数失败时**，返回两个值。

| 值名    | 类型    | 说明                         |
| ------- | ------- | ---------------------------- |
| `ok`    | boolean | 调用是否成功（必为 `false`） |
| `error` | any     | 处理函数返回值               |

### 示例

```lua
local ok, value = debug.xpcall(function(text)
  return string.upper(text)
end, function(err)
  return "failed: " .. tostring(err)
end, "hello")

debug.print(ok)
debug.print(value)

local ok, error, code = debug.xpcall(function(text)
  return string.upper(text)
end, function(err)
  return "failed: " .. tostring(err), 123
end, nil)

debug.print(ok)
debug.print(error)
debug.print(type(code))
```

**输出：**

```lua
true
HELLO
false
[打印处理后的错误信息，脚本继续运行]
nil
```

## 额外说明

- 必填参数 `error_callback` 回调函数接收一个值，为错误信息。
- 必填参数 `error_callback` 回调函数返回值仅保留第一个。