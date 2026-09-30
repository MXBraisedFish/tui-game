# file 库

`file` 提供 `assets/` 目录中的异步文件操作。

---

# 目录

## 常量

| 常量           | 说明                                  | 定位                              |
| ---------------- | ------------------------------------- | --------------------------------- |
| `AUTO`           | 自动检测模式                          | [AUTO](#auto)                     |
| `ALL`            | 全部统一模式                          | [ALL](#all)                       |
| `CR`             | 回车换行符                            | [CR](#cr)                         |
| `LF`             | 换行符                                | [LF](#lf)                         |
| `CRLF`           | 回车换行符组合                        | [CRLF](#crlf)                     |
| `UTF_8`          | UTF-8 编码                            | [UTF_8](#utf_8)                   |
| `UTF_16LE`       | UTF-16 小端编码                       | [UTF_16LE](#utf_16le)             |
| `UTF_16BE`       | UTF-16 大端编码                       | [UTF_16BE](#utf_16be)             |
| `GBK`            | GBK 编码（简体中文）                  | [GBK](#gbk)                       |
| `GB18030`        | GB18030 编码（简体中文）              | [GB18030](#gb18030)               |
| `BIG5`           | BIG5 编码（繁体中文）                 | [BIG5](#big5)                     |
| `SHIFT_JIS`      | Shift JIS 编码（日文）                | [SHIFT_JIS](#shift_jis)           |
| `EUC_JP`         | EUC-JP 编码（日文）                   | [EUC_JP](#euc_jp)                 |
| `ISO_2022_JP`    | ISO-2022-JP 编码（日文）              | [ISO_2022_JP](#iso_2022_jp)       |
| `EUC_KR`         | EUC-KR 编码（韩文）                   | [EUC_KR](#euc_kr)                 |
| `WINDOWS_874`    | Windows-874 编码（泰文）              | [WINDOWS_874](#windows_874)       |
| `WINDOWS_1250`   | Windows-1250 编码（中欧）             | [WINDOWS_1250](#windows_1250)     |
| `WINDOWS_1251`   | Windows-1251 编码（西里尔）           | [WINDOWS_1251](#windows_1251)     |
| `WINDOWS_1252`   | Windows-1252 编码（西欧）             | [WINDOWS_1252](#windows_1252)     |
| `WINDOWS_1253`   | Windows-1253 编码（希腊）             | [WINDOWS_1253](#windows_1253)     |
| `WINDOWS_1254`   | Windows-1254 编码（土耳其）           | [WINDOWS_1254](#windows_1254)     |
| `WINDOWS_1255`   | Windows-1255 编码（希伯来）           | [WINDOWS_1255](#windows_1255)     |
| `WINDOWS_1256`   | Windows-1256 编码（阿拉伯）           | [WINDOWS_1256](#windows_1256)     |
| `WINDOWS_1257`   | Windows-1257 编码（波罗的海）         | [WINDOWS_1257](#windows_1257)     |
| `WINDOWS_1258`   | Windows-1258 编码（越南）             | [WINDOWS_1258](#windows_1258)     |
| `ISO_8859_2`     | ISO-8859-2 编码（中欧）               | [ISO_8859_2](#iso_8859_2)         |
| `ISO_8859_3`     | ISO-8859-3 编码（南欧）               | [ISO_8859_3](#iso_8859_3)         |
| `ISO_8859_4`     | ISO-8859-4 编码（北欧）               | [ISO_8859_4](#iso_8859_4)         |
| `ISO_8859_5`     | ISO-8859-5 编码（西里尔）             | [ISO_8859_5](#iso_8859_5)         |
| `ISO_8859_6`     | ISO-8859-6 编码（阿拉伯）             | [ISO_8859_6](#iso_8859_6)         |
| `ISO_8859_7`     | ISO-8859-7 编码（希腊）               | [ISO_8859_7](#iso_8859_7)         |
| `ISO_8859_8`     | ISO-8859-8 编码（希伯来）             | [ISO_8859_8](#iso_8859_8)         |
| `ISO_8859_8_I`   | ISO-8859-8-I 编码（希伯来，逻辑顺序） | [ISO_8859_8_I](#iso_8859_8_i)     |
| `ISO_8859_10`    | ISO-8859-10 编码（北欧）              | [ISO_8859_10](#iso_8859_10)       |
| `ISO_8859_13`    | ISO-8859-13 编码（波罗的海）          | [ISO_8859_13](#iso_8859_13)       |
| `ISO_8859_14`    | ISO-8859-14 编码（凯尔特）            | [ISO_8859_14](#iso_8859_14)       |
| `ISO_8859_15`    | ISO-8859-15 编码（西欧）              | [ISO_8859_15](#iso_8859_15)       |
| `ISO_8859_16`    | ISO-8859-16 编码（东南欧）            | [ISO_8859_16](#iso_8859_16)       |
| `KOI8_R`         | KOI8-R 编码（俄文）                   | [KOI8_R](#koi8_r)                 |
| `KOI8_U`         | KOI8-U 编码（乌克兰）                 | [KOI8_U](#koi8_u)                 |
| `IBM866`         | IBM866 编码（俄文）                   | [IBM866](#ibm866)                 |
| `MACINTOSH`      | Macintosh 编码（西欧）                | [MACINTOSH](#macintosh)           |
| `X_MAC_CYRILLIC` | x-mac-cyrillic 编码（西里尔）         | [X_MAC_CYRILLIC](#x_mac_cyrillic) |

## 方法

| 方法       | 说明                                          | 定位                      |
| ------------ | --------------------------------------------- | ------------------------- |
| `read`       | 异步读取 `assets/` 目录下的文本文件           | [read](#read)             |
| `write`      | 异步写入文本文件到 `assets/` 目录             | [write](#write)           |
| `list_dir`   | 异步枚举 `assets/` 目录下的条目               | [list_dir](#list_dir)     |
| `create_dir` | 异步创建指定目录到 `assets/` 目录             | [create_dir](#create_dir) |
| `exists`     | 判断 `assets/` 目录下的指定文件或目录是否存在 | [exists](#exists)         |
| `remove`     | 异步删除 `assets/` 目录下指定文件或目录       | [remove](#remove)         |

---

# 常量

## `AUTO`

自动检测模式。

### 调用

```lua
file.AUTO
```

### 可用于

- 参数 `encoding`
- 参数 `end_of_line`

### 示例

```lua
file.read("file.txt", {encoding = file.AUTO, end_of_line = file.AUTO})
```

**输出：**

```lua
```

### 等值

```text
"auto"
```

---

## `ALL`

全部统一模式。

### 调用

```lua
file.ALL
```

### 可用于

- 参数 `file_type`

### 示例

```lua
file.list_dir("dir/", {file_type = file.ALL})
```

**输出：**

```lua
```

### 等值

```text
"all"
```

---

## `CR`

回车换行符。

### 调用

```lua
file.CR
```

### 可用于

- 参数 `end_of_line`

### 示例

```lua
file.read("file.txt", {end_of_line = file.CR})
```

**输出：**

```lua
```

### 等值

```text
"cr"
```

---

## `LF`

换行符。

### 调用

```lua
file.LF
```

### 可用于

- 参数 `end_of_line`

### 示例

```lua
file.read("file.txt", {end_of_line = file.LF})
```

**输出：**

```lua
```

### 等值

```text
"lf"
```

---

## `CRLF`

回车换行符组合。

### 调用

```lua
file.CRLF
```

### 可用于

- 参数 `end_of_line`

### 示例

```lua
file.read("file.txt", {end_of_line = file.CRLF})
```

**输出：**

```lua
```

### 等值

```text
"crlf"
```

---

## `UTF_8`

UTF-8 编码。

### 调用

```lua
file.UTF_8
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.UTF_8})
```

**输出：**

```lua
```

### 等值

```text
"utf-8"
```

---

## `UTF_16LE`

UTF-16 小端编码。

### 调用

```lua
file.UTF_16LE
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.UTF_16LE})
```

**输出：**

```lua
```

### 等值

```text
"utf-16le"
```

---

## `UTF_16BE`

UTF-16 大端编码。

### 调用

```lua
file.UTF_16BE
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.UTF_16BE})
```

**输出：**

```lua
```

### 等值

```text
"utf-16be"
```
## `GBK`

GBK 编码（简体中文）。

### 调用

```lua
file.GBK
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.GBK})
```

**输出：**

```lua
```

### 等值

```text
"gbk"
```

---

## `GB18030`

GB18030 编码（简体中文）。

### 调用

```lua
file.GB18030
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.GB18030})
```

**输出：**

```lua
```

### 等值

```text
"gb18030"
```

---

## `BIG5`

BIG5 编码（繁体中文）。

### 调用

```lua
file.BIG5
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.BIG5})
```

**输出：**

```lua
```

### 等值

```text
"big5"
```

---

## `SHIFT_JIS`

Shift JIS 编码（日文）。

### 调用

```lua
file.SHIFT_JIS
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.SHIFT_JIS})
```

**输出：**

```lua
```

### 等值

```text
"shift_jis"
```

---

## `EUC_JP`

EUC-JP 编码（日文）。

### 调用

```lua
file.EUC_JP
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.EUC_JP})
```

**输出：**

```lua
```

### 等值

```text
"euc-jp"
```

---

## `ISO_2022_JP`

ISO-2022-JP 编码（日文）。

### 调用

```lua
file.ISO_2022_JP
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.ISO_2022_JP})
```

**输出：**

```lua
```

### 等值

```text
"iso-2022-jp"
```

---

## `EUC_KR`

EUC-KR 编码（韩文）。

### 调用

```lua
file.EUC_KR
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.EUC_KR})
```

**输出：**

```lua
```

### 等值

```text
"euc-kr"
```

---

## `WINDOWS_874`

Windows-874 编码（泰文）。

### 调用

```lua
file.WINDOWS_874
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.WINDOWS_874})
```

**输出：**

```lua
```

### 等值

```text
"windows-874"
```

---

## `WINDOWS_1250`

Windows-1250 编码（中欧）。

### 调用

```lua
file.WINDOWS_1250
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.WINDOWS_1250})
```

**输出：**

```lua
```

### 等值

```text
"windows-1250"
```

---

## `WINDOWS_1251`

Windows-1251 编码（西里尔）。

### 调用

```lua
file.WINDOWS_1251
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.WINDOWS_1251})
```

**输出：**

```lua
```

### 等值

```text
"windows-1251"
```

---

## `WINDOWS_1252`

Windows-1252 编码（西欧）。

### 调用

```lua
file.WINDOWS_1252
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.WINDOWS_1252})
```

**输出：**

```lua
```

### 等值

```text
"windows-1252"
```

---

## `WINDOWS_1253`

Windows-1253 编码（希腊）。

### 调用

```lua
file.WINDOWS_1253
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.WINDOWS_1253})
```

**输出：**

```lua
```

### 等值

```text
"windows-1253"
```

---

## `WINDOWS_1254`

Windows-1254 编码（土耳其）。

### 调用

```lua
file.WINDOWS_1254
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.WINDOWS_1254})
```

**输出：**

```lua
```

### 等值

```text
"windows-1254"
```

---

## `WINDOWS_1255`

Windows-1255 编码（希伯来）。

### 调用

```lua
file.WINDOWS_1255
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.WINDOWS_1255})
```

**输出：**

```lua
```

### 等值

```text
"windows-1255"
```

---

## `WINDOWS_1256`

Windows-1256 编码（阿拉伯）。

### 调用

```lua
file.WINDOWS_1256
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.WINDOWS_1256})
```

**输出：**

```lua
```

### 等值

```text
"windows-1256"
```

---

## `WINDOWS_1257`

Windows-1257 编码（波罗的海）。

### 调用

```lua
file.WINDOWS_1257
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.WINDOWS_1257})
```

**输出：**

```lua
```

### 等值

```text
"windows-1257"
```

---

## `WINDOWS_1258`

Windows-1258 编码（越南）。

### 调用

```lua
file.WINDOWS_1258
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.WINDOWS_1258})
```

**输出：**

```lua
```

### 等值

```text
"windows-1258"
```

---

## `ISO_8859_2`

ISO-8859-2 编码（中欧）。

### 调用

```lua
file.ISO_8859_2
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.ISO_8859_2})
```

**输出：**

```lua
```

### 等值

```text
"iso-8859-2"
```

---

## `ISO_8859_3`

ISO-8859-3 编码（南欧）。

### 调用

```lua
file.ISO_8859_3
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.ISO_8859_3})
```

**输出：**

```lua
```

### 等值

```text
"iso-8859-3"
```

---

## `ISO_8859_4`

ISO-8859-4 编码（北欧）。

### 调用

```lua
file.ISO_8859_4
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.ISO_8859_4})
```

**输出：**

```lua
```

### 等值

```text
"iso-8859-4"
```

---

## `ISO_8859_5`

ISO-8859-5 编码（西里尔）。

### 调用

```lua
file.ISO_8859_5
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.ISO_8859_5})
```

**输出：**

```lua
```

### 等值

```text
"iso-8859-5"
```

---

## `ISO_8859_6`

ISO-8859-6 编码（阿拉伯）。

### 调用

```lua
file.ISO_8859_6
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.ISO_8859_6})
```

**输出：**

```lua
```

### 等值

```text
"iso-8859-6"
```

---

## `ISO_8859_7`

ISO-8859-7 编码（希腊）。

### 调用

```lua
file.ISO_8859_7
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.ISO_8859_7})
```

**输出：**

```lua
```

### 等值

```text
"iso-8859-7"
```

---

## `ISO_8859_8`

ISO-8859-8 编码（希伯来）。

### 调用

```lua
file.ISO_8859_8
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.ISO_8859_8})
```

**输出：**

```lua
```

### 等值

```text
"iso-8859-8"
```

---

## `ISO_8859_8_I`

ISO-8859-8-I 编码（希伯来，逻辑顺序）。

### 调用

```lua
file.ISO_8859_8_I
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.ISO_8859_8_I})
```

**输出：**

```lua
```

### 等值

```text
"iso-8859-8-i"
```

---

## `ISO_8859_10`

ISO-8859-10 编码（北欧）。

### 调用

```lua
file.ISO_8859_10
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.ISO_8859_10})
```

**输出：**

```lua
```

### 等值

```text
"iso-8859-10"
```

---

## `ISO_8859_13`

ISO-8859-13 编码（波罗的海）。

### 调用

```lua
file.ISO_8859_13
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.ISO_8859_13})
```

**输出：**

```lua
```

### 等值

```text
"iso-8859-13"
```

---

## `ISO_8859_14`

ISO-8859-14 编码（凯尔特）。

### 调用

```lua
file.ISO_8859_14
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.ISO_8859_14})
```

**输出：**

```lua
```

### 等值

```text
"iso-8859-14"
```

---

## `ISO_8859_15`

ISO-8859-15 编码（西欧）。

### 调用

```lua
file.ISO_8859_15
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.ISO_8859_15})
```

**输出：**

```lua
```

### 等值

```text
"iso-8859-15"
```

---

## `ISO_8859_16`

ISO-8859-16 编码（东南欧）。

### 调用

```lua
file.ISO_8859_16
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.ISO_8859_16})
```

**输出：**

```lua
```

### 等值

```text
"iso-8859-16"
```

---

## `KOI8_R`

KOI8-R 编码（俄文）。

### 调用

```lua
file.KOI8_R
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.KOI8_R})
```

**输出：**

```lua
```

### 等值

```text
"koi8-r"
```

---

## `KOI8_U`

KOI8-U 编码（乌克兰）。

### 调用

```lua
file.KOI8_U
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.KOI8_U})
```

**输出：**

```lua
```

### 等值

```text
"koi8-u"
```

---

## `IBM866`

IBM866 编码（俄文）。

### 调用

```lua
file.IBM866
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.IBM866})
```

**输出：**

```lua
```

### 等值

```text
"ibm866"
```

---

## `MACINTOSH`

Macintosh 编码（西欧）。

### 调用

```lua
file.MACINTOSH
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.MACINTOSH})
```

**输出：**

```lua
```

### 等值

```text
"macintosh"
```

---

## `X_MAC_CYRILLIC`

x-mac-cyrillic 编码（西里尔）。

### 调用

```lua
file.X_MAC_CYRILLIC
```

### 可用于

- 参数 `encoding`

### 示例

```lua
file.read("file.txt", {encoding = file.X_MAC_CYRILLIC})
```

**输出：**

```lua
```

### 等值

```text
"x-mac-cyrillic"
```

---

# 方法

## `read`

异步读取 `assets/` 目录下的文本文件。

### 调用

```lua
file.read
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `path` | string | 相对 `assets/` 的文件路径 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `encoding` | const-file | 默认：`file.AUTO`；文本编码 |
| `end_of_line` | const-file | 默认：`file.AUTO`；换行符规范 |
| `byte` | boolean | 默认：`false`；二进制模式 |
| `event_tip` | string / nil | 默认：`nil`；自定义事件标记 |

## 返回值

成功提交时立即返回一个请求编号。

| 类型 | 说明 |
| --- | --- |
| integer | 请求编号；在完成或失败事件的 `data.request_id` 中对应此编号。 |

### 示例

```lua
-- 准备：assets/file.txt 中写入 Hello Tui Game。

local request_id = file.read("file.txt")

function HandleEvent(event)
  if event.type == "file" then
    debug.print(serialization.json_encode(event))
  end
end
```

**输出：**

```lua
```

### 额外说明

- 路径相对当前包的 `assets/`；不能使用绝对路径或越出该目录。通过 `HandleEvent` 接收结果，查看⌞[文件事件](../EVENT.md#52-file)⌝。

- 参数 `byte` 为 false 时按文本读取，参数 `encoding` 决定解码方式；读取保留文件中的换行符，`end_of_line` 只校验取值，不转换换行。
- 参数 `byte` 为 true 时按二进制读取，参数 `encoding` 与 参数 `end_of_line` **忽略**。

---

## `write`

异步写入文本文件到 `assets/` 目录。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
file.write
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `path` | string | 相对 `assets/` 的文件路径 |
| `text` | string | 要写入的文本；`byte = true` 时为原始字节字符串 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `encoding` | const-file | 默认：`file.AUTO`；文本编码 |
| `end_of_line` | const-file | 默认：`file.AUTO`；换行符规范 |
| `byte` | boolean | 默认：`false`；二进制模式 |
| `event_tip` | string / nil | 默认：`nil`；事件提示文本 |

## 返回值

成功提交时立即返回一个请求编号。

| 类型 | 说明 |
| --- | --- |
| integer | 请求编号；在完成或失败事件的 `data.request_id` 中对应此编号。 |

屏保脚本调用时返回 `nil`，不提交任务，也不产生该请求的结果事件。

### 示例

```lua
-- 准备：包内已有 assets/ 目录。

local request_id = file.write("file.txt", "Hello Tui Game", {event_tip = "Get!"})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(serialization.json_encode(event))
  end
end
```

**输出：**

```lua
```

### 额外说明

- 路径相对当前包的 `assets/`；不能使用绝对路径或越出该目录。通过 `HandleEvent` 接收结果，查看⌞[文件事件](../EVENT.md#52-file)⌝。

- 参数 `byte` 为 false 时按文本写入，参数 `encoding` 与 参数 `end_of_line` **生效**。
- 参数 `byte` 为 true 时按二进制写入，参数 `encoding` 与 参数 `end_of_line` **忽略**。
- 该 API 会自动创建未创建的**文件**。
- 该 API 不会自动补全未创建的**目录**，目录不存在会抛出错误。


---

## `list_dir`

异步枚举 `assets/` 目录下的条目。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
file.list_dir
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `path` | string | 相对 `assets/` 目录路径 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `recursive` | boolean | 默认：`false`；是否递归子目录枚举 |
| `file_type` | string / const-file | 默认：`file.ALL`；仅匹配指定扩展名 |
| `event_tip` | string / nil | 默认：`nil`；事件提示文本 |

## 返回值

成功提交时立即返回一个请求编号。

| 类型 | 说明 |
| --- | --- |
| integer | 请求编号；在完成或失败事件的 `data.request_id` 中对应此编号。 |

屏保脚本调用时返回 `nil`，不提交任务，也不产生该请求的结果事件。

### 示例

```lua
-- 准备以下包内资源：
-- assets/
-- + c/ (game.c, main.c)
-- + js/ (data.json, main.js)
-- + rust/ (src/main.rs, Cargo.toml)
-- - file.txt

file.list_dir(".", {recursive = true})
file.list_dir("rust/")
file.list_dir("js/", {file_type = "json", event_tip = "Only Json"})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(serialization.json_encode(event))
  end
end
```

**输出：**

```lua
```

### 额外说明

- 路径相对当前包的 `assets/`；不能使用绝对路径或越出该目录。通过 `HandleEvent` 接收结果，查看⌞[文件事件](../EVENT.md#52-file)⌝。


---

## `create_dir`

异步创建指定目录到 `assets/` 目录。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
file.create_dir
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `path` | string | 相对 `assets/` 目录路径 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `event_tip` | string / nil | 默认：`nil`；事件提示文本 |

## 返回值

成功提交时立即返回一个请求编号。

| 类型 | 说明 |
| --- | --- |
| integer | 请求编号；在完成或失败事件的 `data.request_id` 中对应此编号。 |

屏保脚本调用时返回 `nil`，不提交任务，也不产生该请求的结果事件。

### 示例

```lua
-- 准备：包内已有 assets/ 目录。

file.create_dir("file")
file.create_dir("test1/test2")

function HandleEvent(event)
  if event.type == "file" then
    debug.print(serialization.json_encode(event))
  end
end
```

**输出：**

```lua
```

### 额外说明

- 路径相对当前包的 `assets/`；不能使用绝对路径或越出该目录。通过 `HandleEvent` 接收结果，查看⌞[文件事件](../EVENT.md#52-file)⌝。

- 该 API **支持**链式创建目录。


---

## `exists`

判断 `assets/` 目录下的指定文件或目录是否存在。

### 调用

```lua
file.exists
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `path` | string | 相对 `assets/` 目录路径 |

## 返回值

直接返回一个值。

| 类型    | 说明               |
| ------- | ------------------ |
| boolean | 文件或目录是否存在 |

### 示例

```lua
-- 准备：包内已有 assets/test/ 目录。

debug.print(tostring(file.exists("test")))
debug.print(tostring(file.exists("none")))
```

**输出：**

```lua
```

---
## `remove`

异步删除 `assets/` 目录下指定文件或目录。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
file.remove
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `path` | string | 相对 `assets/` 目录路径 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `recursive` | boolean | 默认：`false`；是否删除非空目录 |
| `event_tip` | string / nil | 默认：`nil`；事件提示文本 |

## 返回值

成功提交时立即返回一个请求编号。

| 类型 | 说明 |
| --- | --- |
| integer | 请求编号；在完成或失败事件的 `data.request_id` 中对应此编号。 |

屏保脚本调用时返回 `nil`，不提交任务，也不产生该请求的结果事件。

### 示例

```lua
-- 准备以下包内资源：
-- assets/
-- + test/test.txt
-- - file.txt

file.remove("file.txt")
file.remove("test", {recursive = false})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(serialization.json_encode(event))
  end
end
```

**输出：**

```lua
```

### 额外说明

- 路径相对当前包的 `assets/`；不能使用绝对路径或越出该目录。通过 `HandleEvent` 接收结果，查看⌞[文件事件](../EVENT.md#52-file)⌝。

- 一次指定一个目标；`recursive = true` 时可删除目标目录及其中的全部内容，`false` 时不能删除非空目录。
