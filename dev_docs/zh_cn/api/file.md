# file 库

## 基本库说明

`file` 提供异步文件操作。

---

## 目录

### 常量

| 常量名           | 说明                                  | 索引                              |
| ---------------- | ------------------------------------- | --------------------------------- |
| `AUTO`           | 自动检测模式                          | [AUTO](#AUTO)                     |
| `ALL`            | 全部统一模式                          | [ALL](#ALL)                       |
| `CR`             | 回车换行符                            | [CR](#CR)                         |
| `LF`             | 换行符                                | [LF](#LF)                         |
| `CRLF`           | 回车换行符组合                        | [CRLF](#CRLF)                     |
| `UTF_8`          | UTF-8 编码                            | [UTF_8](#UTF_8)                   |
| `UTF_16LE`       | UTF-16 小端编码                       | [UTF_16LE](#UTF_16LE)             |
| `UTF_16BE`       | UTF-16 大端编码                       | [UTF_16BE](#UTF_16BE)             |
| `GBK`            | GBK 编码（简体中文）                  | [GBK](#GBK)                       |
| `GB18030`        | GB18030 编码（简体中文）              | [GB18030](#GB18030)               |
| `BIG5`           | BIG5 编码（繁体中文）                 | [BIG5](#BIG5)                     |
| `SHIFT_JIS`      | Shift JIS 编码（日文）                | [SHIFT_JIS](#SHIFT_JIS)           |
| `EUC_JP`         | EUC-JP 编码（日文）                   | [EUC_JP](#EUC_JP)                 |
| `ISO_2022_JP`    | ISO-2022-JP 编码（日文）              | [ISO_2022_JP](#ISO_2022_JP)       |
| `EUC_KR`         | EUC-KR 编码（韩文）                   | [EUC_KR](#EUC_KR)                 |
| `WINDOWS_874`    | Windows-874 编码（泰文）              | [WINDOWS_874](#WINDOWS_874)       |
| `WINDOWS_1250`   | Windows-1250 编码（中欧）             | [WINDOWS_1250](#WINDOWS_1250)     |
| `WINDOWS_1251`   | Windows-1251 编码（西里尔）           | [WINDOWS_1251](#WINDOWS_1251)     |
| `WINDOWS_1252`   | Windows-1252 编码（西欧）             | [WINDOWS_1252](#WINDOWS_1252)     |
| `WINDOWS_1253`   | Windows-1253 编码（希腊）             | [WINDOWS_1253](#WINDOWS_1253)     |
| `WINDOWS_1254`   | Windows-1254 编码（土耳其）           | [WINDOWS_1254](#WINDOWS_1254)     |
| `WINDOWS_1255`   | Windows-1255 编码（希伯来）           | [WINDOWS_1255](#WINDOWS_1255)     |
| `WINDOWS_1256`   | Windows-1256 编码（阿拉伯）           | [WINDOWS_1256](#WINDOWS_1256)     |
| `WINDOWS_1257`   | Windows-1257 编码（波罗的海）         | [WINDOWS_1257](#WINDOWS_1257)     |
| `WINDOWS_1258`   | Windows-1258 编码（越南）             | [WINDOWS_1258](#WINDOWS_1258)     |
| `ISO_8859_2`     | ISO-8859-2 编码（中欧）               | [ISO_8859_2](#ISO_8859_2)         |
| `ISO_8859_3`     | ISO-8859-3 编码（南欧）               | [ISO_8859_3](#ISO_8859_3)         |
| `ISO_8859_4`     | ISO-8859-4 编码（北欧）               | [ISO_8859_4](#ISO_8859_4)         |
| `ISO_8859_5`     | ISO-8859-5 编码（西里尔）             | [ISO_8859_5](#ISO_8859_5)         |
| `ISO_8859_6`     | ISO-8859-6 编码（阿拉伯）             | [ISO_8859_6](#ISO_8859_6)         |
| `ISO_8859_7`     | ISO-8859-7 编码（希腊）               | [ISO_8859_7](#ISO_8859_7)         |
| `ISO_8859_8`     | ISO-8859-8 编码（希伯来）             | [ISO_8859_8](#ISO_8859_8)         |
| `ISO_8859_8_I`   | ISO-8859-8-I 编码（希伯来，逻辑顺序） | [ISO_8859_8_I](#ISO_8859_8_I)     |
| `ISO_8859_9`     | ISO-8859-9 编码（土耳其）             | [ISO_8859_9](#ISO_8859_9)         |
| `ISO_8859_10`    | ISO-8859-10 编码（北欧）              | [ISO_8859_10](#ISO_8859_10)       |
| `ISO_8859_11`    | ISO-8859-11 编码（泰文）              | [ISO_8859_11](#ISO_8859_11)       |
| `ISO_8859_13`    | ISO-8859-13 编码（波罗的海）          | [ISO_8859_13](#ISO_8859_13)       |
| `ISO_8859_14`    | ISO-8859-14 编码（凯尔特）            | [ISO_8859_14](#ISO_8859_14)       |
| `ISO_8859_15`    | ISO-8859-15 编码（西欧）              | [ISO_8859_15](#ISO_8859_15)       |
| `ISO_8859_16`    | ISO-8859-16 编码（东南欧）            | [ISO_8859_16](#ISO_8859_16)       |
| `KOI8_R`         | KOI8-R 编码（俄文）                   | [KOI8_R](#KOI8_R)                 |
| `KOI8_U`         | KOI8-U 编码（乌克兰）                 | [KOI8_U](#KOI8_U)                 |
| `IBM866`         | IBM866 编码（俄文）                   | [IBM866](#IBM866)                 |
| `MACINTOSH`      | Macintosh 编码（西欧）                | [MACINTOSH](#MACINTOSH)           |
| `X_MAC_CYRILLIC` | x-mac-cyrillic 编码（西里尔）         | [X_MAC_CYRILLIC](#X_MAC_CYRILLIC) |

### 方法

| 方法名       | 说明                                          | 索引                      |
| ------------ | --------------------------------------------- | ------------------------- |
| `read`       | 异步读取 `assets/` 目录下的文本文件           | [read](#read)             |
| `write`      | 异步写入文本文件到 `assets/` 目录             | [write](#write)           |
| `list_dir`   | 异步枚举 `assets/` 目录下的条目               | [list_dir](#list_dir)     |
| `create_dir` | 异步创建指定目录到 `assets/` 目录             | [create_dir](#create_dir) |
| `exists`     | 判断 `assets/` 目录下的指定文件或目录是否存在 | [exists](#exists)         |
| `remove`     | 异步删除 `assets/` 目录下指定文件或目录       | [remove](#remove)         |

---

## 常量

## `AUTO`

自动检测模式。

**可用于**

- 参数 `encoding`
- 参数 `end_of_line`

### 调用

```lua
file.AUTO
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.AUTO, end_of_line = file.AUTO }
```

---

## `ALL`

全部统一模式。

**可用于**

- 参数 `file_type`

### 调用

```lua
file.ALL
```

### 示例

```lua
file.list_dir { path = "dir/", file_type = file.ALL }
```

---

## `CR`

回车换行符。

**可用于**

- 参数 `end_of_line`

### 调用

```lua
file.CR
```

### 示例

```lua
file.read { path = "file.txt", end_of_line = file.CR }
```

---

## `LF`

换行符。

**可用于**

- 参数 `end_of_line`

### 调用

```lua
file.LF
```

### 示例

```lua
file.read { path = "file.txt", end_of_line = file.LF }
```

---

## `CRLF`

回车换行符组合。

**可用于**

- 参数 `end_of_line`

### 调用

```lua
file.CRLF
```

### 示例

```lua
file.read { path = "file.txt", end_of_line = file.CRLF }
```

---

## `UTF_8`

UTF-8 编码。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.UTF_8
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.UTF_8 }
```

---

## `UTF_16LE`

UTF-16 小端编码。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.UTF_16LE
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.UTF_16LE }
```

---

## `UTF_16BE`

UTF-16 大端编码。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.UTF_16BE
```

### 示例

````lua
file.read { path = "file.txt", encoding = file.UTF_16BE }

---

## `GBK`

GBK 编码（简体中文）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.GBK
````

### 示例

```lua
file.read { path = "file.txt", encoding = file.GBK }
```

---

## `GB18030`

GB18030 编码（简体中文）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.GB18030
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.GB18030 }
```

---

## `BIG5`

BIG5 编码（繁体中文）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.BIG5
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.BIG5 }
```

---

## `SHIFT_JIS`

Shift JIS 编码（日文）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.SHIFT_JIS
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.SHIFT_JIS }
```

---

## `EUC_JP`

EUC-JP 编码（日文）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.EUC_JP
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.EUC_JP }
```

---

## `ISO_2022_JP`

ISO-2022-JP 编码（日文）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_2022_JP
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_2022_JP }
```

---

## `EUC_KR`

EUC-KR 编码（韩文）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.EUC_KR
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.EUC_KR }
```

---

## `WINDOWS_874`

Windows-874 编码（泰文）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.WINDOWS_874
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.WINDOWS_874 }
```

---

## `WINDOWS_1250`

Windows-1250 编码（中欧）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.WINDOWS_1250
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.WINDOWS_1250 }
```

---

## `WINDOWS_1251`

Windows-1251 编码（西里尔）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.WINDOWS_1251
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.WINDOWS_1251 }
```

---

## `WINDOWS_1252`

Windows-1252 编码（西欧）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.WINDOWS_1252
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.WINDOWS_1252 }
```

---

## `WINDOWS_1253`

Windows-1253 编码（希腊）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.WINDOWS_1253
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.WINDOWS_1253 }
```

---

## `WINDOWS_1254`

Windows-1254 编码（土耳其）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.WINDOWS_1254
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.WINDOWS_1254 }
```

---

## `WINDOWS_1255`

Windows-1255 编码（希伯来）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.WINDOWS_1255
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.WINDOWS_1255 }
```

---

## `WINDOWS_1256`

Windows-1256 编码（阿拉伯）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.WINDOWS_1256
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.WINDOWS_1256 }
```

---

## `WINDOWS_1257`

Windows-1257 编码（波罗的海）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.WINDOWS_1257
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.WINDOWS_1257 }
```

---

## `WINDOWS_1258`

Windows-1258 编码（越南）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.WINDOWS_1258
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.WINDOWS_1258 }
```

---

## `ISO_8859_2`

ISO-8859-2 编码（中欧）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_2
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_2 }
```

---

## `ISO_8859_3`

ISO-8859-3 编码（南欧）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_3
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_3 }
```

---

## `ISO_8859_4`

ISO-8859-4 编码（北欧）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_4
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_4 }
```

---

## `ISO_8859_5`

ISO-8859-5 编码（西里尔）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_5
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_5 }
```

---

## `ISO_8859_6`

ISO-8859-6 编码（阿拉伯）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_6
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_6 }
```

---

## `ISO_8859_7`

ISO-8859-7 编码（希腊）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_7
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_7 }
```

---

## `ISO_8859_8`

ISO-8859-8 编码（希伯来）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_8
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_8 }
```

---

## `ISO_8859_8_I`

ISO-8859-8-I 编码（希伯来，逻辑顺序）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_8_I
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_8_I }
```

---

## `ISO_8859_9`

ISO-8859-9 编码（土耳其）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_9
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_9 }
```

---

## `ISO_8859_10`

ISO-8859-10 编码（北欧）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_10
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_10 }
```

---

## `ISO_8859_11`

ISO-8859-11 编码（泰文）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_11
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_11 }
```

---

## `ISO_8859_13`

ISO-8859-13 编码（波罗的海）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_13
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_13 }
```

---

## `ISO_8859_14`

ISO-8859-14 编码（凯尔特）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_14
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_14 }
```

---

## `ISO_8859_15`

ISO-8859-15 编码（西欧）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_15
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_15 }
```

---

## `ISO_8859_16`

ISO-8859-16 编码（东南欧）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.ISO_8859_16
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.ISO_8859_16 }
```

---

## `KOI8_R`

KOI8-R 编码（俄文）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.KOI8_R
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.KOI8_R }
```

---

## `KOI8_U`

KOI8-U 编码（乌克兰）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.KOI8_U
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.KOI8_U }
```

---

## `IBM866`

IBM866 编码（俄文）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.IBM866
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.IBM866 }
```

---

## `MACINTOSH`

Macintosh 编码（西欧）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.MACINTOSH
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.MACINTOSH }
```

---

## `X_MAC_CYRILLIC`

x-mac-cyrillic 编码（西里尔）。

**可用于**

- 参数 `encoding`

### 调用

```lua
file.X_MAC_CYRILLIC
```

### 示例

```lua
file.read { path = "file.txt", encoding = file.X_MAC_CYRILLIC }
```

---

## 方法

## `read`

异步读取 `assets/` 目录下的文本文件。

### 调用

```lua
-- 表参数
file.read{}
```

### 参数

| 参数名        | 类型         | 必填 | 默认值        | 说明                      |
| ------------- | ------------ | ---- | ------------- | ------------------------- |
| `path`        | string       | 是   | -             | 相对 `assets/` 的文件路径 |
| `encoding`    | const-file   | 否   | `file.AUTO`   | 文本编码                  |
| `end_of_line` | const-file   | 否   | `file.AUTO` | 换行符规范                |
| `byte`        | boolean      | 否   | `false`       | 二进制模式                |
| `event_tip`   | string / nil | 否   | `nil`         | 自定义事件标记            |

### 返回

事件返回，请查看⌊[事件结构](../EVENT.md)⌉文档⌊file⌉部分。

### 示例

```lua
assets/
- file.txt

file.read { path = "file.txt" }

function HandleEvent(event)
  if event.type == "file" then
    debug.print { message = serialization.json_encode(event) }
  end
end
```

输出：

> X 为占位符

```json
{
  "type": "file",
  "frame": X,
  "sequence": X,
  "data": {
    "request_id": X,
    "path": "file.txt",
    "ok": true,
    "kind": "read_text",
    "text": "Hello Tui Game"
  }
}
```

### 额外补充

- 参数 `byte` 为 false 时按文本读取，参数 `encoding` 与 参数 `end_of_line` **生效**。
- 参数 `byte` 为 false 时按二进制读取，参数 `encoding` 与 参数 `end_of_line` **忽略**。

---

## `write`

异步写入文本文件到 `assets/` 目录。

> 需关闭安全模式。
> 仅游戏脚本可用。

### 调用

```lua
-- 表参数
file.write{}
```

### 参数

| 参数名        | 类型         | 必填 | 默认值      | 说明                      |
| ------------- | ------------ | ---- | ----------- | ------------------------- |
| `path`        | string       | 是   | -           | 相对 `assets/` 的文件路径 |
| `text`        | string       | 是   | -           | 要写入的文本              |
| `encoding`    | const-file   | 否   | `file.AUTO` | 文本编码                  |
| `end_of_line` | const-file   | 否   | `file.AUTO` | 换行符规范                |
| `byte`        | boolean      | 否   | `false`     | 二进制模式                |
| `event_tip`   | string / nil | 否   | `nil`       | 事件提示文本              |

### 返回

事件返回，请查看⌊[事件结构](../EVENT.md)⌉文档⌊file⌉部分。

### 示例

```lua
assets/
- file.txt

file.write { path = "file.txt", text = "Hello Tui Game", event_tip = "Get!" }

function HandleEvent(event)
  if event.type == "file" then
    debug.print { message = serialization.json_encode(event) }
  end
end
```

输出：

> X 为占位符

```json
{
  "type": "file",
  "frame": X,
  "sequence": X,
  "data": {
    "request_id": X,
    "path": "file.txt",
    "ok": true,
    "kind": "write_text",
    "tip": "Get!"
  }
}
```

### 额外补充

- 参数 `byte` 为 false 时按文本读取，参数 `encoding` 与 参数 `end_of_line` **生效**。
- 参数 `byte` 为 false 时按二进制读取，参数 `encoding` 与 参数 `end_of_line` **忽略**。
- 该 API 会自动创建未创建的**文件**。
- 该 API 不会自动补全未创建的**目录**，目录不存在会抛出错误。

---

## `list_dir`

异步枚举 `assets/` 目录下的条目。

> 需关闭安全模式。
> 仅游戏脚本可用。

### 调用

```lua
-- 表参数
file.list_dir{}
```

### 参数

| 参数名      | 类型                | 必填 | 默认值     | 说明                      |
| ----------- | ------------------- | ---- | ---------- | ------------------------- |
| `path`      | string              | 是   | -          | 相对 `assets/` 目录路径 |
| `recursive` | boolean             | 否   | `false`    | 是否递归子目录枚举        |
| `file_type` | string / const-file | 否   | `file.ALL` | 仅匹配指定扩展名          |
| `event_tip` | string / nil        | 否   | `nil`      | 事件提示文本              |

### 返回

事件返回，请查看⌊[事件结构](../EVENT.md)⌉文档⌊file⌉部分。

### 示例

```lua
assets/
+ c/
| + main.c
| - game.c
+ js/
| + data.json
| - main.js
+ rust/
| + src/
| | - main.rs
| - Cargo.toml
- file.txt

file.list_dir { path = ".", recursive = true }
file.list_dir { path = "rust/" }
file.list_dir { path = "js/", file_type = "json", event_tip = "Only Json" }

function HandleEvent(event)
  if event.type == "file" then
    debug.print { message = serialization.json_encode(event) }
  end
end
```

输出：

> X 为占位符

```json
{
  "type": "file",
  "sequence": X,
  "frame": X,
  "data": {
    "request_id": X,
    "ok": true,
    "kind": "list_dir",
    "path": ".",
    "entries": [
      {
        "path": "c/game.c",
        "file_type": "c"
      },
      {
        "path": "c/main.c",
        "file_type": "c"
      },
      {
        "path": "file.txt",
        "file_type": "txt"
      },
      {
        "path": "js/data.json",
        "file_type": "json"
      },
      {
        "path": "js/main.js",
        "file_type": "js"
      },
      {
        "path": "rust/Cargo.toml",
        "file_type": "toml"
      },
      {
        "path": "rust/src/main.rs",
        "file_type": "rs"
      }
    ],
  }
}

{
  "type": "file",
  "frame": X,
  "sequence": X,
  "data": {
    "request_id": X,
    "ok": true,
    "kind": "list_dir",
    "path": "rust/",
    "entries": [
      {
        "path": "Cargo.toml",
        "file_type": "toml"
      }
    ],
  }
}

{
  "type": "file",
  "frame": X,
  "sequence": X,
  "data": {
    "request_id": X,
    "ok": true,
    "tip": "Only Json",
    "kind": "list_dir",
    "path": "js/",
    "entries": [
      {
        "path": "data.json",
        "file_type": "json"
      }
    ],
  }
}
```

---

## `create_dir`

异步创建指定目录到 `assets/` 目录。

> 需关闭安全模式。
> 仅游戏脚本可用。

### 调用

```lua
-- 表参数
file.create_dir{}
```

### 参数

| 参数名      | 类型         | 必填 | 默认值 | 说明                      |
| ----------- | ------------ | ---- | ------ | ------------------------- |
| `path`      | string       | 是   | -      | 相对 `assets/` 目录路径 |
| `event_tip` | string / nil | 否   | `nil`  | 事件提示文本              |

### 返回

事件返回，请查看⌊[事件结构](../EVENT.md)⌉文档⌊file⌉部分。

### 示例

```lua
assets/

file.create_dir { path = "file" }
file.create_dir { path = "test1/test2" }

function HandleEvent(event)
  if event.type == "file" then
    debug.print { message = serialization.json_encode(event) }
  end
end
```

输出：

> X 为占位符

```json
assets/
+ file/
- test1/
  - test2/

{
  "type": "file",
  "frame": X,
  "sequence": X,
  "data": {
    "request_id": X,
    "ok": true,
    "kind": "create_dir",
    "path": "file"
  },
}

{
  "type": "file",
  "frame": X,
  "sequence": X,
  "data": {
    "request_id": X,
    "ok": true,
    "kind": "create_dir",
    "path": "test1/test2"
  },
}
```

### 额外补充

- 该 API **支持**链式创建目录。

---

## `exists`

判断 `assets/` 目录下的指定文件或目录是否存在。

### 调用

```lua
-- 单参数
file.exists()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明                      |
| ------ | ------ | ---- | ------ | ------------------------- |
| `path` | string | 是   | -      | 相对 `assets/` 目录路径 |

### 返回

直接返回一个值。

| 类型    | 说明               |
| ------- | ------------------ |
| boolean | 文件或目录是否存在 |

### 示例

```lua
assets/
- test/

debug.print { message = tostring(file.exists("test")) }
debug.print { message = tostring(file.exists("none")) }
```

输出：

```test
true
false
```

---

## `remove`

异步删除 `assets/` 目录下指定文件或目录。

> 需关闭安全模式。
> 仅游戏脚本可用。

### 调用

```lua
-- 表参数
file.remove{}
```

### 参数

| 参数名      | 类型         | 必填 | 默认值  | 说明                      |
| ----------- | ------------ | ---- | ------- | ------------------------- |
| `path`      | string       | 是   | -       | 相对 `assets/` 目录路径 |
| `recursive` | boolean      | 否   | `false` | 是否删除非空目录          |
| `event_tip` | string / nil | 否   | `nil`   | 事件提示文本              |

### 返回

事件返回，请查看⌊[事件结构](../EVENT.md)⌉文档⌊file⌉部分。

### 示例

```lua
assets/
+ test/
| - test.txt
- file.txt

file.remove { path = "file.txt" }
file.remove { path = "test", recursive = false }

function HandleEvent(event)
  if event.type == "file" then
    debug.print { message = serialization.json_encode(event) }
  end
end
```

输出：

> X 为占位符

```json
assets/
+ test/
  - test.txt

{
  "type": "file",
  "frame": X,
  "sequence": X,
  "data": {
    "request_id": X,
    "ok": true,
    "kind": "remove",
    "path": "file.txt"
  },
}

{
  "type": "file",
  "frame": X,
  "sequence": X,
  "data": {
    "request_id": X,
    "ok": false,
    "kind": "remove",
    "path": "test",
    "error": {
      "code": "io",
      "message": "I/O operation failed"
    },
  }
}
```

### 额外补充

- 该 API 一次仅删除**单个**文件或目录，无法链式删除。
