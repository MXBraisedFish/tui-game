# 模组制作说明

本文档说明当前版本 TUI-GAME 的模组制作方式。当前架构下：

- 官方包和第三方模组共用同一套宿主 API。
- 模组只能通过宿主公开的运行时、渲染、资源、存档和语言接口工作。
- 模组不能直接操作终端、文件系统或其他包资源。

## 1. 程序位置

### 1.1 模组安装目录
程序运行后，会在可执行文件同级目录下创建数据目录：

```text
./tui-game-data/
```

第三方模组应放在：

```text
./tui-game-data/mod/list/<namespace>/
```

示例：

```text
./tui-game-data/mod/list/examplepack/
```

其中 `<namespace>` 必须只包含英文和数字，并且应与包声明保持一致。

### 1.2 仓库内示例目录
仓库中提供了一个开发示例：

```text
./examples/mod/examplepack/
```

建议新模组先复制这个目录，再按自己的内容修改。

## 2. 目录结构

### 2.1 最小可运行结构
最小可运行模组至少需要：

```text
<namespace>/
  package.json
  game.json
  scripts/
    runtime_demo.lua
  assets/
    lang/
      en_us.json
      zh_cn.json
```

### 2.2 建议完整结构
建议完整结构如下：

```text
<namespace>/
  meta.json
  package.json
  game.json
  scripts/
    main.lua
    helpers/
      *.lua
  assets/
    lang/
      en_us.json
      zh_cn.json
    data/
      *.json
      *.txt
    text/
      *.txt
    images/
      *
```

说明：

- `package.json + game.json`：运行时真正识别的清单。
- `meta.json`：模组管理页、缩略图、横幅、启停状态、调试状态等信息的来源。当前如果你希望模组在设置页完整显示，应提供该文件。
- `scripts/helpers/*.lua`：给 `load_helper()` 使用的辅助脚本。
- `assets/data` / `assets/text` / `assets/images`：给 `read_json/read_text/read_bytes` 使用的包内资源。

## 3. 文件内容

### 3.1 `package.json`
描述整个包的信息。

示例：

```json
{
  "namespace": "examplepack",
  "package_name": "example_mod.package_name",
  "author": "YourName",
  "version": "1.0.0",
  "description": "example_mod.package_description",
  "api_version": 1
}
```

字段说明：

| 字段 | 作用 | 说明 |
| --- | --- | --- |
| `namespace` | 包命名空间 | 必须与目录名一致，只允许字母和数字 |
| `package_name` | 包名称语言键 | 建议始终使用语言键，不写裸文本 |
| `author` | 作者名 | 字符串 |
| `version` | 版本号 | 字符串 |
| `description` | 包描述语言键 | 建议使用语言键 |
| `api_version` | 模组 API 版本 | 当前为 `1` |

### 3.2 `game.json`
描述一个具体游戏的运行入口。

示例：

```json
{
  "id": "examplepack:demo",
  "name": "example_mod.game_name",
  "description": "example_mod.game_description",
  "detail": "example_mod.game_detail",
  "entry": "scripts/main.lua",
  "save": true,
  "best_none": "example_mod.best_none",
  "min_width": 60,
  "min_height": 24,
  "actions": {
    "move_left": ["left", "a"],
    "move_right": ["right", "d"],
    "confirm": ["enter", "space"],
    "restart": ["r"],
    "quit_action": ["q"]
  }
}
```

字段说明：

| 字段 | 作用 | 说明 |
| --- | --- | --- |
| `id` | 游戏唯一 ID | 建议格式：`namespace:game_name` 或更细粒度 ID |
| `name` | 游戏名称语言键 | 建议只写语言键 |
| `description` | 游戏简述语言键 | 建议只写语言键 |
| `detail` | 游戏详情语言键 | 建议只写语言键 |
| `entry` | 入口脚本路径 | 相对包根目录 |
| `save` | 是否支持存档 | `true/false` |
| `best_none` | 无最佳记录时的语言键 | 可选 |
| `min_width` | 最小宽度 | `0` 或缺省表示不限制 |
| `min_height` | 最小高度 | `0` 或缺省表示不限制 |
| `max_width` | 最大宽度 | `0` 或缺省表示不限制 |
| `max_height` | 最大高度 | `0` 或缺省表示不限制 |
| `actions` | 动作到按键的映射 | 宿主会把命中的按键转成 `event.type = "action"` |

### 3.3 `meta.json`
这是模组管理页使用的包元信息文件。

示例：

```json
{
  "package_name": "example_mod.package_name",
  "description": "example_mod.package_description",
  "author": "YourName",
  "version": "1.0.0",
  "namespace": "examplepack",
  "api_version": 1,
  "thumbnail": "examplepack:images/thumb.txt",
  "banner": "examplepack:images/banner.txt"
}
```

当前建议：

- 想要模组在设置页、模组管理页正常显示，提供 `meta.json`。
- 只想做最小运行验证，可以先只写 `package.json + game.json`，但管理页信息会不完整。

### 3.4 入口脚本
当前宿主要求的运行时导出如下：

```lua
function init_game()
  return state
end

function handle_event(state, event)
  return state
end

function render(state)
end

function best_score(state)
  return nil
end
```

说明：

| 函数 | 是否必须 | 作用 |
| --- | --- | --- |
| `init_game()` | 必须 | 初始化并返回状态表 |
| `handle_event(state, event)` | 必须 | 处理输入、tick、resize，并返回新状态 |
| `render(state)` | 必须 | 把当前帧绘制到宿主画布 |
| `best_score(state)` | 建议提供 | 返回最佳记录结构，供宿主写入和展示 |

## 4. 文件规范

### 4.1 编码
- 统一使用 UTF-8。
- 不要写 BOM。
- JSON 文件必须是合法 JSON，不能混入注释。

### 4.2 路径规范
- 清单文件里的路径全部相对当前包根目录。
- 不允许绝对路径。
- 不允许 `..`。
- 不允许跨包访问其他资源。

### 4.3 文案规范
- `package.json` 和 `game.json` 中的文案字段建议全部写语言键，不写裸文本。
- 语言键建议用自己包的前缀，例如：

```text
example_mod.package_name
example_mod.game_name
example_mod.controls
```

### 4.4 动作映射规范
- `actions` 只用于“宿主先处理再交给 Lua”的按键。
- 如果某个键在游戏里本来就是原始输入的一部分，例如字母输入、数字输入、自由文本输入，不建议全部绑进 `actions`。
- 一旦按键命中 `actions`，Lua 侧通常拿到的是 `event.type = "action"`，而不是原始 `key`。

## 5. API 和必要函数的使用

完整 API 请参考：

```text
./API.md
```

这里列出模组制作最常用、最关键的部分。

### 5.1 事件模型
宿主会把事件以 `event` 表的形式交给 `handle_event(state, event)`。

常见事件：

| 事件类型 | 结构 | 说明 |
| --- | --- | --- |
| `action` | `{ type = "action", name = "move_left" }` | 命中 `game.json.actions` 时触发 |
| `key` | `{ type = "key", name = "a" }` | 没命中动作映射时的原始按键 |
| `tick` | `{ type = "tick", dt_ms = 16 }` | 固定步长 tick |
| `resize` | `{ type = "resize", width = 120, height = 40 }` | 终端尺寸变化 |
| `quit` | `{ type = "quit" }` | 宿主层退出事件，通常对应 `Esc` |

建议写法：

```lua
local function normalize_key(event)
  if event == nil then return "" end
  if event.type == "quit" then return "esc" end
  if event.type == "key" then return string.lower(event.name or "") end
  if event.type == "action" then return string.lower(event.name or "") end
  return ""
end
```

### 5.2 渲染函数
最常用：

| 函数 | 用途 |
| --- | --- |
| `canvas_clear()` | 清空整帧画布 |
| `canvas_draw_text(x, y, text, fg, bg)` | 绘制文本 |
| `canvas_fill_rect(x, y, w, h, ch, fg, bg)` | 填充矩形 |

注意：

- 坐标从 `0` 开始。
- 如果你在移植旧脚本，旧脚本通常用 `1` 起始坐标，可以自己包一层：

```lua
local function draw_text(x, y, text, fg, bg)
  canvas_draw_text(math.max(0, x - 1), math.max(0, y - 1), text or "", fg, bg)
end

local function clear()
  canvas_clear()
end
```

### 5.3 布局和测量
常用：

| 函数/常量 | 用途 |
| --- | --- |
| `get_terminal_size()` | 获取终端宽高 |
| `get_text_width(text)` | 计算文本宽度 |
| `get_text_size(text)` / `measure_text(text)` | 计算文本宽高 |
| `ANCHOR_LEFT/CENTER/RIGHT` | 水平锚点 |
| `ANCHOR_TOP/MIDDLE/BOTTOM` | 垂直锚点 |
| `resolve_x` / `resolve_y` / `resolve_rect` | 布局定位 |

### 5.4 存档和最佳记录
常用：

| 函数 | 用途 |
| --- | --- |
| `save_data(slot, value)` | 保存普通数据 |
| `load_data(slot)` | 读取普通数据 |
| `save_continue(value)` | 保存继续游戏快照 |
| `load_continue()` | 读取继续游戏快照 |
| `request_refresh_best_score()` | 请求宿主刷新最佳记录 |
| `update_game_stats(game_id, score, duration_sec)` | 更新宿主统计文件 |

建议：

- `save_data/load_data` 用来存长期记录、最佳数据、配置。
- `save_continue/load_continue` 用来存可恢复的中间局面。
- `best_score(state)` 返回最终展示给宿主的最佳记录块。

### 5.5 资源和语言
常用：

| 函数 | 用途 |
| --- | --- |
| `translate(key)` | 读取当前包语言键 |
| `read_text(path)` | 读取包内文本资源 |
| `read_json(path)` | 读取包内 JSON 资源 |
| `read_bytes(path)` | 读取包内二进制资源 |
| `load_helper(path)` | 加载当前包 `scripts/` 下的辅助 Lua 文件 |

### 5.6 时间和随机数
常用：

| 函数 | 用途 |
| --- | --- |
| `time_now_ms()` | 获取当前 runtime 毫秒时间 |
| `after_ms(delay_ms)` | 创建未来截止时间 |
| `deadline_passed(deadline_ms)` | 判断截止时间是否已到 |
| `remaining_ms(deadline_ms)` | 计算剩余毫秒数 |
| `random()` / `random(max)` / `random(min, max)` | 宿主随机数 |
| `debug_log(message)` | 向当前游戏日志追加一行调试信息 |
| `clear_debug_log()` | 清空当前游戏的调试日志 |

说明：

- 需要停顿、阶段切换、动画时，优先用 `tick + after_ms/deadline_passed`。
- `random(max)` 返回 `0..max-1`，这是为了兼容旧脚本中常见的 `random(n) + 1` 写法。
- `debug_log(message)` 和 `clear_debug_log()` 适合调试输入、状态切换和资源读取，日志文件位于 `tui-game-data/log/<game_id>.log`。

### 5.7 其他必要函数
| 函数 | 用途 |
| --- | --- |
| `get_launch_mode()` | 获取本次是 `new` 还是 `continue` |
| `clear_input_buffer()` | 清理阶段切换时的残留输入 |
| `request_exit()` | 请求退出当前游戏 |
| `debug_log(message)` | 追加一行调试日志 |
| `clear_debug_log()` | 清空当前游戏日志 |
| `was_terminal_resized()` / `consume_resize_event()` | 检测和消费 resize 事件 |

## 6. 资源的存储与调用

### 6.1 语言文件
放在：

```text
assets/lang/en_us.json
assets/lang/zh_cn.json
```

示例：

```json
{
  "example_mod.game_name": "Example Game",
  "example_mod.controls": "[Enter] Confirm [Q] Exit"
}
```

读取：

```lua
local title = translate("example_mod.game_name")
```

### 6.2 文本资源
放在：

```text
assets/text/help.txt
```

读取：

```lua
local help = read_text("text/help.txt")
```

### 6.3 JSON 数据
放在：

```text
assets/data/config.json
assets/data/word.json
```

读取：

```lua
local config = read_json("data/config.json")
local words = read_json("data/word.json")
```

### 6.4 辅助脚本
放在：

```text
scripts/helpers/layout.lua
```

加载：

```lua
load_helper("helpers/layout.lua")
```

注意：

- 只能加载当前包 `scripts/` 下的 `.lua` 文件。
- 不能跨包加载。
- 不能加载宿主根目录脚本。

### 6.5 图片和二进制资源
放在：

```text
assets/images/*
```

读取：

```lua
local raw = read_bytes("images/logo.bin")
```

当前宿主没有给 Lua 提供通用图片解码 API。图片一般用于：
- 模组管理页的 `meta.json` 缩略图和横幅
- 或由你自己读取字节后做自定义解析

## 7. 注意事项

1. 官方包和第三方模组没有特权差异。
2. 不要在脚本里使用：
   - `io.open`
   - `dofile`
   - `require`
   - `os.execute`
   - 其他文件系统或系统调用
3. 不要把所有字母键、数字键都塞进 `actions`。
   - 需要自由输入的游戏应优先吃原始 `key` 事件。
4. 脚本必须返回状态表。
   - `init_game()` 必须返回 state
   - `handle_event()` 必须返回 state
5. 如果游戏自己处理尺寸不足提示：
   - 可以把 `min_width/min_height` 设成 `1/1`
   - 然后在脚本里自己调用 `get_terminal_size()` 判断
6. 如果游戏不自己处理尺寸不足提示：
   - 就在 `game.json` 里写真实限制，让宿主统一挡住
7. 资源路径一律写逻辑路径，不写物理路径。
   - 对：`read_json("data/word.json")`
   - 错：`read_json("assets/data/word.json")`
8. 尽量把可复用逻辑拆进 `scripts/helpers/*.lua`，不要把所有东西塞进一个脚本。

## 8. 权限范围

当前 Lua 模组的权限边界如下。

### 8.1 允许的范围
- 接收宿主事件
- 使用画布绘制内容
- 读取自己包内的文本、JSON、字节资源
- 读取自己包内的语言文件
- 加载自己包内的辅助 Lua 脚本
- 读写宿主提供的存档槽和继续游戏槽
- 请求退出当前游戏
- 请求刷新最佳记录

### 8.2 不允许的范围
- 不能直接操作终端输入输出
- 不能直接访问操作系统命令
- 不能直接访问任意文件系统路径
- 不能访问其他模组包资源
- 不能访问官方包资源
- 不能加载包外 Lua 文件
- 不能使用宿主未公开的内部函数

### 8.3 资源访问边界
- `read_text/read_json/read_bytes/load_helper` 只能访问当前包根目录下的允许区域。
- 宿主会拒绝：
  - 绝对路径
  - `..`
  - 跨目录逃逸
  - 非 `.lua` 的 helper 加载

## 9. 建议的开发流程

1. 复制 `./examples/mod/examplepack/`
2. 修改 `package.json` / `game.json`
3. 写入口脚本并导出：
   - `init_game`
   - `handle_event`
   - `render`
   - `best_score`
4. 把文案放进 `assets/lang/`
5. 把词库、配置、文本放进 `assets/data/` 或 `assets/text/`
6. 用 `load_helper()` 拆分辅助脚本
7. 测试：
   - 新开游戏
   - 继续游戏
   - resize
   - 中文/英文切换
   - 保存/读取
   - 退出回列表
