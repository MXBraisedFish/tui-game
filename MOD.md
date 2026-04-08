# 模组制作说明

这份文档只讲一套规则：现在的模组该怎么组织目录、怎么写 `game.json`、怎么接宿主 API。

先说边界：

- Lua 只负责游戏闭包内的逻辑、状态和绘制
- Rust 宿主负责输入、主循环、渲染、资源读取、存档、最佳记录和日志
- 模组不能自己开主循环，不能自己控制宿主帧率
- 模组只能通过宿主开放的 API 读写数据

## 运行目录

运行时数据都在：

```text
tui-game-data/
  language.txt
  best_scores.json
  saves.json
  updater_cache.json
  mod/
  official/
  log/
```

模组安装目录在：

```text
tui-game-data/mod/
```

推荐一个模组包的目录长这样：

```text
my_mod/
  package.json
  game.json
  scripts/
    main.lua
    helpers/
      util.lua
  assets/
    lang/
      en_us.json
      zh_cn.json
    data/
      word.json
```

## `package.json`

包清单还是包级别信息，最少需要这些：

```json
{
  "namespace": "examplepack",
  "package_name": "Example Pack",
  "author": "Your Name",
  "version": "1.0.0",
  "description": "A small example package",
  "api_version": 1
}
```

说明：

- `namespace`：包命名空间
- `package_name`：包名
- `author`：包作者
- `version`：版本号
- `description`：包简介
- `api_version`：当前保持 `1`

## `game.json`

### 自定义包

自定义游戏清单现在统一按这个结构：

```json
{
  "package": "package",
  "name": "game.example.name",
  "description": "game.example.description",
  "detail": "game.example.detail",
  "author": "Your Name",
  "introduction": "game.example.introduction",
  "icon": null,
  "banner": null,
  "entry": "scripts/main.lua",
  "save": true,
  "best_none": "game.example.best_none",
  "min_width": 60,
  "min_height": 24,
  "write": true,
  "actions": {
    "move_left": ["left", "a"],
    "move_right": ["right", "d"]
  },
  "runtime": {
    "target_fps": 60
  }
}
```

### 官方包

官方包字段更少，但核心规则一样：

```json
{
  "id": "tui_game_2048_pQc2haTtPbX0Pt6T",
  "name": "game.2048.name",
  "description": "game.2048.description",
  "detail": "game.2048.details",
  "author": "TUI-GAME",
  "entry": "scripts/2048.lua",
  "save": true,
  "best_none": "game.2048.best_none_block",
  "min_width": 52,
  "min_height": 26,
  "actions": {
    "move_left": ["left"]
  },
  "runtime": {
    "target_fps": 60
  }
}
```

## 各字段是什么意思

| 字段 | 作用 |
| --- | --- |
| `id` | 游戏唯一 ID。必须唯一，不能拿语言键充当 ID。 |
| `name` | 游戏显示名。可以是真实文本，也可以是语言键。 |
| `description` | 游戏列表里的简短介绍。可以是真实文本，也可以是语言键。 |
| `detail` | 游戏详情页里的详细介绍。可以是真实文本，也可以是语言键。 |
| `author` | 游戏作者。官方包必须是 `TUI-GAME`。 |
| `introduction` | 自定义包在模组列表里的简介。官方包不要求。 |
| `icon` | 先保留，不要求实现。可以写 `null`。 |
| `banner` | 先保留，不要求实现。可以写 `null`。 |
| `entry` | 入口脚本路径。宿主会从这个脚本里固定找 `init_game / handle_event / render / best_score`。 |
| `save` | 是否允许保存。 |
| `best_none` | 没有最佳记录时显示什么。写字符串表示“这个游戏有最佳记录”；写 `null` 表示“这个游戏没有最佳记录”。 |
| `min_width` | 最小终端宽度。可选，必须大于 `0`。 |
| `min_height` | 最小终端高度。可选，必须大于 `0`。 |
| `actions` | 物理按键到动作名的映射。 |
| `runtime.target_fps` | 宿主目标 FPS，只允许 `30`、`60`、`120`。不写时默认 `60`。 |

## `best_score(state)` 的规则

这个规则现在定死了：

- `best_score(state)` 不是永远强制必写
- 是否必须存在，由 `game.json.best_none` 决定

### 情况 1：有最佳记录

如果你这样写：

```json
"best_none": "game.example.best_none"
```

那就表示：

- 这个游戏有最佳记录概念
- 必须导出 `best_score(state)`
- `request_refresh_best_score()` 才有意义

### 情况 2：没有最佳记录

如果你这样写：

```json
"best_none": null
```

那就表示：

- 这个游戏没有最佳记录概念
- 可以不写 `best_score(state)`
- 宿主不会展示最佳记录块
- `request_refresh_best_score()` 会被忽略

## 文本字段可以怎么写

这些字段支持两种写法：

- 直接写真实文本
- 写语言键

支持双模式的字段：

- `name`
- `description`
- `detail`
- `author`
- `introduction`
- `best_none`

例如这两种都合法：

```json
"name": "Word Puzzle"
```

```json
"name": "game.word_puzzle.name"
```

宿主的处理规则是：

- 如果看起来像语言键，就按当前包语言去查
- 查不到时直接回退原字符串
- 如果本身就是普通文本，就直接显示

## 游戏唯一 ID 规则

### 官方游戏 ID

官方游戏必须是：

```text
tui_game_<package_name>_<hash16>
```

并且宿主会按内置白名单强校验，不匹配就当作非法清单。

### 自定义游戏 ID

自定义游戏必须是：

```text
mod_game_<package_name>_<hash16>
```

`hash16` 的输入固定是：

```text
author + "\n" + package_name + "\n" + game_name
```

特点：

- 只用 `0-9a-zA-Z`
- 长度固定 16
- 同样输入一定生成同样结果
- 不是安全哈希，只是稳定且足够区分

仓库里的 `hash.py` 已经和宿主用的是同一套算法，可以直接拿来算。

## 脚本必须导出的函数

每个游戏脚本至少要有：

```lua
function init_game()
end

function handle_event(state, event)
  return state
end

function render(state)
end
```

如果 `best_none` 不是 `null`，还必须有：

```lua
function best_score(state)
end
```

## 帧模型

宿主现在是固定锁帧模型，不是旧的“没事件才补 tick”。

每一帧顺序固定为：

```text
收集本帧累积事件
-> 按顺序逐个调用 handle_event(state, event)
-> 再调用一次 handle_event(state, tick_event)
-> 清空画布
-> 调用一次 render(state)
-> 输出当前帧
```

所以脚本作者应当这样理解：

- 输入先改状态
- 时间推进再改状态
- `render` 永远画本帧最终状态

`tick` 现在是：

- 每帧固定一次
- `dt_ms` 是这一帧真实经过的毫秒数
- 不要写死成 `16`

动态游戏推荐：

- 用 `after_ms(...)`
- 用 `deadline_passed(...)`
- 用 `remaining_ms(...)`

不要在 `render()` 里推进游戏逻辑。

## 资源的存放与读取

游戏自己的资源放在包内，例如：

```text
assets/
  lang/
    en_us.json
    zh_cn.json
  data/
    word.json
  text/
    help.txt
```

读取方式：

- `translate("game.xxx.name")`
- `read_text("text/help.txt")`
- `read_json("data/word.json")`
- `read_bytes("images/logo.bin")`
- `load_helper("helpers/util.lua")`

只能读取**当前包自己的资源**，不能跨包，也不能随便读系统路径。

## 存储会写到哪里

| 接口 | 最终位置 |
| --- | --- |
| `save_data(slot, value)` | `tui-game-data/saves.json` 的 `data[game_id][slot]` |
| `save_continue(value)` | `tui-game-data/saves.json` 的 `continue[game_id]` |
| `load_best_score()` | `tui-game-data/best_scores.json` 的当前游戏条目 |
| `debug_log(message)` | `tui-game-data/log/<game_id>.log` |

包目录里不要写运行时文件。运行时数据都交给宿主 API。

## 注意事项

1. `entry` 是脚本路径，不是函数名。
2. `best_none == null` 的游戏，不要再去做最佳记录刷新。
3. `random(max)` 返回的是 `0..max-1`，不是 Lua `math.random(max)` 的语义。
4. `save_data/load_data` 是普通数据槽，不是最佳记录接口。
5. 继续游戏用 `save_continue/load_continue`，不要自己发明另一套继续槽。
6. 游戏不能自己锁帧，只能在 `game.json.runtime.target_fps` 里声明目标 FPS。

## 权限范围

模组能做的事，只有宿主明确开放出来的这些：

- 处理输入事件
- 更新状态
- 画画布
- 读取自己包里的资源
- 保存普通数据
- 保存继续游戏
- 读取最佳记录
- 写自己的调试日志

模组不能做的事：

- 不能 `require` 外部 Lua
- 不能 `dofile`
- 不能 `io.open`
- 不能读取别的包的资源
- 不能直接操作宿主主循环
- 不能动态修改宿主 FPS

## 一个最小可运行例子

```lua
local state = {
  x = 1,
  y = 1,
}

function init_game()
  return state
end

function handle_event(state, event)
  if event.type == "action" and event.name == "move_right" then
    state.x = state.x + 1
  elseif event.type == "tick" then
    -- 时间推进逻辑
  end
  return state
end

function render(state)
  canvas_draw_text(state.x, state.y, "@", "yellow", nil)
end
```
