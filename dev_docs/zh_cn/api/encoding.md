# encoding 库

## 基本库说明

`encoding` 提供字符串与二进制数据的编码转换。
所有方法均使用单个命名参数表，输入字段名为 `s`。

---

## 目录

### 方法

| 方法名          | 说明                                      | 索引                            |
| --------------- | ----------------------------------------- | ------------------------------- |
| `base64_encode` | 将字符串数据编码为 Base64 字符串          | [base64_encode](#base64_encode) |
| `base64_decode` | 将 Base64 字符串解码为原始字符串          | [base64_decode](#base64_decode) |
| `url_encode`    | 将字符串编码为 URL 安全格式（百分号编码） | [url_encode](#url_encode)       |
| `url_decode`    | 将 URL 编码字符串解码为原始字符串         | [url_decode](#url_decode)       |
| `hex_encode`    | 将字符串编码为十六进制字符串              | [hex_encode](#hex_encode)       |
| `hex_decode`    | 将十六进制字符串解码为原始字符串          | [hex_decode](#hex_decode)       |

---

## 方法

## `base64_encode`

将字符串数据编码为 Base64 字符串。

### 调用

```lua
-- 单参数
encoding.base64_encode{ s = "Hello Tui Game" }
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明         |
| ------ | ------ | ---- | ------ | ------------ |
| `s`    | string | 是   | -      | 要编码的数据 |

### 返回

直接返回一个值。

| 类型   | 说明          |
| ------ | ------------- |
| string | Base64 字符串 |

### 示例

```lua
debug.print { message = encoding.base64_encode{ s = "Hello Tui Game" } }
```

输出

```text
SGVsbG8gVHVpIEdhbWU=
```

---

## `base64_decode`

将 Base64 字符串解码为原始字符串。

### 调用

```lua
-- 单参数
encoding.base64_decode{ s = "SGVsbG8gVHVpIEdhbWU=" }
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明          |
| ------ | ------ | ---- | ------ | ------------- |
| `s`    | string | 是   | -      | Base64 字符串 |

### 返回

直接返回一个值。

| 类型   | 说明             |
| ------ | ---------------- |
| string | 解码后的原始数据 |

### 示例

```lua
debug.print { message = encoding.base64_decode{ s = "SGVsbG8gVHVpIEdhbWU=" } }
```

输出：

```text
Hello Tui Game
```

---

## `url_encode`

将字符串编码为 URL 安全格式（百分号编码）。

### 调用

```lua
-- 单参数
encoding.url_encode{ s = "exe=Hello Tui Game" }
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明           |
| ------ | ------ | ---- | ------ | -------------- |
| `s`    | string | 是   | -      | 要编码的字符串 |

### 返回

直接返回一个值。

| 类型   | 说明           |
| ------ | -------------- |
| string | 百分号编码结果 |

### 示例

```lua
debug.print { message = encoding.url_encode{ s = "exe=Hello Tui Game" } }
```

输出：

```text
exe%3DHello%20Tui%20Game
```

### 额外补充

- 该 API 为严格的百分号编码

---

## `url_decode`

将 URL 编码字符串解码为原始字符串。

### 调用

```lua
-- 单参数
encoding.url_decode{ s = "exe%3DHello%20Tui%20Game" }
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明               |
| ------ | ------ | ---- | ------ | ------------------ |
| `s`    | string | 是   | -      | 百分号编码的字符串 |

### 返回

直接返回一个值。

| 类型   | 说明               |
| ------ | ------------------ |
| string | 解码后的原始字符串 |

### 示例

```lua
debug.print { message = encoding.url_decode{ s = "exe%3DHello%20Tui%20Game" } }
```

输出：

```text
exe=Hello Tui Game
```

### 额外补充

- 该 API 为严格的百分号解码，所有不满足的格式均会拒绝解码并抛出错误

---

## `hex_encode`

将字符串编码为十六进制字符串。

### 调用

```lua
-- 单参数
encoding.hex_encode{ s = "Hello Tui Game" }
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明         |
| ------ | ------ | ---- | ------ | ------------ |
| `s`    | string | 是   | -      | 要编码的数据 |

### 返回

直接返回一个值。

| 类型   | 说明           |
| ------ | -------------- |
| string | 十六进制字符串 |

### 示例

```lua
debug.print { message = encoding.hex_encode{ s = "Hello Tui Game" } }
```

输出：

```text
48656c6c6f205475692047616d65
```

---

## `hex_decode`

将十六进制字符串解码为原始字符串。

### 调用

```lua
-- 单参数
encoding.hex_decode{ s = "48656c6c6f205475692047616d65" }
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明           |
| ------ | ------ | ---- | ------ | -------------- |
| `s`    | string | 是   | -      | 十六进制字符串 |

### 返回

直接返回一个值。

| 类型   | 说明             |
| ------ | ---------------- |
| string | 解码后的原始数据 |

### 示例

```lua
debug.print { message = encoding.hex_decode{ s = "48656c6c6f205475692047616d65" } }
```

输出：

```text
Hello Tui Game
```

### 额外补充

- 参数 `s` 接受大小写十六进制字符。
- 参数 `s` 输入长度必须为偶数。
