# event 库

## 基本库说明

`event` 提供事件处理高级控制。

---

## 目录

### 方法

| 方法名         | 说明             | 索引                          |
| -------------- | ---------------- | ----------------------------- |
| `skip_action`  | 跳过本帧动作队列 | [skip_action](#skip_action)   |
| `clear_action` | 清空本帧动作队列 | [clear_action](#clear_action) |

---

## 方法

## `skip_action`

请求宿主跳过本帧输入动作队列的处理。

> 仅游戏会话可调用。
> 仅游戏脚本可用。

### 调用

```lua
-- 单参数
event.skip_action()
```

### 参数

无。

### 返回

无。

### 示例

```lua
event.skip_action()
```

### 额外补充

- 所有**全局事件**不会被清空。

---

## `clear_action`

请求宿主清空当前积压的动作队列。

> 仅游戏会话可调用。
> 仅游戏脚本可用。

### 调用

```lua
-- 单参数
event.clear_action()
```

### 参数

无。

### 返回

无。

### 示例

```lua
event.clear_action()
```

### 额外补充

- 所有**全局事件**不会被清空。
