# event 库

`event` 提供对当前游戏输入动作队列的控制。

---

# 目录

## 方法

| 方法                    | 说明                                       | 定位                                            |
| ----------------------- | ------------------------------------------ | ----------------------------------------------- |
| `skip_action`           | 将本帧剩余等待处理的输入动作留到后续帧处理 | [skip_action](#skip_action)                     |
| `clear_action`          | 清除当前等待处理的输入动作                 | [clear_action](#clear_action)                   |
| `enable_focus_release`  | 启用游戏失焦时的释放事件补发               | [enable_focus_release](#enable_focus_release)   |
| `disable_focus_release` | 关闭游戏失焦时的释放事件补发               | [disable_focus_release](#disable_focus_release) |

---

# 方法

## `skip_action`

将本帧剩余等待处理的输入动作留到后续帧处理。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
event.skip_action
```

## 返回值

无返回值。

### 示例

```lua
event.skip_action()
```

## 额外说明

- 仅跳过用户输入的 `action` 事件。

---

## `clear_action`

清除当前等待处理的输入动作。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
event.clear_action
```

## 返回值

无返回值。

### 示例

```lua
event.clear_action()
```

## 额外说明

- 仅清楚用户输入的 `action` 事件。

---

## `enable_focus_release`

启用游戏失焦时的释放事件补发。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
event.enable_focus_release
```

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 设置是否成功 |

### 示例

```lua
local success = event.enable_focus_release()
debug.print(success)
```

**输出：**

```lua
true
```

---

## `disable_focus_release`

关闭游戏失焦时的释放事件补发。

### 限制

- 仅游戏脚本可用。

### 调用

```lua
event.disable_focus_release
```

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 设置是否成功 |

### 示例

```lua
local success = event.disable_focus_release()
debug.print(success)
```

**输出：**

```lua
true
```
