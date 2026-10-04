# encoding 库

`encoding` 用来转换文本编码格式，得到 Base64、URL 百分号编码或十六进制字符串，也可以把这些格式还原为原始字符串。

---

# 目录

## 方法

| 方法 | 说明 | 定位 |
| --- | --- | --- |
| `base64_encode` | 将字符串编码为 Base64 | [base64_encode](#base64_encode) |
| `base64_decode` | 将 Base64 解码为字符串 | [base64_decode](#base64_decode) |
| `url_encode` | 对字符串进行 URL 百分号编码 | [url_encode](#url_encode) |
| `url_decode` | 解码 URL 百分号编码字符串 | [url_decode](#url_decode) |
| `hex_encode` | 将字符串编码为十六进制 | [hex_encode](#hex_encode) |
| `hex_decode` | 将十六进制解码为字符串 | [hex_decode](#hex_decode) |

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

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `s` | string | 要编码的字符串 |

## 返回值

返回一个 Base64 字符串。

| 类型 | 说明 |
| --- | --- |
| string | 编码结果 |

### 示例

```lua
local encoded = encoding.base64_encode("Hello Tui Game")
debug.print(encoded)
```

**输出：**

```lua
```

---

## `base64_decode`

将 Base64 字符串解码为原始字符串。输入不是有效 Base64 时会报错。

### 调用

```lua
encoding.base64_decode
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `s` | string | 要解码的 Base64 字符串 |

## 返回值

返回解码后的原始字符串。

| 类型 | 说明 |
| --- | --- |
| string | 解码结果 |

### 示例

```lua
local decoded = encoding.base64_decode("SGVsbG8gVHVpIEdhbWU=")
debug.print(decoded)
```

**输出：**

```lua
```

---

## `url_encode`

将字符串中的非 URL 安全字节转换为百分号编码。

### 调用

```lua
encoding.url_encode
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `s` | string | 要编码的字符串 |

## 返回值

返回百分号编码后的字符串。

| 类型 | 说明 |
| --- | --- |
| string | 编码结果 |

### 示例

```lua
local encoded = encoding.url_encode("name=Hello Tui Game")
debug.print(encoded)
```

**输出：**

```lua
```

---

## `url_decode`

将 URL 百分号编码字符串还原为原始字符串。格式无效时会报错。

### 调用

```lua
encoding.url_decode
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `s` | string | 要解码的百分号编码字符串 |

## 返回值

返回解码后的字符串。

| 类型 | 说明 |
| --- | --- |
| string | 解码结果 |

### 示例

```lua
local decoded = encoding.url_decode("name%3DHello%20Tui%20Game")
debug.print(decoded)
```

**输出：**

```lua
```

---

## `hex_encode`

将字符串的每个字节编码为两位十六进制数。

### 调用

```lua
encoding.hex_encode
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `s` | string | 要编码的字符串 |

## 返回值

返回小写十六进制字符串。

| 类型 | 说明 |
| --- | --- |
| string | 编码结果 |

### 示例

```lua
local encoded = encoding.hex_encode("Hello")
debug.print(encoded)
```

**输出：**

```lua
```

---

## `hex_decode`

将偶数长度的十六进制字符串解码为原始字节。输入含非十六进制字符或长度为奇数时会报错。

### 调用

```lua
encoding.hex_decode
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `s` | string | 要解码的十六进制字符串 |

## 返回值

返回解码后的原始字符串。

| 类型 | 说明 |
| --- | --- |
| string | 解码结果 |

### 示例

```lua
local decoded = encoding.hex_decode("48656c6c6f")
debug.print(decoded)
```

**输出：**

```lua
```
