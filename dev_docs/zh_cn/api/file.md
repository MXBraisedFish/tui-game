# file 库

`file` 提供 `assets/` 目录中的异步文件操作。

---

# 目录

## 常量

| 常量             | 说明                                  | 定位                              |
| ---------------- | ------------------------------------- | --------------------------------- |
| `AUTO`           | 自动检测模式                          | [AUTO](#auto)                     |
| `ALL`            | 全部统一模式                          | [ALL](#all)                       |
| `CR`             | 回车符                                | [CR](#cr)                         |
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

| 方法         | 说明                                          | 定位                      |
| ------------ | --------------------------------------------- | ------------------------- |
| `read`       | 异步读取 `assets/` 目录下的文件               | [read](#read)             |
| `write`      | 异步写入文件到 `assets/` 目录                 | [write](#write)           |
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

> assets/file.txt

```lua
local request_id = file.read("file.txt", {encoding = file.AUTO, end_of_line = file.AUTO})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/dir/...

```lua
local request_id = file.list_dir("dir/", {file_type = file.ALL})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(table.pretty(event.data.entries))
  end
end
```

**输出：**

```lua
{
  [1] = {
    file_type = "lua",
    path = "helper.lua"
  },
  [2] = {
    file_type = "html",
    path = "index.html"
  },
  [3] = {
    file_type = "rs",
    path = "main.rs"
  },
  [4] = {
    file_type = "js",
    path = "script.js"
  }
}
```

### 等值

```text
"all"
```

---

## `CR`

回车符。

### 调用

```lua
file.CR
```

### 可用于

- 参数 `end_of_line`

### 示例

> assets/file.txt

```lua
function Init(ctx)
  file.read("file.txt", {end_of_line = file.CR})
end

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt

```lua
function Init(ctx)
  file.read("file.txt", {end_of_line = file.LF})
end

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt

```lua
function Init(ctx)
  file.read("file.txt", {end_of_line = file.CRLF})
end

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (UTF-8)

```lua
local request_id = file.read("file.txt", {encoding = file.UTF_8})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (UTF-16 LE)

```lua
local request_id = file.read("file.txt", {encoding = file.UTF_16LE})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (UTF-16 BE)

```lua
local request_id = file.read("file.txt", {encoding = file.UTF_16BE})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
```

### 等值

```text
"utf-16be"
```

---

## `GBK`

GBK 编码（简体中文）。

### 调用

```lua
file.GBK
```

### 可用于

- 参数 `encoding`

### 示例

> assets/file.txt (GBK)

```lua
local request_id = file.read("file.txt", {encoding = file.GBK})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (GB18030)

```lua
local request_id = file.read("file.txt", {encoding = file.GB18030})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (Big5)

```lua
local request_id = file.read("file.txt", {encoding = file.BIG5})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (Shift_JIS)

```lua
local request_id = file.read("file.txt", {encoding = file.SHIFT_JIS})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (EUC-JP)

```lua
local request_id = file.read("file.txt", {encoding = file.EUC_JP})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (ISO-2022-JP)

```lua
local request_id = file.read("file.txt", {encoding = file.ISO_2022_JP})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (EUC-KR)

```lua
local request_id = file.read("file.txt", {encoding = file.EUC_KR})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (Windows-874)

```lua
local request_id = file.read("file.txt", {encoding = file.WINDOWS_874})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (Windows-1250)

```lua
local request_id = file.read("file.txt", {encoding = file.WINDOWS_1250})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (Windows-1251)

```lua
local request_id = file.read("file.txt", {encoding = file.WINDOWS_1251})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (Windows-1252)

```lua
local request_id = file.read("file.txt", {encoding = file.WINDOWS_1252})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (Windows-1253)

```lua
local request_id = file.read("file.txt", {encoding = file.WINDOWS_1253})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (Windows-1254)

```lua
local request_id = file.read("file.txt", {encoding = file.WINDOWS_1254})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (Windows-1255)

```lua
local request_id = file.read("file.txt", {encoding = file.WINDOWS_1255})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (Windows-1256)

```lua
local request_id = file.read("file.txt", {encoding = file.WINDOWS_1256})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (Windows-1257)

```lua
local request_id = file.read("file.txt", {encoding = file.WINDOWS_1257})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (Windows-1258)

```lua
local request_id = file.read("file.txt", {encoding = file.WINDOWS_1258})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (ISO-8859-2)

```lua
local request_id = file.read("file.txt", {encoding = file.ISO_8859_2})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (ISO-8859-3)

```lua
local request_id = file.read("file.txt", {encoding = file.ISO_8859_3})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (ISO-8859-4)

```lua
local request_id = file.read("file.txt", {encoding = file.ISO_8859_4})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (ISO-8859-5)

```lua
local request_id = file.read("file.txt", {encoding = file.ISO_8859_5})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (ISO-8859-6)

```lua
local request_id = file.read("file.txt", {encoding = file.ISO_8859_6})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (ISO-8859-7)

```lua
local request_id = file.read("file.txt", {encoding = file.ISO_8859_7})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (ISO-8859-8)

```lua
local request_id = file.read("file.txt", {encoding = file.ISO_8859_8})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (ISO-8859-8-I)

```lua
local request_id = file.read("file.txt", {encoding = file.ISO_8859_8_I})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (ISO-8859-10)

```lua
local request_id = file.read("file.txt", {encoding = file.ISO_8859_10})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (ISO-8859-13)

```lua
local request_id = file.read("file.txt", {encoding = file.ISO_8859_13})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (ISO-8859-14)

```lua
local request_id = file.read("file.txt", {encoding = file.ISO_8859_14})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (ISO-8859-15)

```lua
local request_id = file.read("file.txt", {encoding = file.ISO_8859_15})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (ISO-8859-16)

```lua
local request_id = file.read("file.txt", {encoding = file.ISO_8859_16})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (KOI8-R)

```lua
local request_id = file.read("file.txt", {encoding = file.KOI8_R})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (KOI8-U)

```lua
local request_id = file.read("file.txt", {encoding = file.KOI8_U})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (IBM866)

```lua
local request_id = file.read("file.txt", {encoding = file.IBM866})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (Macintosh)

```lua
local request_id = file.read("file.txt", {encoding = file.MACINTOSH})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
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

> assets/file.txt (x-mac-cyrillic)

```lua
local request_id = file.read("file.txt", {encoding = file.X_MAC_CYRILLIC})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(event.data.text)
  end
end
```

**输出：**

```lua
Hello Tui Game
```

### 等值

```text
"x-mac-cyrillic"
```

---

# 方法

## `read`

异步读取 `assets/` 目录下的文件。

### 调用

```lua
file.read
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明                      |
| ------ | ------ | ------------------------- |
| `path` | string | 相对 `assets/` 的文件路径 |

### 选填参数

| 参数名        | 类型                | 默认值      | 说明           |
| ------------- | ------------------- | ----------- | -------------- |
| `encoding`    | string / const-file | `file.AUTO` | 文本编码       |
| `end_of_line` | string / const-file | `file.AUTO` | 换行符规范     |
| `byte`        | boolean             | `false`     | 二进制模式     |
| `event_tip`   | string / nil        | `nil`       | 自定义事件标记 |

## 返回值

**请求提交成功时**，返回一个值。

| 类型   | 说明    |
| ------ | ------- |
| string | 事件 ID |

**请求提交失败时**，返回一个值。

| 类型 | 说明         |
| ---- | ------------ |
| nil  | 请求提交失败 |

### 示例

> assets/file.txt

```lua
local request_id = file.read("file.txt")

function HandleEvent(event)
  if event.type == "file" then
    debug.print(table.pretty(event))
  end
end
```

**输出：**

```lua
{
  type = "file",
  frame = X,        -- 占位符
  sequence = X,     -- 占位符
  data = {
    request_id = X, -- 占位符
    kind = "read_text",
    ok = true,
    path = "file.txt",
    text = "Hello Tui Game"
  }
}
```

## 额外说明

- 事件返回值见⌊[事件协议](../EVENT.md)⌉
- 路径相对当前包的 `assets/`。
- 选填参数 `byte` 为 `false` 时按文本读取。
- 选填参数 `byte` 为 `true` 时按二进制读取，选填参数 `encoding` 与选填参数 `end_of_line` 会被忽略。

---

## `write`

异步写入文件到 `assets/` 目录。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
file.write
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明                      |
| ------ | ------ | ------------------------- |
| `path` | string | 相对 `assets/` 的文件路径 |
| `text` | string | 要写入的内容              |

### 选填参数

| 参数名        | 类型                | 默认值      | 说明           |
| ------------- | ------------------- | ----------- | -------------- |
| `encoding`    | string / const-file | `file.AUTO` | 文本编码       |
| `end_of_line` | string / const-file | `file.AUTO` | 换行符规范     |
| `byte`        | boolean             | `false`     | 二进制模式     |
| `event_tip`   | string / nil        | `nil`       | 自定义事件标记 |

## 返回值

**请求提交成功时**，返回一个值。

| 类型   | 说明    |
| ------ | ------- |
| string | 事件 ID |

**请求提交失败时**，返回一个值。

| 类型 | 说明         |
| ---- | ------------ |
| nil  | 请求提交失败 |

### 示例

```lua
local request_id = file.write("file.txt", "Hello Tui Game", {event_tip = "Get!"})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(table.pretty(event))
  end
end
```

**输出：**

```lua
{
  type = "file",
  frame = X,        -- 占位符
  sequence = X,     -- 占位符
  data = {
    request_id = X, -- 占位符
    kind = "write_text",
    ok = true,
    path = "file.txt",
  }
}
```

## 额外说明

- 事件返回值见⌊[事件协议](../EVENT.md)⌉
- 路径相对当前包的 `assets/`。
- 选填参数 `byte` 为 `false` 时按文本读取。
- 选填参数 `byte` 为 `true` 时按二进制读取，选填参数 `encoding` 与选填参数 `end_of_line` 会被忽略。
- 方法会自动创建未创建的**文件**，但不会自动补全未创建的**目录**。

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

| 参数名 | 类型   | 说明                    |
| ------ | ------ | ----------------------- |
| `path` | string | 相对 `assets/` 目录路径 |

### 选填参数

| 参数名      | 类型                      | 默认值  | 说明               |
| ----------- | ------------------------- | ------- | ------------------ |
| `recursive` | boolean                   | `false` | 是否递归子目录枚举 |
| `file_type` | string / const-file | `file.ALL`   | 仅匹配指定扩展名   |
| `event_tip` | string / nil              | `nil`   | 自定义事件标记     |

## 返回值

**请求提交成功时**，返回一个值。

| 类型   | 说明    |
| ------ | ------- |
| string | 事件 ID |

**请求提交失败时**，返回一个值。

| 类型 | 说明         |
| ---- | ------------ |
| nil  | 请求提交失败 |

### 示例

> assets/dir/...

```lua
local request_id = file.list_dir("dir/", {file_type = file.ALL})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(table.pretty(event.data.entries))
  end
end
```

**输出：**

```lua
{
  [1] = {
    file_type = "lua",
    path = "helper.lua"
  },
  [2] = {
    file_type = "html",
    path = "index.html"
  },
  [3] = {
    file_type = "rs",
    path = "main.rs"
  },
  [4] = {
    file_type = "js",
    path = "script.js"
  }
}
```

## 额外说明

- 事件返回值见⌊[事件协议](../EVENT.md)⌉
- 路径相对当前包的 `assets/`。
- 选填参数 `file_type` 只接受不含点号的扩展名，且不区分大小写。

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

| 参数名 | 类型   | 说明                    |
| ------ | ------ | ----------------------- |
| `path` | string | 相对 `assets/` 目录路径 |

### 选填参数

| 参数名      | 类型         | 默认值 | 说明           |
| ----------- | ------------ | ------ | -------------- |
| `event_tip` | string / nil | `nil`  | 自定义事件标记 |

## 返回值

**请求提交成功时**，返回一个值。

| 类型   | 说明    |
| ------ | ------- |
| string | 事件 ID |

**请求提交失败时**，返回一个值。

| 类型 | 说明         |
| ---- | ------------ |
| nil  | 请求提交失败 |

### 示例

```lua
local request_id1 = file.create_dir("file")
local request_id2 = file.create_dir("test1/test2/")

function HandleEvent(event)
  if event.type == "file" then
    debug.print(table.pretty(event))
  end
end
```

**输出：**

```lua
{
  type = "file",
  frame = X,        -- 占位符
  sequence = X,     -- 占位符
  data = {
    request_id = X, -- 占位符
    kind = "create_dir",
    ok = true,
    path = "file",
  }
}
{
  type = "file",
  frame = X,        -- 占位符
  sequence = X,     -- 占位符
  data = {
    request_id = X, -- 占位符
    kind = "create_dir",
    ok = true,
    path = "test1/test2",
  }
}
```

## 额外说明

- 事件返回值见⌊[事件协议](../EVENT.md)⌉
- 路径相对当前包的 `assets/`。
- 方法会逐级创建缺失的目录。
- 返回值最后一级目录不含带 `/` 字符结尾。

---

## `exists`

判断 `assets/` 目录下的指定文件或目录是否存在。

### 调用

```lua
file.exists
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明                  |
| ------ | ------ | --------------------- |
| `path` | string | 相对 `assets/` 的路径 |

## 返回值

返回一个值。

| 类型    | 说明               |
| ------- | ------------------ |
| boolean | 文件或目录是否存在 |

### 示例

> assets/test/

```lua
debug.print(tostring(file.exists("test")))
debug.print(tostring(file.exists("none")))
```

**输出：**

```lua
true
false
```

## 额外说明

- 路径相对当前包的 `assets/`。

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

| 参数名 | 类型   | 说明                  |
| ------ | ------ | --------------------- |
| `path` | string | 相对 `assets/` 的路径 |

### 选填参数

| 参数名      | 类型         | 默认值  | 说明             |
| ----------- | ------------ | ------- | ---------------- |
| `recursive` | boolean      | `false` | 是否删除非空目录（递归删除） |
| `event_tip` | string / nil | `nil`   | 自定义事件标记   |

## 返回值

**请求提交成功时**，返回一个值。

| 类型   | 说明    |
| ------ | ------- |
| string | 事件 ID |

**请求提交失败时**，返回一个值。

| 类型 | 说明         |
| ---- | ------------ |
| nil  | 请求提交失败 |

### 示例

> assets/test/test.txt

```lua
file.remove("test", {recursive = false})

function HandleEvent(event)
  if event.type == "file" then
    debug.print(table.pretty(event))
  end
end
```

**输出：**

```lua
{
  type = "file",
  frame = X,        -- 占位符
  sequence = X,     -- 占位符
  data = {
    request_id = X, -- 占位符
    kind = "remove",
    ok = false,
    error = {
      code = "io", 
      message = "I/O operation failed"
    },
    path = "test1",
  }
}
```

## 额外说明

- 路径相对当前包的 `assets/`。
