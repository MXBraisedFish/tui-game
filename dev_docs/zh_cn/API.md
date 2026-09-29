# API 文档

| 项目         | 内容                       |
| ---------- | ------------------------ |
| **API 版本** | 1                        |
| **最后更新日期** | （待补充）                    |
| **更新作者**   | MXFish                   |
| **文档作用**   | 本文档展示所有 API 的基本使用标准和总索引。 |

---

## 目录

- [API 使用注意事项](#api-使用注意事项)
- [API 子库文档索引](#api-子库文档索引)
- [API 使用标准](#api-使用标准)

---

## API 使用注意事项

Tui Game 使用 Lua 作为脚本语言。
Lua 基础语法保持不变，但脚本运行环境不会直接提供完整的标准 Lua 库。
所有可用函数、库和常量均由 Tui Game 根据脚本运行需求重新定义

因此：
> 原生 Lua API 不保证可用，请以本文档列出的 API 为准。

---

## API 子库文档索引

| 库名              | 作用            | 索引                                     | 包含      |
| --------------- | ------------- | -------------------------------------- | ------- |
| `lifecycle`     | 脚本生命周期回调      | [LIFECYCLE](./api/lifecycle.md)        | 方法      |
| `base`          | 提供 Lua 基础值操作  | [BASE](./api/base.md)                  | 方法      |
| `draw`          | 提供终端绘制指令      | [DRAW](./api/draw.md)                  | 方法      |
| `align`         | 辅助计算布局坐标      | [ALIGN](./api/align.md)                | 方法 / 常量 |
| `char`          | 常用的字符表        | [CHAR](./api/char.md)                  | 常量      |
| `color`         | 颜色控制          | [COLOR](./api/color.md)                | 方法 / 常量 |
| `debug`         | 用于脚本信息调试与错误捕获 | [DEBUG](./api/debug.md)                | 方法 / 常量 |
| `encoding`      | 提供基础的编码与解码转换  | [ENCODING](./api/encoding.md)          | 方法      |
| `event`         | 事件队列控制        | [EVENT](./api/event.md)                | 方法      |
| `file`          | 异步文件读写与目录枚举   | [FILE](./api/file.md)                  | 方法 / 常量 |
| `game`          | 游戏独立的生命周期控制   | [GAME](./api/game.md)                  | 方法      |
| `i18n`          | 国际化语言文本管理     | [I18N](./api/i18n.md)                  | 方法      |
| `loader`        | 加载 Lua 模块     | [LOADER](./api/loader.md)              | 方法      |
| `math`          | 数学运算          | [MATH](./api/math.md)                  | 方法 / 常量 |
| `measurement`   | 辅助文本字符尺寸测量    | [MEASUREMENT](./api/measurement.md)    | 方法      |
| `random`        | 随机数生成         | [RANDOM](./api/random.md)              | 方法 / 常量 |
| `serialization` | 多格式序列化与反序列化   | [SERIALIZTION](./api/serialization.md) | 方法      |
| `slice`         | 图层切片对象管理      | [SLICE](./api/slice.md)                | 方法 / 常量 |
| `string`        | 字符串处理         | [STRING](./api/string.md)              | 方法 / 常量 |
| `table`         | 表操作           | [TABLE](./api/table.md)                | 方法      |
| `utf8`          | UTF-8 字符串处理   | [UTF8](./api/utf8.md)                  | 方法      |

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
local is_equal = rawequal {left = 1, right = 2} -- 返回 false
```

### 扩展库 API

**库**：除 lifecycle 和 base 之外的所有库

通过对象方式调用

**示例**：
```lua
draw.text(1, 2, "Hello Tui Game") -- 在 base 切片坐标 (1, 2) 作为起始位置绘制字符串 "Hello Tui Game"
```