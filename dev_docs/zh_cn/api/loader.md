# loader 库

`loader` 用来从包内 `scripts/` 目录加载并执行 Lua 模块。

---

# 目录

## 方法

| 方法       | 说明                                 | 定位                  |
| ---------- | ------------------------------------ | --------------------- |
| `require`  | 加载并执行模块文件，结果缓存在内存中 | [require](#require)   |
| `dofile`   | 加载并执行模块文件，结果不缓存       | [dofile](#dofile)     |
| `loadfile` | 读取并编译模块文件                   | [loadfile](#loadfile) |

---

# 方法

## `require`

加载并执行模块文件，结果缓存在内存中。

### 调用

```lua
loader.require
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明                            |
| ------ | ------ | ------------------------------- |
| `path` | string | 相对 `scripts/` 的 Lua 文件路径 |

## 返回值

返回任意数量值。

| 类型   | 说明       |
| ------ | ---------- |
| any... | 模块返回值 |

### 示例

> scripts/value.lua -> return "ready", nil

```lua
local status, optional = loader.require("value.lua")
debug.print(status)
debug.print(type(optional))
```

**输出：**

```lua
ready
nil
```

## 额外说明

- 路径相对当前包的 `scripts/`。

---

## `dofile`

加载并执行模块文件，结果不缓存。

### 调用

```lua
loader.dofile
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明                            |
| ------ | ------ | ------------------------------- |
| `path` | string | 相对 `scripts/` 的 Lua 文件路径 |

## 返回值

返回任意数量值。

| 类型   | 说明       |
| ------ | ---------- |
| any... | 模块返回值 |

### 示例

> scripts/value.lua -> return "ready", nil

```lua
local status, optional = loader.dofile("value.lua")
debug.print(status)
debug.print(type(optional))
```

**输出：**

```lua
ready
nil
```

## 额外说明

- 路径相对当前包的 `scripts/`。

---

## `loadfile`

读取并编译模块文件。

### 调用

```lua
loader.loadfile
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明                            |
| ------ | ------ | ------------------------------- |
| `path` | string | 相对 `scripts/` 的 Lua 文件路径 |

## 返回值

返回一个值。

| 类型     | 说明                  |
| -------- | --------------------- |
| function | 已编译的 Lua 模块函数 |

### 示例

> scripts/value.lua -> return "ready", nil

```lua
local module = loader.loadfile("value.lua")
local status, optional = module()
debug.print(status)
debug.print(type(optional))
```

**输出：**

```lua
ready
nil
```

## 额外说明

- 路径相对当前包的 `scripts/`。
