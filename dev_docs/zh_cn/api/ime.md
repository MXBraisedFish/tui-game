# ime 库

`ime` 用于锁定或解锁输入法、接收终端提交的文字，以及写入剪贴板。

---

# 目录

## 方法

| 方法 | 说明 | 定位 |
| --- | --- | --- |
| `lock` | 锁定输入法 | [lock](#lock) |
| `unlock` | 解锁输入法 | [unlock](#unlock) |
| `receive_input_event` | 开始接收文字输入事件 | [receive_input_event](#receive_input_event) |
| `reject_input_event` | 停止接收文字输入事件 | [reject_input_event](#reject_input_event) |
| `write_clipboard` | 将文字写入剪贴板 | [write_clipboard](#write_clipboard) |

---

# 方法

## `lock`

锁定输入法，使用英文输入。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
ime.lock()
```

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 锁定是否成功 |

### 示例

```lua
local success = ime.lock()
debug.print(success)
```

**输出：**

```text
true
```

---

## `unlock`

解锁输入法。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
ime.unlock({ restore = true })
```

## 参数

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| restore | boolean | true | 是否恢复锁定前的输入法 |

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 解锁是否成功 |

### 示例

```lua
local success = ime.unlock({ restore = true })
debug.print(success)
```

**输出：**

```text
true
```

---

## `receive_input_event`

开始接收终端提交的文字。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
ime.receive_input_event()
```

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 设置是否成功 |

### 示例

```lua
local success = ime.receive_input_event()
debug.print(success)
```

**输出：**

```text
```

---

## `reject_input_event`

停止接收终端提交的文字。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
ime.reject_input_event()
```

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 设置是否成功 |

### 示例

```lua
local success = ime.reject_input_event()
debug.print(success)
```

**输出：**

```text
true
```

---

## `write_clipboard`

将文字写入系统剪贴板。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
ime.write_clipboard(text)
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| text | string | 写入剪贴板的文字 |

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 写入是否成功 |

### 示例

```lua
local success = ime.write_clipboard("Hello Tui Game")
debug.print(success)
```

**输出：**

```text
true
```
