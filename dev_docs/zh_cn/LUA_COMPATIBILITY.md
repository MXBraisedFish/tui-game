# Lua 兼容性与当前注册面

更新时间：2026-09-29。本文说明脚本能使用哪些接口，以及与标准 Lua 调用方式的区别。

## 环境边界

脚本只能调用本页列出的接口。`io`、`os`、`package`、原生 `debug` 不开放；项目提供的 `debug` 用于输出信息、断言和受保护调用。

### 基础接口

下表列出当前可用的基础接口。`base`、受保护调用与标准 `table` 方法保留位置参数或变参；`math`、`string`、`utf8` 的项目接口以各自文档为准，不保证与同名标准函数有相同的签名和结果。

| 位置 | 当前安装项 |
|---|---|
| 全局及 `base` | `ipairs`, `pairs`, `next`, `select`, `rawequal`, `rawget`, `rawset`, `rawlen`, `tonumber`, `tostring`, `type`, `setmetatable`, `getmetatable` |
| `math` | 函数：`abs`, `ceil`, `floor`, `round`, `round_to`, `fmod`, `pow`, `exp`, `log`, `lg`, `ln`, `sqrt`, `ldexp`, `frexp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `deg`, `rad`, `normalize_angle`, `max`, `min`, `modf`, `tointeger`, `type`, `ult`, `approx_equal`, `percent`, `factorial`, `combination`；常量：`PI`, `E`, `POSITIVE_INFINITE`, `INFINITE`, `NEGATIVE_INFINITE`, `DEG`, `RAD`, `MAX_INTEGER`, `MIN_INTEGER` |
| `table` | 原生 Lua 5.4：`concat`, `insert`, `move`, `pack`, `remove`, `sort`, `unpack`（由宿主包装保留操作上限）；项目扩展：`count`, `count_array`, `count_hash`, `compact`, `deepcopy`, `pretty` |
| `string` | 常量 `AUTO`, `PLAIN_TEXT`, `RICH_TEXT`；函数 `lower`, `upper`, `reverse`, `regex_escape`, `split`, `sub`, `rep`, `find`, `match`, `gmatch`, `gsub`, `regex_find`, `regex_match`, `regex_gmatch`, `regex_gsub`, `regex_test`, `regex_split`, `format`, `rich_text_to_plain_text` |
| `utf8` | `len`, `byte_len`, `is_ascii`, `codepoint_to_char`, `ascii_to_char`, `char_to_codepoint`, `char_to_ascii`, `char_position`, `codepoints`, `next` |

更多调用示例见⌞[API 总览](API.md)⌝。

### 项目扩展

下表列出项目扩展。部分方法名与标准 Lua 相同，仍需使用项目文档中的参数与返回值。

| 命名空间 | 当前注册项 |
|---|---|
| `math` | `round`, `round_to`, `pow`, `lg`, `ln`, `ldexp`, `frexp`, `atan2`, `normalize_angle`, `approx_equal`, `percent`, `factorial`, `combination`, `PI`, `E`, `POSITIVE_INFINITE`, `INFINITE`, `NEGATIVE_INFINITE`, `DEG`, `RAD` |
| `table` | `count`, `count_array`, `count_hash`, `compact`, `deepcopy`, `pretty` |
| `string` | 常量 `AUTO`, `PLAIN_TEXT`, `RICH_TEXT`；`split`, `regex_escape`, `regex_find`, `regex_match`, `regex_gmatch`, `regex_gsub`, `regex_test`, `regex_split`, `rich_text_to_plain_text` |
| `utf8` | `byte_len`, `is_ascii`, `codepoint_to_char`, `ascii_to_char`, `char_to_codepoint`, `char_to_ascii`, `char_position`, `codepoints` |
| `color` | `BLACK`, `RED`, `GREEN`, `YELLOW`, `BLUE`, `MAGENTA`, `CYAN`, `GRAY`, `GREY`, `BRIGHT_GRAY`, `BRIGHT_GREY`, `BRIGHT_RED`, `BRIGHT_GREEN`, `BRIGHT_YELLOW`, `BRIGHT_BLUE`, `BRIGHT_MAGENTA`, `BRIGHT_CYAN`, `WHITE`, `NONE`, `TRANSPARENT`, `rgb`, `hex` |
| `char` | `LINE`, `BOLD_LINE`, `DOUBLE_LINE`, `ROUNDED_LINE`, `ASCII_NUMBER`, `ASCII_LOWERCASE`, `ASCII_UPPERCASE`, `ASCII_LETTER`, `ASCII_CHARACTER`, `ASCII` |
| `align` | 常量 `AUTO`, `LEFT`, `HORIZONTAL_CENTER`, `RIGHT`, `TOP`, `VERTICAL_CENTER`, `BOTTOM`, `CENTER`；函数 `resolve_x`, `resolve_y`, `resolve_rect` |
| `measurement` | `get_text_size`, `get_text_width`, `get_text_height` |
| `random` | 常量 `INT`, `FLOAT`；函数 `randint`, `randfloat`, `create`, `delete`, `clear`, `list`, `count`, `generate`, `set`, `set_type`, `set_range`, `set_seed`, `set_step`, `get_type`, `get_seed`, `get_step`, `exists`, `get_range`, `get_info` |
| `slice` | `create`, `delete`, `clear`, `set`, `set_size`, `set_width`, `set_height`, `set_background`, `set_layer`, `draw`, `exists`, `get_size`, `get_width`, `get_height`, `get_layer`, `get_background`, `get_info`, `list`, `count` |
| `serialization` | 常量 `NULL`；`json_encode/decode`, `csv_encode/decode`, `yaml_encode/decode`, `toml_encode/decode`, `ini_encode/decode`, `xml_encode/decode`, `binary_pack`, `binary_unpack`, `binary_packsize` |
| `encoding` | `base64_encode`, `base64_decode`, `url_encode`, `url_decode`, `hex_encode`, `hex_decode` |
| `draw` | `text`, `fill_rect`, `stroke_rect`, `erase_rect`, `render` |
| `debug` | `VERSION`, `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL`, `print`, `info`, `warn`, `error`, `assert`, `pcall`, `xpcall` |
| `game` | `exit_game`, `save_game`, `save_best` |
| `i18n` | `create`, `get_value`, `get_language_code`, `reload` |
| `image` | `load` |
| `event` | `skip_action`, `clear_action` |
| `loader` | `require`, `dofile`, `loadfile` |
| `file` | `read`, `write`, `create_dir`, `exists`, `remove`, `list_dir`；另有编码与换行常量，详见 `api/file.md` |

各库的参数、返回值和使用限制见⌞[API 总览](API.md)⌝。

### 尚未开放

`image.load` 已开放，通过请求编号关联异步结果。`audio`、`animation`、`http`、`timer`、`effect`、`widget`、`ime`、`keyboard` 未由当前 `install` 注册。存在相应服务、事件类型或文档页不等于 Lua 脚本可以调用。

## 调用差异

- 使用 Lua 5.4，不把 Lua 5.5 新增能力视为已提供。
- `base` 方法也能直接使用全局名称，例如 `pairs(t)`。`debug.pcall` 和 `debug.xpcall` 使用变参；全局 `pcall`、`xpcall`、`assert`、`print` 不开放。
- `table.concat/insert/move/pack/remove/sort/unpack` 使用标准位置参数；其余项目接口有选填参数时使用末尾严格选项表。
- `string.find` 返回起点、终点、捕获表；`string.match` 和 `string.gmatch` 返回捕获表。字符位置按 Unicode 字符计数，不能直接套用标准 Lua 的字节位置和捕获多返回值写法。
- `math.max/min` 接收一个数值数组，`math.log` 要求显式给出底数，`math.fmod` 只接受整数。其余限制查看各方法说明。
- `utf8` 仅提供上表列出的名字，没有 `utf8.codepoint`、`utf8.byte_position` 或 `utf8.position` 别名。
- 使用 `string.sub(text, start, options)` 等库函数；不支持借助字符串冒号方法调用原生接口。
- 相关多个结果按顺序返回；捕获列表、解码对象和配置列表等数据本身仍为表。查看⌞[Lua API 调用约定](LUA_API_MIGRATION.md)⌝。
