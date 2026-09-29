# Lua 兼容性与当前注册面

更新时间：2026-09-29。本文记录 `crates/service/lua/src/api/libraries.rs::install` 实际装入的 Lua 环境；标准能力按 Lua 5.4 协议，项目扩展按各 API 页面说明执行。

## 环境边界

宿主只选择性启用当前兼容组需要的 Lua 原生库，再将允许的 API 装入独立脚本 `_ENV`。当前只启用了安全的原生 `table` 库以复用 Lua 5.4 标准函数；其余未装入库不能因为 Lua 5.4 有该能力就视为可用。`io`、`os`、`package`、`debug` 原生库不开放；宿主提供的 `debug` 是受控扩展。

### 标准库兼容层（B2 已完成逐组审计）

这些命名空间实现或包装 Lua 常见标准能力。下表是当前注册面；B2 已逐组核对 Lua 5.4 语义、项目协议、错误和安全边界。项目专属成员单列在宿主扩展表。

| 位置 | 当前安装项 |
|---|---|
| 全局及 `base` | `ipairs`, `pairs`, `next`, `select`, `rawequal`, `rawlen`, `tonumber`, `tostring`, `type`, `setmetatable`, `getmetatable` |
| `math` | 函数：`abs`, `ceil`, `floor`, `round`, `round_to`, `fmod`, `pow`, `exp`, `log`, `lg`, `ln`, `sqrt`, `ldexp`, `frexp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `deg`, `rad`, `normalize_angle`, `max`, `min`, `modf`, `tointeger`, `type`, `ult`, `approx_equal`, `percent`, `factorial`, `combination`；常量：`PI`, `E`, `POSITIVE_INFINITE`, `INFINITE`, `NEGATIVE_INFINITE`, `DEG`, `RAD`, `MAX_INTEGER`, `MIN_INTEGER` |
| `table` | 原生 Lua 5.4：`concat`, `insert`, `move`, `pack`, `remove`, `sort`, `unpack`（由宿主包装保留操作上限）；项目扩展：`count`, `count_array`, `count_hash`, `compact`, `deepcopy`, `pretty` |
| `string` | 常量 `AUTO`, `PLAIN_TEXT`, `RICH_TEXT`；函数 `lower`, `upper`, `reverse`, `regex_escape`, `split`, `sub`, `rep`, `find`, `match`, `gmatch`, `gsub`, `regex_find`, `regex_match`, `regex_gmatch`, `regex_gsub`, `regex_test`, `regex_split`, `format`, `rich_text_to_plain_text` |
| `utf8` | `len`, `byte_len`, `is_ascii`, `codepoint_to_char`, `ascii_to_char`, `char_to_codepoint`, `char_to_ascii`, `char_position`, `codepoints`, `byte_position`, `codepoint`, `next`, `position` |

`base` 的迭代器、表长、模式和类型行为由 Lua 5.4 同版本用例审计；项目扩展的参数/结果形状按对应 API 文档说明。标准 `table.*` 使用 Lua 5.4 实现并由宿主包装限制操作量；只读 API 表不能通过 rawset、insert、remove、sort 或 move 被修改。错误、预算、GC、终结器、`<close>` 和会话结束行为由 Lua 宿主回归覆盖。

### 宿主扩展

以下成员属于项目扩展；其余列在标准库兼容层的成员也要由 B2 验证是否真正遵循 Lua 5.4。

| 命名空间 | 当前注册项 |
|---|---|
| `math` | `round`, `round_to`, `pow`, `lg`, `ln`, `ldexp`, `frexp`, `atan2`, `normalize_angle`, `approx_equal`, `percent`, `factorial`, `combination`, `PI`, `E`, `POSITIVE_INFINITE`, `INFINITE`, `NEGATIVE_INFINITE`, `DEG`, `RAD` |
| `table` | `count`, `count_array`, `count_hash`, `compact`, `deepcopy`, `pretty` |
| `string` | 常量 `AUTO`, `PLAIN_TEXT`, `RICH_TEXT`；`split`, `regex_escape`, `regex_find`, `regex_match`, `regex_gmatch`, `regex_gsub`, `regex_test`, `regex_split`, `rich_text_to_plain_text` |
| `utf8` | `byte_len`, `is_ascii`, `codepoint_to_char`, `ascii_to_char`, `char_to_codepoint`, `char_to_ascii`, `char_position`, `codepoints`, `byte_position`, `position` |
| `color` | `BLACK`, `RED`, `GREEN`, `YELLOW`, `BLUE`, `MAGENTA`, `CYAN`, `GRAY`, `GREY`, `BRIGHT_GRAY`, `BRIGHT_GREY`, `BRIGHT_RED`, `BRIGHT_GREEN`, `BRIGHT_YELLOW`, `BRIGHT_BLUE`, `BRIGHT_MAGENTA`, `BRIGHT_CYAN`, `WHITE`, `NONE`, `TRANSPARENT`, `rgb`, `hex` |
| `char` | `LINE`, `BOLD_LINE`, `DOUBLE_LINE`, `ROUNDED_LINE`, `ASCII_NUMBER`, `ASCII_LOWERCASE`, `ASCII_UPPERCASE`, `ASCII_LETTER`, `ASCII_CHARACTER`, `ASCII` |
| `align` | 常量 `AUTO`, `LEFT`, `HORIZONTAL_CENTER`, `RIGHT`, `TOP`, `VERTICAL_CENTER`, `BOTTOM`, `CENTER`；函数 `resolve_x`, `resolve_y`, `resolve_rect` |
| `measurement` | `get_text_size`, `get_text_width`, `get_text_height` |
| `random` | 常量 `INT`, `FLOAT`；函数 `randint`, `randfloat`, `create`, `delete`, `clear`, `list`, `count`, `generate`, `set`, `set_type`, `set_range`, `set_seed`, `set_step`, `get_type`, `get_seed`, `get_step`, `exists`, `get_range`, `get_info` |
| `slice` | `create`, `delete`, `clear`, `set`, `set_size`, `set_width`, `set_height`, `set_background`, `set_layer`, `draw`, `exists`, `get_size`, `get_width`, `get_height`, `get_layer`, `get_background`, `get_info`, `list`, `count` |
| `serialization` | `json_encode/decode`, `csv_encode/decode`, `yaml_encode/decode`, `toml_encode/decode`, `ini_encode/decode`, `xml_encode/decode`, `binary_pack`, `binary_unpack`, `binary_packsize` |
| `encoding` | `base64_encode`, `base64_decode`, `url_encode`, `url_decode`, `hex_encode`, `hex_decode` |
| `draw` | `text`, `fill_rect`, `stroke_rect`, `erase_rect`, `render` |
| `debug` | `VERSION`, `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL`, `print`, `info`, `warn`, `error`, `assert`, `pcall`, `xpcall` |
| `game` | `exit_game`, `save_game`, `save_best` |
| `i18n` | `create`, `get_value`, `get_language_code`, `reload` |
| `event` | `skip_action`, `clear_action` |
| `loader` | `require`, `dofile`, `loadfile` |
| `file` | `read`, `write`, `create_dir`, `exists`, `remove`, `list_dir`；另有编码与换行常量，详见 `api/file.md` |

`char` 的数组内容、`random` / `slice` 的对象字段及各函数详细形状以后续代码审计为准。上述扩展名来自当前注册实现，不从文档占位页推导 API。

### 尚未注册

`image` 已由 B6 注册 `image.load`，使用 request id 关联异步 `HandleEvent` 终态。`audio`、`animation`、`http`、`timer`、`effect`、`widget`、`ime`、`keyboard` 未由当前 `install` 注册。存在相应服务、事件类型或文档页不等于 Lua 脚本可以调用。

## B2 对照约定

- 用户已确认保留 vendored Lua 5.4；不升级到 Lua 5.5。
- 对标准库按 Lua 5.4 同版本行为作为主对照。Lua 5.5 只记录版本差异，不作为本项目目标。
- `io`、`os`、`package`、原生 `debug` 等能力不自动开放；任何权限变化需单独说明沙箱边界与测试。
- 用户已确认字符串原生元表保持隔离：`getmetatable("x")` 在宿主环境返回 nil；不能借 `("x"):sub(...)` 绕过受控的项目 `string.*` API。
- 项目扩展按必填参数顺序使用位置参数；方法声明了选项时，选项放在末尾表中。原生 Lua 5.4 接口仍保留其原生参数协议。
- 宿主自定义的多个相关结果按 Lua 多返回值传递；捕获列表、解码对象、配置列表等本来就是数据的结果仍保留为 Lua 表。
- 用户已确认字符串元表保持隔离；`getmetatable("x")` 不暴露原生字符串方法。受控的项目 `string.*` 接口仍遵循项目自己的位置参数和返回形状。
- Lua 5.4 基线用于核对标准接口行为、安全边界和错误路径；项目扩展的签名、返回顺序与限制见对应 API 页面及 [Lua API 调用约定](LUA_API_MIGRATION.md)。
