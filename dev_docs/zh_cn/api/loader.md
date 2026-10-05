# loader 库

`loader` 用来从包内 `scripts/` 目录加载并执行 Lua 模块。

---

# 目录

## 方法

| 方法       | 说明                                                                       | 定位                  |
| ---------- | -------------------------------------------------------------------------- | --------------------- |
| `require`  | 加载并执行模块；首次调用会执行文件，之后从当前脚本环境的模块缓存中返回结果 | [require](#require)   |
| `dofile`   | 加载并执行模块文件，每次调用都会重新执行                                   | [dofile](#dofile)     |
| `loadfile` | 读取并编译模块文件，但不会立即执行它                                       | [loadfile](#loadfile) |

---

# 方法

## `require`

加载并执行模块。首次调用会执行文件，之后从当前脚本环境的模块缓存中返回结果。

### 调用

```lua
loader.require
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `path` | string | 相对 `scripts/` 的 Lua 文件路径 |

## 返回值

透传模块返回的原始 Lua 多返回值，包括其中的 `nil`。

| 类型 | 说明 |
| --- | --- |
| any... | 模块自身返回的多返回值，按原顺序透传，`nil` 位置保留 |

### 示例

```lua
-- 准备 scripts/value.lua，内容为：return "ready", nil
local status, optional = loader.require("value.lua")
debug.print(status)
```

**输出：**

```lua
```

## 额外说明

- 模块路径指向 `.lua` 文件：可省略扩展名，其它扩展名会抛出错误。
- 模块缓存仅在当前脚本环境内共享。
- 模块源码最多 1 MiB，一次加载链累计最多 4 MiB。
- 嵌套加载超过 16 层，或加载链中重复加载同一路径（循环 require）会抛出错误。
- 模块源码必须是 UTF-8 文本，不接受 Lua 字节码。

---

## `dofile`

加载并执行模块文件，每次调用都会重新执行。

### 调用

```lua
loader.dofile
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `path` | string | 相对 `scripts/` 的 Lua 文件路径 |

## 返回值

透传模块返回的原始 Lua 多返回值，包括其中的 `nil`。

| 类型 | 说明 |
| --- | --- |
| any... | 模块自身返回的多返回值，按原顺序透传，`nil` 位置保留 |

### 示例

```lua
-- 准备 scripts/value.lua，内容为：return "ready", nil
local status, optional = loader.dofile("value.lua")
debug.print(status)
```

**输出：**

```lua
```

---

## `loadfile`

读取并编译模块文件，但不会立即执行它。

### 调用

```lua
loader.loadfile
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `path` | string | 相对 `scripts/` 的 Lua 文件路径 |

## 返回值

返回可调用的函数。调用该函数时会执行已加载文件，并透传文件的原始多返回值和 `nil`。

| 类型 | 说明 |
| --- | --- |
| function | 已编译的 Lua 模块函数 |

### 示例

```lua
-- 准备 scripts/value.lua，内容为：return "ready", nil
local module = loader.loadfile("value.lua")
local status, optional = module()
debug.print(status)
```

**输出：**

```lua
```

## 额外说明

- `require`、`dofile` 和 `loadfile` 的路径必须指向包内 `scripts/`，不能通过绝对路径或路径穿越读取包外文件。
