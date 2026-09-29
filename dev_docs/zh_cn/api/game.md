# game 库

`game` 提供退出游戏、保存继续游戏数据和保存最佳记录的命令。

---

# 目录

## 方法

| 方法 | 说明 | 定位 |
| --- | --- | --- |
| `exit_game` | 请求结束当前游戏 | [exit_game](#exit_game) |
| `save_game` | 请求保存继续游戏数据 | [save_game](#save_game) |
| `save_best` | 请求保存最佳记录 | [save_best](#save_best) |

---

# 方法

## `exit_game`

请求结束当前游戏。此命令不会自动保存游戏数据或最佳记录。

### 限制

- 仅游戏脚本可用

### 调用

```lua
game.exit_game
```

## 返回值

无返回值。

### 示例

```lua
game.exit_game()
```

**输出：**

```lua
```

### 额外说明

- 需要保存时，应先调用 `game.save_game()` 或 `game.save_best()`，再请求退出。
- 不可在 `Init`、`SaveGame` 或 `SaveBest` 回调中调用。

---

## `save_game`

请求执行一次 `SaveGame` 回调，将游戏数据保存到继续游戏槽位。

### 限制

- 仅游戏脚本可用

### 调用

```lua
game.save_game
```

## 返回值

无返回值。

### 示例

```lua
game.save_game()
```

**输出：**

```lua
```

### 额外说明

- 包清单需启用 `save_game`。
- 保存的数据会在玩家继续游戏时传给 `Init` 回调。
- 不可在 `SaveGame` 回调中调用。

---

## `save_best`

请求执行一次 `SaveBest` 回调，将最佳记录提供给游戏列表展示。

### 限制

- 仅游戏脚本可用

### 调用

```lua
game.save_best
```

## 返回值

无返回值。

### 示例

```lua
game.save_best()
```

**输出：**

```lua
```

### 额外说明

- 包清单需启用 `best_score.enable`。
- 不可在 `SaveBest` 回调中调用。
