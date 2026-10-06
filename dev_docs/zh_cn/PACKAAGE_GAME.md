# 游戏包 schema 2

游戏包由包头、展示配置、游戏配置和可选动作配置组成。每个 JSON 文件都独立限于 1 MiB；未知字段、错误类型、缺失必需文件或越界路径都会使该包本次扫描失败。

## 目录结构

```text
game_package/
├── package.json          # 身份、版本与 API 区间
├── display.json          # 标题、简介、作者与图标/横幅
├── game.json             # 游戏运行配置与入口
├── actions.json          # 可选；动作说明与默认按键
├── scripts/main.lua
└── assets/
    ├── ui/icon.png
    └── language/
        ├── en_us/package/  # package.json, display.json, game.json, actions.json
        └── zh_cn/package/  # package.json, display.json, game.json, actions.json
```

官方包位于 `scripts/game/<包目录>/`，模组包位于 `data/mod/game/<包目录>/`。包相对路径不得使用绝对路径、`..`、反斜杠或越界符号链接。

## `package.json`

```json
{
  "package": "tui game",
  "mod_id": "example_game",
  "schema_version": 2,
  "type": "game",
  "version": "1.0.0",
  "version_code": 1,
  "api": { "min": 1, "max": 1 }
}
```

`package` 可省略；存在时必须是字符串 `"tui game"`，用于 IDE 插件识别 TUI GAME 开发包。`mod_id` 是包身份，长度为 1 到 128 字节，且只能包含 ASCII 字母、数字或下划线。`version_code` 必须大于 0；宿主 API 版本必须位于 `api.min..=api.max`。宿主只接受 `schema_version: 2`，不读取旧的单文件清单。

## `display.json`

```json
{
  "title": "Example Game",
  "description": "An example terminal game.",
  "author": "Author",
  "icon": { "type": "image", "path": "ui/icon.png", "block": "mix_block" },
  "banner": { "type": "text", "path": "ui/banner.txt" }
}
```

`title`、`description`、`author` 必填。`icon` 和 `banner` 可省略，省略时使用宿主默认资源。图片支持 PNG/JPG/JPEG，`block` 可选 `half_block`（默认）或 `mix_block`；文本资源只能是 UTF-8 `.txt`，并按图标 `8×4`、横幅 `60×14` 单元格规范化。文本资源忽略 `block`。

文本值既可直接写字符串，也可写 `{ "type": "text", "text": "..." }`。i18n 文本格式为：

```json
{ "type": "i18n", "key": "title", "callback": "Example Game" }
```

不接受 `path`。每个清单文件读取自己的同名翻译文件：`display.json` 对应 `assets/language/<当前语言>/package/display.json`，其他文件同理。宿主先查当前语言，再查 `en_us` 下的同名文件，都未找到时使用 `callback`；不读取旧路径 `assets/language/<语言>/package.json`。翻译文件是扁平的字符串键值对象。点号是键名的一部分。语言包示例：`{ "title": "示例游戏" }`。

## `game.json`

```json
{
  "name": "Example Game",
  "detail": "Game details.",
  "command": "example_game",
  "entry": "main",
  "min_width": 60,
  "min_height": 20,
  "mouse": false,
  "truecolor": false,
  "target_fps": 60,
  "save_game": false,
  "language": ["en_us", "zh_cn"],
  "best_score": { "enable": true, "empty_text": "No best score" }
}
```

`name`、`detail`、`command`、`entry`、`language` 必填。`command` 本轮只解析并校验，CLI 启动命令留待后续任务。入口相对于 `scripts/`，可省略 `.lua`；尺寸省略时为 0（不限制）。`mouse`、`truecolor`、`save_game` 默认 `false`；`target_fps 可省略，省略时跟随玩家帧率设置；指定时只接受 30、60、120，并与玩家上限取较小值。玩家设为 Unlimited 时由包目标值限速。`best_score` 可省略；存在时 `enable` 必填，`empty_text` 可省略；`enable:false` 时宿主忽略 `empty_text`。

## `actions.json`

文件可省略，等价于 `{}`。示例：

```json
{
  "move_up": {
    "description": "Move up",
    "keys": [["w"], ["up"]],
    "lock": false,
    "priority": 10
  },
  "unbound": { "description": "Unbound action", "keys": [] }
}
```

每项必填 `description` 和 `keys`；`lock` 默认 `false`。`priority` 可选，省略为 `0`。

| 字段 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `description` | string 或 i18n 表 | 是 | — | 显示动作的用途。 |
| `keys` | array | 是 | — | 动作的默认绑定，可为空。 |
| `lock` | boolean | 否 | false | 是否禁止玩家改键。 |
| `priority` | 非负整数 | 否 | 0 | 数字越大，同一层内越先派发。 |

`priority` 范围为 `0`～`18446744073709551615`（u64）。负数、小数（包括 `1.0`）、字符串、boolean、null 和超范围数值均抛出错误，定位到对应动作的 `priority` 字段。动作名称不允许重复声明。

宿主动作先于游戏动作，各层内部按 priority 降序；同值时实际命中的两键组合先于单键；完全同级时按注册顺序，游戏以本文件从上到下的声明顺序为准。优先级只决定顺序，所有命中动作都会发送，不能用游戏 priority 越过宿主。一个动作的多个备选绑定同时有效只发送一次，使用当前有效绑定中排名最高的一项；最后一个备选绑定结束才释放。玩家改键不改变包声明的优先级或注册顺序，重启和热加载保持同样的规则。改键列表显示包优先级，按首个显示绑定预览组合键次序；运行时始终按当前实际命中的绑定排序，列表排序不改变注册次序。清单仍为 schema 2，旧包无需添加此字段，已有改键存档不迁移。

`keys: []` 表示无默认按键；每个动作最多两个绑定，每个绑定最多两个键。空组合、未知键、规范化后重复键、同一动作重复绑定都会抛出错误；不同动作可以共享相同绑定，改键界面仅提示共享，不阻止保存。非法包本次从列表下架，修复后可重新出现；已运行会话按既有生命周期继续。

## 文本字段与错误

`version`、展示文本、`game.name/detail`、`best_score.empty_text` 和动作 `description` 使用相同文本形式。错误诊断会指出对应配置文件和字段。包身份、profile 行为和跨平台数据处理见重构计划记录。

所有支持 i18n 的字段如下；不在表内的字段不接受 i18n 对象：

| 清单文件 | 字段 | 翻译文件（相对包目录） |
| --- | --- | --- |
| `package.json` | `version` | `assets/language/<语言>/package/package.json` |
| `display.json` | `title`、`description`、`author` | `assets/language/<语言>/package/display.json` |
| `game.json` | `name`、`detail`、`best_score.empty_text` | `assets/language/<语言>/package/game.json` |
| `actions.json` | `<动作名>.description` | `assets/language/<语言>/package/actions.json` |

`best_score.enable` 为 `false` 时忽略 `empty_text`，不加载其翻译。

热加载跟踪当前语言和 `en_us` 的翻译文件及其父目录。修改、删除、替换文件或补建、重建目录后都会重新扫描包；脚本使用的同级 `ui.json` 等语言文件保持原有位置。

## 最佳记录展示文本

`SaveBest` 返回的 `best_string` 及 `value` 中的替换参数同样支持字符串或包清单文本表。它们使用独立的 `assets/language/<语言代码>/package/best_string.json`，与 `game.json` 中的 `best_score.empty_text` 区分。完整返回结构与替换示例见[SaveBest 回调](CALLBACK.md#9-savebest)。
