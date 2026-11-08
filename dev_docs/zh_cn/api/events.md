# events 库

`events` 提供对当前游戏输入动作队列的控制。

---

# 目录

## 方法

| 方法                    | 说明                                       | 定位                                            |
| ----------------------- | ------------------------------------------ | ----------------------------------------------- |
| `skip_actions`           | 将本帧剩余等待处理的输入动作留到后续帧处理 | [skip_actions](#skip_actions)                     |
| `clear_actions`          | 清除当前等待处理的输入动作                 | [clear_actions](#clear_actions)                   |
| `enable_focus_release_events`  | 启用游戏失焦时的释放事件补发               | [enable_focus_release_events](#enable_focus_release_events)   |
| `disable_focus_release_events` | 关闭游戏失焦时的释放事件补发               | [disable_focus_release_events](#disable_focus_release_events) |

---

# 方法

## `skip_actions`

将本帧剩余等待处理的输入动作留到后续帧处理。

### 限制

- 仅游戏脚本可用。
- 需开启调试模式。

### 调用

```lua
events.skip_actions
```

### 返回值

无返回值。

### 示例

```lua
events.skip_actions()
```

### 额外说明

- 仅跳过玩家输入的 `action` 事件。

---

## `clear_actions`

清除当前等待处理的输入动作。

### 限制

- 仅游戏脚本可用。
- 需开启调试模式。

### 调用

```lua
events.clear_actions
```

### 返回值

无返回值。

### 示例

```lua
events.clear_actions()
```

### 额外说明

- 仅清除玩家输入的 `action` 事件。

---

## `enable_focus_release_events`

启用游戏失焦时的释放事件补发。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
events.enable_focus_release_events
```

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 设置是否成功 |

### 示例

```lua
local success = events.enable_focus_release_events()
debug.print(success)
```

**输出：**

```lua
true
```

---

## `disable_focus_release_events`

关闭游戏失焦时的释放事件补发。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
events.disable_focus_release_events
```

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 设置是否成功 |

### 示例

```lua
local success = events.disable_focus_release_events()
debug.print(success)
```

**输出：**

```lua
true
```
