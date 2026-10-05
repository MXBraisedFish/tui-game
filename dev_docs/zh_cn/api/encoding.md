# encoding 库

`encoding` 用于在原始字符串与 Base64、URL 百分号编码、十六进制字符串之间互相转换。

---

# 目录

## 方法

| 方法            | 说明                                        | 定位                            |
| --------------- | ------------------------------------------- | ------------------------------- |
| `base64_encode` | 将字符串编码为 Base64 字符串                | [base64_encode](#base64_encode) |
| `base64_decode` | 将 Base64 字符串解码为原始字符串            | [base64_decode](#base64_decode) |
| `url_encode`    | 将字符串中的非 URL 安全字节转换为百分号编码 | [url_encode](#url_encode)       |
| `url_decode`    | 将 URL 百分号编码字符串还原为原始字符串     | [url_decode](#url_decode)       |
| `hex_encode`    | 将字符串的每个字节编码为两位十六进制数字    | [hex_encode](#hex_encode)       |
| `hex_decode`    | 将偶数长度的十六进制字符串解码为原始字节    | [hex_decode](#hex_decode)       |

---

# 方法

## `base64_encode`

将字符串编码为 Base64 字符串。

### 调用

```lua
encoding.base64_encode
```

## 参数

### 必填参数

| 参数名  | 类型   | 说明           |
| ------- | ------ | -------------- |
| `sting` | string | 要编码的字符串 |

## 返回值

返回一个 Base64 字符串。

| 类型   | 说明     |
| ------ | -------- |
| string | 编码结果 |

### 示例

```lua
local encoded = encoding.base64_encode("Hello Tui Game")
debug.print(encoded)
```

**输出：**

```lua
SGVsbG8gVHVpIEdhbWU=
```

## 额外说明

- Base64 使用标准字母表，编码结果一律带 `=` 填充。
- 不支持 URL-safe 字母表。

---

## `base64_decode`

将 Base64 字符串解码为原始字符串。

### 调用

```lua
encoding.base64_decode
```

## 参数

### 必填参数

| 参数名  | 类型   | 说明                   |
| ------- | ------ | ---------------------- |
| `sting` | string | 要解码的 Base64 字符串 |

## 返回值

返回解码后的原始字符串。

| 类型   | 说明     |
| ------ | -------- |
| string | 解码结果 |

### 示例

```lua
local decoded = encoding.base64_decode("SGVsbG8gVHVpIEdhbWU=")
debug.print(decoded)
```

**输出：**

```lua
Hello Tui Game
```

## 额外说明

- 解码要求规范的 `=` 填充。

---

## `url_encode`

将字符串中的非 URL 安全字节转换为百分号编码。

### 调用

```lua
encoding.url_encode
```

## 参数

### 必填参数

| 参数名  | 类型   | 说明           |
| ------- | ------ | -------------- |
| `sting` | string | 要编码的字符串 |

## 返回值

返回百分号编码后的字符串。

| 类型   | 说明     |
| ------ | -------- |
| string | 编码结果 |

### 示例

```lua
local encoded = encoding.url_encode("name=Hello Tui Game")
debug.print(encoded)
```

**输出：**

```lua
name%3DHello%20Tui%20Game
```

---

## `url_decode`

将 URL 百分号编码字符串还原为原始字符。

### 调用

```lua
encoding.url_decode
```

## 参数

### 必填参数

| 参数名  | 类型   | 说明                     |
| ------- | ------ | ------------------------ |
| `sting` | string | 要解码的百分号编码字符串 |

## 返回值

返回解码后的字符串。

| 类型   | 说明     |
| ------ | -------- |
| string | 解码结果 |

### 示例

```lua
local decoded = encoding.url_decode("name%3DHello%20Tui%20Game")
debug.print(decoded)
```

**输出：**

```lua
name=Hello Tui Game
```

---

## `hex_encode`

将字符串的每个字节编码为两位十六进制数字。

### 调用

```lua
encoding.hex_encode
```

## 参数

### 必填参数

| 参数名  | 类型   | 说明           |
| ------- | ------ | -------------- |
| `sting` | string | 要编码的字符串 |

## 返回值

返回小写十六进制字符串。

| 类型   | 说明     |
| ------ | -------- |
| string | 编码结果 |

### 示例

```lua
local encoded = encoding.hex_encode("Hello")
debug.print(encoded)
```

**输出：**

```lua
48656c6c6f
```

---

## `hex_decode`

将偶数长度的十六进制字符串解码为原始字节。

### 调用

```lua
encoding.hex_decode
```

## 参数

### 必填参数

| 参数名  | 类型   | 说明                   |
| ------- | ------ | ---------------------- |
| `sting` | string | 要解码的十六进制字符串 |

## 返回值

返回解码后的原始字符串。

| 类型   | 说明     |
| ------ | -------- |
| string | 解码结果 |

### 示例

```lua
local decoded = encoding.hex_decode("48656c6c6f")
debug.print(decoded)
```

**输出：**

```lua
Hello
```

## 额外说明

- 十六进制字母不区分大小写。
