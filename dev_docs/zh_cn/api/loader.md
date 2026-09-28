# loader 库

## 基本库说明

`loader` 提供脚本加载额外的 Lua 模块。

---

## 目录

### 方法

| 方法名     | 说明                                         | 索引                  |
| ---------- | -------------------------------------------- | --------------------- |
| `require`  | 加载并执行模块，返回模块结果，并缓存加载结果 | [require](#require)   |
| `dofile`   | 加载并执行模块，返回模块结果，不缓存加载结果 | [dofile](#dofile)     |
| `loadfile` | 加载并编译模块，返回编译后的函数，不执行模块 | [loadfile](#loadfile) |

---

## 方法

## `require`

加载并执行模块，返回模块结果，并缓存加载结果。

### 调用

```lua
-- 命名参数表
loader.require{ path = "helper.lua" }
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明                     |
| ------ | ------ | ---- | ------ | ------------------------ |
| `path` | string | 是   | -      | 相对 `scripts/` 目录路径 |

### 返回

自定义返回值。

| 类型     | 说明       |
| -------- | ---------- |
| `any...` | 模块返回值（保留模块定义的多返回值与 nil 位置） |

### 示例

```lua
scripts/
+ main.lua
- helper.lua

-- main.lua
local helper1 = loader.require{ path = "helper.lua" }
local helper2 = loader.require{ path = "helper.lua" }

helper1.print()

debug.print { message = tostring(helper1 == helper2) }

-- helper.lua
local M = {}

function M.print()
  debug.print { message = "Helper Function" }
end

return M
```

输出：

```text
Helper Function
true
```

## 额外补充

- 模块必须为 `lua` 文件。
- 该 API 调用后的返回值共享全局环境缓存，每次调用不会重新执行。

---

## `dofile`

加载并执行模块，返回模块结果，不缓存加载结果。

### 调用

```lua
-- 命名参数表
loader.dofile{ path = "helper.lua" }
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明                     |
| ------ | ------ | ---- | ------ | ------------------------ |
| `path` | string | 是   | -      | 相对 `scripts/` 目录路径 |

### 返回

自定义返回值。

| 类型     | 说明       |
| -------- | ---------- |
| `any...` | 模块返回值（保留模块定义的多返回值与 nil 位置） |

### 示例

```lua
scripts/
+ main.lua
- helper.lua

-- main.lua
local helper1 = loader.dofile{ path = "helper.lua" }
local helper2 = loader.dofile{ path = "helper.lua" }

helper1.print()

debug.print { message = tostring(helper1 == helper2) }

-- helper.lua
local M = {}

function M.print()
  debug.print { message = "Helper Function" }
end

return M
```

输出：

```text
Helper Function
false
```

## 额外补充

- 模块必须为 `lua` 文件。
- 该 API 调用后的返回值不缓存，每次调用都会重新执行。

---

## `loadfile`

加载并编译模块，返回编译后的函数，不执行模块。

### 调用

```lua
-- 命名参数表
loader.loadfile{ path = "helper.lua" }
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明                     |
| ------ | ------ | ---- | ------ | ------------------------ |
| `path` | string | 是   | -      | 相对 `scripts/` 目录路径 |

### 返回

直接返回一个值。

| 类型       | 说明             |
| ---------- | ---------------- |
| `function` | 模块编译后的函数 |

### 示例

```lua
scripts/
+ main.lua
- helper.lua

-- main.lua
local func1 = loader.loadfile{ path = "helper.lua" }
local func2 = loader.loadfile{ path = "helper.lua" }

debug.print { message = tostring(func1 == func2) }

local helper = func1()

helper.print()

-- helper.lua
local M = {}

function M.print()
  debug.print { message = "Helper Function" }
end

return M
```

输出：

```text
false
Helper Function
```

## 额外补充

- 模块必须为 `lua` 文件。
- 该 API 调用后的编译值不缓存，每次调用都会重新编译。
