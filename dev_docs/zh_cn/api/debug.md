# debug 库

## 基本库说明

`debug` 提供调试输出、断言与受保护调用。

---

## 目录

### 常量

| 常量名    | 说明                 | 索引                |
| --------- | -------------------- | ------------------- |
| `VERSION` | 运行时版本标识字符串 | [VERSION](#VERSION) |
| `TRACE`   | 日志等级-追踪        | [TRACE](#TRACE)     |
| `DEBUG`   | 日志等级-调试        | [DEBUG](#DEBUG)     |
| `INFO`    | 日志等级-信息        | [INFO](#INFO)       |
| `WARN`    | 日志等级-警告        | [WARN](#WARN)       |
| `ERROR`   | 日志等级-错误        | [ERROR](#ERROR)     |
| `FATAL`   | 日志等级-致命        | [FATAL](#FATAL)     |

### 方法

| 方法名   | 说明                                             | 索引              |
| -------- | ------------------------------------------------ | ----------------- |
| `print`  | 输出一条自定义日志                               | [print](#print)   |
| `info`   | 输出一条信息级别的日志                           | [info](#info)     |
| `warn`   | 输出一条警告级别的日志                           | [warn](#warn)     |
| `error`  | 输出一条错误级别的日志                           | [error](#error)   |
| `assert` | 断言值非 `nil` 且非 `false`；否则抛出错误        | [assert](#assert) |
| `pcall`  | 受保护地调用函数，返回成功标志与结果（或错误值） | [pcall](#pcall)   |
| `xpcall` | 受保护地调用函数，失败时先执行错误处理函数再返回 | [xpcall](#xpcall) |

---

## 常量

## `VERSION`

运行时版本标识字符串。

**可用于**

- 任意

### 调用

```lua
debug.VERSION
```

### 示例

```lua
debug.print { message = debug.VERSION }
```

输出：

```text
Lua 5.4 / TUI GAME API 1
```

---

## `TRACE`

日志等级-追踪。

**可用于**

- 参数 `level`

### 调用

```lua
debug.TRACE
```

### 示例

```lua
debug.print { message = "TRACE", level = debug.TRACE }
```

输出：

```text
[追踪] TRACE
```

---

## `DEBUG`

日志等级-调试。

**可用于**

- 参数 `level`

### 调用

```lua
debug.DEBUG
```

### 示例

```lua
debug.print { message = "DEBUG", level = debug.DEBUG }
```

输出：

```text
[调试] DEBUG
```

---

## `INFO`

日志等级-信息。

**可用于**

- 参数 `level`

### 调用

```lua
debug.INFO
```

### 示例

```lua
debug.print { message = "INFO", level = debug.INFO }
```

输出：

```text
[信息] INFO
```

---

## `WARN`

日志等级-警告。

**可用于**

- 参数 `level`

### 调用

```lua
debug.WARN
```

### 示例

```lua
debug.print { message = "WARN", level = debug.WARN }
```

输出：

```text
[警告] WARN
```

---

## `ERROR`

日志等级-错误。

**可用于**

- 参数 `level`

### 调用

```lua
debug.ERROR
```

### 示例

```lua
debug.print { message = "ERROR", level = debug.ERROR }
```

输出：

```text
[错误] ERROR
```

---

## `FATAL`

日志等级-致命。

**可用于**

- 参数 `level`

### 调用

```lua
debug.FATAL
```

### 示例

```lua
debug.print { message = "FATAL", level = debug.FATAL }
```

输出：

```text
[致命] FATAL
```

---

## 方法

## `print`

输出一条自定义日志。

> 需开启调试模式。

### 调用

```lua
-- 表参数
debug.print{}
```

### 参数

| 参数名      | 类型              | 必填 | 默认值  | 说明           |
| ----------- | ----------------- | ---- | ------- | -------------- |
| `message`   | string            | 是   | -       | 日志内容       |
| `title`     | string / nil      | 否   | `nil`   | 自定义日志标题 |
| `level`     | const-debug / nil | 否   | `nil`   | 日志级别       |
| `time`      | boolean           | 否   | `false` | 时间           |
| `type_head` | boolean           | 否   | `false` | 类型头         |

### 返回

无。

### 示例

```lua
debug.print { message = "Print A Log" }
debug.print { message = "Print A Warn", level = debug.WARN }
debug.print { message = "Print A Info", title = "A Title", level = debug.INFO, time = true, type_head = true }
```

输出

```text
Print A Log
[警告] Print A Warn
[游戏][yyyy-mm-dd hh:mm:ss.ms][信息][A Title] Print A Info
```

---

## `info`

输出一条信息级别的日志。

> 需开启调试模式。

### 调用

```lua
-- 单参数
debug.info()
```

### 参数

| 参数名    | 类型   | 必填 | 默认值 | 说明     |
| --------- | ------ | ---- | ------ | -------- |
| `message` | string | 是   | -      | 日志内容 |

### 返回

无。

### 示例

```lua
debug.info("This is a INFO")
```

输出

```text
[游戏][yyyy-mm-dd hh:mm:ss.ms][信息] This is a INFO
```

---

## `warn`

输出一条警告级别的日志。

> 需开启调试模式。

### 调用

```lua
-- 单参数
debug.warn()
```

### 参数

| 参数名    | 类型   | 必填 | 默认值 | 说明     |
| --------- | ------ | ---- | ------ | -------- |
| `message` | string | 是   | -      | 日志内容 |

### 返回

无。

### 示例

```lua
debug.warn("This is a WARN")
```

输出

```text
[游戏][yyyy-mm-dd hh:mm:ss.ms][警告] This is a WARN
```

---

## `error`

输出一条错误级别的日志。

> 需开启调试模式。

### 调用

```lua
-- 单参数
debug.error()
```

### 参数

| 参数名    | 类型   | 必填 | 默认值 | 说明     |
| --------- | ------ | ---- | ------ | -------- |
| `message` | string | 是   | -      | 日志内容 |

### 返回

无。

### 示例

```lua
debug.error("This is a ERROR")
```

输出

```text
[游戏][yyyy-mm-dd hh:mm:ss.ms][错误] This is a ERROR
```

### 额外补充

- **仅为日志信息输出**，不会中断脚本运行

---

## `assert`

断言值非 `nil` 且非 `false`；否则抛出错误。

### 调用

```lua
-- 表参数
debug.assert{}
```

### 参数

| 参数名    | 类型   | 必填 | 默认值               | 说明             |
| --------- | ------ | ---- | -------------------- | ---------------- |
| `value`   | any    | 是   | -                    | 断言表达式结果   |
| `message` | string | 否   | `"assertion failed"` | 失败时的错误信息 |

### 返回

若 `value` 参数断言通过，直接返回一个值。

| 类型 | 说明               |
| ---- | ------------------ |
| any  | 断言通过时原样返回 |

### 示例

```lua
local v1 = debug.assert { value = 1 == 1, message = "Right" }
debug.print { message = tostring(v1) }

local v2 = debug.assert { value = 0 == 1, message = "Asser Error" }
```

输出

```text
true

[运行][Lua][yyyy-mm-dd hh:mm:ss.ms][错误] Lua game session 'test.package' failed during Callback (Update): runtime error: Asser Error
stack traceback: [Error Message]
【脚本终止运行】
```

---

## `pcall`

受保护地调用函数，返回成功标志与结果（或错误值）。

### 调用

```lua
-- 表参数
debug.pcall{}
```

### 参数

| 参数名   | 类型        | 必填 | 默认值 | 说明                   |
| -------- | ----------- | ---- | ------ | ---------------------- |
| `func`   | function    | 是   | -      | 要调用的函数           |
| `values` | table / nil | 否   | `nil`  | 传给 `func` 的参数数组表 |

### 返回

若参数 `func` **执行成功**，返回一个对象表

| 字段     | 类型    | 说明             |
| -------- | ------- | ---------------- |
| `ok`     | boolean | 是否成功（true） |
| `values` | table   | 函数返回值       |

若参数 `func` **执行失败**，返回一个对象表

| 字段    | 类型    | 说明              |
| ------- | ------- | ----------------- |
| `ok`    | boolean | 是否成功（false） |
| `error` | string  | 错误信息          |

### 示例

```lua
local result1 = debug.pcall {
  func = function(a, b)
    return a + b, "Clear"
  end,
  values = { 10, 20 }
}

if result1.ok then
  for item in pairs(result1.values) do
    debug.print { message = tostring(item.index) .. " " .. tostring(item.value) }
  end
end

local result2 = debug.pcall {
  func = function(a, b)
    debug.assert { value = a == b }
  end,
  values = { 10, 20 }
}

if not result2.ok then
  debug.error(result2.error)
end
```

输出

```text
1 30
2 Clear
n 2

[运行][Lua][yyyy-mm-dd hh:mm:ss.ms][错误] runtime error: assertion failed
stack traceback: [Error Message]
【脚本继续运行，不会终止】
```

### 额外补充

- 返回值 `values` 表结构如下：

```lua
{
  [1] = ..., -- any
  [2] = ...,
  ...
  [x] = ..., -- any
  n = x      -- integer
} -- 共有 x+1 个元素，所有返回值连续排序，最后 n 为返回值个数
```

---

## `xpcall`

受保护地调用函数，失败时先执行错误处理函数再返回。

**可用于**

- 任意

### 调用

```lua
-- 表参数
debug.xpcall{}
```

### 参数

| 参数名           | 类型           | 必填 | 默认值 | 说明                   |
| ---------------- | -------------- | ---- | ------ | ---------------------- |
| `func`           | function       | 是   | -      | 要调用的函数           |
| `values`         | table / nil    | 否   | `nil`  | 传给 `func` 的参数数组表 |
| `error_callback` | function / nil | 否   | `nil`  | 错误处理函数           |

### 返回

若参数 `func` **执行成功**，返回一个对象表

| 字段     | 类型    | 说明             |
| -------- | ------- | ---------------- |
| `ok`     | boolean | 是否成功（true） |
| `values` | table   | 函数返回值       |

若参数 `func` **执行失败**，执行参数 `error_callback`，返回一个对象表

| 字段    | 类型    | 说明              |
| ------- | ------- | ----------------- |
| `ok`    | boolean | 是否成功（false） |
| `error` | any     | 错误信息          |

### 示例

```lua
local result1 = debug.xpcall {
  func = function(a, b)
    return a + b, "Clear"
  end,
  values = { 10, 20 }
}

if result1.ok then
  for item in pairs(result1.values) do
    debug.print { message = tostring(item.index) .. " " .. tostring(item.value) }
  end
end

local result2 = debug.xpcall {
  func = function(a, b)
    debug.assert { value = a == b }
  end,
  values = { 10, 20 },
  error_callback = function(error)
    return "Xpcall Error Callback Return: " .. error
  end
}

if not result2.ok then
  debug.error(result2.error)
end
```

输出

```text
1 30
2 Clear
n 2

[运行][Lua][yyyy-mm-dd hh:mm:ss.ms][错误] Xpcall Error Callback Return: runtime error: assertion failed
stack traceback: [Error Message]
【脚本继续运行，不会终止】
```

### 额外补充

- 返回值 `values` 表结构如下：

```lua
{
  [1] = ...,
  [2] = ...,
  ...
  [x] = ...,
  n = x
} -- 共有 x+1 个元素，所有返回值连续排序，最后 n 为返回值个数
```

- 参数 `error_callback` 函数结构如下：

```lua
function(error)
  -- 错误处理逻辑
  return any
end
```

- 返回值 `error` 在接收多值时只保留一个值

```lua
return "apple", "banner", "orange"
result.error = "apple"
-- "banner", "orange" 均忽略

return { "apple", "banner", "orange" }
result.error = { "apple", "banner", "orange" }
-- 若需要多值返回，可以打包成一个表
```
