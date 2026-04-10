# 文档信息

1. 更新日期：2026年4月10日
2. 本文档旨在为模组开发者提供完整的规范化指引与教程，涵盖模组结构、最佳实践及常见问题。

# 文档导航

- [README](../../README-i18n/README-zh-cn.md)
- [API 规范与查询](./API.md)
- [富文本指令](./RICH_TEXT.md)

# 目录

---

# 模组放置目录

所有 MOD 文件必须放置在宿主执行目录下的 `tui-game-data/mod/` 目录中，按命名空间组织。

```text
宿主执行目录/
└─ tui-game-data/
    └─ mod/
        └─ <namespace>/    -- 命名空间
            └─ *           -- 该模组的所有文件
```

---

# 模组目录结构

一个合规的模组必须遵循以下目录结构，否则宿主将无法识别和加载该模组。

```text
<namespace>/               -- 模组命名空间/根目录
├─ package.json            -- 模组包信息（名称、作者、版本等）
├─ game.json               -- 模组游戏信息（配置、入口、权限等）
├─ scripts/                -- 脚本目录
│  ├─ main.lua             -- 脚本入口文件
│  └─ function/            -- 辅助脚本目录
│     └─ *.lua             -- 辅助脚本
└─ assets/                 -- 资源目录
   ├─ lang/                -- 语言资源目录
   │  ├─ en_us.json        -- 英语（美国）
   │  ├─ zh_cn.json        -- 简体中文
   │  └─ *.json            -- 其他语言文件
   └─ *                    -- 其他资源（图片、字体、音频等）
```

> 注：`package.json`、`game.json` 的具体字段含义请参考后续章节。

---

# 模组配置文件

## 目录结构

```text
<namespace>/               -- 模组命名空间/根目录
├─ package.json            -- 模组包信息（名称、作者、版本等）
└─ game.json               -- 模组游戏信息（配置、入口、权限等）
```

## 命名空间

- 模组根目录为 `<namespace>/`，`<namespace>` 即为该模组的命名空间。
- 命名空间在全局必须唯一，宿主将优先加载首个遇到的同名命名空间模组。
- 命名空间仅允许包含以下字符：小写字母 `a-z`、大写字母 `A-Z`、数字 `0-9`、下划线 `_`。

## `package.json`

> 注：
> - `key` 表示语言键，需配合语言文件使用。
> - `image` 表示图片路径，相对于 `assets/` 目录。

该文件用于声明模组的基本信息，格式如下：

```json
{
  "package": string,                -- 包名
  "introduction": string | key,     -- 包简介
  "author": string | key,           -- 作者信息
  "name": string | key,             -- 游戏显示名称
  "description": string | key,      -- 游戏简短描述
  "detail": string | key,           -- 游戏详细描述
  "icon": Array | string | image,   -- 图标
  "banner": Array | string | image  -- 横幅
}
```

**字段说明**

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `package` | <font color="#92cddc">string</font> | 包名，用于区分不同模组，全局唯一。仅允许字符串。 |
| `introduction` | <font color="#92cddc">string</font> \| <font color="#92cddc">key</font> | 包简介，在模组列表中展示，由开发者编写。可填写字符串或语言键。 |
| `author` | <font color="#92cddc">string</font> \| <font color="#92cddc">key</font> | 作者名称。可填写字符串或语言键。 |
| `name` | <font color="#92cddc">string</font> \| <font color="#92cddc">key</font> | 游戏显示名称，在游戏列表中展示。可填写字符串或语言键。 |
| `description` | <font color="#92cddc">string</font> \| <font color="#92cddc">key</font> | 游戏简短描述，建议一句话概括玩法或目标。可填写字符串或语言键。 |
| `detail` | <font color="#92cddc">string</font> \| <font color="#92cddc">key</font> | 游戏详细描述，建议包含：游戏目标、核心机制、操作方式、特殊警告（如与原版差异）等。可填写字符串或语言键。 |
| `icon` | <font color="#92cddc">Array</font> \| <font color="#92cddc">string</font> \| <font color="#92cddc">image</font> | 图标，在模组列表中展示。推荐使用 4 行 × 8 列的二维数组（字符）。若使用字符串，请用 `\n` 换行，否则宿主仅单行解析。若使用图片路径，建议宽高比 1:1，但宿主图片渲染效果通常不理想。若不填写会使用默认图标，见 附录-[默认图标](#默认图标) |
| `banner` | <font color="#92cddc">Array</font> \| <font color="#92cddc">string</font> \| <font color="#92cddc">image</font> | 横幅，在模组详情页展示。推荐使用 13 行 × 43 列的二维数组。若使用字符串，请用 `\n` 换行。若使用图片路径，建议宽高比 13:43，宿主图片渲染效果通常不理想。若不填写会使用默认头图，见 附录-[默认头图](#默认头图) |

## `game.json`

> 注：
> - `key` 表示语言键。
> - `path` 表示脚本路径，相对于 `scripts/` 目录。

该文件用于声明游戏的核心配置，格式如下：

```json
{
  "api": Array | int,                -- 支持的 API 版本范围
  "entry": path,                     -- 入口脚本路径
  "save": boolean,                   -- 是否支持存档
  "best_none": string | key | null,  -- 最佳记录占位文本（null 表示禁用）
  "min_width": int,                  -- 最小终端宽度（字符行数）
  "min_height": int,                 -- 最小终端高度（字符列数）
  "write": boolean,                  -- 是否请求直写权限
  "actions": object,                 -- 按键动作映射表
  "runtime": {
    "target_fps": int                -- 目标帧率
  }
}
```

**字段说明**

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `api` | <font color="#92cddc">Array</font> \| <font color="#92cddc">int</font> | 支持的 API 版本。数组格式 `[min, max]` 表示支持从 `min` 到 `max` 的版本（含端点）；整数表示仅支持该单一版本。若版本不符合宿主要求，模组将不被加载并抛出异常（详见 API 文档「通用异常」）。 |
| `entry` | <font color="#92cddc">path</font> | 入口脚本路径，相对于 `scripts/` 目录。若路径错误，模组将不被加载并抛出异常。 |
| `save` | <font color="#92cddc">boolean</font> | 是否支持存档。`true` 表示需要实现声明式 API `save_game(state)`；`false` 则忽略相关调用。 |
| `best_none` | <font color="#92cddc">string</font> \| <font color="#92cddc">key</font> \| <font color="#92cddc">null</font> | 无最佳记录时显示的文本。若不为 `null`，需实现声明式 API `save_best_score(state)`；若为 `null`，表示不启用最佳记录功能，相关调用被忽略。 |
| `min_width` | <font color="#92cddc">int</font> | 游戏所需的最小终端宽度（字符列数）。终端尺寸不足时会显示提示。 |
| `min_height` | <font color="#92cddc">int</font> | 游戏所需的最小终端高度（字符行数）。终端尺寸不足时会显示提示。 |
| `write` | <font color="#92cddc">boolean</font> | 是否请求直写权限。`true` 表示模组需要文件写入权限，加载时会向用户申请；`false` 表示不需要权限，所有直写请求将被宿主忽略。<font color="red">直写操作为高风险操作，请最大程度避免使用！</font> |
| `actions` | <font color="#92cddc">object</font> | 按键动作映射表，格式见下方「注册表格式」。宿主会将物理按键映射为语义化动作。 |
| `runtime` | <font color="#92cddc">object</font> | 运行时设置。 |
| `runtime.target_fps` | <font color="#92cddc">int</font> | 目标帧率，支持 `30`、`60`、`120`。其他值将被忽略并回退为 `60`。实际帧率受机器性能影响，该值为上限。 |

## 注册表格式

> 注：
> - `#` 表示自定义或可变内容。
> - `[]` 表示字段可重复或扩展。
> - `<>` 表示类型约束。
> - `key` 表示按键映射名，具体按键映射见 附录-[调试输出目录](#调试输出目录)。

```json
"actions": {
  [#action]: key | Array<key>
}
```

**示例**：

```json
"actions": {
  "jump": "space",
  "move": ["up", "down", "left", "right"]
}
```

> 每个动作可绑定单个按键或多个按键（数组形式）。宿主会将按键事件转换为动作事件，通过 `handle_event` 传递给脚本（事件类型 `action`）。

## UID

UID 是宿主为每个模组生成的唯一标识码，用于区分不同模组。

**构成格式**：`mod_game_{编码}`

**编码生成规则**：

1. 将模组的 `命名空间`、`包名（package）`、`作者（author）` 按顺序拼接成一个字符串。
2. 对该字符串进行哈希运算，然后使用 Base64 编码。
3. 取编码结果的前 16 位字符作为最终编码。

> 上述过程可用以下伪代码表示：
> ```
> encoding = base64(hash(namespace + package + author)).substring(0, 16)
> uid = "mod_game_" + encoding
> ```

**稳定性**：只要 `命名空间`、`包名`、`作者` 三者保持不变，生成的 UID 就不会改变。这确保了模组在不同环境中的一致性识别。

---

# 模组脚本规范

## 目录结构

```text
<namespace>/               -- 模组命名空间/根目录
└─ scripts/                -- 脚本目录（必须）
   ├─ main.lua             -- 脚本入口文件（必须）
   └─ function/            -- 辅助脚本目录（可选）
      └─ *.lua             -- 辅助脚本
```

## 规范要求

1. 所有脚本文件必须放置在 `scripts/` 目录下，且仅支持 `.lua` 扩展名。
2. 入口脚本建议直接放在 `scripts/` 目录下，默认文件名为 `main.lua`（由 `game.json` 中的 `entry` 字段指定，可自定义）。
3. 辅助脚本必须存放在 `scripts/function/` 目录下，用于组织可复用的模块化代码。

## 沙箱限制（禁用 API）

以下 Lua 内置 API 在脚本中**严格禁止使用**，宿主沙箱会阻止其执行：

- `os.execute`
- `os.remove`
- `os.rename`
- `os.exit`
- `io.*`（所有输入输出函数）
- `debug.*`（所有调试函数）

## 不建议使用的 API

为保证游戏性能和宿主稳定性，以下 API 不建议在脚本中使用，推荐使用宿主提供的替代方案：

| 不建议使用的 API | 推荐替代方案 |
| --- | --- |
| `require` | 使用直用式 API `load_function` 加载辅助 |
| `dofile` | 使用 `load_function` |
| `loadfile` | 使用 `load_function` |
| `while true do ... end`（死循环） | 依赖宿主每帧调用的声明式 API `handle_event` 实现循环逻辑 |
| `math.random` | 使用直用式 API `random_*` 系列函数（可复现、更安全） |
| `print` | 使用直用式 API `debug_*`（输出到日志文件） |

## 主脚本规范

主脚本（即 `game.json` 中 `entry` 字段指定的入口文件）必须满足以下要求：

1. **必须实现**以下四个声明式 API：
   - `init_game(state)`
   - `handle_event(state, event)`
   - `render(state)`
   - `exit_game(state)`

2. **至少存在一条可执行路径**能够调用直用式 API `request_exit()`，以确保游戏能够正常退出。

3. 其余游戏逻辑（如状态管理、事件响应、画面绘制、辅助函数调用等）由开发者自行编写，宿主不做额外限制。

## 辅助脚本规范

辅助脚本必须返回一个 Lua 表，表中可包含变量和函数。示例：

### 导出辅助函数和变量

`scripts/function/hello.lua`

```lua
local M = {}

M.name = "Function"

M.sayHello = function() -- 一种函数方式
    debug_log("Hello")
end

function M.sayAny(text) -- 另一种函数方式
    debug_log(text)
end

return M
```

### 在入口脚本中引用

`scripts/main.lua`

```lua
local hello = load_function("hello.lua")   -- 注意：路径相对于 function/ 目录

debug_log(hello.name)      -- 日志输出 "Function"
hello.sayHello()           -- 日志输出 "Hello"
hello.sayAny("tui game")   -- 日志输出 "tui game"
```

> 注：`load_function` 的参数为相对于 `scripts/function/` 的路径.

---

# 模组资源目录

## 目录结构

```text
<namespace>/               -- 模组命名空间/根目录
└─ assets/                 -- 资源目录
   ├─ lang/                -- 语言资源目录
   │  ├─ en_us.json        -- 英语（美国）
   │  ├─ zh_cn.json        -- 简体中文
   │  └─ *.json            -- 其他语言文件
   └─ *                    -- 其他资源（图片、字体、音频等）
```

## 语言文件

### 文件规范

- 所有语言文件必须存放在 `assets/lang/` 目录下。
- **`en_us.json` 必须提供**，作为默认回退语言。当宿主请求的语言模组未实现时，会自动使用 `en_us.json` 中的对应键值；若该键在 `en_us.json` 中也不存在，则返回 `[missing-i18n-key:key]`。
- **`zh_cn.json` 建议提供**（软规范）。由于仓库作者来自中文社区，提供简体中文支持有助于本地化体验，但非强制。
- 其他语言文件请按照 `{语言代码}.json` 的命名规则创建，确保宿主能够根据用户选择的语言正确加载。宿主支持的语言扩展详见 `LANGUAGE.md`。

### 键值规范

> 注：
> - `#` 表示自定义或可变内容。
> - `[]` 表示字段可重复或扩展。

语言文件采用键值对结构，键可使用点号 `.` 进行语义化分隔，值必须为字符串。字符串中可包含：
- **动态变量**：使用 `{变量名}` 占位符，运行时由脚本传入实际值。
- **富文本标记**：支持宿主定义的富文本格式（如颜色、样式等），具体语法参见宿主文档。

**结构示例**：

```json
{
  "[#key]": "string"
}
```

**完整示例**：

```json
{
  "game.title": "推箱子",
  "game.score": "当前得分：{score}",
  "game.hint": "<color=green>按 R 键重新开始</color>"
}
```

## 其他资源文件

### 支持的类型

| 类别 | 支持格式 | 说明 |
| --- | --- | --- |
| 文本文件 | `json`, `yaml`, `toml`, `csv`, `xml`, `txt` | 可通过 `read_*` 系列 API 读取并自动解析 |
| 二进制文件 | 任意格式 | 通过 `read_bytes` 读取为 Lua 字符串，需自行解析 |
| 图像文件 | `png`, `jpg`, `jpeg` | 用于 `icon`、`banner` 等字段，支持图片路径引用 |

> 注：其他资源文件可放置在 `assets/` 下的任意子目录中，使用 API 时需提供相对于 `assets/` 的路径。