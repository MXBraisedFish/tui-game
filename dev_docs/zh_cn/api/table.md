# table 库

## 基本库说明

`table` 提供表操作。

## 目录

### 方法

| 方法名        | 说明                                           | 索引                        |
| ------------- | ---------------------------------------------- | --------------------------- |
| `concat`      | 拼接数组表中的元素                             | [concat](#concat)           |
| `insert`      | 在指定位置插入一个元素，并将后续元素后移       | [insert](#insert)           |
| `move`        | 将表中的指定范围元素复制并覆盖到目标索引       | [move](#move)               |
| `pack`        | 将数组表打包为新的表，并记录原始数组表元素数量 | [pack](#pack)               |
| `unpack`      | 展开数组表                                     | [unpack](#unpack)           |
| `remove`      | 删除指定位置的一个元素，并将后续元素前移       | [remove](#remove)           |
| `sort`        | 排序数组表                                     | [sort](#sort)               |
| `deepcopy`    | 深拷贝表                                       | [deepcopy](#deepcopy)       |
| `pretty`      | 将表转换为有可读性的字符串                     | [pretty](#pretty)           |
| `count`       | 查询表中真实的元素数量                         | [count](#count)             |
| `count_array` | 查询表中真实的数组元素数量及下标               | [count_array](#count_array) |
| `count_hash`  | 查询表中真实的哈希键数量                       | [count_hash](#count_hash)   |
| `compact`     | 将数组部分前压为从 1 开始的连续排列            | [compact](#compact)         |

## 方法

## `concat`

拼接数组表中的元素。

### 调用

```lua
-- 表参数
table.concat{}
```

### 参数

| 参数名   | 类型    | 必填 | 默认值     | 说明               |
| -------- | ------- | ---- | ---------- | ------------------ |
| `table`  | table   | 是   | -          | 源数组表           |
| `sep`    | string  | 否   | `""`       | 相邻元素间的分隔符 |
| `start`  | integer | 否   | `1`        | 起始索引           |
| `finish` | integer | 否   | 数组表长度 | 结束索引           |

### 返回

直接返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| string | 拼接结果 |

### 示例

```lua
local t1 = { "apple", "banana", "grape" }
debug.print { message = table.concat { table = t1 } }

local t2 = { "a", "b", "c" }
debug.print { message = table.concat { table = t2, sep = " | " } }
```

输出：

```text
applebananagrape
a | b | c
```

---

## `insert`

在指定位置插入一个元素，并将后续元素后移。

### 调用

```lua
-- 表参数
table.insert{}
```

### 参数

| 参数名     | 类型    | 必填 | 默认值    | 说明       |
| ---------- | ------- | ---- | --------- | ---------- |
| `table`    | table   | 是   | -         | 目标表     |
| `value`    | any     | 是   | -         | 要插入的值 |
| `position` | integer | 否   | 末尾 `+1` | 插入位置   |

### 返回

无。

### 示例

```lua
local t1 = { "x", "y" }
table.insert { table = t1, value = "z" }
debug.print { message = table.pretty(t1) .. "\n" }

local t2 = { "a", "c" }
table.insert { table = t2, value = "b", position = 2 }
debug.print { message = table.pretty(t2) }
```

输出：

```lua
{
  [1] = "x",
  [2] = "y",
  [3] = "z"
}

{
  [1] = "a",
  [2] = "b",
  [3] = "c"
}
```

---

## `move`

将数组表中的指定范围元素复制并覆盖到目标索引。

### 调用

```lua
-- 表参数
table.move{}
```

### 参数

| 参数名         | 类型    | 必填 | 默认值   | 说明         |
| -------------- | ------- | ---- | -------- | ------------ |
| `source`       | table   | 是   | -        | 源数组表     |
| `start`        | integer | 是   | -        | 起始索引     |
| `finish`       | integer | 是   | -        | 结束索引     |
| `target_index` | integer | 是   | -        | 目标起始索引 |
| `target`       | table   | 否   | 源数组表 | 目标表       |

### 返回

返回一个数组表。

| 类型  | 说明   |
| ----- | ------ |
| table | 目标表 |

### 示例

```lua
local t1 = { "a", "b", "c", "d" }
local t_m1 = table.move { source = t1, start = 2, finish = 3, target_index = 4 }
debug.print { message = table.pretty(t1) }
debug.print { message = t1 }
debug.print { message = t_m1 .. "\n" }

local t2 = { 1, 2, 3, 4, 5 }
table.move { source = t2, start = 1, finish = 2, target_index = 4 }
debug.print { message = table.pretty(t2) }
```

输出：

```lua
{
  [1] = "a",
  [2] = "b",
  [3] = "c",
  [4] = "b",
  [5] = "c"
}
table: 0x19be6539b30
table: 0x19be6539b30

{
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 1,
  [5] = 2
}
```

### 额外补充

- 该 API 实际操作为复制元素并覆盖目标位置的元素，而非剪切并移动。
- 返回值的是被修改后的源数组表地址，而非拷贝后的表。

---

## `pack`

将数组表打包为新的表，并记录原始数组表元素数量。

### 调用

```lua
-- 单参数
table.pack()
```

### 参数

| 参数名   | 类型  | 必填 | 默认值 | 说明     |
| -------- | ----- | ---- | ------ | -------- |
| `values` | table | 是   | -      | 源数组表 |

### 返回

返回一个数组表。

| 类型  | 说明   |
| ----- | ------ |
| table | 数组表 |

### 示例

```lua
local packed1 = table.pack { "a", "b", "c" }
debug.print { message = table.pretty(packed1) .. "\n" }

local packed2 = table.pack { 1, nil, 3 }
debug.print { message = table.pretty(packed2) }
```

输出：

```lua
{
  [1] = "a",
  [2] = "b",
  [3] = "c",
  n = 3
}

{
  [1] = 1,
  [3] = 3,
  n = 3
}
```

### 额外补充

- 返回值数组表结构如下：

```lua
{
  [1] = ...,
  [2] = ...,
  ...
  [x] = ...,
  n = x
} -- 共有 x+1 个元素，所有返回值连续排序，最后 n 为返回值个数
```

- `nil` 值不会被显式存储，但表结构中的 `n` 字段为源数组表中包含 `nil` 值的长度。

---

## `unpack`

展开数组表。

### 调用

```lua
-- 表参数
table.unpack{}
```

### 参数

| 参数名   | 类型    | 必填 | 默认值     | 说明     |
| -------- | ------- | ---- | ---------- | -------- |
| `table`  | table   | 是   | -          | 源数组表 |
| `start`  | integer | 否   | `1`        | 起始索引 |
| `finish` | integer | 否   | 数组表长度 | 结束索引 |

### 返回

返回多个值。

| 类型   | 说明       |
| ------ | ---------- |
| any... | 展开的元素 |

### 示例

```lua
local t1 = { "a", "b", "c" }
local a1, b1, c1 = table.unpack { table = t1 }
debug.print { message = a1 .. " " .. b1 .. " " .. c1 }

local t2 = { 10, 20, 30, 40 }
local a2, b2 = table.unpack { table = t2, start = 2 }
debug.print { message = a2 .. " " .. b2 }
```

输出：

```text
a b c
20 30
```

### 额外补充

- 该 API 返回多参数而非表。

---

## `remove`

删除指定位置的一个元素，并将后续元素前移。

### 调用

```lua
-- 表参数
table.remove{}
```

### 参数

| 参数名     | 类型    | 必填 | 默认值     | 说明     |
| ---------- | ------- | ---- | ---------- | -------- |
| `table`    | table   | 是   | -          | 目标表   |
| `position` | integer | 否   | 数组表长度 | 删除位置 |

### 返回

| 类型 | 说明         |
| ---- | ------------ |
| any  | 被删除的元素 |

### 示例

```lua
local t1 = { "a", "b", "c", "d" }
local removed1 = table.remove { table = t1 }
debug.print { message = removed1 .. " " .. table.pretty(t1) .. "\n" }

local t2 = { 10, 20, 30, 40 }
local removed2 = table.remove { table = t2, position = 2 }
debug.print { message = removed2 .. " " .. table.pretty(t2) }
```

输出：

```lua
d
{
  [1] = "a",
  [2] = "b",
  [3] = "c"
}

20
{
  [1] = 10,
  [2] = 30,
  [3] = 40
}
```

---

## `sort`

排序数组表。

### 调用

```lua
-- 表参数
table.sort{}
```

### 参数

| 参数名       | 类型     | 必填 | 默认值 | 说明       |
| ------------ | -------- | ---- | ------ | ---------- |
| `table`      | table    | 是   | -      | 目标数组表 |
| `comparator` | function | 否   | `nil`  | 比较函数   |

### 返回

无。

### 示例

```lua
local t1 = { 3, 1, 4, 2 }
table.sort { table = t1 }
debug.print { message = table.pretty(t1) .. "\n" }

local t2 = { "banana", "apple", "grape", "cherry" }
table.sort { table = t2 }
debug.print { message = table.pretty(t2) .. "\n" }

local t3 = { 5, 2, 8, 1 }
table.sort {
  table = t3,
  comparator = function(left, right)
    return left > right
  end
  }
debug.print { message = table.pretty(t3) .. "\n" }

local t4 = { "abc", "a", "abcdef", "ab" }
table.sort {
  table = t4,
  comparator = function(left, right)
    return #left < #right
  end
}
debug.print { message = table.pretty(t4) }
```

输出：

```lua
{
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 4
}

{
  [1] = "apple",
  [2] = "banana",
  [3] = "cherry",
  [4] = "grape"
}

{
  [1] = 8,
  [2] = 5,
  [3] = 2,
  [4] = 1
}

{
  [1] = "a",
  [2] = "ab",
  [3] = "abc",
  [4] = "abcdef"
}
```

### 额外补充

- 参数 `comparator` 函数结构如下：

```lua
function(left, right)
  -- 比较处理逻辑
  return boolean
end
```

- 参数 `comparator` 函数返回值为 `true` 时，表示 `left` 排在 `right` 之前；返回值为 `false` 时，表示 `left` 排在 `right` 之后。

---

## `deepcopy`

深拷贝表。

### 调用

```lua
-- 单参数
table.deepcopy()
```

### 参数

| 参数名  | 类型  | 必填 | 默认值 | 说明   |
| ------- | ----- | ---- | ------ | ------ |
| `table` | table | 是   | -      | 目标表 |

### 返回

返回一个混合表。

| 类型  | 说明           |
| ----- | -------------- |
| table | 深拷贝后的新表 |

### 示例

```lua
local t = { 1, 2, 3 }
local t_copy = table.deepcopy(t)

debug.print { message = tostring(t) }
debug.print { message = tostring(t_copy) }
```

输出：

```text
table: 0x24076b69b50  -- 两个表地址不同，非引用。
table: 0x24076b6a190
```

---

## `pretty`

将表转换为有可读性的字符串。

### 调用

```lua
-- 单参数
table.pretty()
```

### 参数

| 参数名  | 类型  | 必填 | 默认值 | 说明   |
| ------- | ----- | ---- | ------ | ------ |
| `table` | table | 是   | -      | 目标表 |

### 返回

直接返回一个值。

| 类型   | 说明           |
| ------ | -------------- |
| string | 转换后的字符串 |

### 示例

```lua
local t = { "apple", "banana", "grape" }
debug.print { message = table.pretty(t) }
```

输出：

```lua
{
  [1] = "apple",
  [2] = "banana",
  [3] = "grape"
}
```

---

## `count`

查询表中真实存在的元素数量。

### 调用

```lua
-- 单参数
table.count()
```

### 参数

| 参数名  | 类型  | 必填 | 默认值 | 说明   |
| ------- | ----- | ---- | ------ | ------ |
| `table` | table | 是   | -      | 目标表 |

### 返回

返回一个对象表。

| 字段         | 类型    | 说明                              |
| ------------ | ------- | --------------------------------- |
| `n`          | integer | 表中真实存在的全部元素数量        |
| `contiguous` | boolean | 数组部分是否从下标 1 开始连续排列 |

### 示例

```lua
local t = { [1] = "a", [3] = "c", name = "Tui Game" }
local result = table.count(t)

debug.print { message = result.n }
debug.print { message = result.contiguous }
```

输出：

```text
3
false
```

### 额外补充

- 空数组的返回值 `contiguous` 为 `true`。
- 值为 `nil` 的键在 Lua 表中表示该键不存在，因此不会计数。

---

## `count_array`

查询表中真实存在的数组元素数量。

### 调用

```lua
-- 单参数
table.count_array()
```

### 参数

| 参数名  | 类型  | 必填 | 默认值 | 说明   |
| ------- | ----- | ---- | ------ | ------ |
| `table` | table | 是   | -      | 目标表 |

### 返回

返回一个对象表。

| 字段         | 类型    | 说明                              |
| ------------ | ------- | --------------------------------- |
| `n`          | integer | 真实存在的数组元素数量            |
| `contiguous` | boolean | 数组部分是否从下标 1 开始连续排列 |
| `indexes`    | table   | 所有有效数组下标组成的升序数组表  |

### 示例

```lua
local t = { [1] = "a", [3] = "c", name = "Tui Game" }
local result = table.count_array { table = t }

debug.print { message = result.n }
debug.print { message = result.contiguous }
debug.print { message = table.pretty(result.indexes) }
```

输出：

```lua
2
false
{
  [1] = 1, 
  [2] = 3
}
```

### 额外补充

- `indexes` 按下标从小到大排序。

---

## `count_hash`

查询表中真实存在的哈希项数量。

### 调用

```lua
-- 单参数
table.count_hash()
```

### 参数

| 参数名  | 类型  | 必填 | 默认值 | 说明   |
| ------- | ----- | ---- | ------ | ------ |
| `table` | table | 是   | -      | 目标表 |

### 返回

直接返回一个值。

| 类型    | 说明                 |
| ------- | -------------------- |
| integer | 真实存在的哈希项数量 |

### 示例

```lua
local t = { [1] = "a", [3] = "c", name = "Tui Game", [0] = "zero" }
local result = table.count_hash(t)

debug.print { message = result }
```

输出：

```text
2
```

---

## `compact`

压实目标表的数组部分，将所有数组元素前压至从下标 1 开始连续排列。

### 调用

```lua
-- 单参数
table.compact()
```

### 参数

| 参数名  | 类型  | 必填 | 默认值 | 说明       |
| ------- | ----- | ---- | ------ | ---------- |
| `table` | table | 是   | -      | 要压实的表 |

### 返回

直接返回一个值。

| 类型  | 说明                         |
| ----- | ---------------------------- |
| table | 压实后的原始表，与参数表相同 |

### 示例

```lua
local t = { [1] = "a", [3] = "c", [8] = "h", name = "Tui Game" }
local result = table.compact { table = t }

debug.print { message = tostring(result == t) }
debug.print { message = table.pretty(t) }
```

输出：

```lua
true
{
  [1] = "a", 
  [2] = "c", 
  [3] = "h", 
  name = "Tui Game"
}
```

### 额外补充

- 数组元素按照压实前的下标升序排列，元素之间的相对顺序不会改变。
- 哈希项不会被删除或移动。
- 该方法直接修改原始表，不会创建新表。
