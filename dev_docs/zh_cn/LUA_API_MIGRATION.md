# Lua API 调用约定

更新时间：2026-09-29。本文说明当前注册的 Lua API 调用和返回规则。各方法的字段、默认值、单位及权限限制以对应 API 页面为准。

## 调用参数

- Lua 5.4 原生接口保持原生参数协议，包括迭代器、`pcall`、`select` 和原生 `table.*`。
- 项目 API 按必填参数的顺序直接传参。方法声明了选项时，可在末尾传一个选项表；不需要选项时可省略。
- 没有选项的方法不接受额外参数。无参方法使用空括号调用。
- 选项表只接受文档列出的字段。未知字段、非字符串键、字段类型错误和多余位置参数会报错。
- 真实业务数据表仍按位置参数传递，不会被当作选项表解析。`string.format`、`serialization.binary_pack`、`debug.pcall` 和 `debug.xpcall` 的变参原样传递，变参中的表和 nil 都会保留。

```lua
draw.text(2, 3, "Hello", {fg = color.WHITE, bold = true})
local encoded = encoding.base64_encode("Hello")
local rows = serialization.csv_encode({{"name", "score"}, {"Ada", 10}})
local id = file.read("notes.txt", {encoding = file.UTF_8})
```

## 返回值

- 一个结果直接返回该值；本来就是数据表的结果仍返回表。
- 相关多个结果使用 Lua 多返回值，接收时按文档顺序赋值。
- 条件结果按实际分支返回。例如查询到对象时返回多个值，未找到时返回一个 nil。
- 无返回值与返回一个 nil 不同。多返回值里的 nil 位置也会保留。
- `loader.require` 和 `loader.dofile` 保留模块自己的多返回值；`loader.loadfile` 返回函数，调用该函数后也保留模块的多返回值。

```lua
local start_pos, end_pos, captures = string.find("item-42", "(%a+)%-(%d+)")
local width, height = measurement.get_text_size("Hello")
local bytes = serialization.binary_pack("<I2", 42)
local values, next_pos = serialization.binary_unpack("<I2", bytes)
```

## 异步方法

`file` 的异步操作、`i18n.create/reload` 和 `image.load`在成功入队后立即返回 request id，不会同步等待任务完成。终态事件在 `data.request_id` 中携带同一个 id。权限门控或请求去重导致任务未入队时，方法按各自契约返回 nil，且不产生该请求的完成事件。`image.load` 的完成事件还会在 `data.output` 中返回可供 `draw.text` 使用的富文本字符串。

## 序列化空值

JSON 和 YAML 使用 `serialization.NULL` 表示并保留嵌套 null。CSV、INI、TOML 和 XML 不接受该哨兵，会返回明确错误。Lua 表中普通 nil 的处理仍遵循 Lua 表自身的规则。

详细接口见各库页面：

- [API 总览](API.md)
- [Lua 兼容性与注册面](LUA_COMPATIBILITY.md)
- [file](api/file.md)、[i18n](api/i18n.md)、[image](api/image.md)、[serialization](api/serialization.md)
