# 游戏包 `package.json`

本文描述当前宿主实际支持的游戏包清单结构。`package.json` 必须是合法 JSON；下方示例使用 JSONC 注释辅助说明，实际文件中不要保留注释。

## 目录结构

```text
game_package/
├── package.json
├── scripts/
│   └── main.lua
└── assets/
    ├── icon.png
    ├── banner.txt
    └── language/
        ├── en_us/
        │   └── display.json
        └── zh_cn/
            └── display.json
```

官方游戏包位于 `scripts/game/<包目录>/`，模组游戏包位于 `data/mod/game/<包目录>/`。清单中的所有相对路径均以包目录为边界，不能使用绝对路径、`..`、反斜杠或符号链接越界。

## 完整结构

```jsonc
{
  "mod_id": "example.game",
  "schema_version": 1,
  "type": "game",

  "version": "1.0.0",
  "version_code": 1,

  "api": {
    "min": 1,
    "max": 1
  },

  "entry": "main",

  "display": {
    "title": "Example Game",
    "description": {
      "type": "i18n",
      "path": "display.json",
      "key": "description",
      "callback": "An example terminal game."
    },
    "author": {
      "type": "text",
      "text": "Author"
    },
    "icon": {
      "type": "image",
      "path": "icon.png"
    },
    "banner": {
      "type": "text",
      "path": "banner.txt"
    }
  },

  "runtime": {
    "min_width": 60,
    "min_height": 20
  },

  "game": {
    "name": "Example Game",
    "detail": "Game description shown in the game list.",

    "mouse": false,
    "truecolor": false,
    "target_fps": 60,
    "save": false,

    "language": ["en_us", "zh_cn"],

    "score": {
      "enabled": true,
      "empty_text": "No best score"
    },

    "actions": {
      "move_up": {
        "description": "Move Up",
        "keys": [
          ["w"],
          ["up"]
        ],
        "lock": false
      },
      "confirm": {
        "description": {
          "type": "i18n",
          "path": "display.json",
          "key": "action.confirm",
          "callback": "Confirm"
        },
        "keys": [
          ["enter"]
        ],
        "lock": true
      },
      "unbound_action": {
        "description": "Unbound Action",
        "keys": [],
        "lock": false
      }
    }
  }
}
```

## 顶层字段

| 字段 | 类型 | 必填 | 约束与含义 |
| --- | --- | --- | --- |
| `mod_id` | string | 是 | 包标识，1–128 字节；仅允许 ASCII 字母、数字、`.`、`_`、`-`。同一来源、同一包类型内必须唯一。 |
| `schema_version` | integer | 是 | 必须等于宿主当前清单版本，当前为 `1`。 |
| `type` | string | 是 | 游戏包固定为 `"game"`。 |
| `version` | package-text | 是 | 展示版本文本，解析后不可为空。 |
| `version_code` | integer | 是 | 必须大于 `0`。 |
| `api` | object | 是 | 宿主 API 兼容区间，包含 `min` 和 `max`。必须满足 `min <= max`，且当前宿主 API 位于该闭区间内。 |
| `entry` | string | 是 | 相对于 `scripts/` 的 Lua 入口。可省略 `.lua`；例如 `ui/main` 会解析为 `scripts/ui/main.lua`。 |
| `display` | object | 是 | 包的展示信息。 |
| `runtime` | object | 是 | 运行所需的最小 Base 画布尺寸。 |
| `game` | object | 是 | 游戏专属配置。 |
| `screensaver` | object | 否 | 游戏包不得提供该对象。 |

## 文本字段 `package-text`

`version`、`display.title`、`display.description`、`display.author`、`game.name`、`game.detail`、`game.score.empty_text` 和动作 `description` 均使用同一文本结构。

### 简写纯文本

```json
"title": "Example Game"
```

等价于：

```json
"title": {
  "type": "text",
  "text": "Example Game"
}
```

### i18n 文本

```json
"title": {
  "type": "i18n",
  "path": "display.json",
  "key": "title",
  "callback": "Example Game"
}
```

| 字段 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `type` | string | 是 | `"text"` 或 `"i18n"`。 |
| `text` | string | `type=text` 时是 | 直接使用的文本。 |
| `path` | string | `type=i18n` 时是 | 相对于 `assets/language/<language_code>/` 的安全 `.json` 路径。 |
| `key` | string | `type=i18n` 时是 | JSON 文件中的键，不可为空。 |
| `callback` | string | `type=i18n` 时是 | 当前语言和 `en_us` 均无有效值时使用的最终回退文本。 |

i18n 的查询顺序固定为：用户当前语言 → `en_us` → `callback`。这些展示文本可以包含宿主支持的富文本语法。

## `display`

| 字段 | 类型 | 必填 | 默认值/约束 |
| --- | --- | --- | --- |
| `title` | package-text | 是 | 包标题，解析后不可为空。 |
| `description` | package-text | 是 | 包简介。 |
| `author` | package-text | 是 | 作者信息。 |
| `icon` | display-asset | 否 | 未提供时使用宿主默认图标。 |
| `banner` | display-asset | 否 | 未提供时使用宿主默认横幅。 |

展示资源结构：

```json
{
  "type": "image",
  "path": "ui/icon.png"
}
```

- `type="image"` 支持 `.png`、`.jpg`、`.jpeg`。
- `type="text"` 仅支持 UTF-8 `.txt`。
- `path` 相对于包内 `assets/`，必须为安全的正向斜杠相对路径。
- 文本图标按 `8×4` 单元格规范化，文本横幅按 `60×14` 单元格规范化。
- 显式资源的类型、扩展名或路径不合法时，整个包会被拒绝扫描；只有省略字段才使用默认资源。

## `runtime`

| 字段 | 类型 | 必填 | 含义 |
| --- | --- | --- | --- |
| `min_width` | integer | 是 | 最小 Base 画布宽度；`0` 表示不限制。 |
| `min_height` | integer | 是 | 最小 Base 画布高度；`0` 表示不限制。 |

宿主依据 Base 画布尺寸判断是否满足要求；尺寸提醒中显示的是对应的终端实际尺寸。

## `game`

| 字段 | 类型 | 必填 | 默认值/约束 |
| --- | --- | --- | --- |
| `name` | package-text | 是 | 游戏名称，解析后不可为空。 |
| `detail` | package-text | 是 | 游戏详情。 |
| `mouse` | boolean | 否 | 默认 `false`；表示游戏需要鼠标，仅作为提示。 |
| `truecolor` | boolean | 否 | 默认 `false`；表示游戏需要真彩色，仅作为提示。 |
| `target_fps` | integer | 否 | 默认 `60`；只接受 `30`、`60`、`120`。 |
| `save` | boolean | 否 | 默认 `false`；是否启用宿主“继续游戏”单槽位存档。 |
| `language` | string[] | 是 | 支持的语言代码，仅作为提示，不限制游戏启动。代码会去除首尾空白、转为小写并去重。 |
| `score` | object | 否 | 最佳成绩配置。 |
| `actions` | object | 是 | 动作注册表；允许空对象。 |

语言代码不可为空，最长 64 字节，只允许 ASCII 字母、数字、`.`、`_`、`-`。

### `score`

| 字段 | 类型 | 必填 | 默认值/含义 |
| --- | --- | --- | --- |
| `enabled` | boolean | 否 | 默认 `false`。 |
| `empty_text` | package-text | 否 | 无最佳成绩时的文本；省略时使用宿主默认文本。 |

### `actions`

`actions` 的每个键是传递给 Lua `action` 事件的动作名：

```json
"actions": {
  "move_left": {
    "description": "Move Left",
    "keys": [["a"], ["left"]],
    "lock": false
  }
}
```

| 字段 | 类型 | 必填 | 默认值/约束 |
| --- | --- | --- | --- |
| `description` | package-text | 是 | 按键设置界面显示的动作名称。 |
| `keys` | string[][] | 是 | 默认绑定组合；允许 `[]`，表示该动作没有默认按键。 |
| `lock` | boolean | 否 | 默认 `false`。它属于具体动作，即 `game.actions.<动作名>.lock`；为 `true` 时用户不能修改该动作的按键。 |

绑定规则：

- 每个动作最多保留前两个绑定，依次显示为“键 1”和“键 2”。
- 每个绑定最多保留前两个键，用于组合键。
- 单个绑定不能是空数组；无绑定动作应直接使用 `"keys": []`。
- 键名必须能被宿主输入服务识别，并会被规范化；未知键名会导致该包被拒绝扫描。
- 启动游戏前宿主会再次验证默认映射与用户映射，非法外部数据只会阻止该包启动，不会导致宿主崩溃。

## 必填字段缺失时

缺少必填字段、字段类型错误、值超出约束、入口越界或类型专属配置不一致时，该包会被单独拒绝扫描并记录原因，不影响其他包和宿主继续运行。
