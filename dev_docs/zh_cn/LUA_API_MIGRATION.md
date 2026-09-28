# Lua 扩展 API 契约迁移表（初稿）

更新时间：2026-09-28。以 `crates/service/lua/src/api/libraries.rs::install` 为实际注册源；只记录宿主扩展，不把标准 Lua API、lifecycle callback 或未注册文档页误列为可调用扩展。Q6 的统一参数/结果协议已确认；B10.2 已迁移到 game/event/debug/loader，并核对 B6 image 异步契约。

## 当前注册映射

| 命名空间 | 当前实际注册成员 | 参数、返回、权限、测试与文档核对位置 |
|---|---|---|
| `math`（项目扩展） | `round`, `round_to`, `pow`, `lg`, `ln`, `ldexp`, `frexp`, `atan2`, `normalize_angle`, `approx_equal`, `percent`, `factorial`, `combination` 与常量 | B2 已确认保留项目协议；函数级范围/数值边界：[math API](api/math.md)；实现 `libraries/math.rs`，Lua 行为回归 `session.rs` |
| `table`（项目扩展） | `count`, `count_array`, `count_hash`, `compact`, `deepcopy`, `pretty` | 与原生 Lua `table.*` 分开；[table API](api/table.md)；实现 `libraries/table.rs`，回归 `session.rs` |
| `string`（项目扩展） | `AUTO`, `PLAIN_TEXT`, `RICH_TEXT`, `split`, `regex_escape`, `regex_find`, `regex_match`, `regex_gmatch`, `regex_gsub`, `regex_test`, `regex_split`, `rich_text_to_plain_text` | 用户确认命名参数/项目结果表/字符串元表隔离；[string API](api/string.md)；实现 `libraries/string.rs` 和 `string/pattern.rs`，回归 `session.rs` |
| `utf8`（项目扩展） | `byte_len`, `is_ascii`, `codepoint_to_char`, `ascii_to_char`, `char_to_codepoint`, `char_to_ascii`, `char_position`, `codepoints`, `byte_position`, `position` | B2 已确认保留数组/命名参数、记录迭代器及项目结果表；[utf8 API](api/utf8.md)；实现 `libraries/utf8.rs`，回归 `session.rs` |
| `color` / `char` | `color`: 调色板常量、`NONE`/`TRANSPARENT`/灰阶别名、`rgb`, `hex`; `char`: 边框与 ASCII 字符数组常量 | 范围、字段、只读语义：[color API](api/color.md)、[char API](api/char.md)；实现/测试 `libraries/color.rs`, `char.rs` |
| `align` / `measurement` | `align`: 常量、`resolve_x/y/rect`; `measurement`: `get_text_size/width/height` | 对齐坐标、surface 单位、文本尺寸和结果字段：[align API](api/align.md)、[measurement API](api/measurement.md)；实现同名 modules，回归 `session.rs` |
| `random` / `slice` | `random`: `INT/FLOAT`, `randint/randfloat/create/delete/clear/list/count/generate/set/set_type/set_range/set_seed/set_step/get_type/get_seed/get_step/exists/get_range/get_info`; `slice`: `create/delete/clear/set/set_size/set_width/set_height/set_background/set_layer/draw/exists/get_size/get_width/get_height/get_layer/get_background/get_info/list/count` | 对象 ID、生命周期、坐标/尺寸、变更与查询记录：[random API](api/random.md)、[slice API](api/slice.md)；实现同名 modules，回归 `session.rs` |
| `serialization` / `encoding` | `serialization`: `json_encode/decode`, `csv_encode/decode`, `yaml_encode/decode`, `toml_encode/decode`, `ini_encode/decode`, `xml_encode/decode`, `binary_pack/unpack/packsize`; `encoding`: Base64/URL/hex encode/decode | Lua ↔ JSON/YAML null、跨格式拒绝、深度/节点/字节上限及 binary format：[serialization API](api/serialization.md)、[encoding API](api/encoding.md)；实现 `libraries/serialization/*`, `encoding.rs`，回归 `session.rs`。encoding 与 serialization 均已改为具名参数并通过定向/完整 Lua 回归 |
| `draw` | `text`, `fill_rect`, `stroke_rect`, `erase_rect`, `render` | 坐标、尺寸、目标层、绘制模式与宿主命令：[draw API](api/draw.md)；实现 `libraries/draw.rs`, `measurement.rs`，回归 `session.rs` |
| `debug` | 常量、`print/info/warn/error/assert/pcall/xpcall` | 所有方法用单个命名表；日志便捷方法用 `{message=...}`，protected-call 继续返回 `{ok, values/error}`：[debug API](api/debug.md)；实现 `libraries/debug.rs`，回归 `session.rs` |
| `game` / `event` | `game.exit_game/save_game/save_best`; `event.skip_action/clear_action` | 零参数操作使用显式空命名表 `{}`；Game session 与 callback 阶段限制保持：[game API](api/game.md)、[event API](api/event.md)；实现同名 modules，回归 `session.rs` |
| `i18n` / `file` / `image` | `i18n.create/get_value/get_language_code/reload`; `file.read/write/create_dir/exists/remove/list_dir`; `image.load` | 本地资源、sandbox、编码、request id 和异步终态：[i18n API](api/i18n.md)、[file API](api/file.md)、[image API](api/image.md)；异步成功入队返回 ID，终态回传相同 ID；image 保留 B6 富文本结果；实现同名 modules、event broker 和 `session.rs` |
| `loader` | `require`, `dofile`, `loadfile` | 所有调用使用 `{path=...}`；按 OQ-B10-04 保留模块原始多返回值；包内路径、缓存与加载错误：[loader API](api/loader.md)；实现 `libraries/loader.rs`，回归 `session.rs` |

标准/扩展边界见 [LUA_COMPATIBILITY.md](LUA_COMPATIBILITY.md)。每个成员的现行调用字段、默认值、单位与返回值在对应链接的 API 文档中；已迁移成员以实现、session 回归和对应 API 文档为准。相邻 namespace 仅为排版分组，不代表调用协议相同。

## Q6 当前协议样例与统一建议

下表记录当前调用形状和已确认的全局协议。按 OQ-B10-01，宿主扩展使用一个命名参数表；简单结果直接返回，复合结果使用具名表；异步调用立即返回 request id，终态事件回传同一 id。B2 string/math/utf8 与 B6 image 使用已单独确认的契约；不保留旧 API alias。

| 类型 | 当前代码/文档示例 | 需要锁定的内容 |
|---|---|---|
| 纯值 `color.rgb` | `local gray = color.rgb{r=85, g=87, b=83}`；接收一个命名表，返回字符串 `"rgb(85,87,83)"`；[color API](api/color.md) | 保持命名表；复核字段与错误上下文 |
| 选项多的 `draw.text` | `draw.text{x=2, y=3, text="Hello", fg=color.WHITE}`；只接收一个命名表并拒绝未知字段，排入绘制命令且无同步结果；[draw API](api/draw.md) | 保持命名表；逐字段复核默认值、尺寸单位、错误与画布目标 |
| 异步 `file.read` / `image.load` | 两者接收命名字段表并通过 `HandleEvent` 取终态。`file.read{path="x.txt"}` 与 `image.load{path="x.png"}` 均立即返回 request id，终态回传相同 id；image 完全遵循 B6 已确认契约 | 统一成功/失败/取消形状并验证 session token；image 不重复裁决 |

当前协议已经确认并开始落实：除单独确认的 B2 与 B6 契约外，宿主扩展统一使用单一命名参数表；简单结果返回标量，复合结果返回具名表；异步调用立即返回 request id，终态事件回传该 id。全项目不做旧内容兼容，不添加旧名称 alias。逐 API 签名迁移按纯值工具、绘制与对象 API、异步 API、生命周期和事件 API 分组进行。`serialization.NULL` 的跨格式范围见 [OPEN_QUESTIONS.md](../refactor/OPEN_QUESTIONS.md) OQ-B10-03。

## 尚未进入实现的范围

- 不将原生 `assert`、`pcall`、`require` 等仅凭名字相似而纳入扩展迁移；先按 B2 标准 Lua 契约划分。
- `image.load` 已于 B6 注册并实现；参数、request id、富文本 `output` 与完成/失败事件遵循已确认契约。
- 组件事件与独立 callback 当前缺少生产来源/注册 API；不得只去掉 `cfg(test)` 暴露半成品。
- 每组迁移需要同时更新 Rust 注册、Lua 调用、示例、事件和开发文档，并以脚本测试验证。
- encoding 六个转换 API 已统一为 `encoding.*{s = ...}`；不再接受位置参数。
- 项目自定义 `table.deepcopy/pretty/count/count_array/count_hash/compact` 已统一使用 `{table = ...}`；原生 Lua `table.*` 保持 Lua 5.4 标准参数。
- `random` 的生成器操作统一使用 `{id = ...}`；`clear/list/count` 使用空命名表 `{}`。
- `slice` 的对象操作统一使用 `{id = ...}`；`clear/list/count` 与 `draw.render` 使用空命名表 `{}`。`measurement` 查询方法原已使用命名参数表。
- file/i18n 异步调用在任务成功入队后立即返回会话 request ID，终态事件回传该 ID；单例/权限门控而未入队时返回 `nil` 且不发事件。`file.exists` 使用 `{path=...}`。
- `game`/`event` 无参命令必须传 `{}`；`debug.info/warn/error` 使用 `{message=...}`。权限拒绝仍在参数校验前处理，回调重入限制不变。
- loader 的 `require/dofile/loadfile` 路径统一用 `{path=...}`；`require/dofile` 仍透传模块自己的多返回值，`loadfile` 返回函数。
- 当前生产事件源与测试专用结构的逐项依据见 [B10_EVENT_AUDIT.md](../refactor/B10_EVENT_AUDIT.md)。
