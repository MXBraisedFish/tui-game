# random 库

`random` 提供随机数生成和随机数生成器管理。

---

# 目录

## 常量

| 常量    | 说明                   | 定位            |
| ------- | ---------------------- | --------------- |
| `INT`   | 整数类型随机数生成器   | [INT](#int)     |
| `FLOAT` | 浮点数类型随机数生成器 | [FLOAT](#float) |

## 方法

| 方法        | 说明                         | 定位                    |
| ----------- | ---------------------------- | ----------------------- |
| `randint`   | 随机生成指定区间的整数       | [randint](#randint)     |
| `randfloat` | 随机生成指定区间的浮点数     | [randfloat](#randfloat) |
| `create`    | 创建一个随机数生成器对象     | [create](#create)       |
| `delete`    | 删除指定生成器               | [delete](#delete)       |
| `clear`     | 删除所有生成器               | [clear](#clear)         |
| `list`      | 获取所有生成器的信息         | [list](#list)           |
| `count`     | 返回当前生成器的总数         | [count](#count)         |
| `generate`  | 使用指定生成器生成一个随机数 | [generate](#generate)   |
| `set`       | 修改生成器的参数             | [set](#set)             |
| `get_info`  | 获取生成器的完整信息表       | [get_info](#get_info)   |
| `exists`    | 检查生成器是否存在           | [exists](#exists)       |

---

# 常量

## `INT`

整数类型随机数生成器。

### 调用

```lua
random.INT
```

### 可用于

- 参数 `type`

### 示例

```lua
local generator = random.create({type = random.INT})
local value = random.generate(generator)
```

**输出：**

```lua

```

### 等值

```text
"int"
```

---

## `FLOAT`

浮点数类型随机数生成器。

### 调用

```lua
random.FLOAT
```

### 可用于

- 参数 `type`

### 示例

```lua
local generator = random.create({type = random.FLOAT})
local value = random.generate(generator)
```

**输出：**

```lua

```

### 等值

```text
"float"
```

---

# 方法

## `randint`

随机生成指定区间的整数。

### 调用

```lua
random.randint
```

## 参数

### 选填参数

| 参数名 | 类型    | 默认值        | 说明           |
| ------ | ------- | ------------- | -------------- |
| `min`  | integer | `-2147483648` | 区间下界（含） |
| `max`  | integer | `2147483647`  | 区间上界（含） |

## 返回值

返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| integer | 区间内的随机整数 |

### 示例

```lua
local r1 = random.randint()
debug.print(r1)

local r2 = random.randint({min = 1, max = 10})
debug.print(r2)

local r3 = random.randint({min = -20, max = -8})
debug.print(r3)
```

**输出：**

```lua

```

## 额外说明

- 随机区间为闭区间 $[min, max]$。
- 选填参数 `min` 取值范围为 $[-9223372036854775808, 9223372036854775807]$，不得大于 `max`，否则抛出错误。
- 可设置区间；若要控制种子或步进，请使用 `random.create` 创建生成器。
- 选填参数 `max` 取值范围为 $[-9223372036854775808, 9223372036854775807]$，不得小于 `min`，否则抛出错误。

---

## `randfloat`

随机生成指定区间的浮点数。

### 调用

```lua
random.randfloat
```

## 参数

### 选填参数

| 参数名 | 类型            | 默认值 | 说明           |
| ------ | --------------- | ------ | -------------- |
| `min`  | integer / float | `0`    | 区间下界（含） |
| `max`  | integer / float | `1`    | 区间上界（含） |

## 返回值

返回一个值。

| 类型  | 说明               |
| ----- | ------------------ |
| float | 区间内的随机浮点数 |

### 示例

```lua
local r1 = random.randfloat()
debug.print(r1)

local r2 = random.randfloat({min = 1, max = 10})
debug.print(r2)

local r3 = random.randfloat({min = -20, max = -8})
debug.print(r3)
```

**输出：**

```lua

```

## 额外说明

- 随机区间为闭区间 $[min, max]$。
- 选填参数 `min` 取值范围为 $(-\infty, +\infty)$，必须为有限数值，且不得大于 `max`，否则抛出错误。
- 可设置区间；若要控制种子或步进，请使用 `random.create` 创建生成器。
- 选填参数 `max` 取值范围为 $(-\infty, +\infty)$，必须为有限数值，且不得小于 `min`，否则抛出错误。

---

## `create`

创建一个随机数生成器对象。

### 限制

同一会话最多同时存在 4096 个生成器，超出后再创建会抛出错误。

### 调用

```lua
random.create
```

## 参数

### 选填参数

| 参数名 | 类型            | 默认值         | 说明               |
| ------ | --------------- | -------------- | ------------------ |
| `type` | const-random    | `random.INT`   | 生成器类型         |
| `min`  | integer / float | 跟随生成器类型 | 区间下界（含）     |
| `max`  | integer / float | 跟随生成器类型 | 区间上界（含）     |
| `seed` | integer         | 系统随机生成   | 随机种子           |
| `step` | integer         | `0`            | 初始步进数（非负） |

## 返回值

返回一个值。

| 类型   | 说明      |
| ------ | --------- |
| string | 生成器 ID |

### 示例

```lua
local r1 = random.create()
debug.print(r1)
debug.print(random.generate(r1))

local r2 = random.create({type = random.INT, min = 1, max = 30, seed = 520})
debug.print(r2)
debug.print(random.generate(r2))

debug.print(table.pretty(random.list()))
```

**输出：**

```lua

```

## 额外说明

- 随机区间为闭区间 $[min, max]$。
- 参数 `type` 为 `random.INT` 时，参数 `min` 默认值为 `-2147483648`，参数 `max` 默认值为 `2147483647`。
- 参数 `type` 为 `random.FLOAT` 时，参数 `min` 默认值为 `0`，参数 `max` 默认值为 `1`。
- 参数 `type` 只能为 `random.INT` 或 `random.FLOAT`。
- 返回的生成器 ID 形如 `rng_001`；其它方法要求的 `id` 必须是同格式字符串，格式不合法会抛出错误。
- 选填参数 `min` 取值范围为 $[-9223372036854775808, 9223372036854775807]$，`type` 为 `random.FLOAT` 时为 $(-\infty, +\infty)$ 内的有限数值，且不得大于 `max`，否则抛出错误。
- 选填参数 `max` 取值范围为 $[-9223372036854775808, 9223372036854775807]$，`type` 为 `random.FLOAT` 时为 $(-\infty, +\infty)$ 内的有限数值，且不得小于 `min`，否则抛出错误。
- 选填参数 `seed` 取值范围为 $[-9223372036854775808, 9223372036854775807]$。
- 选填参数 `step` 取值范围为 $[0, 9223372036854775807]$。

---

## `delete`

删除指定生成器。

### 调用

```lua
random.delete
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 生成器 ID |

## 返回值

**对象不存在时**，返回 `false`；**对象存在时**，返回以下结果。

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否删除成功 |

### 示例

```lua
local r = random.create()
debug.print(r)
debug.print(random.generate(r))

debug.print(random.delete(r))

debug.print(table.pretty(random.list()))
```

**输出：**

```lua

```

---

## `clear`

删除所有生成器。

### 调用

```lua
random.clear
```

## 返回值

返回一个值。

| 类型    | 说明        |
| ------- | ----------- |
| boolean | 恒为 `true` |

### 示例

```lua
random.create()
random.create()
random.create()
random.create()

debug.print(random.clear())

debug.print(table.pretty(random.list()))
```

**输出：**

```lua

```

---

## `list`

获取所有生成器的信息。

### 调用

```lua
random.list
```

## 返回值

返回一个混合表。

| 类型  | 说明           |
| ----- | -------------- |
| table | 所有生成器信息 |

### 示例

```lua
random.create()
random.create()

debug.print(table.pretty(random.list()))
```

**输出：**

```lua

```

## 额外说明

- 返回值混合表结构如下：

```lua
local generators = {
  {
    id = "rng_001", -- 示例 ID；使用 random.create 的实际返回值
    type = "int", -- random.INT；浮点类型为 "float"
    min = 1, -- float / integer
    max = 10, -- float / integer
    seed = 42, -- integer
    step = 1, -- integer
  },
  n = 1, -- integer；生成器数量
}
```

---

## `count`

返回当前生成器的总数。

### 调用

```lua
random.count
```

## 返回值

返回一个值。

| 类型    | 说明       |
| ------- | ---------- |
| integer | 生成器总数 |

### 示例

```lua
random.create()
random.create()
random.create()
random.create()
random.create()

debug.print(random.count())
```

**输出：**

```lua

```

---

## `generate`

使用指定生成器生成一个随机数。

### 调用

```lua
random.generate
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 生成器 ID |

## 返回值

**对象不存在时**，返回 `nil`；**对象存在时**，返回以下结果。

返回一个值。

| 类型            | 说明         |
| --------------- | ------------ |
| integer / float | 生成的随机数 |

### 示例

```lua
local r = random.create({min = -5, max = 30})

debug.print(random.generate(r))
debug.print(random.generate(r))
debug.print(random.generate(r))
debug.print(random.generate(r))
debug.print(random.generate(r))
```

**输出：**

```lua

```

## 额外说明

- 每次成功生成后步进数加 1；步进数达到上限后再生成会抛出错误。

---

## `set`

修改生成器的参数。

### 调用

```lua
random.set
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 生成器 ID |

### 选填参数

| 参数名 | 类型            | 默认值   | 说明           |
| ------ | --------------- | -------- | -------------- |
| `type` | const-random    | 保持原值 | 生成器类型     |
| `min`  | integer / float | 保持原值 | 区间下界（含） |
| `max`  | integer / float | 保持原值 | 区间上界（含） |
| `seed` | integer         | 保持原值 | 随机种子       |
| `step` | integer         | 保持原值 | 步进数（非负） |

## 返回值

**对象不存在时**，返回 `false`；**对象存在时**，返回以下结果。

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否修改成功 |

### 示例

```lua
local r = random.create()

debug.print(table.pretty(random.get_info(r)))

random.set(r, {type = random.FLOAT, min = 3.2, max = 5.8, seed = 123456})

debug.print(table.pretty(random.get_info(r)))
```

**输出：**

```lua

```

## 额外说明

- 随机区间为闭区间 $[min, max]$。
- 参数 `type` 只能为 `random.INT` 或 `random.FLOAT`。
- 选填参数 `min` 取值范围为 $[-9223372036854775808, 9223372036854775807]$，目标类型为 `random.FLOAT` 时为 $(-\infty, +\infty)$ 内的有限数值，须与目标类型匹配且不得大于 `max`，否则抛出错误。
- 选填参数 `max` 取值范围为 $[-9223372036854775808, 9223372036854775807]$，目标类型为 `random.FLOAT` 时为 $(-\infty, +\infty)$ 内的有限数值，须与目标类型匹配且不得小于 `min`，否则抛出错误。
- 选填参数 `seed` 取值范围为 $[-9223372036854775808, 9223372036854775807]$。
- 选填参数 `step` 取值范围为 $[0, 9223372036854775807]$。

---

## `get_info`

获取生成器的完整信息表。

### 调用

```lua
random.get_info
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 生成器 ID |

## 返回值

**对象存在时**，返回一个表；**不存在时**，返回 `nil`。

| 类型        | 说明                     |
| ----------- | ------------------------ |
| table / nil | 生成器配置，或对象不存在 |

### 示例

```lua
local r = random.create()
debug.print(table.pretty(random.get_info(r)))
```

**输出：**

```lua

```

## 额外说明

返回表包含以下字段：

| 字段   | 类型            | 说明       |
| ------ | --------------- | ---------- |
| `id`   | string          | 生成器 ID  |
| `type` | string          | 生成器类型 |
| `min`  | float / integer | 区间下界   |
| `max`  | float / integer | 区间上界   |
| `seed` | integer         | 生成器种子 |
| `step` | integer         | 步进数     |

---

## `exists`

检查生成器是否存在。

### 调用

```lua
random.exists
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 生成器 ID |

## 返回值

返回一个值。

| 类型    | 说明     |
| ------- | -------- |
| boolean | 是否存在 |

### 示例

```lua
local r = random.create()
debug.print(random.exists(r))

random.delete(r)
debug.print(random.exists(r))
```

**输出：**

```lua

```
