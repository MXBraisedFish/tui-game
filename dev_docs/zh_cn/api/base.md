# base 库

## 基本库说明

`base` 提供 Lua 基础操作。

---

## 目录

### 方法

| 方法名     | 说明                                                  | 索引                  |
| ---------- | ----------------------------------------------------- | --------------------- |
| `ipairs`   | 遍历表中的连续整数索引元素                            | [ipairs](#ipairs)     |
| `pairs`    | 遍历表中的全部键值对                                  | [pairs](#pairs)       |
| `next`     | 获取表中指定索引之后的下一个键值对                    | [next](#next)         |
| `select`   | 获取表中指定位置开始的连续元素，或查询连续元素数量    | [select](#select)     |
| `rawequal` | 比较两个值是否直接相等，不触发元方法                  | [rawequal](#rawequal) |
| `rawlen`   | 字符串返回其 UTF-8 字节数；表返回其连续数组部分的长度 | [rawlen](#rawlen)     |
| `tonumber` | 将值转换为数字，可指定进制                            | [tonumber](#tonumber) |
| `tostring` | 将任意值安全的转换为字符串                            | [tostring](#tostring) |
| `type`     | 返回值的类型名                                        | [type](#type)         |

---

## 方法

## `ipairs`

遍历表中的连续整数索引元素。

### 调用

```lua
-- 单参数
ipairs()
```

### 参数

| 参数    | 类型  | 必填 | 默认值 | 说明       |
| ------- | ----- | ---- | ------ | ---------- |
| `table` | table | 是   | -      | 要遍历的表 |

### 返回

直接返回一个值。

| 类型     | 说明       |
| -------- | ---------- |
| function | 迭代器函数 |

**迭代器函数**，返回一个对象表。

| 字段    | 类型    | 说明 |
| ------- | ------- | ---- |
| `index` | integer | 索引 |
| `value` | any     | 值   |

### 示例

```lua
t = {"a", "b", "c", [10] = "x"}

for item in ipairs(t) do
	debug.print {message = item.index .. " " .. item.value}
end
```

输出：

```text
1 a
2 b
3 c
```

---

## `pairs`

遍历表中的全部键值对。

### 调用

```lua
-- 单参数
pairs()
```

### 参数

| 参数    | 类型  | 必填 | 默认值 | 说明       |
| ------- | ----- | ---- | ------ | ---------- |
| `table` | table | 是   | -      | 要遍历的表 |

### 返回

直接返回一个值。

| 类型     | 说明       |
| -------- | ---------- |
| function | 迭代器函数 |

**迭代器函数**，返回一个对象表。

| 字段    | 类型             | 说明 |
| ------- | ---------------- | ---- |
| `index` | integer / string | 索引 |
| `value` | any              | 值   |

### 示例

```lua
t = { "a", "b", x = 1 }

for item in pairs(t) do
	debug.print {message = tostring(item.index) .. " " .. tostring(item.value)}
end
```

输出：

```text
1 a
2 b
x 1
```

---

## `next`

获取表中指定索引之后的下一个键值对。

### 调用

```lua
-- 表参数
next{}
```

### 参数

| 参数    | 类型          | 必填 | 默认值 | 说明                                              |
| ------- | ------------- | ---- | ------ | ------------------------------------------------- |
| `table` | table         | 是   | -      | 要查询的表                                        |
| `index` | integer / nil | 否   | `nil`  | 当前索引；省略或传入 `nil` 时从表的第一个元素开始 |

### 返回

**找到下一个元素时**，返回一个对象表。

| 字段    | 类型             | 说明           |
| ------- | ---------------- | -------------- |
| `value` | any              | 下一个值       |
| `index` | integer / string | 下一个值的索引 |

**没有后续元素时**，直接返回一个值。

| 类型 | 说明       |
| ---- | ---------- |
| nil  | 无后续元素 |

### 示例

```lua
t = { "a", "b", x = 1 }
index = nil

while true do
	item = next { table = t, index = index }

	-- 没有后续元素时结束遍历
	if item == nil then
		break
	end

	debug.print { message = tostring(item.index) .. " " .. tostring(item.value) }

	-- 使用当前索引继续查找下一个元素
	index = item.index
end
```

输出：

```text
1 a
2 b
x 1
```

---

### `select`

获取表中指定位置开始的连续元素，或查询连续元素数量。

### 调用

```lua
-- 表参数
select{}
```

### 参数

| 参数     | 类型            | 必填 | 默认值 | 说明                                        |
| -------- | --------------- | ---- | ------ | ------------------------------------------- |
| `index`  | integer / `"#"` | 是   | -      | 索引起始位置；传入 `"#"` 时查询连续元素数量 |
| `values` | table           | 是   | -      | 要查询的表                                  |

### 返回

**参数 `index` 为 `"#"` 时**，直接返回一个值。

| 类型    | 说明               |
| ------- | ------------------ |
| integer | 表中连续元素的数量 |

**参数 `index` 为 integer 时**，从指定索引开始，依次返回连续存在的元素值。

| 类型   | 说明                                                   |
| ------ | ------------------------------------------------------ |
| any... | 从指定位置开始的连续元素；没有可返回的元素时返回 `nil` |

### 示例

```lua
t = { "a", "b", "c" , x = 1, [5] = "d" }

count = select { index = "#", values = t }
debug.print {message = tostring(count)}

value1, value2 = select { index = 2, values = t }
debug.print { message = tostring(value1) .. " " .. tostring(value2) }

value3 = select { index = -1, values = t }
debug.print { message = tostring(value3) }

value4 = select { index = 4, values = t}
debug.print { message = tostring(value4) }
```

输出：

```text
2
b c
c
nil
```

### 额外补充

- 参数 `index` 为 integer 时，该 API 返回多参数而非表。

---

### `rawequal`

比较两个值是否直接相等，不触发元方法。

### 调用

```lua
-- 表参数
rawequal{}
```

### 参数

| 参数    | 类型 | 必填 | 默认值 | 说明     |
| ------- | ---- | ---- | ------ | -------- |
| `left`  | any  | 是   | -      | 左操作符 |
| `right` | any  | 是   | -      | 右操作符 |

### 返回

直接返回一个值。

| 类型    | 说明           |
| ------- | -------------- |
| boolean | 两个值是否相等 |

### 示例

```lua
t1 = { "a" }
t2 = { "a" }

b1 = rawequal { left = 1, right = 1 }
debug.print { message = tostring(b1) }

b2 = rawequal { left = 1, right = 2 }
debug.print { message = tostring(b2) }

b3 = rawequal { left = t1, right = t1 }
debug.print { message = tostring(b3) }

b4 = rawequal { left = t1, right = t2 }
debug.print { message = tostring(b4) }
```

输出：

```text
true
false
true
false
```

---

### `rawlen`

字符串返回其 UTF-8 字节数；表返回其连续数组部分的长度。

### 调用

```lua
-- 单参数
rawlen()
```

### 参数

| 参数    | 类型           | 必填 | 默认值 | 说明         |
| ------- | -------------- | ---- | ------ | ------------ |
| `value` | string / table | 是   | -      | 需要测量的值 |

### 返回

直接返回一个值。

| 类型    | 说明                               |
| ------- | ---------------------------------- |
| integer | 字符串总字节数；表中连续元素的数量 |

### 示例

```lua
t = { "a", "b", "c", x = 1, [5] = "d" }

l1 = rawlen("Hello")
debug.print { message = tostring(l1) }

l2 = rawlen("你好")
debug.print { message = tostring(l2) }

l3 = rawlen { value = "Hello" }
debug.print { message = tostring(l3) }

l4 = rawlen { value = t }
debug.print { message = tostring(l4) }
```

输出：

```text
5
6
5
3
```

---

### `tonumber`

将值转换为数字，可指定进制。

### 调用

```lua
-- 表参数
tonumber{}
```

### 参数

| 参数    | 类型                      | 必填 | 默认值 | 说明                                           |
| ------- | ------------------------- | ---- | ------ | ---------------------------------------------- |
| `value` | float / integer / string | 是   | -      | 需要转换的值                                   |
| `base`  | integer                   | 否   | `10`   | 指定进制；指定进制时，`value` 参数必须为字符串 |

### 返回

**转换成功**，直接返回一个值。

| 类型             | 说明         |
| ---------------- | ------------ |
| float / integer | 转换后的数字 |

**转换失败**，直接返回一个值。

| 类型 | 说明     |
| ---- | -------- |
| nil  | 转换失败 |

### 示例

```lua
n1 = tonumber { value = "42" }
debug.print { message = tostring(n1) }

n2 = tonumber { value = "3.14" }
debug.print { message = tostring(n2) }

n3 = tonumber { value = "invalid" }
debug.print { message = tostring(n3) }

n4 = tonumber { value = "1010", base = 2 }
debug.print { message = tostring(n4) }
```

输出：

```text
42
3.14
nil
10
```

---

### `tostring`

将任意值安全的转换为字符串。

### 调用

```lua
-- 单参数
tostring()
```

### 参数

| 参数    | 类型 | 必填 | 默认值 | 说明         |
| ------- | ---- | ---- | ------ | ------------ |
| `value` | any  | 是   | -      | 需要转换的值 |

### 返回

直接返回一个值。

| 类型   | 说明           |
| ------ | -------------- |
| string | 转换后的字符串 |

### 示例

```lua
t = { "a", "b", "c" }

debug.print { message = tostring(nil) }

debug.print { message = tostring { value = true } }

debug.print { message = tostring(t) }

debug.print { message = tostring(10.5) }
```

输出：

```text
nil
true
table: 0x114514   -- 表的内部身份指针
10.5
```

---

### `type`

返回值的类型名。

### 调用

```lua
-- 单参数
type()
```

### 参数

| 参数    | 类型 | 必填 | 默认值 | 说明           |
| ------- | ---- | ---- | ------ | -------------- |
| `value` | any  | 是   | -      | 要判断类型的值 |

### 返回

直接返回一个值。

| 类型   | 说明           |
| ------ | -------------- |
| string | 传递值的类型名 |

### 示例

```lua
t = { "a", "b", "c" }

debug.print { message = type(nil) }

debug.print { message = type { value = 10 } }

debug.print { message = type(2.7) }

debug.print { message = type(t) }

debug.print { message = type("Hello") }

debug.print { message = type(false) }
```

输出：

```text
nil
integer  -- 整型
number   -- 浮点数
table
string
boolean
```
