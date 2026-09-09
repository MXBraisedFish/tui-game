# serialization 库

## 基本库说明

`serialization` 提供多格式序列化与反序列化。

---

## 目录

### 方法

| 方法名            | 说明                           | 索引                                |
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

## 方法

## `json_encode`

将 Lua 值编码为 JSON 字符串。

### 调用

```lua
-- 单参数
serialization.json_encode()
```

### 参数

| 参数名  | 类型 | 必填 | 默认值 | 说明            |
| ------- | ---- | ---- | ------ | --------------- |
| `value` | any  | 是   | -      | 要编码的 Lua 值 |

### 返回

直接返回一个值。

| 类型   | 说明        |
| ------ | ----------- |
| string | JSON 字符串 |

### 示例

```lua
data = { name = "TUI", version = 1, features = { "draw", "event" } }
json = serialization.json_encode(data)
debug.print { message = json }
```

输出：

```json
{"features":["draw","event"],"name":"TUI","version":1}
```

### 额外补充

- 参数 `value` 必须可序列化。

---

## `json_decode`

将 JSON 字符串解码为 Lua 值。

### 调用

```lua
-- 单参数
serialization.json_decode()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明        |
| ------ | ------ | ---- | ------ | ----------- |
| `s`    | string | 是   | -      | JSON 字符串 |

### 返回

直接返回一个值。

| 类型 | 说明     |
| ---- | -------- |
| any  | 解码结果 |

### 示例

```lua
json = '{"name":"TUI","version":1}'
data = serialization.json_decode(json)
debug.print { message = data.name .. ", v" .. tostring(data.version) }
```

输出：

```text
TUI, v1
```

### 额外补充

- 参数 `s` 必须可反序列化。

---

## `csv_encode`

将二维数组表编码为 CSV 字符串。

### 调用

```lua
-- 单参数
serialization.csv_encode()
```

### 参数

| 参数名 | 类型  | 必填 | 默认值 | 说明     |
| ------ | ----- | ---- | ------ | -------- |
| `rows` | table | 是   | -      | 二维数组表 |

### 返回

直接返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| string | CSV 字符串 |

### 示例

```lua
data = {
    { "Name", "Score" },
    { "Alice", 95 },
    { "Bob", 87 }
}
csv = serialization.csv_encode(data)
debug.print { message = csv }
```

输出：

```csv
Name,Score
Alice,95
Bob,87
```

### 额外补充

- 参数 `rows` 必须可序列化。

---

## `csv_decode`

将 CSV 字符串解码为二维数组表。

### 调用

```lua
-- 单参数
serialization.csv_decode()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明       |
| ------ | ------ | ---- | ------ | ---------- |
| `s`    | string | 是   | -      | CSV 字符串 |

### 返回

返回一个数组表。

| 类型  | 说明     |
| ----- | -------- |
| table | 二维数组表 |

### 示例

```lua
csv = "Name,Score\nAlice,95\nBob,87"
data = serialization.csv_decode(csv)
debug.print { message = data[2][1] .. ": " .. tostring(data[2][2]) }
```

输出：

```text
Alice: 95
```

### 额外补充

- 参数 `s` 必须可反序列化。

---

## `yaml_encode`

将 Lua 值编码为 YAML 字符串。

### 调用

```lua
-- 单参数
serialization.yaml_encode()
```

### 参数

| 参数名  | 类型             | 必填 | 默认值 | 说明            |
| ------- | ---------------- | ---- | ------ | --------------- |
| `value` | table / 基本类型 | 是   | -      | 要编码的 Lua 值 |

### 返回

直接返回一个值。

| 类型   | 说明        |
| ------ | ----------- |
| string | YAML 字符串 |

### 示例

```lua
data = { name = "TUI", version = 1 }
yaml = serialization.yaml_encode(data)
debug.print { message = yaml }
```

输出：

```text
name: TUI
version: 1
```

### 额外补充

- 参数 `value` 必须可序列化。

---

## `yaml_decode`

将 YAML 字符串解码为 Lua 值。

### 调用

```lua
-- 单参数
serialization.yaml_decode()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明        |
| ------ | ------ | ---- | ------ | ----------- |
| `s`    | string | 是   | -      | YAML 字符串 |

### 返回

直接返回一个值。

| 类型 | 说明     |
| ---- | -------- |
| any  | 解码结果 |

### 示例

```lua
yaml = "name: TUI\nversion: 1"
data = serialization.yaml_decode(yaml)
debug.print { message = data.name }
```

输出：

```text
TUI
```

### 额外补充

- 参数 `s` 必须可反序列化。

---

## `toml_encode`

将 Lua 值编码为 TOML 字符串。

### 调用

```lua
-- 单参数
serialization.toml_encode()
```

### 参数

| 参数名  | 类型 | 必填 | 默认值 | 说明            |
| ------- | ---- | ---- | ------ | --------------- |
| `value` | any  | 是   | -      | 要编码的 Lua 值 |

### 返回

直接返回一个值。

| 类型   | 说明        |
| ------ | ----------- |
| string | TOML 字符串 |

### 示例

```lua
data = { name = "TUI", version = 1 }
toml = serialization.toml_encode(data)
debug.print { message = toml }
```

输出：

```text
name = "TUI"
version = 1
```

### 额外补充

- 参数 `value` 必须可序列化。

---

## `toml_decode`

将 TOML 字符串解码为 Lua 值。

### 调用

```lua
-- 单参数
serialization.toml_decode()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明        |
| ------ | ------ | ---- | ------ | ----------- |
| `s`    | string | 是   | -      | TOML 字符串 |

### 返回

直接返回一个值。

| 类型 | 说明     |
| ---- | -------- |
| any  | 解码结果 |

### 示例

```lua
toml = 'name = "TUI"\nversion = 1'
data = serialization.toml_decode(toml)
debug.print { message = data.name }
```

输出：

```text
TUI
```

### 额外补充

- 参数 `s` 必须可反序列化。

---

## `ini_encode`

将 Lua 表编码为 INI 字符串。

### 调用

```lua
-- 单参数
serialization.ini_encode(t)
```

### 参数

| 参数名 | 类型  | 必填 | 默认值 | 说明            |
| ------ | ----- | ---- | ------ | --------------- |
| `t`    | table | 是   | -      | 要编码的 Lua 表 |

### 返回

直接返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| string | INI 字符串 |

### 示例

```lua
data = {
  server = { host = "127.0.0.1", port = 8080 },
  logging = { level = "debug" }
}
ini = serialization.ini_encode(data)
debug.print { message = ini }
```

输出：

```text
[server]
host = 127.0.0.1
port = 8080

[logging]
level = debug
```

### 额外补充

- 参数 `t` 必须可序列化。

---

## `ini_decode`

将 INI 字符串解码为 Lua 表。

### 调用

```lua
-- 单参数
serialization.ini_decode()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明       |
| ------ | ------ | ---- | ------ | ---------- |
| `s`    | string | 是   | -      | INI 字符串 |

### 返回

返回一个对象表。

| 类型  | 说明     |
| ----- | -------- |
| table | 解码结果 |

### 示例

```lua
ini = "[server]\nhost = 127.0.0.1\nport = 8080"
data = serialization.ini_decode(ini)
debug.print { message = data.server.host }
```

输出：

```text
127.0.0.1
```

### 额外补充

- 参数 `s` 必须可反序列化。

---

## `xml_encode`

将 Lua 值编码为 XML 字符串。

### 调用

```lua
-- 单参数
serialization.xml_encode()
```

### 参数

| 参数名  | 类型  | 必填 | 默认值 | 说明            |
| ------- | ----- | ---- | ------ | --------------- |
| `value` | table | 是   | -      | 要编码的 Lua 值 |

### 返回

直接返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| string | XML 字符串 |

### 示例

```lua
data = {
  root = {
    _attr = { version = "1.0" },
    child = { "Hello", _attr = { id = 1 } }
  }
}
xml = serialization.xml_encode(data)
debug.print { message = xml }
```

输出：

```text
<root version="1.0"><child id="1">Hello</child></root>
```

### 额外补充

- 参数 `value` 必须可序列化。

---

## `xml_decode`

将 XML 字符串解码为 Lua 值。

### 调用

```lua
-- 单参数
serialization.xml_decode()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明       |
| ------ | ------ | ---- | ------ | ---------- |
| `s`    | string | 是   | -      | XML 字符串 |

### 返回

返回一个对象表。

| 类型  | 说明     |
| ----- | -------- |
| table | 解码结果 |

### 示例

```lua
xml = '<root version="1.0"><child id="1">Hello</child></root>'
data = serialization.xml_decode(xml)
debug.print { message = data.root.child._text }
```

输出：

```text
Hello
```

### 额外补充

- 参数 `s` 必须可反序列化。

---

## `binary_pack`

按格式串将数据打包为二进制字符串。

### 调用

```lua
-- 表参数
serialization.binary_pack{}
```

### 参数

| 参数名   | 类型   | 必填 | 默认值 | 说明       |
| -------- | ------ | ---- | ------ | ---------- |
| `fmt`    | string | 是   | -      | 打包格式串 |
| `values` | table  | 是   | -      | 数据数组表   |

### 返回

直接返回一个值。

| 类型   | 说明                 |
| ------ | -------------------- |
| binary | 打包后的二进制字符串 |

### 示例

```lua
bytes = serialization.binary_pack {
  fmt = "<I4 I4",
  values = { 100, 200 }
}
debug.print { message = "packed " .. tostring(#bytes) .. " bytes" }
```

输出：

```text
packed 8 bytes
```

---

## `binary_unpack`

按格式串从二进制字符串中解包数据。

### 调用

```lua
-- 表参数
serialization.binary_unpack{}
```

### 参数

| 参数名 | 类型    | 必填 | 默认值 | 说明                         |
| ------ | ------- | ---- | ------ | ---------------------------- |
| `fmt`  | string  | 是   | -      | 解包格式串                   |
| `data` | binary  | 是   | -      | 二进制数据，可包含任意字节   |
| `pos`  | integer | 否   | `1`    | 基起始字节位置             |

### 返回

返回一个对象表。

| 字段       | 类型    | 说明                         |
| ---------- | ------- | ---------------------------- |
| `values`   | table   | 解出的数据表               |
| `next_pos` | integer | 下一次解包的一基起始字节位置 |

### 示例

```lua
bytes = serialization.binary_pack {
  fmt = "<I4 I4",
  values = { 100, 200 }
}
result = serialization.binary_unpack {
  fmt = "<I4 I4",
  data = bytes
}
debug.print { message = tostring(result.values[1]) .. ", " .. tostring(result.values[2]) }
```

输出：

```text
100, 200
```

---

## `binary_packsize`

返回按格式串打包所需的总字节数。

### 调用

```lua
-- 单参数
serialization.binary_packsize()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明       |
| ------ | ------ | ---- | ------ | ---------- |
| `fmt`  | string | 是   | -      | 打包格式串 |

### 返回

直接返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| integer | 打包所需总字节数 |

### 示例

```lua
size = serialization.binary_packsize("<I4 I4")
debug.print { message = tostring(size) }
```

输出：

```text
8
```
