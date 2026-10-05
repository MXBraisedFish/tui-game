# ime 库

`ime` 决定游戏是否接收动作事件和原始按键事件。

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
ime.receive_action_event
```

## 返回值

返回一个值。

| 类型    | 说明        |
| ------- | ----------- |
| boolean | 是否设置成功 |

### 示例

```lua
ime.reject_action_event()
debug.assert(ime.receive_action_event())
```

**输出：**

```lua
```

## 额外说明

- 无参数，传入任何参数都会抛出错误。
- 游戏默认已经接收动作事件。重新开启后，先前仍按着的输入不会重新产生 pressed；松开后再次按下才开始接收。
- 两个接收开关互相独立，只影响当前游戏会话。屏保调用会抛出错误。
- 调用立即改变接收状态；必要的 released 在当前回调结束后交付，不会嵌套调用 HandleEvent。
- 宿主快捷键、输入框和改键捕获仍按各自规则工作；观察原始键不能阻止宿主行为。
- pressed 发送一次；下一宿主帧仍有效时发送一次 held；结束时发送一次 released。持续 held 不会逐帧重复，快速点按可以只有 pressed 与 released。查看[事件协议](../EVENT.md)。

- 事件返回值见⌊[事件协议](../EVENT.md)⌉

---

## `reject_action_event`

停止接收动作事件。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
ime.reject_action_event
```

## 返回值

返回一个值。

| 类型    | 说明        |
| ------- | ----------- |
| boolean | 是否设置成功 |

### 示例

```lua
debug.assert(ime.reject_action_event())
```

**输出：**

```lua
```

## 额外说明

- 无参数，传入任何参数都会抛出错误。
- 已收到 pressed 且尚未释放的动作会收到一次 released；未交付的普通动作会作废。不会停止原始按键事件。
- 宿主快捷键、输入框和改键捕获仍按各自规则工作；观察原始键不能阻止宿主行为。
- 接收开关独立性、屏保行为与事件发送规则见[`receive_action_event`](#receive_action_event)。

---

## `receive_key_event`

开始接收原始按键事件。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
ime.receive_key_event
```

## 返回值

返回一个值。

| 类型    | 说明        |
| ------- | ----------- |
| boolean | 是否设置成功 |

### 示例

```lua
debug.assert(ime.receive_key_event())
```

**输出：**

```lua
```

## 额外说明

- 无参数，传入任何参数都会抛出错误。
- 游戏默认不接收原始按键事件。按键名沿用动作绑定的规范化键名，包括左右修饰键、数字小键盘与未知键表示。它不表示输入法提交的文字。
- 宿主快捷键、输入框和改键捕获仍按各自规则工作；观察原始键不能阻止宿主行为。
- 接收开关独立性、屏保行为与事件发送规则见[`receive_action_event`](#receive_action_event)。

---

## `reject_key_event`

停止接收原始按键事件。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
ime.reject_key_event
```

## 返回值

返回一个值。

| 类型    | 说明        |
| ------- | ----------- |
| boolean | 是否设置成功 |

### 示例

```lua
ime.receive_key_event()
debug.assert(ime.reject_key_event())
```

**输出：**

```lua
```

## 额外说明

- 无参数，传入任何参数都会抛出错误。
- 已收到 pressed 且尚未释放的键会收到一次 released；未交付的普通键事件会作废。不会停止动作事件。
- 宿主快捷键、输入框和改键捕获仍按各自规则工作；观察原始键不能阻止宿主行为。
- 接收开关独立性、屏保行为与事件发送规则见[`receive_action_event`](#receive_action_event)。
