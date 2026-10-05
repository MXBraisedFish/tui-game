# game 库

`game` 提供退出游戏、保存继续游戏数据和保存最佳记录的命令。

---

# 目录

## 方法

| 方法        | 说明                                                       | 定位                    |
| ----------- | ---------------------------------------------------------- | ----------------------- |
| `exit_game` | 请求结束当前游戏脚本     | [exit_game](#exit_game) |
| `save_game` | 请求执行一次 `SaveGame` 回调 | [save_game](#save_game) |
| `save_best` | 请求执行一次 `SaveBest` 回调 | [save_best](#save_best) |

---

# 方法

## `exit_game`

请求结束当前游戏脚本。

### 限制

- 仅游戏脚本可用。

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
[脚本终止运行]
```

## 额外说明

- 不可在 `Init`、`SaveGame` 或 `SaveBest` 回调中调用。

---

## `save_game`

请求执行一次 `SaveGame` 回调。

### 限制

- 仅游戏脚本可用。
- 游戏包 `game.json` 配置文件字段 `save_game` 为 `true`

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

## 额外说明

- 不可在 `SaveGame` 回调中调用。

---

## `save_best`

请求执行一次 `SaveBest` 回调。

### 限制

- 仅游戏脚本可用。
- 游戏包 `game.json` 配置文件字段 `best_score.enable` 为 `true`

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

## 额外说明

- 不可在 `SaveBest` 回调中调用。
