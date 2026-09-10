# 多格式序列化与反序列化规范

## 前言

序列化与反序列化是数据持久化与跨环境交换的基础机制，负责在 Lua 值与存储格式之间建立双向映射。本文档旨在定义上述转换过程中所遵循的数据格式规范，确保序列化与反序列化行为的一致性和兼容性。

---

## 目录

| 章节       | 说明                 | 索引                      |
| ---------- | -------------------- | ------------------------- |
| 通用规则   | 所有序列化的通用规则 | [通用规则](#通用规则)     |
| JSON       | JSON 格式            | [JSON](#json)             |
| TOML       | TOML 格式            | [TOML](#toml)             |
| YAML       | YAML 格式            | [YAML](#yaml)             |
| CSV        | CSV 格式             | [CSV](#csv)               |
| XML        | XML 格式             | [XML](#xml)               |
| INI        | INI 格式             | [INI](#ini)               |
| 二进制数据 | 二进制数据           | [二进制数据](#二进制数据) |

## 链接

| 说明                            | 链接                                  |
| ------------------------------- | ------------------------------------- |
| `lifecycle` 库 API 使用文档     | [lifecycle](api/lifecycle.md)         |
| `serialization` 库 API 使用文档 | [serialization](api/serialization.md) |

---

## 通用规则

- 所有序列化操作最后返回的均为字符串类型，只有被写入到对应的文件当中才会被解析。
- 所有文件类型中的空值（例如 `null`，`~` 等），转换后的 `nil` 不会被 Lua 值解释器保留；反之 `nil` 并不会被转换为对应的空值，而是留空。

  **示例**

  ```lua
  {
    is_null = nil -- JSON 为 {}
  }
  ```

---

## JSON

### 根结构要求

JSON 的根值**可为任意可序列化的值**。

```lua
data = any...
```

### 数据结构映射

**双向映射**

> 该部分值的映射可逆

| Lua       | JSON      |
| --------- | --------- |
| `boolean` | `boolean` |
| `integer` | `integer` |
| `number`  | `float`   |
| `string`  | `string`  |
| 数组表    | 数组      |
| 对象表    | 对象      |

**单向映射**

> 该部分值的映射不可逆

| Lua   | 方向 | JSON   |
| ----- | :--: | ------ |
| `nil` | $→$  | 空字段 |
| `nil` | $←$  | `null` |

### 示例

**正确结构**

序列化：

```lua
data = {
  name = "TUI GAME",
  enabled = true,
  values = { 1, 2, 3 }
}

json = serialization.json_encode(data)
debug.print { message = json }
```

输出：

```json
{ "enabled": true, "name": "TUI GAME", "values": [1, 2, 3] }
```

反序列化：

```lua
json = '{"name":"TUI GAME","values":[1,2,3]}'
data = serialization.json_decode(json)
debug.print { message = tostring(data.name) }  -- TUI GAME
```

输出：

```text
TUI GAME
```

**错误结构**

> Lua 表值混合

```lua
{
  1,
  "A",
  obj = { ... }
}
```

> 数组表不连续，存在数据空洞

```lua
{
  [1] = "A",
  [3] = "C"
}
```

> `nil` 被当做可显式的 `null`

```lua
{
  is_null = nil -- JSON 为 {}
}
```

---

## TOML

### 根结构要求

TOML 的根值**必须是对象表**。

```lua
data = {
  key = value,
  ...
}
```

### 数据结构映射

**双向映射**

> 该部分值的映射可逆

| Lua       | TOML      |
| --------- | --------- |
| `boolean` | `boolean` |
| `integer` | `integer` |
| `number`  | `float`   |
| `string`  | `string`  |
| 数组表    | 数组      |
| 对象表    | 表        |

**单向映射**

> 该部分值的映射不可逆

| Lua      | 方向 | TOML   |
| -------- | :--: | ------ |
| `string` | $←$  | `date` |

### 示例

**正确示例**

序列化：

```lua
data = {
  title = "TUI GAME",
  window = { width = 120, height = 40 }
}

toml = serialization.toml_encode(data)
debug.print { message = toml }
```

输出：

```toml
title = "TUI GAME"

[window]
width = 120
height = 40
```

反序列化：：

```lua
toml = 'title = "TUI GAME"\n[window]\nwidth = 120'
data = serialization.toml_decode(toml)
debug.print { message = tostring(data.window.width) }
```

输出：

```text
120
```

**错误示例**

> 根值类型错误

```lua
{ 1, 2, 3 }
```

> 对象键为非字符串

```lua
{ [2] = "value" }
```

> 误认为单向映射值可逆

```toml
date = 2026-10-01T15:20:45
```

```lua
"2026-10-01T15:20:45" -- 被转换为字符串，且不可再转回日期类型
```

---

## YAML

### 根结构要求

YAML 的根值**必须是对象表**。

```lua
data = {
  key = value,
  ...
}
```

### 数据结构映射

**双向映射**

> 该部分值的映射可逆

| Lua       | YAML      |
| --------- | --------- |
| `boolean` | `boolean` |
| `integer` | `integer` |
| `number`  | `float`   |
| `string`  | `string`  |
| 数组表    | 序列      |
| 对象表    | 映射      |

**单向映射**

> 该部分值的映射不可逆

| Lua      | 方向 | YAML         |
| -------- | :--: | ------------ |
| `string` | $←$  | `date`       |
| `nil`    | $→$  | 空字段       |
| `nil`    | $←$  | `null` / `~` |

### 示例

**正确示例**

序列化：

```lua
data = {
  name = "TUI GAME",
  enabled = true,
  tags = { "tui", "lua" }
}

yaml = serialization.yaml_encode(data)
debug.print { message = yaml }
```

输出：

```yaml
enabled: true
name: TUI GAME
tags:
  - tui
  - lua
```

反序列化：：

```lua
yaml = "name: TUI GAME\ntags:\n- tui\n- lua"
data = serialization.yaml_decode(yaml)
debug.print { message = tostring(data.tags[1]) }
```

输出：

```text
tui
```

**错误示例**

> 原始数据中包含自定义标签

```yaml
name: !person TUI # 反序列化不支持自定义标签
```

> 映射键不是字符串

```yaml
1: one
```

> `nil` 被当做可显式的 `null`

```lua
{
  is_null = nil -- YAML 为空文件
}
```

---

## CSV

### 根结构要求

CSV 的根值**必须是二维数组表**。

```lua
data = {
  {string...},
  ...
}
```

### 数据结构映射

**双向映射**

> 该部分值的映射可逆

| Lua      | YAML     |
| -------- | -------- |
| `string` | `string` |

**单向映射**

> 该部分值的映射不可逆

| Lua       | 方向 | CSV      |
| --------- | :--: | -------- |
| `boolean` | $→$  | `string` |
| `integer` | $→$  | `string` |
| `number`  | $→$  | `string` |
| `string`  | $←$  | 空文本   |

### 示例

**正确示例**

序列化：

```lua
rows = {
  { "name", "age", "work" },
  { "Alice", 12, false },
  { "Bob", 30, true }
}

csv = serialization.csv_encode(rows)
debug.print { message = csv }
```

输出：

```csv
name,age,work
Alice,12,false
Bob,30,true
```

反序列化：：

```lua
csv = "name,age,work\nAlice,12,false\nBob,30,true"
rows = serialization.csv_decode(csv)
debug.print { message = tostring(rows[2][1]) }
debug.print { message = tostring(type(rows[2][2])) }
```

输出：

```text
Alice
string
```

**错误示例**

> 各行列数不一致

```lua
{
  { "name", "age" },
  { "Alice" }
}
```

> 过度嵌套

```lua
{
  { "name", { "nested" } }
}
```

> 误认为单向映射值可逆

```csv
true -- Lua 为 "true"
1    -- Lua 为 "1"
```

---

## XML

### 根结构要求

XML 的根值**必须遵循特定的表结构**。

```lua
data = {
  root = {            -- 根标签（仅一个）
    _attr = {         -- 属性
      key = value
      ...
    },
    _text = value,    -- 子元素（与子标签冲突，见下文）
    element = { ... } -- 子标签（与子元素冲突，见下文）
  }
}
```

XML 结构：

```xml
<root key = value>
  value                  <!-- 与子标签冲突，见下文 -->
  <element>...</element> <!-- 与子元素冲突，见下文 -->
</root>
```

### 数据结构映射

**双向映射**

> 该部分值的映射可逆

| Lua      | XML      |
| -------- | -------- |
| `string` | `string` |

**单向映射**

> 该部分值的映射不可逆

| Lua       | 方向 | XML      |
| --------- | :--: | -------- |
| `boolean` | $→$  | `string` |
| `integer` | $→$  | `string` |
| `number`  | $→$  | `string` |
| `nil`     | $→$  | 单标签   |
| `string`  | $←$  | 单标签   |

### 属性

属性保存在对象键 `_attr` 表中。

**示例**

序列化：

```lua
data = {
  root = {
    _attr = {
      version = "1.0",
      enabled = true
    },
    _text = "Hello"
  }
}

xml = serialization.xml_encode(data)
debug.print { message = xml }
```

输出：

```xml
<root enabled="true" version="1.0">Hello</root>
```

### 子元素

子元素保存在对象键 `_text` 中，或直接保存在对象键中。

> 子标签与子元素冲突，当存在子元素时，该标签不可包含子标签。

**示例**

序列化：

```lua
data1 = {
  root = {
    _text = "Hello"
  }
}

data2 = {
  root = "Hello"
}

xml1 = serialization.xml_encode(data1)
debug.print { message = xml1 }

xml2 = serialization.xml_encode(data2)
debug.print { message = xml2 }
```

输出：

```xml
<root>Hello</root>
<root>Hello</root>
```

### 子标签

其他任何键作为子标签。

> 子标签与子元素冲突，当存在子标签时，该标签不可包含子元素。

**示例**

序列化：

```lua
{
  player = {
    name = "Alice",
    score = 95
  }
}

{
  players = {
    player = {
      { name = "Alice" }, -- 同名使用连续数组表表示
      { name = "Bob" }
    }
  }
}
```

苏处：

```xml
<player>
  <name>Alice</name>
  <score>95</score>
</player>

<players>
  <player><name>Alice</name></player>
  <player><name>Bob</name></player>
</players>
```

### 转义规则

| 原字符 | 转义符   |
| ------ | -------- |
| `&`    | `&amp;`  |
| `<`    | `&lt;`   |
| `>`    | `&gt;`   |
| `"`    | `&quot;` |
| `'`    | `&apos;` |

### 示例

**正确示例**

序列化：

```lua
data = {
  root = {
    _attr = { version = "1.0" },
    item = {
      { name = "Alice" },
      { name = "Bob" }
    }
  }
}

xml = serialization.xml_encode(data)
debug.print { message = xml }
```

输出：

```xml
<root version="1.0">
  <item>
    <name>Alice</name>
  </item>
  <item>
    <name>Bob</name>
  </item>
</root>
```

反序列化：

```lua
xml = '<root version="1.0"><item>A</item><item>B</item></root>'
data = serialization.xml_decode(xml)
debug.print { message = tostring(data.root.item[1].name) }
```

输出：

```
Alice
```

**错误示例**

> 根标签不唯一

```lua
data = {
  root1 = "value1",
  root2 = "value2"
}
```

> 标签同时包含子标签和子元素

```lua
{
  root = {
    _text = "before",
    child = "value"
  }
}
```

> 子元素使用命名空间语法

```xml
<ns:root>value</ns:root> <!-- 反序列化不支持命名空间语法 -->
```

---

## INI

### 根结构要求

INI 的根值**必须是对象表**。

```lua
data = {
  key = value,
  ...
}
```

### 数据结构映射

**双向映射**

> 该部分值的映射可逆

| Lua      | INI      |
| -------- | -------- |
| `string` | `string` |
| 对象表   | 节       |

**单向映射**

> 该部分值的映射不可逆

| Lua       | 方向 | INI      |
| --------- | :--: | -------- |
| `boolean` | $→$  | `string` |
| `integer` | $→$  | `string` |
| `number`  | $→$  | `string` |
| `nil`     | $→$  | 空字段   |

### 示例

**正确示例**

```lua
data = {
  application = "TUI GAME",
  server = {
    host = "127.0.0.1",
    port = 8080
  }
}

ini = serialization.ini_encode(data)
debug.print { message = ini }
```

输出：

```ini
application = TUI GAME

[server]
host = 127.0.0.1
port = 8080
```

反序列化：：

```lua
ini = "[server]\nhost=127.0.0.1\nport=8080"
data = serialization.ini_decode(ini)
debug.print { message = data.server.host }
```

输出：

```text
127.0.0.1
```

**错误示例**

> 过度嵌套

```lua
data = {
  server = {
    network = {
      host = "127.0.0.1" -- ini 最多嵌套两层
    }
  }
}
```

> 键为空值

```ini
; 键不可为空
key =
```

> 键名包含非法字符

```ini
; 键名不可包含 # / ; / = / [ / ]
key# = value
```

---

## 二进制数据

## 示例

**正确示例**

打包：

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

解包：

```lua
bytes = serialization.binary_pack {
  fmt = "<I4 I4",
  values = { 100, 200 }
}

result = serialization.binary_unpack {
  fmt = "<I4 I4",
  data = bytes
}

debug.print { message = result.values[1] .. ", " .. result.values[2] }
```

输出：

```text
100, 200
```

查询大小：

```lua
size = serialization.binary_packsize("<I4 I4")
debug.print { message = tostring(size) }
```

输出：

```text
8
```

**错误示例**

> 参数 `fmt` 和参数 `values` 数量不匹配

```lua
serialization.binary_pack {
  fmt = "<I4 I4",
  values = { 100 }
}
```

> 参数 `data` 长度不足

```lua
serialization.binary_unpack {
  fmt = "<I4",
  data = "\x01"  -- 需要4字节
}
```

> 参数 `pos` 超出数据范围

```lua
serialization.binary_unpack {
  fmt = "<I2",
  data = "\1\2\3\4",
  pos = 6 -- 最大为 3
}
```
