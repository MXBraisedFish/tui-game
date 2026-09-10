# Lua 二进制格式字符串的语法规范

## 前言

Lua 二进制格式字符串是二进制数据包操作的核心参数，用于精确描述 Lua 值与原始字节流之间的双向映射关系。本文档旨在为开发者提供一份语法速查手册。

> 格式串的语法规则与 Lua 官方原版完全一致，仅在 Tui Game 所签名的 API 有修改。如需完整的规范说明，请参阅 Lua 语言官方文档。

---

## 目录

| 章节             | 说明                                           | 索引                                  |
| ---------------- | ---------------------------------------------- | ------------------------------------- |
| 字节序控制       | 用于指定多字节数值在二进制数据中的字节排列顺序 | [字节序控制](#字节序控制)             |
| 对齐控制         | 用于控制数据项在二进制结构中的对齐方式         | [对齐控制](#对齐控制)                 |
| 原生整数类型     | 用于按照运行环境对应的原生整数类型保存数值     | [原生整数类型](#原生整数类型)         |
| 固定尺寸整数类型 | 用于显式指定整数占用的字节数                   | [固定尺寸整数类型](#固定尺寸整数类型) |
| 浮点数类型       | 用于保存带小数的数值                           | [浮点数类型](#浮点数类型)             |
| 字符串类型       | 用于存储指定长度的字符串                       | [字符串类型](#字符串类型)             |
| 填充与空白符     | 用于添加固定填充字节                           | [填充与空白符](#填充与空白符)         |

---

## 字节序控制

用于指定多字节数值在二进制数据中的字节排列顺序。

### 语法

| 符号 | 说明               |
| ---- | ------------------ |
| `<`  | 使用小端序         |
| `>`  | 使用大端序         |
| `=`  | 使用系统默认字节序 |

### 示例

```lua
little = serialization.binary_pack {
  fmt = "<I4",
  values = { 0x12345678 }
}

big = serialization.binary_pack {
  fmt = ">I4",
  values = { 0x12345678 }
}

debug.print { message = tostring(#little) }
debug.print { message = tostring(#big) }
```

输出：

```text
4
4
```

---

## 对齐控制

用于控制数据项在二进制结构中的对齐方式。

### 语法

| 符号    | 说明                                 |
| ------- | ------------------------------------ |
| `![n]`  | 设置最大对齐值为 `[n]`               |
| `X[op]` | 按 `[op]` 对应的数据类型执行一次对齐 |

### 示例

```lua
size1 = serialization.binary_packsize("c1 i4")
size2 = serialization.binary_packsize("!4 c1 i4")

debug.print { message = tostring(size1) }
debug.print { message = tostring(size2) }

bytes1 = serialization.binary_pack {
  fmt = "c1 Xi4 i4",
  values = { "A", 100 }
}
bytes2 = serialization.binary_pack {
  fmt = "!4 c1 Xi4 i4", -- Xop 需要 !n 开头
  values = { "A", 100 }
}

debug.print { message = tostring(#bytes1) }
debug.print { message = tostring(#bytes2) }
```

输出：

```text
5
8
5
8
```

### 额外补充

- `[n]` 为对齐字节数。
- `[n]` 必须为 $2^x$，且范围为 $[1, 16]$。

---

## 原生整数类型

用于按照运行环境对应的原生整数类型保存数值。

### 语法

| 符号 | 说明             | 类型        | 有符号 |
| ---- | ---------------- | ----------- | ------ |
| `b`  | 1 字节有符号整数 | char        | 是     |
| `B`  | 1 字节无符号整数 | char        | 否     |
| `h`  | 短字节有符号整数 | short       | 是     |
| `H`  | 短字节无符号整数 | short       | 否     |
| `i`  | 有符号整数       | int         | 是     |
| `I`  | 无符号整数       | int         | 否     |
| `l`  | 长字节有符号整数 | long        | 是     |
| `L`  | 长字节无符号整数 | long        | 否     |
| `j`  | Lua 有符号整数   | Lua Integer | 是     |
| `J`  | Lua 无符号整数   | Lua Integer | 否     |

### 示例

```lua
size1 = serialization.binary_packsize("b B")
size2 = serialization.binary_packsize("h H")
size3 = serialization.binary_packsize("i I")
size4 = serialization.binary_packsize("l L")
size5 = serialization.binary_packsize("j J")

debug.print { message = tostring(size1) }
debug.print { message = tostring(size2) }
debug.print { message = tostring(size3) }
debug.print { message = tostring(size4) }
debug.print { message = tostring(size5) }
```

输出：

```text
运行系统：Windows11 x64 - Lua 5.4

2  -- 每个数据为 1 字节
4  -- 每个数据为 2 字节
8  -- 每个数据为 4 字节
16 -- 每个数据为 8 字节
16 -- 每个数据为 8 字节
```

### 额外补充

- 小写符号表示有符号整数，大写符号表示无符号整数。
- `h`/`H`/`i`/`I`/`l`/`L` 的实际大小由运行环境决定。

---

## 固定尺寸整数类型

用于显式指定整数占用的字节数。

### 语法

| 格式   | 说明                 |
| ------ | -------------------- |
| `i[n]` | `[n]` 字节有符号整数 |
| `I[n]` | `[n]` 字节无符号整数 |

### 示例

```lua
size = serialization.binary_packsize("i1 I2 i4")

debug.print { message = tostring(size) }
```

输出：

```text
7
```

### 额外补充

- 小写符号表示有符号整数，大写符号表示无符号整数。
- `[n]` 为整数占用的字节数。
- `[n]` 必须为 $2^x$，且范围为 $[1, 8]$。
- 固定尺寸不会随运行环境变化。

---

## 浮点数类型

用于保存带小数的数值。

### 语法

| 符号 | 说明         |
| ---- | ------------ |
| `f`  | 单精度浮点数 |
| `d`  | 双精度浮点数 |
| `n`  | Lua 浮点数   |

### 示例

```lua
size1 = serialization.binary_packsize("f")
size2 = serialization.binary_packsize("d")
size3 = serialization.binary_packsize("n")

debug.print { message = tostring(size1) }
debug.print { message = tostring(size2) }
debug.print { message = tostring(size3) }
```

输出：

```text
运行系统：Windows11 x64 - Lua 5.4

4
8
8
```

### 额外补充

- `n` 的实际大小由运行环境决定。

---

## 字符串类型

用于存储指定长度的字符串。

### 语法

| 格式   | 说明                          |
| ------ | ----------------------------- |
| `c[n]` | 固定 `[n]` 字节字符串         |
| `z`    | 以 `\0` 结尾的字符串          |
| `s[n]` | 带 `[n]` 字节长度前缀的字符串 |

### 示例

```lua
bytes1 = serialization.binary_pack {
  fmt = "c5 c4 c4",
  values = { "Hello", "Tui", "Game" }
}

debug.print { message = tostring(#bytes1) }

bytes2 = serialization.binary_pack {
  fmt = "z",
  values = { "Hello" }
}

debug.print { message = tostring(#bytes2) }

bytes3 = serialization.binary_pack {
  fmt = "s4",
  values = { "Hello" }
}

debug.print { message = tostring(#bytes3) }
```

输出：

```text
13 -- Tui 被补到 4 字节
6  -- Hello 后多一个 \0
9  -- Hello 前被填充 4 字节的占位数据
```

### 额外补充

- `c[n]` 的强制固定长度，数据长度大于 `[n]` 会自动截断，小于则会补 `\0`。
- `z` 存储的总长度为 $数据长度 + 1$。
- `s[n]` 中的 `[n]` 范围为 $[1, 16]$。

---

## 填充与空白符

用于添加固定填充字节。

### 语法

| 符号 | 说明                         |
| ---- | ---------------------------- |
| `x`  | 插入或跳过一个空字节         |
| 空格 | 无实际作用，仅用于可读性优化 |

### 示例

```lua
bytes = serialization.binary_pack {
  fmt = "<I2 x I2",
  values = {
    100,
    200
  }
}

debug.print { message = tostring(#bytes) }
```

输出：

```text
5
```

### 额外补充

- `x` 实际含义为打包时插入一字节占位符或解包时跳过一字节。
- 格式字符串中的空格在实际的处理中会被忽略，仅用于可读性优化。
