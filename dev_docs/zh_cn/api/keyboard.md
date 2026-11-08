# keyboard 库

`keyboard` 用于控制键盘动作事件。

---

# 目录

## 方法

| 方法                   | 说明                 | 定位                                          |
| ---------------------- | -------------------- | --------------------------------------------- |
| `enable_action_event` | 启用接收动作事件     | [enable_action_event](#enable_action_event) |
| `disable_action_event`  | 关闭接收动作事件     | [disable_action_event](#disable_action_event)   |
| `enable_key_event`    | 启用接收原始按键事件 | [enable_key_event](#enable_key_event)       |
| `disable_key_event`     | 关闭接收原始按键事件 | [disable_key_event](#disable_key_event)         |

---

# 方法

## `enable_action_event`

启用接收动作事件。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
keyboard.enable_action_event
```

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 设置是否成功 |

### 示例

```lua
local success = keyboard.enable_action_event()
debug.print(success)
```

**输出：**

```lua
true
```

---

## `disable_action_event`

关闭接收动作事件。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
keyboard.disable_action_event
```

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 设置是否成功 |

### 示例

```lua
local success = keyboard.disable_action_event()
debug.print(success)
```

**输出：**

```lua
true
```

---

## `enable_key_event`

开始接收原始按键事件。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
keyboard.enable_key_event
```

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 设置是否成功 |

### 示例

```lua
local success = keyboard.enable_key_event()
debug.print(success)
```

**输出：**

```lua
true
```

---

## `disable_key_event`

关闭接收原始按键事件。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
keyboard.disable_key_event
```

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 设置是否成功 |

### 示例

```lua
keyboard.enable_key_event()
debug.assert(keyboard.disable_key_event())
```

**输出：**

```lua
true
```
