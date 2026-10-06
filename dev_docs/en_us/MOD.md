# 文档信息

1. 更新日期：2026年4月10日
2. 本文档旨在为模组开发者提供完整的规范化指引与教程，涵盖模组结构、最佳实践及常见问题。

# 文档导航

- [README](../../README-i18n/README-zh-cn.md)
- [API 规范与查询](./API.md)
- [富文本指令](./RICH_TEXT.md)

# 目录

- [模组放置目录](#模组放置目录模组放置目录)
- [模组目录结构](#模组目录结构)
- [模组配置文件](#模组配置文件)
  - [目录结构](#目录结构1)
  - [命名空间](#命名空间)
  - [package.json](#packagejson)
  - [game.json](#packagejson)
  - [注册表格式](#注册表格式)
  - [UID](#uid)
- [模组脚本规范](#模组脚本规范)
  - [目录结构](#目录结构2)
  - [规范要求](#规范要求)
  - [沙箱限制](#沙箱限制禁用-api)
  - [不建议使用的 API](#不建议使用的-api)
  - [入口脚本规范](#入口脚本规范)
  - [辅助脚本规范](#辅助脚本规范)
- [模组资源目录](#模组资源目录)
  - [目录结构](#目录结构3)
  - [语言文件](#语言文件)
  - [其它资源文件](#其它资源文件)
- [附录](#附录)
  - [物理按键语义映射表](#物理按键语义映射表)

---

# 模组放置目录

所有 MOD 文件必须放置在宿主执行目录下的 `data/mod/` 目录中，按命名空间组织。

```text
宿主执行目录/
└─ data/
    └─ mod/
        └─ <namespace>/    -- 命名空间
            └─ *           -- 该模组的所有文件
```

---

# 模组目录结构

一个合规的模组必须遵循以下目录结构，否则宿主将无法识别和加载该模组。

```text
<package>/
├─ package.json
├─ display.json
├─ game.json or screensaver.json
├─ actions.json             # optional for games
├─ scripts/main.lua
└─ assets/language/<code>/package/<配置文件名>.json
```

> 注：`package.json`、`game.json` 的具体字段含义请参考后续章节。

---

# 模组配置文件

The host uses the schema 2 split-file package format. The old single-file examples below have been retired. See the current [game package schema](../zh_cn/PACKAAGE_GAME.md) and [screensaver package schema](../zh_cn/PACKAAGE_SCREENSAVER.md).

```text
<package>/
├─ package.json
├─ display.json
├─ game.json or screensaver.json
├─ actions.json             # optional for games
├─ scripts/main.lua
└─ assets/language/<code>/package/<配置文件名>.json
```

# 模组脚本规范

## 目录结构<font style="opacity:0;">2</font>

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

## 入口脚本规范

入口脚本（即 `game.json` 中 `entry` 字段指定的入口文件）必须满足以下要求：

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

## 目录结构<font style="opacity:0;">3</font>

```text
<namespace>/               -- 模组命名空间/根目录
└─ assets/                 -- 资源目录
   ├─ lang/                -- 语言资源目录
   │  ├─ en_us.json        -- 英语（美国）
   │  ├─ zh_cn.json        -- 简体中文
   │  └─ *.json            -- 其它语言文件
   └─ *                    -- 其它资源（图片、字体、音频等）
```

## 语言文件

### 文件规范

- 所有语言文件必须存放在 `assets/lang/` 目录下。
- **`en_us.json` 必须提供**，作为默认回退语言。当宿主请求的语言模组未实现时，会自动使用 `en_us.json` 中的对应键值；若该键在 `en_us.json` 中也不存在，则返回 `[missing-i18n-key:key]`。
- **`zh_cn.json` 建议提供**（软规范）。由于仓库作者来自中文社区，提供简体中文支持有助于本地化体验，但非强制。
- 其它语言文件请按照 `{语言代码}.json` 的命名规则创建，确保宿主能够根据用户选择的语言正确加载。宿主支持的语言扩展详见 `LANGUAGE.md`。

### 键值规范

> 注：
> - `#` 表示自定义或可变内容。
> - `[]` 表示字段可重复或扩展。

语言文件采用键值对结构，键可使用点号 `.` 进行语义化分隔，值必须为字符串。字符串中可包含：
- **动态变量**：使用 `{变量名}` 占位符，运行时由脚本传入实际值。
- **富文本标记**：支持宿主定义的富文本格式（如颜色、样式等），具体语法参见`RICH_TEXT.md`。

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

## 其它资源文件

### 支持的类型

| 类别 | 支持格式 | 说明 |
| --- | --- | --- |
| 文本文件 | `json`, `yaml`, `toml`, `csv`, `xml`, `txt` | 可通过 `read_*` 系列 API 读取并自动解析 |
| 二进制文件 | 任意格式 | 通过 `read_bytes` 读取为 Lua 字符串，需自行解析 |
| 图像文件 | `png`, `jpg`, `jpeg` | 用于 `icon`、`banner` 等字段，支持图片路径引用 |

> 注：其它资源文件可放置在 `assets/` 下的任意子目录中，使用 API 时需提供相对于 `assets/` 的路径。

---

# 其它

## 图标与头图

### 图标

图标用于在模组列表中展示，显示区域为 **4 行 × 8 列**（终端字符）。

**支持参数类型**：数组 / 字符串 / 图片

#### 数组

- 传递一个二维数组，最多包含 4 个子数组，每个子数组最多包含 8 个元素。
- **行数处理**：
  - 若子数组不足 4 行，宿主会在上下交替补充空行补齐至 4 行（**先上后下**）。
  - 若子数组超过 4 行，仅保留前 4 行。
- **列数处理**：
  - 若子数组内元素不足 8 个，宿主会在左右交替补充空格补齐至 8 个元素（**先右后左**）。
  - 若子数组内元素超过 8 个，仅保留前 8 个。
- 完成上述填充后，已填写的图标元素会被**居中显示**。
- **推荐写法**：将所有元素左对齐，剩余对齐与填充工作交由宿主完成。

#### 字符串

- 传递一个单行字符串，使用 `\n` 表示换行。
- 宿主会根据 `\n` 将字符串拆分为二维数组，后续处理规则与数组一致。
- **不推荐使用**：可读性极差，有时会被误识别为图片路径。

#### 图片

- 填写相对于 `assets/` 目录的路径。
- 建议图片比例为 **1:1**。
- 宿主会根据图片比例生成一个 1:1 的比例框进行截取，然后将图片符号化并染色。
- **不推荐使用**：生成效果通常严重偏离预期，仅作为功能扩展保留。

#### 默认值

- 若该字段不填写或传递空数组，将使用默认图标，见附录 [默认图标](#默认图标)。

---

### 头图

头图用于在模组详情的详细信息展示，显示区域为 **13 行 × 86 列**（终端字符）。

**支持参数类型**：数组 / 字符串 / 图片

#### 数组

- 传递一个二维数组，最多包含 13 个子数组，每个子数组最多包含 86 个元素。
- **行数处理**：
  - 若子数组不足 13 行，宿主会在上下交替补充空行补齐至 13 行（**先上后下**）。
  - 若子数组超过 13 行，仅保留前 13 行。
- **列数处理**：
  - 若子数组内元素不足 86 个，宿主会在左右交替补充空格补齐至 86 个元素（**先左后右**）。
  - 若子数组内元素超过 86 个，仅保留前 86 个。
- 完成上述填充后，已填写的头图元素会被**居中显示**。
- **推荐写法**：将所有元素左对齐，剩余对齐与填充工作交由宿主完成。

#### 字符串

- 传递一个单行字符串，使用 `\n` 表示换行。
- 宿主会根据 `\n` 将字符串拆分为二维数组，后续处理规则与数组一致。
- **不推荐使用**：可读性极差，有时会被误识别为图片路径。

#### 图片

- 填写相对于 `assets/` 目录的路径。
- 建议图片比例为 **13:43**。
- 宿主会生成一个最大可被 13×43 整除的比例框进行截取，然后将图片符号化并染色。
- **不推荐使用**：生成效果通常严重偏离预期，仅作为功能扩展保留。

#### 默认值

- 若该字段不填写或传递空数组，将使用默认头图，见附录 [默认头图](#默认头图)。

---

## 绘制坐标

绘制原点位于终端的**左上角**。坐标系定义如下：

- **X 轴**：水平向右为正方向
- **Y 轴**：垂直向下为正方向

示意图如下：

![绘制坐标](./image/axis.png)

---

## `introduction` / `description` / `detail` 展示位置

各字段在界面中的展示位置如下：

- **`introduction`**：展示于**模组列表**的详细信息区域，用于描述整个模组包的概要信息。
- **`description`**：展示于**游戏列表**的详细信息区域，用于说明游戏的玩法或基本规则。
- **`detail`**：展示于**游戏列表**的详细信息区域，用于提供游戏的详细说明。

示意图如下：

![introduction 展示位置](./image/introduction)  
![description 与 detail 展示位置](./image/description_detail)

---

# 附录

## 物理按键语义映射表

> 键盘监听基于 `crossterm` 与 `rdev` 两库联合实现，尽可能覆盖绝大多数按键的检测。为确保兼容性，建议优先使用显式声明的键位，避免因特殊键无法匹配而导致输入失效。
>
> 当前版本**不支持组合键**（如 `Ctrl+C`、`Shift+A` 等）。所有与 `Shift` 键组合的输入，其语义仍会被解析为对应的单键（例如 `Shift + A` 映射为 `A`）。

### 字母键（小写）
| 物理按键 | 返回值 |
|----------|--------|
| `A` ~ `Z` | `a` ~ `z` |

### 字母键（大写）
| 物理按键 | 返回值 |
|----------|--------|
| `Shift` + `A` ~ `Z` | `A` ~ `Z` |

### 数字键（主键盘）
| 物理按键 | 返回值 |
|----------|--------|
| `0` ~ `9` | `0` ~ `9` |

### 数字键（Shift 组合 / 符号上档）
| 物理按键 | 返回值 |
|----------|--------|
| `Shift` + `1` | `!` |
| `Shift` + `2` | `@` |
| `Shift` + `3` | `#` |
| `Shift` + `4` | `$` |
| `Shift` + `5` | `%` |
| `Shift` + `6` | `^` |
| `Shift` + `7` | `&` |
| `Shift` + `8` | `*` |
| `Shift` + `9` | `(` |
| `Shift` + `0` | `)` |

### 符号键（无 Shift）
| 物理按键 | 返回值 |
|----------|--------|
| ``` ` ``` | ``` ` ``` |
| `-` | `-` |
| `=` | `=` |
| `[` | `[` |
| `]` | `]` |
| `\` | `\` |
| `;` | `;` |
| `'` | `'` |
| `,` | `,` |
| `.` | `.` |
| `/` | `/` |

### 符号键（Shift 组合）
| 物理按键 | 返回值 |
|----------|--------|
| `Shift` + ``` ` ``` | `~` |
| `Shift` + `-` | `_` |
| `Shift` + `=` | `+` |
| `Shift` + `[` | `{` |
| `Shift` + `]` | `}` |
| `Shift` + `\` | `\`| |
| `Shift` + `;` | `:` |
| `Shift` + `'` | `"` |
| `Shift` + `,` | `<` |
| `Shift` + `.` | `>` |
| `Shift` + `/` | `?` |

### 功能键（F1 ~ F12）
| 物理按键 | 返回值 |
|----------|--------|
| `F1` ~ `F12` | `f1` ~ `f12` |

### 导航键
| 物理按键 | 返回值 |
|----------|--------|
| `↑` | `up` |
| `↓` | `down` |
| `←` | `left` |
| `→` | `right` |
| `Home` | `home` |
| `End` | `end` |
| `PageUp` | `pageup` |
| `PageDown` | `pagedown` |

### 编辑键
| 物理按键 | 返回值 |
|----------|--------|
| `Enter` | `enter` |
| `Backspace` | `backspace` |
| `Delete` | `del` |
| `Insert` | `ins` |
| `Tab` | `tab` |
| `Shift` + `Tab` | `back_tab` |
| `Space` | `space` |

### 修饰键
| 物理按键 | 返回值 |
|----------|--------|
| `左 Ctrl` | `left_ctrl` |
| `右 Ctrl` | `right_ctrl` |
| `左 Shift` | `left_shift` |
| `右 Shift` | `right_shift` |
| `左 Alt` | `left_alt` |
| `右 Alt` | `right_alt` |
| `左 Meta` (Win / Cmd) | `left_meta` |
| `右 Meta` (Win / Cmd)| `right_meta` |

### 锁定键
| 物理按键 | 返回值 |
|----------|--------|
| `CapsLock` | `capslock` |
| `NumLock` | `numlock` |
| `ScrollLock` | `scrolllock` |

### 系统功能键
| 物理按键 | 返回值 |
|----------|--------|
| `Esc` | `esc` |
| `PrintScreen` | `printscreen` |
| `Pause` | `pause` |
| `Menu` | `menu` |

### 小键盘
| 物理按键 | 返回值 |
|----------|--------|
| 小键盘 `0` ~ `9` | `0` ~ `9` |
| 小键盘 `+` | `+` |
| 小键盘 `-` | `-` |
| 小键盘 `*` | `*` |
| 小键盘 `/` | `/` |
| 小键盘 `Del` | `del` |
| 小键盘 `Enter` | `enter` |

### 未知键
| 物理按键 | 返回值 |
|----------|--------|
| 无法识别的按键 | `key(扫描码)` |

---

## 默认图标

**代码**

```json
[
  "████████", 
  "██ ██ ██",
  "   ██   ",
  "  ████  "
]
```

**样图**
![默认图标](./image/mod_icon.png)

## 默认头图

**代码**

```json
[
  "`7MMM.     ,MMF' .g8\"\"8q. `7MM\"\"\"Yb.   ",
  "  MMMb    dPMM .dP'    `YM. MM    `Yb. ",
  "  M YM   ,M MM dM'      `MM MM     `Mb ",
  "  M  Mb  M' MM MM        MM MM      MM ",
  "  M  YM.P'  MM MM.      ,MP MM     ,MP ",
  "  M  `YM'   MM `Mb.    ,dP' MM    ,dP' ",
  ".JML. `'  .JMML. `\"bmmd\"' .JMMmmmdP'   ",
]
```

**样图**
![默认头图](./image/mod_banner.png)

---

# 模组最小示例

> 该部分是纯文本展示，可查看仓库的 examples/ 目录了解详细代码

## 结构
```text
宿主执行目录/data/mod/
└─ example/
   ├─ package.json
   ├─ game.json
   ├─ scripts/
   │  ├─ main.lua
   │  └─ function/
   │     └─ helper.lua
   └─ assets/
      ├─ lang/
      │  ├─ en_us.json
      │  └─ zh_cn.json
      └─ json/
         └─ word.json
```

## 文件

### `package.json`

```json
{
  "package": "tui game",
  "introduction": "example.introduction",
  "author": "TUI GAME",
  "name": "example.name",
  "description": "example.description",
  "detail": "example.detail",
  "icon": [],
  "banner": []
}
```
