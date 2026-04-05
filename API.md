# Lua Runtime API

本文档列出当前宿主实际注入给 Lua 的公开 API。官方包和第三方模组共用同一套接口。

## 函数
| 函数 | 函数作用 | 参数解析 |
| --- | --- | --- |
| `canvas_clear()` | 清空当前画布。 | 无参数。 |
| `canvas_draw_text(x, y, text, fg, bg)` | 在画布指定位置绘制文本。坐标从 `0` 开始。 | `x: integer` 列坐标；`y: integer` 行坐标；`text: string` 文本；`fg: string \| nil` 前景色；`bg: string \| nil` 背景色。 |
| `canvas_fill_rect(x, y, width, height, ch, fg, bg)` | 用单个字符填充矩形区域。 | `x: integer`；`y: integer`；`width: integer`；`height: integer`；`ch: string` 只取第一个字符；`fg: string \| nil`；`bg: string \| nil`。 |
| `measure_text(text)` | 计算文本显示宽高。 | `text: string`。返回 `(width, height)`。 |
| `get_text_width(text)` | 计算文本显示宽度。 | `text: string`。返回 `width`。 |
| `get_text_size(text)` | 计算文本显示宽高。 | `text: string`。返回 `(width, height)`。 |
| `get_terminal_size()` | 获取当前终端宽高。 | 无参数。返回 `(width, height)`。 |
| `resolve_x(anchor, content_width, offset)` | 按水平锚点计算最终 `x`。 | `anchor: integer`；`content_width: integer`；`offset: integer \| nil`，缺省为 `0`。 |
| `resolve_y(anchor, content_height, offset)` | 按垂直锚点计算最终 `y`。 | `anchor: integer`；`content_height: integer`；`offset: integer \| nil`，缺省为 `0`。 |
| `resolve_rect(h_anchor, v_anchor, width, height, offset_x, offset_y)` | 按锚点计算矩形左上角坐标。 | `h_anchor: integer`；`v_anchor: integer`；`width: integer`；`height: integer`；`offset_x: integer \| nil`；`offset_y: integer \| nil`。返回 `(x, y)`。 |
| `was_terminal_resized()` | 查询自上次消费后是否收到终端尺寸变化。 | 无参数。返回 `boolean`。 |
| `consume_resize_event()` | 读取并清除终端尺寸变化标记。 | 无参数。返回 `boolean`。 |
| `request_exit()` | 请求宿主退出当前游戏并返回上层页面。 | 无参数。 |
| `request_refresh_best_score()` | 请求宿主立即刷新最佳记录。 | 无参数。 |
| `debug_log(message)` | 向当前游戏的运行时日志文件追加一行调试信息。 | `message: string`。日志文件位于 `tui-game-data/runtime-logs/<game_id>.log`。 |
| `clear_debug_log()` | 清空当前游戏的运行时日志文件。 | 无参数。 |
| `save_data(slot, value)` | 保存当前游戏的存档槽。 | `slot: string`；`value: nil \| boolean \| number \| string \| table`。 |
| `load_data(slot)` | 读取当前游戏的存档槽。 | `slot: string`。返回对应 Lua 值，不存在时返回 `nil`。 |
| `save_game_slot(slot, value)` | 保存“继续游戏”快照槽。 | `slot: string`；`value: nil \| boolean \| number \| string \| table`。返回 `true`。 |
| `load_game_slot(slot)` | 读取“继续游戏”快照槽。 | `slot: string`。返回对应 Lua 值，不存在时返回 `nil`。 |
| `update_game_stats(game_id, score, duration_sec)` | 更新宿主统计文件中的分数和时长。 | `game_id: string`；`score: integer`；`duration_sec: integer`。 |
| `translate(key)` | 读取当前包语言域中的语言键。 | `key: string`。查询顺序为当前语言、包内 `en_us`、缺失提示。 |
| `read_text(path)` | 读取当前包 `assets/` 下的文本资源。 | `path: string` 包内逻辑路径，例如 `data/help.txt`。禁止绝对路径和 `..`。 |
| `read_bytes(path)` | 读取当前包 `assets/` 下的二进制资源。 | `path: string` 包内逻辑路径。返回 Lua 字符串形式的原始字节。 |
| `read_json(path)` | 读取当前包 `assets/` 下的 JSON 资源并转换为 Lua 值。 | `path: string` 包内逻辑路径。返回 `table / string / number / boolean / nil`。 |
| `load_helper(path)` | 加载当前包 `scripts/` 下的辅助 Lua 脚本。 | `path: string` 包内逻辑路径，例如 `helpers/layout.lua`。只允许 `.lua`，禁止绝对路径和 `..`。 |
| `get_launch_mode()` | 获取当前游戏启动方式。 | 无参数。返回 `"new"` 或 `"continue"`。 |
| `clear_input_buffer()` | 清空当前终端事件缓冲。 | 无参数。用于阶段切换时丢弃残留按键。 |
| `time_now_ms()` | 获取当前 runtime 自启动以来的毫秒数。 | 无参数。返回 `integer`。 |
| `after_ms(delay_ms)` | 以当前时间为基准创建未来截止时间。 | `delay_ms: integer`。返回 `deadline_ms`。 |
| `deadline_passed(deadline_ms)` | 判断截止时间是否已到。 | `deadline_ms: integer`。返回 `boolean`。 |
| `remaining_ms(deadline_ms)` | 计算距离截止时间还剩多少毫秒。 | `deadline_ms: integer`。返回 `integer`，已超时则返回 `0`。 |
| `random()` | 获取一个宿主生成的随机整数。 | 无参数。返回 `integer`。 |
| `random(max)` | 获取 `0..max-1` 的随机整数。 | `max: integer`。若 `max <= 0` 则返回 `0`。这一语义用于兼容旧脚本中的 `random(n) + 1`。 |
| `random(min, max)` | 获取闭区间随机整数。 | `min: integer`；`max: integer`。若顺序相反，宿主会自动交换。 |

## 常量
| 常量 | 作用 | 取值说明 |
| --- | --- | --- |
| `ANCHOR_LEFT` | 水平左对齐锚点。 | `0` |
| `ANCHOR_CENTER` | 水平居中锚点。 | `1` |
| `ANCHOR_RIGHT` | 水平右对齐锚点。 | `2` |
| `ANCHOR_TOP` | 垂直顶部锚点。 | `0` |
| `ANCHOR_MIDDLE` | 垂直居中锚点。 | `1` |
| `ANCHOR_BOTTOM` | 垂直底部锚点。 | `2` |

## 补充说明
| 项目 | 说明 |
| --- | --- |
| 颜色参数 | 当前颜色参数统一为字符串或 `nil`，例如 `white`、`black`、`yellow`、`light_cyan`、`dark_gray`、`rgb(255,0,0)`。 |
| 坐标系 | `canvas_*` 系列函数全部使用从 `0` 开始的画布坐标。旧脚本若是从 `1` 开始，需要自行做偏移。 |
| 资源路径 | `read_text/read_json/read_bytes/load_helper` 都只接收包内逻辑路径，不要写 `assets/...`，不要写绝对路径，也不能跨包。 |
| 存档值 | `save_data` 与 `save_game_slot` 只应保存可序列化的 Lua 值，避免函数、userdata、线程等内容。 |
| 随机数 | `random(max)` 不是 Lua 标准库 `math.random(max)` 语义，而是兼容旧宿主脚本的 `0..max-1`。 |
