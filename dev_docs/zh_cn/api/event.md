# event 库

`event` 提供对当前游戏输入动作队列的控制。

---

# 目录

## 方法

| 方法 | 说明 | 定位 |
| --- | --- | --- |
| `skip_action` | 跳过本帧动作队列 | [skip_action](#skip_action) |
| `clear_action` | 清除当前积压动作 | [clear_action](#clear_action) |

---

# 方法

## `skip_action`

跳过本帧剩余的输入动作。

### 限制

- 仅游戏脚本可用

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

**输出：**

```lua
```

### 额外说明

- 全局事件不会被清空。

---

## `clear_action`

清除当前等待处理的输入动作。

### 限制

- 仅游戏脚本可用

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

**输出：**

```lua
```

### 额外说明

- 全局事件不会被清空。
