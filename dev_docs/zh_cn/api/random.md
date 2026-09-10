# random 库

## 基本库说明

`random` 提供可控且安全的随机数生成。

---

## 目录

### 常量

| 常量名  | 说明                   | 索引            |
| ------- | ---------------------- | --------------- |
| `INT`   | 整数类型随机数生成器   | [INT](#INT)     |
| `FLOAT` | 浮点数类型随机数生成器 | [FLOAT](#FLOAT) |

### 方法

| 方法名      | 说明                         | 索引                    |
| ----------- | ---------------------------- | ----------------------- |
| `randint`   | 随机生成指定区间的整数       | [randint](#randint)     |
| `randfloat` | 随机生成指定区间的浮点数     | [randfloat](#randfloat) |
| `create`    | 创建一个随机数生成器对象     | [create](#create)       |
| `delete`    | 删除指定生成器               | [delete](#delete)       |
| `clear`     | 删除所有生成器               | [clear](#clear)         |
| `list`      | 返回所有生成器的信息         | [list](#list)           |
| `count`     | 返回当前生成器的总数         | [count](#count)         |
| `generate`  | 使用指定生成器生成一个随机数 | [generate](#generate)   |
| `set`       | 修改生成器的参数             | [set](#set)             |
| `set_type`  | 修改生成器的类型             | [set_type](#set_type)   |
| `set_range` | 修改生成器的随机区间         | [set_range](#set_range) |
| `set_seed`  | 修改生成器的种子             | [set_seed](#set_seed)   |
| `set_step`  | 修改生成器的步进数           | [set_step](#set_step)   |
| `get_type`  | 获取生成器的类型             | [get_type](#get_type)   |
| `get_range` | 获取生成器的随机区间         | [get_range](#get_range) |
| `get_seed`  | 获取生成器的种子             | [get_seed](#get_seed)   |
| `get_step`  | 获取生成器当前的步进数       | [get_step](#get_step)   |
| `get_info`  | 获取生成器的完整信息表       | [get_info](#get_info)   |
| `exists`    | 检查生成器是否存在           | [exists](#exists)       |

---

## 常量

## `INT`

整数类型随机数生成器。

**可用于**

- 参数 `type`

### 调用

```lua
random.INT
```

### 示例

---

## `FLOAT`

浮点数类型随机数生成器。

**可用于**

- 参数 `type`

### 调用

```lua
random.FLOAT
```

### 示例

---

## 方法

## `randint`

随机生成指定区间的整数。

### 调用

```lua
-- 表参数
random.randint{}
```

### 参数

| 参数名 | 类型    | 必填 | 默认值        | 说明           |
| ------ | ------- | ---- | ------------- | -------------- |
| `min`  | integer | 否   | `-2147483648` | 区间下界（含） |
| `max`  | integer | 否   | `2147483647`  | 区间上界（含） |

### 返回

| 类型    | 说明             |
| ------- | ---------------- |
| integer | 区间内的随机整数 |

### 示例

```lua
r1 = random.randint {}
debug.print { message = r1 }

r2 = random.randint { min = 1, max = 10 }
debug.print { message = r2 }

r3 = random.randint { min = -20, max = -8 }
debug.print { message = r3 }
```

输出：

```text
1812592315
7
-9
```

### 额外补充

- 随机区间为闭区间 $[min, max]$。
- 直接生成无法控制相关参数。

---

## `randfloat`

随机生成指定区间的浮点数。

### 调用

```lua
-- 表参数
random.randfloat{}
```

### 参数

| 参数名 | 类型             | 必填 | 默认值 | 说明           |
| ------ | ---------------- | ---- | ------ | -------------- |
| `min`  | integer / float | 否   | `0`    | 区间下界（含） |
| `max`  | integer / float | 否   | `1`    | 区间上界（含） |

### 返回

| 类型   | 说明               |
| ------ | ------------------ |
| float | 区间内的随机浮点数 |

### 示例

```lua
r1 = random.randfloat {}
debug.print { message = r1 }

r2 = random.randfloat { min = 1, max = 10 }
debug.print { message = r2 }

r3 = random.randfloat { min = -20, max = -8 }
debug.print { message = r3 }
```

输出：

```text
0.7669397909242387
7.324927707931401
-13.663908207215046
```

### 额外补充

- 随机区间为闭区间 $[min, max]$。
- 直接生成无法控制相关参数。

---

## `create`

创建一个随机数生成器对象。

### 调用

```lua
-- 表参数
random.create{}
```

### 参数

| 参数名 | 类型             | 必填 | 默认值         | 说明           |
| ------ | ---------------- | ---- | -------------- | -------------- |
| `min`  | float / integer | 否   | 跟随生成器类型 | 区间下界（含） |
| `max`  | float / integer | 否   | 跟随生成器类型 | 区间上界（含） |
| `type` | const-random     | 否   | `random.INT`   | 生成器类型     |
| `seed` | integer          | 否   | 系统随机生成   | 随机种子       |
| `step` | integer          | 否   | `0`            | 初始步进数     |

### 返回

| 类型   | 说明      |
| ------ | --------- |
| string | 生成器 ID |

### 示例

```lua
r1 = random.create {}
debug.print { message = r1 }
debug.print { message = random.generate(r1) }

r2 = random.create { type = random.INT, min = 1, max = 30, seed = 520 }
debug.print { message = r2 }
debug.print { message = random.generate(r2) }


debug.print { message = table.pretty(random.list()) }
```

输出：

```lua
rng_001
1277268617
rng_002
14
{
  [1] = {
    id = "rng_001",
    max = 2147483647,
    min = -2147483648,
    seed = 1789000067301558980,
    step = 1,
    type = "int"
  },
  [2] = {
    id = "rng_002",
    max = 30,
    min = 1,
    seed = 520,
    step = 1,
    type = "int"
  },
  n = 2
}
```

### 额外补充

- 随机区间为闭区间 $[min, max]$。
- 参数 `type` 为 `random.INT` 时，参数 `min` 默认值为 `-2147483648`，参数 `max` 默认值为 `2147483647`。
- 参数 `type` 为 `random.FLOAT` 时，参数 `min` 默认值为 `0`，参数 `max` 默认值为 `1`。

---

## `delete`

删除指定生成器。

### 调用

```lua
-- 单参数
random.delete()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明      |
| ------ | ------ | ---- | ------ | --------- |
| `id`   | string | 是   | -      | 生成器 ID |

### 返回

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否删除成功 |

### 示例

```lua
r = random.create {}
debug.print { message = r }
debug.print { message = random.generate(r) }

debug.print { message = random.delete(r) }


debug.print { message = table.pretty(random.list()) }
```

输出：

```lua
rng_001
553823262
true
{
  n = 0
}
```

---

## `clear`

删除所有生成器。

### 调用

```lua
-- 单参数
random.clear()
```

### 参数

无。

### 返回

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否删除成功 |

### 示例

```lua
random.create {}
random.create {}
random.create {}
random.create {}

debug.print { message = random.clear() }

debug.print { message = table.pretty(random.list()) }
```

输出：

```lua
true
{
  n = 0
}
```

---

## `list`

获取所有生成器的信息。

### 调用

```lua
-- 单参数
random.list()
```

### 参数

无。

### 返回

返回一个混合表。

| 类型  | 说明           |
| ----- | -------------- |
| table | 所有生成器信息 |

### 示例

```lua
random.create {}
random.create {}

debug.print { message = table.pretty(random.list()) }
```

输出：

```lua
{
  [1] = {
    id = "rng_001",
    max = 2147483647,
    min = -2147483648,
    seed = 1789001961255729648,
    step = 0,
    type = "int"
  },
  [2] = {
    id = "rng_002",
    max = 2147483647,
    min = -2147483648,
    seed = 1789001959106153348,
    step = 0,
    type = "int"
  },
  n = 2
}
```

### 额外补充

- 返回值混合表结构如下：

```lua
{
  {
    id = ...,   -- string
    type = ..., -- string
    min = ...,  -- float / integer
    max = ...,  -- float / integer
    seed = ..., -- integer,
    step = ..., -- integer
  },
  ...
  n = x,        -- integer
} -- 共有 x+1 个元素，所有返回值连续排序，最后 n 为返回值个数
```

---

## `count`

返回当前生成器的总数。

### 调用

```lua
-- 单参数
random.count()
```

### 参数

无。

### 返回

直接返回一个值。

| 类型    | 说明       |
| ------- | ---------- |
| integer | 生成器总数 |

### 示例

```lua
random.create {}
random.create {}
random.create {}
random.create {}
random.create {}

debug.print { message = random.count() }
```

输出：

```lua
5
```

---

## `generate`

使用指定生成器生成一个随机数。

### 调用

```lua
-- 单参数
random.generate()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明      |
| ------ | ------ | ---- | ------ | --------- |
| `id`   | string | 是   | -      | 生成器 ID |

### 返回

直接返回一个值。

| 类型             | 说明         |
| ---------------- | ------------ |
| integer / float | 生成的随机数 |

### 示例

```lua
r = random.create { min = -5, max = 30 }

debug.print { message = random.generate(r) }
debug.print { message = random.generate(r) }
debug.print { message = random.generate(r) }
debug.print { message = random.generate(r) }
debug.print { message = random.generate(r) }
```

输出：

```text
12
18
7
12
6
```

---

## `set`

修改生成器的参数。

### 调用

```lua
-- 表参数
random.set{}
```

### 参数

| 参数名 | 类型             | 必填 | 默认值   | 说明           |
| ------ | ---------------- | ---- | -------- | -------------- |
| `id`   | string           | 是   | -        | 生成器 ID      |
| `type` | const-random     | 否   | 保持原值 | 生成器类型     |
| `min`  | float / integer | 否   | 保持原值 | 区间下界（含） |
| `max`  | float / integer | 否   | 保持原值 | 区间上界（含） |
| `seed` | integer          | 否   | 保持原值 | 随机种子       |
| `step` | integer          | 否   | 保持原值 | 步进数         |

### 返回

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否修改成功 |

### 示例

```lua
r = random.create {}

debug.print { message = table.pretty(random.get_info(r)) }

random.set { id = r,  type = random.FLOAT, min = 3.2, max = 5.8, seed = 123456 }

debug.print { message = table.pretty(random.get_info(r)) }
```

输出：

```lua
{
  id = "rng_001", 
  max = 2147483647, 
  min = -2147483648, 
  seed = 1789002380334941420, 
  step = 0, 
  type = "int"
}
{
  id = "rng_001", 
  max = 5.8, 
  min = 3.2, 
  seed = 123456, 
  step = 0, 
  type = "float"
}
```

### 额外补充

- 随机区间为闭区间 $[min, max]$。

---

## `set_type`

修改生成器的类型。

### 调用

```lua
-- 表参数
random.set_type{}
```

### 参数

| 参数名 | 类型         | 必填 | 默认值   | 说明       |
| ------ | ------------ | ---- | -------- | ---------- |
| `id`   | string       | 是   | -        | 生成器 ID  |
| `type` | const-random | 否   | 保持原值 | 生成器类型 |

### 返回

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否修改成功 |

### 示例

```lua

```

输出;

```text
int
float
```

---

## `set_range`

修改生成器的随机区间。

### 调用

```lua
-- 表参数
random.set_range{}
```

### 参数

| 参数名 | 类型             | 必填 | 默认值   | 说明           |
| ------ | ---------------- | ---- | -------- | -------------- |
| `id`   | string           | 是   | -        | 生成器 ID      |
| `min`  | float / integer | 是   | 保持原值 | 区间下界（含） |
| `max`  | float / integer | 是   | 保持原值 | 区间上界（含） |

### 返回

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否修改成功 |

### 示例

```lua
r = random.create { min = 10, max = 20 }

debug.print { message = table.pretty(random.get_range(r)) }

random.set_range { id = r, min = 5, max = 7 }

debug.print { message = table.pretty(random.get_range(r)) }
```

输出;

```lua
{
  max = 20, 
  min = 10
}
{
  max = 7, 
  min = 5
}
```

### 额外补充

- 随机区间为闭区间 $[min, max]$。

---

## `set_seed`

修改生成器的种子。

### 调用

```lua
-- 表参数
random.set_seed{}
```

### 参数

| 参数名 | 类型    | 必填 | 默认值   | 说明      |
| ------ | ------- | ---- | -------- | --------- |
| `id`   | string  | 是   | -        | 生成器 ID |
| `seed` | integer | 否   | 保持原值 | 随机种子  |

### 返回

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否修改成功 |

### 示例

```lua
r = random.create {}

debug.print { message = random.get_info(r).seed }

random.set_seed { id = r,  seed = 1314 }

debug.print { message = random.get_info(r).seed }
```

输出;

```text
1789004668875999436
1314
```

---

## `set_step`

修改生成器的步进数。

### 调用

```lua
-- 表参数
random.set_step{}
```

### 参数

| 参数名 | 类型    | 必填 | 默认值   | 说明      |
| ------ | ------- | ---- | -------- | --------- |
| `id`   | string  | 是   | -        | 生成器 ID |
| `step` | integer | 是   | 保持原值 | 步进数    |

### 返回

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否修改成功 |

### 示例

```lua
r = random.create {}

debug.print { message = random.get_info(r).step }

random.set_step { id = r,  step = 30 }

debug.print { message = random.get_info(r).step }
```

输出;

```text
0
30
```

---

## `get_type`

获取生成器的类型。

### 调用

```lua
-- 单参数
random.get_type()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明      |
| ------ | ------ | ---- | ------ | --------- |
| `id`   | string | 是   | -      | 生成器 ID |

### 返回

直接返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| string | 生成器类型 |

### 示例

```lua
r = random.create { type = random.INT }
debug.print { message = random.get_type(r) }
```

输出;

```text
int
```

---

## `get_range`

获取生成器的随机区间。

### 调用

```lua
-- 单参数
random.get_range()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明      |
| ------ | ------ | ---- | ------ | --------- |
| `id`   | string | 是   | -      | 生成器 ID |

### 返回

返回一个对象表。

| 字段  | 类型             | 说明     |
| ----- | ---------------- | -------- |
| `min` | float / integer | 区间下界 |
| `max` | float / integer | 区间上界 |

### 示例

```lua
r = random.create { min = 10, max = 20 }
debug.print { message = table.pretty(random.get_range(r)) }
```

输出;

```lua
{
  max = 20, 
  min = 10
}
```

---

## `get_seed`

获取生成器的种子。

### 调用

```lua
-- 单参数
random.get_seed()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明      |
| ------ | ------ | ---- | ------ | --------- |
| `id`   | string | 是   | -      | 生成器 ID |

### 返回

直接返回一个值。

| 类型    | 说明     |
| ------- | -------- |
| integer | 随机种子 |

### 示例

```lua
r = random.create { seed = 2233 }
debug.print { message = random.get_seed(r) }
```

输出;

```text
2233
```

---

## `get_step`

获取生成器当前的步进数。

### 调用

```lua
-- 单参数
random.get_step()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明      |
| ------ | ------ | ---- | ------ | --------- |
| `id`   | string | 是   | -      | 生成器 ID |

### 返回

| 类型    | 说明       |
| ------- | ---------- |
| integer | 当前步进数 |

### 示例

```lua
r = random.create { step = 50 }
debug.print { message = random.get_step(r) }
```

输出;

```text
50
```

---

## `get_info`

获取生成器的完整信息表。

### 调用

```lua
-- 单参数
random.get_info()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明      |
| ------ | ------ | ---- | ------ | --------- |
| `id`   | string | 是   | -      | 生成器 ID |

### 返回

返回一个对象表。

| 字段   | 类型             | 说明         |
| ------ | ---------------- | ------------ |
| `id`   | string           | 生成器 ID    |
| `type` | string           | 生成器类型   |
| `min`  | float / integer | 区间下界     |
| `max`  | float / integer | 区间上界     |
| `seed` | integer          | 生成器种子   |
| `step` | integer          | 生成及步进数 |

### 示例

```lua
r = random.create {}
debug.print { message = table.pretty(random.get_info(r)) }
```

输出;

```lua
{
  id = "rng_001", 
  max = 2147483647, 
  min = -2147483648, 
  seed = 1789005094764332964, 
  step = 0, 
  type = "int"
}
```

---

## `exists`

检查生成器是否存在。

### 调用

```lua
-- 单参数
random.exists()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明      |
| ------ | ------ | ---- | ------ | --------- |
| `id`   | string | 是   | -      | 生成器 ID |

### 返回

| 类型    | 说明     |
| ------- | -------- |
| boolean | 是否存在 |

### 示例

```lua
r = random.create {}
debug.print { message = random.exists(r) }

random.delete(r)
debug.print { message = random.exists(r) }
```

输出;

```text
true
false
```
