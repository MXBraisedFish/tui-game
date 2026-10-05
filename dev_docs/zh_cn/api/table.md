# table 库

`table` 提供表的拼接、插入、排序、复制和统计等操作。

---

# 目录

## 方法

| 方法          | 说明                                                            | 定位                        |
| ------------- | --------------------------------------------------------------- | --------------------------- |
| `concat`      | 拼接数组表中的元素                                              | [concat](#concat)           |
| `insert`      | 在指定位置插入一个元素，并将后续元素后移                        | [insert](#insert)           |
| `move`        | 将数组表中的指定范围元素复制并覆盖到目标索引                    | [move](#move)               |
| `pack`        | 将所有变参打包为新的表，并记录变参数量                          | [pack](#pack)               |
| `unpack`      | 将数组表元素作为多个返回值展开                                  | [unpack](#unpack)           |
| `remove`      | 删除指定位置的一个元素，并将后续元素前移                        | [remove](#remove)           |
| `sort`        | 排序数组表                                                      | [sort](#sort)               |
| `deepcopy`    | 深拷贝表                                                        | [deepcopy](#deepcopy)       |
| `pretty`      | 将表转换为有可读性的字符串                                      | [pretty](#pretty)           |
| `count`       | 查询表中真实存在的元素数量                                      | [count](#count)             |
| `count_array` | 查询表中真实存在的数组元素数量                                  | [count_array](#count_array) |
| `count_hash`  | 查询表中真实存在的哈希项数量                                    | [count_hash](#count_hash)   |
| `compact`     | 压实目标表的数组部分，将所有数组元素前压至从下标 1 开始连续排列 | [compact](#compact)         |

---

# 方法

## `concat`

拼接数组表中的元素。

### 调用

```lua
table.concat
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `list` | table | 源数组表 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `sep` | string | `""` | 相邻元素间的分隔符 |
| `i` | integer | `1` | 起始索引 |
| `j` | integer | `#list` | 结束索引 |

## 返回值

返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| string | 拼接结果 |

### 示例

```lua
local t1 = { "apple", "banana", "grape" }
debug.print(table.concat(t1))

local t2 = { "a", "b", "c" }
debug.print(table.concat(t2, { sep = " | " }))
```

**输出：**

```lua
```

## 额外说明

- `concat`、`insert`、`move`、`unpack`、`remove`、`sort` 的必填参数按顺序传入，选填参数放在末尾选项表中；未知字段、错误类型和多余位置参数会报错。
- `table` 库和其它库的 API 表本身只读；`insert`、`remove`、`sort`、`move`、`compact` 不能以只读 API 表作为写入目标，`count`、`count_array`、`count_hash`、`pretty`、`deepcopy` 则可以传入。

---

## `insert`

在指定位置插入一个元素，并将后续元素后移。

### 调用

```lua
table.insert
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `list` | table | 目标表 |
| `value` | any | 要插入的值；追加形式为 `table.insert(list, value)` |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `pos` | integer | `#list + 1` | 插入位置 |

## 返回值

无返回值。

### 示例

```lua
local t1 = { "x", "y" }
table.insert(t1, "z")
debug.print(table.pretty(t1) .. "\n")

local t2 = { "a", "c" }
table.insert(t2, "b", { pos = 2 })
debug.print(table.pretty(t2))
```

**输出：**

```lua
```

---

## `move`

将数组表中的指定范围元素复制并覆盖到目标索引。

### 调用

```lua
table.move
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `src` | table | 源表 |
| `first` | integer | 要复制范围的起始索引 |
| `last` | integer | 要复制范围的结束索引 |
| `target_start` | integer | 目标表中的起始索引 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `target` | table | `src` | 接收复制内容的表 |

## 返回值

返回目标表 `target`。

| 类型  | 说明   |
| ----- | ------ |
| table | 目标表 |

### 示例

```lua
local t1 = { "a", "b", "c", "d" }
local t_m1 = table.move(t1, 2, 3, 4)
debug.print(table.pretty(t1))
debug.print(tostring(t_m1 == t1))
debug.print(table.pretty(t_m1) .. "\n")

local t2 = { 1, 2, 3, 4, 5 }
table.move(t2, 1, 2, 4)
debug.print(table.pretty(t2))
```

**输出：**

```lua
```

## 额外说明

- 该 API 实际操作为复制元素并覆盖目标位置的元素，而非剪切并移动。
- 未给 `target` 时，返回源表 `src`。

---

## `pack`

将所有变参打包为新的表，并记录变参数量。

### 调用

```lua
table.pack
```

## 返回值

返回一个混合表。

| 类型  | 说明       |
| ----- | ---------- |
| table | 打包结果表 |

### 示例

```lua
local packed1 = table.pack("a", "b", "c")
debug.print(table.pretty(packed1) .. "\n")

local packed2 = table.pack(1, nil, 3)
debug.print(table.pretty(packed2))
```

**输出：**

```lua
```

## 额外说明

- 要保存的值依次传入，嵌套表按引用保留（不深拷贝）；没有值时使用 `table.pack()`。

- 返回值混合表结构如下：

```lua
local packed = {
  [1] = 1,
  [3] = 3,
  n = 3,
}
```

- `nil` 值不会被显式存储，但 `n` 保留所有参数位置，包括末尾的 `nil`。

---

## `unpack`

将数组表元素作为多个返回值展开。

### 调用

```lua
table.unpack
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `list` | table | 源数组表 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `i` | integer | `1` | 起始索引 |
| `j` | integer | `#list` | 结束索引 |

## 返回值

返回多个值。

| 类型   | 说明       |
| ------ | ---------- |
| any... | 展开的元素 |

### 示例

```lua
local t1 = { "a", "b", "c" }
local a1, b1, c1 = table.unpack(t1)
debug.print(a1 .. " " .. b1 .. " " .. c1)

local t2 = { 10, 20, 30, 40 }
local a2, b2 = table.unpack(t2, { i = 2 })
debug.print(a2 .. " " .. b2)
```

**输出：**

```lua
```

## 额外说明

- 该 API 返回多参数而非表。

---

## `remove`

删除指定位置的一个元素，并将后续元素前移。

### 调用

```lua
table.remove
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `list` | table | 目标表 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `pos` | integer | `#list` | 删除位置 |

## 返回值

返回一个值。

| 类型 | 说明         |
| ---- | ------------ |
| any  | 被删除的元素 |

### 示例

```lua
local t1 = { "a", "b", "c", "d" }
local removed1 = table.remove(t1)
debug.print(removed1 .. " " .. table.pretty(t1) .. "\n")

local t2 = { 10, 20, 30, 40 }
local removed2 = table.remove(t2, { pos = 2 })
debug.print(removed2 .. " " .. table.pretty(t2))
```

**输出：**

```lua
```

---

## `sort`

排序数组表。

### 调用

```lua
table.sort
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `list` | table | 目标数组表 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `comp` | function | `nil` | 比较函数 |

## 返回值

无返回值。

### 示例

```lua
local t1 = { 3, 1, 4, 2 }
table.sort(t1)
debug.print(table.pretty(t1) .. "\n")

local t2 = { "banana", "apple", "grape", "cherry" }
table.sort(t2)
debug.print(table.pretty(t2) .. "\n")

local t3 = { 5, 2, 8, 1 }
table.sort(t3, { comp = function(left, right)
  return left > right
end })
debug.print(table.pretty(t3) .. "\n")

local t4 = { "abc", "a", "abcdef", "ab" }
table.sort(t4, { comp = function(left, right)
  return #left < #right
end })
debug.print(table.pretty(t4))
```

**输出：**

```lua
```

## 额外说明

- 目标数组表最多 4096 个元素，超出会报错。
- 参数 `comp` 函数返回值为 `true` 时，表示 `left` 排在 `right` 之前；返回值为 `false` 时，表示不要求 `left` 排在 `right` 前面（也可能两者相等）。

---

## `deepcopy`

深拷贝表。

### 调用

```lua
table.deepcopy
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `table` | table | 目标表 |

## 返回值

返回一个混合表。

| 类型  | 说明           |
| ----- | -------------- |
| table | 深拷贝后的新表 |

### 示例

```lua
local t = { 1, 2, 3, child = { value = 7 } }
local t_copy = table.deepcopy(t)
t_copy.child.value = 9

debug.print(tostring(t_copy ~= t))
debug.print(t.child.value)
```

**输出：**

```lua
```

---

## `pretty`

将表转换为有可读性的字符串。

### 调用

```lua
table.pretty
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `table` | table | 目标表 |

## 返回值

返回一个值。

| 类型   | 说明           |
| ------ | -------------- |
| string | 转换后的字符串 |

### 示例

```lua
local t = { "apple", "banana", "grape" }
debug.print(table.pretty(t))
```

**输出：**

```lua
```

---

## `count`

查询表中真实存在的元素数量。

### 调用

```lua
table.count
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `table` | table | 目标表 |

## 返回值

返回两个值，依次为表中元素总数和数组部分是否连续。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `count` | integer | 表中真实存在的元素数量 |
| `contiguous` | boolean | 数组部分是否从下标 1 开始连续排列 |

### 示例

```lua
local t = { [1] = "a", [3] = "c", name = "Tui Game" }
local count, contiguous = table.count(t)

debug.print(count)
debug.print(contiguous)
```

**输出：**

```lua
```

## 额外说明

- 数组部分为空时（包括空表和只有哈希项的表），`contiguous` 为 `true`。
- 下标为 0 或负整数的项按哈希项计数，不计入数组元素数量。
- 值为 `nil` 的键在 Lua 表中表示该键不存在，因此不会计数。

---

## `count_array`

查询表中真实存在的数组元素数量。

### 调用

```lua
table.count_array
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `table` | table | 目标表 |

## 返回值

返回三个值，依次为数组元素数量、数组部分是否连续和有效下标表。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `count` | integer | 真实存在的数组元素数量 |
| `contiguous` | boolean | 数组部分是否从下标 1 开始连续排列 |
| `indexes` | table | 所有有效数组下标组成的升序数组表 |

### 示例

```lua
local t = { [1] = "a", [3] = "c", name = "Tui Game" }
local count, contiguous, indexes = table.count_array(t)

debug.print(count)
debug.print(contiguous)
debug.print(table.pretty(indexes))
```

**输出：**

```lua
```

## 额外说明

- `indexes` 按下标从小到大排序。

---

## `count_hash`

查询表中真实存在的哈希项数量。

### 调用

```lua
table.count_hash
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `table` | table | 目标表 |

## 返回值

返回一个值。

| 类型    | 说明                 |
| ------- | -------------------- |
| integer | 真实存在的哈希项数量 |

### 示例

```lua
local t = { [1] = "a", [3] = "c", name = "Tui Game", [0] = "zero" }
local result = table.count_hash(t)

debug.print(result)
```

**输出：**

```lua
```

---

## `compact`

压实目标表的数组部分，将所有数组元素前压至从下标 1 开始连续排列。

### 调用

```lua
table.compact
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `table` | table | 要压实的表 |

## 返回值

返回一个值。

| 类型  | 说明         |
| ----- | ------------ |
| table | 压实后的原表 |

### 示例

```lua
local t = { [1] = "a", [3] = "c", [8] = "h", name = "Tui Game" }
local result = table.compact(t)

debug.print(tostring(result == t))
debug.print(table.pretty(t))
```

**输出：**

```lua
```

## 额外说明

- 数组元素按照压实前的下标升序排列，元素之间的相对顺序不会改变。
- 哈希项不会被删除或移动。
- 该方法直接修改原始表，不会创建新表。
