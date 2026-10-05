# keyboard 库

`keyboard` 决定游戏是否接收动作事件和原始按键事件。

---

# 目录

## 方法

| 方法                   | 说明                 | 定位                                          |
| ---------------------- | -------------------- | --------------------------------------------- |
| `receive_action_event` | 开始接收动作事件     | [receive_action_event](#receive_action_event) |
| `reject_action_event`  | 停止接收动作事件     | [reject_action_event](#reject_action_event)   |
| `receive_key_event`    | 开始接收原始按键事件 | [receive_key_event](#receive_key_event)       |
| `reject_key_event`     | 停止接收原始按键事件 | [reject_key_event](#reject_key_event)         |

---

# 方法

## `receive_action_event`

开始接收动作事件。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
keyboard.receive_action_event
```

## 返回值

返回一个值。

| 类型    | 说明        |
| ------- | ----------- |
| boolean | 设置是否成功 |

### 示例

```lua
local success = keyboard.receive_action_event()
debug.print(success)
```

**输出：**

```lua
true
```

---

## `reject_action_event`

停止接收动作事件。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
keyboard.reject_action_event
```

## 返回值

返回一个值。

| 类型    | 说明        |
| ------- | ----------- |
| boolean | 设置是否成功 |

### 示例

```lua
local success = keyboard.reject_action_event()
debug.print(success)
```

**输出：**

```lua
true
```

---

## `receive_key_event`

开始接收原始按键事件。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
keyboard.receive_key_event
```

## 返回值

返回一个值。

| 类型    | 说明        |
| ------- | ----------- |
| boolean | 设置是否成功 |

### 示例

```lua
local success = keyboard.receive_key_event()
debug.print(success)
```

**输出：**

```lua
true
```

---

## `reject_key_event`

停止接收原始按键事件。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
keyboard.reject_key_event
```

## 返回值

返回一个值。

| 类型    | 说明        |
| ------- | ----------- |
| boolean | 设置是否成功 |

### 示例

```lua
keyboard.receive_key_event()
debug.assert(keyboard.reject_key_event())
```

**输出：**

```lua
true
```
