# serialization 库

`serialization` 提供多种格式的数据编码和解码。

---

# 目录

## 常量

| 常量 | 说明 | 定位 |
| --- | --- | --- |
| `NULL` | 表示 JSON/YAML 中的 null 值 | [NULL](#null) |

## 方法

| 方法            | 说明                           | 定位                                |
| ----------------- | ------------------------------ | ----------------------------------- |
| `json_encode`     | 将 Lua 值编码为 JSON 字符串    | [json_encode](#json_encode)         |
| `json_decode`     | 将 JSON 字符串解码为 Lua 值    | [json_decode](#json_decode)         |
| `csv_encode`      | 将二维数组表编码为 CSV 字符串    | [csv_encode](#csv_encode)           |
| `csv_decode`      | 将 CSV 字符串解码为二维数组表    | [csv_decode](#csv_decode)           |
| `yaml_encode`     | 将 Lua 值编码为 YAML 字符串    | [yaml_encode](#yaml_encode)         |
| `yaml_decode`     | 将 YAML 字符串解码为 Lua 值    | [yaml_decode](#yaml_decode)         |
| `toml_encode`     | 将 Lua 值编码为 TOML 字符串    | [toml_encode](#toml_encode)         |
| `toml_decode`     | 将 TOML 字符串解码为 Lua 值    | [toml_decode](#toml_decode)         |
| `ini_encode`      | 将 Lua 表编码为 INI 字符串     | [ini_encode](#ini_encode)           |
| `ini_decode`      | 将 INI 字符串解码为 Lua 表     | [ini_decode](#ini_decode)           |
| `xml_encode`      | 将 Lua 值编码为 XML 字符串     | [xml_encode](#xml_encode)           |
| `xml_decode`      | 将 XML 字符串解码为 Lua 值     | [xml_decode](#xml_decode)           |
| `binary_pack`     | 按格式串打包数据为二进制字符串 | [binary_pack](#binary_pack)         |
| `binary_unpack`   | 按格式串从二进制字符串解包数据 | [binary_unpack](#binary_unpack)     |
| `binary_packsize` | 返回按格式打包所需的总字节数   | [binary_packsize](#binary_packsize) |

---

# 常量

## `NULL`

用来表示 JSON/YAML 中的 `null`，并与 Lua 的 `nil` 区分开。

### 调用

```lua
serialization.NULL
```

### 可用于

- JSON/YAML 编码参数中的对象字段或数组元素

### 示例

```lua
local encoded = serialization.json_encode({value = serialization.NULL})
local decoded = serialization.json_decode(encoded)
local encoded_again = serialization.json_encode(decoded)
```

**输出：**

```lua
```

### 等值

```json
null
```

### 额外说明

- JSON/YAML 编码与解码都保留此值；CSV、INI、TOML、XML 遇到此值会报错。

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

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `value` | any | 要编码的 Lua 值 |

## 返回值

返回一个值。

| 类型   | 说明        |
| ------ | ----------- |
| string | JSON 字符串 |

### 示例

```lua
local data = { name = "TUI", version = 1, features = { "draw", "event" } }
local json = serialization.json_encode(data)
debug.print(json)
```

**输出：**

```lua
```

### 额外说明

- 参数 `value` 必须可序列化；使用 `serialization.NULL` 表示对象字段或数组位置中的 JSON null。
- `value` 直接作为位置参数传入；`serialization.NULL` 用于保留嵌套的 JSON null。
- JSON/YAML 编码和解码共用以下数据限制：嵌套最多 32 层、最多 16,384 个值节点，编码结果最多 1 MiB；拒绝非有限数字、无效 UTF-8、循环表、稀疏数组、非正整数数组键，以及同一表混用数组索引和字符串键。
- 空 Lua 表编码为 JSON/YAML 对象 `{}`；顶层 `json_encode(nil)` 编码为 `null`。发生错误时会说明被拒绝的值或限制。

---

## `json_decode`

将 JSON 字符串解码为 Lua 值。

### 调用

```lua
serialization.json_decode
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
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
debug.print(data.name .. ", v" .. tostring(data.version))
```

**输出：**

```lua
```

### 额外说明

- 参数 `text` 必须是有效文本；JSON null 解码为 `serialization.NULL`，包含该哨兵的值可以原样重新编码。

---

## `csv_encode`

将二维数组表编码为 CSV 字符串。

### 调用

```lua
serialization.csv_encode
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
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

### 额外说明

- 参数 `rows` 必须可序列化；CSV 不支持 `serialization.NULL`，空字段解码为普通空字符串。

---

## `csv_decode`

将 CSV 字符串解码为二维数组表。

### 调用

```lua
serialization.csv_decode
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | CSV 字符串 |

## 返回值

返回一个数组表。

| 类型  | 说明     |
| ----- | -------- |
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

### 额外说明

- 参数 `text` 必须是对应格式的有效文本。

---

## `yaml_encode`

将 Lua 值编码为 YAML 字符串。

### 调用

```lua
serialization.yaml_encode
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
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

### 额外说明

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

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
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

### 额外说明

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

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `value` | any | 要编码的 Lua 值 |

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

### 额外说明

- 参数 `value` 必须可序列化；TOML 不支持 null，包含 `serialization.NULL` 时返回错误。

---

## `toml_decode`

将 TOML 字符串解码为 Lua 值。

### 调用

```lua
serialization.toml_decode
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
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

### 额外说明

- 参数 `text` 必须是对应格式的有效文本。

---

## `ini_encode`

将 Lua 表编码为 INI 字符串。

### 调用

```lua
serialization.ini_encode
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
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

### 额外说明

- 参数 `value` 必须可序列化；INI 不支持 `serialization.NULL`。

---

## `ini_decode`

将 INI 字符串解码为 Lua 表。

### 调用

```lua
serialization.ini_decode
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
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

### 额外说明

- 参数 `text` 必须是对应格式的有效文本。

---

## `xml_encode`

将 Lua 值编码为 XML 字符串。

### 调用

```lua
serialization.xml_encode
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `value` | table | 要编码的 Lua 值 |

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

### 额外说明

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

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
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

### 额外说明

- 属性在 `_attr` 中，文本在 `_text` 中；重复子元素组成数组。不支持 DTD、实体声明及文本与子元素混排。

- 参数 `text` 必须是对应格式的有效文本。

---

## `binary_pack`

按格式串将数据打包为二进制字符串。

### 调用

```lua
serialization.binary_pack
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `fmt` | string | 打包格式串 |

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

### 额外说明

- 格式串后依次传入数值或字节字符串，数量与类型必须匹配；这些值是打包数据，不作为选项表解析，也不接受表或 nil。

---

## `binary_unpack`

按格式串从二进制字符串中解包数据。

### 调用

```lua
serialization.binary_unpack
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `fmt` | string | 解包格式串 |
| `data` | string | 二进制数据，可包含任意字节 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `pos` | integer | `1` | 从 1 开始的起始字节位置 |

## 返回值

返回两个值：解出的数据数组表和下一次解包的一基字节位置。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `values` | table | 按格式顺序解出的数据数组表 |
| `next_pos` | integer | 下一次解包的一基起始字节位置 |

### 示例

```lua
local bytes = serialization.binary_pack("<I4 I4", table.unpack({ 100, 200 }))
local values, next_pos = serialization.binary_unpack("<I4 I4", bytes)
debug.print(tostring(values[1]) .. ", " .. tostring(values[2]) .. "; next=" .. next_pos)
```

**输出：**

```lua
```

---

## `binary_packsize`

返回按格式串打包所需的总字节数。

### 调用

```lua
serialization.binary_packsize
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `fmt` | string | 打包格式串 |

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
