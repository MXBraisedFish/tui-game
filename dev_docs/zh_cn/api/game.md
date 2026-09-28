# game 库

## 基本库说明

`game` 提供游戏脚本生命周期控制回调。

---

## 目录

### 方法

| 方法名 | 说明 | 索引 |
| ----- | ---- | ---- |
| `exit_game` | 请求结束当前游戏脚本运行 | [exit_game](#exit_game) |
| `save_game` | 请求执行一次 `SaveGame` 回调，将游戏数据保存在"继续游戏"槽位 | [save_game](#save_game) |
| `save_best` | 请求执行一次 `SaveBest` 回调，在游戏列表展示本游戏最佳记录记录 | [save_best](#save_best) |

---

## 方法

## `exit_game`

请求结束当前游戏脚本运行。

> 仅游戏脚本可用。

### 调用

```lua
-- 空命名参数表
game.exit_game{}
```

### 参数

无。

### 返回

无。

### 示例

```lua
game.exit_game{}
```

### 额外补充

- 调用该 API 不会自动调用 `SaveGame` 或 `SaveBest`。
- 如需保存继续游戏数据或最佳记录，应在退出前分别显式调用 `game.save_game{}` 或 `game.save_best{}`。
- 同一回调中先请求保存、再请求退出时，宿主会按照调用顺序先完成保存，再结束游戏。
- **不可**在 `Init`, `SaveGame`, `SaveBest` 回调中调用。

---

## `save_game`

请求执行一次 `SaveGame` 回调, 将游戏数据保存在 `继续游戏` 槽位。

> 仅游戏脚本可用。
> 仅当 `game.json` 中 `save_game` 字段为 true 时可用。

### 调用

```lua
-- 空命名参数表
game.save_game{}
```

### 参数

无。

### 返回

无。

### 示例

```lua
game.save_game{}
```

### 额外补充

- 保存的数据会在玩家使用 `继续游戏` 进入时，传递给 `Init` 回调。
- **不可**在 `SaveGame` 回调中调用。

---

## `save_best`

请求执行一次 `SaveBest` 回调, 在游戏列表展示本游戏最佳记录记录。

> 仅游戏脚本可用。
> 仅当 `game.json` 中 `best_score.enable` 字段为 true 时可用。

### 调用

```lua
-- 空命名参数表
game.save_best{}
```

### 参数

无。

### 返回

无。

### 示例

```lua
game.save_best{}
```

### 额外补充

- **不可**在 `SaveBest` 回调中调用。
