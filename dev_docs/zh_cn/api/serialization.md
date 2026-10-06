# serialization 库

`serialization` 提供多种格式的数据编码和解码。

---

# 目录

## 常量

| 常量   | 说明                                                     | 定位          |
| ------ | -------------------------------------------------------- | ------------- |
| `NULL` | JSON / YAML 文件中的 `null | [NULL](#null) |

## 方法

| 方法              | 说明                             | 定位                                |
| ----------------- | -------------------------------- | ----------------------------------- |
| `json_encode`     | 将 Lua 值编码为 JSON 字符串      | [json_encode](#json_encode)         |
| `json_decode`     | 将 JSON 字符串解码为 Lua 值      | [json_decode](#json_decode)         |
| `csv_encode`      | 将二维数组表编码为 CSV 字符串    | [csv_encode](#csv_encode)           |
| `csv_decode`      | 将 CSV 字符串解码为二维数组表    | [csv_decode](#csv_decode)           |
| `yaml_encode`     | 将 Lua 值编码为 YAML 字符串      | [yaml_encode](#yaml_encode)         |
| `yaml_decode`     | 将 YAML 字符串解码为 Lua 值      | [yaml_decode](#yaml_decode)         |
| `toml_encode`     | 将 Lua 值编码为 TOML 字符串      | [toml_encode](#toml_encode)         |
| `toml_decode`     | 将 TOML 字符串解码为 Lua 值      | [toml_decode](#toml_decode)         |
| `ini_encode`      | 将 Lua 表编码为 INI 字符串       | [ini_encode](#ini_encode)           |
| `ini_decode`      | 将 INI 字符串解码为 Lua 表       | [ini_decode](#ini_decode)           |
| `xml_encode`      | 将 Lua 值编码为 XML 字符串       | [xml_encode](#xml_encode)           |
| `xml_decode`      | 将 XML 字符串解码为 Lua 值       | [xml_decode](#xml_decode)           |
| `binary_pack`     | 按格式串将数据打包为二进制字符串 | [binary_pack](#binary_pack)         |
| `binary_unpack`   | 按格式串从二进制字符串中解包数据 | [binary_unpack](#binary_unpack)     |
| `binary_packsize` | 返回按格式串打包所需的总字节数   | [binary_packsize](#binary_packsize) |

---

# 常量

## `NULL`

JSON / YAML 文件中的 `null`。

### 调用

```lua
serialization.NULL
```

### 可用于

- JSON / YAML 编码参数中的对象字段或数组元素。

### 示例

```lua
local encoded = serialization.json_encode({value = serialization.NULL})
debug.print(encoded)
```

**输出：**

```json
{
  "value": null
}
```

## 额外说明

- 仅适用于 JSON / YAML 文件。

---

# 方法

## `json_encode`

将 Lua 值编码为 JSON 字符串。

### 调用

```lua
serialization.json_encode
```

## 参数

### 必填参数

| 参数名  | 类型 | 说明            |
| ------- | ---- | --------------- |
| `value` | any  | 要编码的 Lua 值 |

## 返回值

返回一个值。

| 类型   | 说明        |
| ------ | ----------- |
| string | JSON 字符串 |

### 示例

```lua
local data = { name = "TUI", version = 1, features = { "draw", "events" } }
local json = serialization.json_encode(data)
debug.print(json)
```

**输出：**

```json
{
  "features": [
    "draw",
    "events"
  ],
  "version": 1,
  "name": "TUI"
}
```

## 额外说明

- 必填参数 `value` 必须可序列化，详细结构见⌊[多格式序列化与反序列化规范](../format/SERIALIZATION_FORMATS.md)⌉。

---

## `json_decode`

将 JSON 字符串解码为 Lua 值。

### 调用

```lua
serialization.json_decode
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明        |
| ------ | ------ | ----------- |
| `text` | string | JSON 字符串 |

## 返回值

返回一个值。

| 类型 | 说明     |
| ---- | -------- |
| any  | 解码结果 |

### 示例

```lua
local json = '{"name":"TUI","version":1}'
local data = serialization.json_decode(json)
debug.print(table.pretty(data))
```

**输出：**

```lua
{
  name = "TUI",
   version = 1
  }
```

## 额外说明

- JSON `null` 会被解析为 `serialization.NULL`

---

## `csv_encode`

将二维数组表编码为 CSV 字符串。

### 调用

```lua
serialization.csv_encode
```

## 参数

### 必填参数

| 参数名 | 类型  | 说明       |
| ------ | ----- | ---------- |
| `rows` | table | 二维数组表 |

## 返回值

返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| string | CSV 字符串 |

### 示例

```lua
local data = {
    { "Name", "Score" },
    { "Alice", 95 },
    { "Bob", 87 }
}
local csv = serialization.csv_encode(data)
debug.print(csv)
```

**输出：**

```lua

```

## 额外说明

- 参数 `rows` 必须是二维数组表，单元格只能是标量；CSV 不支持 `serialization.NULL`。

---

## `csv_decode`

将 CSV 字符串解码为二维数组表。

### 调用

```lua
serialization.csv_decode
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明       |
| ------ | ------ | ---------- |
| `text` | string | CSV 字符串 |

## 返回值

返回一个数组表。

| 类型  | 说明       |
| ----- | ---------- |
| table | 二维数组表 |

### 示例

```lua
local csv = "Name,Score\nAlice,95\nBob,87"
local data = serialization.csv_decode(csv)
debug.print(data[2][1] .. ": " .. tostring(data[2][2]))
```

**输出：**

```lua

```

## 额外说明

- CSV 的每个字段都解码为字符串，空字段为普通空字符串。

---

## `yaml_encode`

将 Lua 值编码为 YAML 字符串。

### 调用

```lua
serialization.yaml_encode
```

## 参数

### 必填参数

| 参数名  | 类型             | 说明            |
| ------- | ---------------- | --------------- |
| `value` | table / 基本类型 | 要编码的 Lua 值 |

## 返回值

返回一个值。

| 类型   | 说明        |
| ------ | ----------- |
| string | YAML 字符串 |

### 示例

```lua
local data = { name = "TUI", version = 1 }
local yaml = serialization.yaml_encode(data)
debug.print(yaml)
```

**输出：**

```lua

```

## 额外说明

- 参数 `value` 必须可序列化；`serialization.NULL` 编码为 YAML null。

---

## `yaml_decode`

将 YAML 字符串解码为 Lua 值。

### 调用

```lua
serialization.yaml_decode
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明        |
| ------ | ------ | ----------- |
| `text` | string | YAML 字符串 |

## 返回值

返回一个值。

| 类型 | 说明     |
| ---- | -------- |
| any  | 解码结果 |

### 示例

```lua
local yaml = "name: TUI\nversion: 1"
local data = serialization.yaml_decode(yaml)
debug.print(data.name)
```

**输出：**

```lua

```

## 额外说明

- 参数 `text` 必须是有效文本；YAML null 解码为 `serialization.NULL`。

---

## `toml_encode`

将 Lua 值编码为 TOML 字符串。

### 调用

```lua
serialization.toml_encode
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明            |
| ------- | ----- | --------------- |
| `value` | table | 要编码的 Lua 表 |

## 返回值

返回一个值。

| 类型   | 说明        |
| ------ | ----------- |
| string | TOML 字符串 |

### 示例

```lua
local data = { name = "TUI", version = 1 }
local toml = serialization.toml_encode(data)
debug.print(toml)
```

**输出：**

```lua

```

## 额外说明

- 参数 `value` 必须可序列化；TOML 不支持 null，包含 `serialization.NULL` 时返回错误。
- TOML 的根必须是对象表，数组或标量根会抛出错误。

---

## `toml_decode`

将 TOML 字符串解码为 Lua 值。

### 调用

```lua
serialization.toml_decode
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明        |
| ------ | ------ | ----------- |
| `text` | string | TOML 字符串 |

## 返回值

返回一个值。

| 类型 | 说明     |
| ---- | -------- |
| any  | 解码结果 |

### 示例

```lua
local toml = 'name = "TUI"\nversion = 1'
local data = serialization.toml_decode(toml)
debug.print(data.name)
```

**输出：**

```lua

```

## 额外说明

- TOML 文档的根总是对象表，解码结果总是表，且不会包含 `serialization.NULL`。

---

## `ini_encode`

将 Lua 表编码为 INI 字符串。

### 调用

```lua
serialization.ini_encode
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明            |
| ------- | ----- | --------------- |
| `value` | table | 要编码的 Lua 表 |

## 返回值

返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| string | INI 字符串 |

### 示例

```lua
local data = {
  server = { host = "127.0.0.1", port = 8080 },
  logging = { level = "debug" }
}
local ini = serialization.ini_encode(data)
debug.print(ini)
```

**输出：**

```lua

```

## 额外说明

- 参数 `value` 必须可序列化；INI 不支持 `serialization.NULL`。
- 参数 `value` 必须是对象表；只支持一层节，节内值必须是标量。
- 键名与节名不能为空，也不能含 `[`、`]`、`=`、`;`、`#` 或换行；值不能含换行。

---

## `ini_decode`

将 INI 字符串解码为 Lua 表。

### 调用

```lua
serialization.ini_decode
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明       |
| ------ | ------ | ---------- |
| `text` | string | INI 字符串 |

## 返回值

返回一个对象表。

| 类型  | 说明     |
| ----- | -------- |
| table | 解码结果 |

### 示例

```lua
local ini = "[server]\nhost = 127.0.0.1\nport = 8080"
local data = serialization.ini_decode(ini)
debug.print(data.server.host)
```

**输出：**

```lua

```

## 额外说明

- INI 的值都解码为字符串；节解码为嵌套表，节外的键直接放在根表中。

---

## `xml_encode`

将 Lua 值编码为 XML 字符串。

### 调用

```lua
serialization.xml_encode
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明            |
| ------- | ----- | --------------- |
| `value` | table | 要编码的 Lua 表 |

## 返回值

返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| string | XML 字符串 |

### 示例

```lua
local data = {
  root = {
    _attr = { version = "1.0" },
    child = { "Hello", _attr = { id = 1 } }
  }
}
local xml = serialization.xml_encode(data)
debug.print(xml)
```

**输出：**

```lua

```

## 额外说明

- 最外层表必须只有一个根元素。元素的 `_attr` 表表示属性，`_text` 表示文本；同名子元素可用数组表示。不能在同一元素中混合文本与子元素。

- 参数 `value` 必须可序列化；XML 不支持 `serialization.NULL`。

---

## `xml_decode`

将 XML 字符串解码为 Lua 值。

### 调用

```lua
serialization.xml_decode
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明       |
| ------ | ------ | ---------- |
| `text` | string | XML 字符串 |

## 返回值

返回一个对象表。

| 类型  | 说明     |
| ----- | -------- |
| table | 解码结果 |

### 示例

```lua
local xml = '<root version="1.0"><child id="1">Hello</child></root>'
local data = serialization.xml_decode(xml)
debug.print(data.root.child._text)
```

**输出：**

```lua

```

## 额外说明

- XML 中无属性且无子元素的元素解码为字符串本身，其余元素解码为表；属性的 `_attr`、文本的 `_text` 仅在非空时出现。

- 重复子元素组成数组，根元素名是结果表的唯一键。不支持 DTD、实体声明及文本与子元素混排。

---

## `binary_pack`

按格式串将数据打包为二进制字符串。

### 调用

```lua
serialization.binary_pack
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明       |
| ------ | ------ | ---------- |
| `fmt`  | string | 打包格式串 |

## 返回值

返回一个值。

| 类型   | 说明                 |
| ------ | -------------------- |
| string | 打包后的二进制字符串 |

### 示例

```lua
local bytes = serialization.binary_pack("<I4 I4", table.unpack({ 100, 200 }))
debug.print("packed " .. tostring(#bytes) .. " bytes")
```

**输出：**

```lua

```

## 额外说明

- 格式串后依次传入数值或字节字符串，数量与类型必须匹配；这些值是打包数据，不作为选项表解析，也不接受表或 `nil`。
- 格式串项：`b`/`B`、`h`/`H`、`l`/`j`/`L`/`J`/`T` 分别是 1、2、8 字节整数（小写有符号、大写无符号）；`i`/`I` 后跟可选字节数（默认 4，取值范围 $[1, 16]$）；`f`/`d`/`n` 是 4、8 字节浮点数；`c` 后必须跟字节数，表示定长字符串；`z` 是零结尾字符串；`s` 后跟可选的长度前缀字节数（默认 8，取值范围 $[1, 8]$）；`x` 是一个填充字节。
- 字节序默认为本机；`<` 改为小端，`>` 改为大端，`=` 恢复本机。
- 默认最大对齐为 1，即默认不填充；`!n` 把最大对齐设为 $[1, 16]$ 的 2 的幂（省略 `n` 时为本机指针宽度），`X` 按后随的定长项对齐。

---

## `binary_unpack`

按格式串从二进制字符串中解包数据。

### 调用

```lua
serialization.binary_unpack
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明                       |
| ------ | ------ | -------------------------- |
| `fmt`  | string | 解包格式串                 |
| `data` | string | 二进制数据，可包含任意字节 |

### 选填参数

| 参数名 | 类型    | 默认值 | 说明                    |
| ------ | ------- | ------ | ----------------------- |
| `pos`  | integer | `1`    | 从 1 开始的起始字节位置 |

## 返回值

返回两个值：解出的数据数组表和下一次解包的起始字节位置。

| 值名       | 类型    | 说明                       |
| ---------- | ------- | -------------------------- |
| `values`   | table   | 按格式顺序解出的数据数组表 |
| `next_pos` | integer | 下一次解包的起始字节位置   |

### 示例

```lua
local bytes = serialization.binary_pack("<I4 I4", table.unpack({ 100, 200 }))
local values, next_pos = serialization.binary_unpack("<I4 I4", bytes)
debug.print(tostring(values[1]) .. ", " .. tostring(values[2]) .. "; next=" .. next_pos)
```

**输出：**

```lua

```

## 额外说明

- 必填参数按顺序传入，选填参数放在末尾选项表中；`pos` 是选项表字段，不是第三个位置参数。
- 返回的第二个值可作为下一次解包调用的 `pos`。
- 选填参数 `pos` 取值范围为 $[1, x]$，$x$ 为 `data` 的字节数加 $1$，超出时抛出错误。

---

## `binary_packsize`

返回按格式串打包所需的总字节数。

### 调用

```lua
serialization.binary_packsize
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明       |
| ------ | ------ | ---------- |
| `fmt`  | string | 打包格式串 |

## 返回值

返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| integer | 打包所需总字节数 |

### 示例

```lua
local size = serialization.binary_packsize("<I4 I4")
debug.print(tostring(size))
```

**输出：**

```lua

```

## 额外说明

- 参数 `fmt` 只能包含定长项，含 `z`、`s` 等变长项时会抛出错误。
