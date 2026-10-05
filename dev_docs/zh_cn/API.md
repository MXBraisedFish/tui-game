# API 文档

| 项目             | 内容                                        |
| ---------------- | ------------------------------------------- |
| **API 版本**     | 1                                           |
| **最后更新日期** | 2026-10-02                                  |
| **更新作者**     | MXFish                                      |
| **文档作用**     | 本文档展示所有 API 的基本使用标准和总索引。 |

---

## 目录

- [API 使用注意事项](#api-使用注意事项)
- [API 子库文档索引](#api-子库文档索引)
- [API 使用标准](#api-使用标准)

---

## API 使用注意事项

Tui Game 使用 Lua 作为脚本语言。
Lua 基础语法保持不变，但脚本运行环境不会直接提供完整的标准 Lua 库。
可用函数、库和常量以本页索引为准；部分接口沿用 Lua 5.4 调用方式，项目扩展使用各页面列出的参数。

因此：
> 原生 Lua API 不保证可用，请以本文档列出的 API 为准。

---

## API 子库文档索引

| 库名            | 作用                       | 索引                                    | 包含        |
| --------------- | -------------------------- | --------------------------------------- | ----------- |
| `lifecycle`     | 脚本生命周期回调           | [LIFECYCLE](./api/lifecycle.md)         | 方法        |
| `base`          | 提供 Lua 基础值操作        | [BASE](./api/base.md)                   | 方法        |
| `draw`          | 提供终端绘制指令           | [DRAW](./api/draw.md)                   | 方法        |
| `align`         | 辅助计算布局坐标           | [ALIGN](./api/align.md)                 | 方法 / 常量 |
| `char`          | 常用的字符表               | [CHAR](./api/char.md)                   | 常量        |
| `color`         | 颜色控制                   | [COLOR](./api/color.md)                 | 方法 / 常量 |
| `timer`         | 计时与循环提醒             | [TIMER](./api/timer.md)                 | 方法        |
| `date`          | 获取时间及日期与时间戳转换 | [DATE](./api/date.md)                   | 方法 / 常量 |
| `debug`         | 用于脚本信息调试与错误捕获 | [DEBUG](./api/debug.md)                 | 方法 / 常量 |
| `encoding`      | 提供基础的编码与解码转换   | [ENCODING](./api/encoding.md)           | 方法        |
| `event`         | 事件队列控制               | [EVENT](./api/event.md)                 | 方法        |
| `file`          | 异步文件读写与目录枚举     | [FILE](./api/file.md)                   | 方法 / 常量 |
| `game`          | 游戏独立的生命周期控制     | [GAME](./api/game.md)                   | 方法        |
| `image`         | 将图片转换为终端字符画     | [IMAGE](./api/image.md)                 | 方法        |
| `i18n`          | 国际化语言文本管理         | [I18N](./api/i18n.md)                   | 方法        |
| `loader`        | 加载 Lua 模块              | [LOADER](./api/loader.md)               | 方法        |
| `math`          | 数学运算                   | [MATH](./api/math.md)                   | 方法 / 常量 |
| `measurement`   | 辅助文本字符尺寸测量       | [MEASUREMENT](./api/measurement.md)     | 方法        |
| `random`        | 随机数生成                 | [RANDOM](./api/random.md)               | 方法 / 常量 |
| `serialization` | 多格式序列化与反序列化     | [SERIALIZATION](./api/serialization.md) | 方法 / 常量 |
| `slice`         | 图层切片对象管理           | [SLICE](./api/slice.md)                 | 方法        |
| `string`        | 字符串处理                 | [STRING](./api/string.md)               | 方法 / 常量 |
| `table`         | 表操作                     | [TABLE](./api/table.md)                 | 方法        |
| `utf8`          | UTF-8 字符串处理           | [UTF8](./api/utf8.md)                   | 方法        |

---

## API 使用标准

### 生命周期回调 API

**库**：lifecycle

通过重写函数的方式使用，并接收固定参数。

*部分回调 API 需要按照需求 return 相应的值*

**示例**：
```lua
function Init(ctx)
	-- 初始化逻辑
end
```

### 基础库 API

**库**：base

直接使用即可

**示例**：
```lua
local is_equal = rawequal(1, 2) -- 返回 false
```

### 扩展库 API

**库**：除 lifecycle 和 base 之外的所有库

使用 `库名.方法名(...)` 调用，不使用冒号调用。必填参数按顺序传递，选填参数放在末尾的表中；无选填参数时可省略该表。未知选项字段、错误类型和多余位置参数会抛出错误。

**示例**：
```lua
draw.text(1, 2, "Hello Tui Game", {fg = color.WHITE}) -- 在 base 切片坐标 (1, 2) 作为起始位置绘制字符串 "Hello Tui Game"
```

### 参数与结果的补充约定


- 所有公开 API 的选填参数统一放在末尾严格选项表中，例如 `tonumber("ff", {base = 16})`、`table.insert(list, value, {pos = 2})`；旧的位置选填参数写法不再接受。
- `select`、`table.pack`、`string.format`、`serialization.binary_pack`、`debug.pcall` 和 `debug.xpcall` 接收变参；这些值是业务数据，表和 nil 都按原样传递。
- 当前 `string`、`math`、`utf8` 是项目提供的接口；不能仅凭与 Lua 标准函数同名，就套用标准签名。比如 `string.find(text, pattern, {init = 2})` 返回起点、终点和捕获表。
- 多个结果按顺序接收，如 `local width, height = measurement.get_text_size("文字")`。结果本身是数据表时仍返回表，未找到对象时的 `nil` 等分支见对应方法。
- 绘制坐标从 `0` 开始，单位是终端字符格；Lua 数组索引从 `1` 开始。字符位置与字节位置按各方法说明区分。
- 文档中的 `integer`、`float` 是 `number` 的两种形式；`const-color` 等表示该库提供的常量，并非额外的 Lua 类型。二进制数据使用 Lua 字符串保存。
- 示例是单个功能片段。完整脚本还需定义 `Init`、`HandleEvent`、`Update`、`UpdateFrame`、`Render`；显示 `debug.print` 等输出需要开启调试模式。示例下的“输出”留空，供人工实测填写。
- 文件、图片及语言加载需要包内资源；异步方法返回请求编号，实际结果在 `HandleEvent` 中接收，不能把请求编号当作读取结果。
- `audio`、`animation`、`http`、`effect`、`widget` 和 `keyboard` 当前未开放，相关占位页不代表可调用。查看⌊[调用约定](LUA_API_MIGRATION.md)⌉与⌊[可用接口](LUA_COMPATIBILITY.md)⌉。

动作与原始键接收由 [ime 库](api/ime.md) 分别控制。游戏默认接收动作，不接收原始键；held 只发送一次，持续移动在 Update 中使用保存的按住状态。
