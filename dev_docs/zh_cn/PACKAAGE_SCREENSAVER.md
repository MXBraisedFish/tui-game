# 屏保包 schema 2

屏保包由包头、展示配置和屏保配置组成。每个 JSON 文件独立限于 1 MiB；未知字段、错误类型、缺失必需文件或越界路径都会使该包本次扫描失败。

## 目录结构

```text
screensaver_package/
├── package.json          # 身份、版本与 API 区间
├── display.json          # 标题、简介、作者与图标/横幅
├── screensaver.json      # 屏保运行配置与入口
├── scripts/main.lua
└── assets/language/
    ├── en_us/package.json
    └── zh_cn/package.json
```

官方包位于 `scripts/screensaver/<包目录>/`，模组包位于 `data/mod/screensaver/<包目录>/`。包相对路径不得使用绝对路径、`..`、反斜杠或越界符号链接。

## `package.json`

```json
{
  "package": "tui game",
  "mod_id": "example_screensaver",
  "schema_version": 2,
  "type": "screensaver",
  "version": "1.0.0",
  "version_code": 1,
  "api": { "min": 1, "max": 1 }
}
```

`package` 可省略；存在时必须是字符串 `"tui game"`，用于 IDE 插件识别 TUI GAME 开发包。`mod_id` 是包身份，长度为 1 到 128 字节，且只能包含 ASCII 字母、数字或下划线。`version_code` 必须大于 0；宿主 API 版本必须位于 `api.min..=api.max`。宿主只接受 `schema_version: 2`，不读取旧的单文件清单。

## `display.json`

```json
{
  "title": "Example Screensaver",
  "description": "An example terminal screensaver.",
  "author": "Author",
  "icon": { "type": "text", "path": "ui/icon.txt" },
  "banner": { "type": "image", "path": "ui/banner.png", "block": "half_block" }
}
```

`title`、`description`、`author` 必填。`icon` 和 `banner` 可省略，省略时使用宿主默认资源。图片支持 PNG/JPG/JPEG，`block` 可选 `half_block`（默认）或 `mix_block`；文本资源只能是 UTF-8 `.txt`，并按图标 `8×4`、横幅 `60×14` 单元格规范化。文本资源忽略 `block`。

文本值既可直接写字符串，也可写 `{ "type": "text", "text": "..." }`。i18n 文本格式为 `{ "type": "i18n", "key": "title", "callback": "Example Screensaver" }`；不接受 `path`。宿主依次从 `assets/language/<当前语言>/package.json`、`assets/language/en_us/package.json` 读取扁平的字符串键值对象；都未找到时使用 `callback`。语言包示例：`{ "title": "示例屏保" }`。

## `screensaver.json`

```json
{
  "name": "Example Screensaver",
  "entry": "main",
  "min_width": 60,
  "min_height": 20,
  "truecolor": false,
  "command": "example_screen"
}
```

`name`、`entry`、`command` 必填。`command` 本轮只解析并校验，CLI 启动命令留待后续任务。入口相对于 `scripts/`，可省略 `.lua`；尺寸省略时为 0（不限制）；`truecolor` 默认 `false`。屏保没有动作映射配置。

错误诊断会指出对应配置文件和字段。坏包本次从列表下架，修复后可重新出现；已运行会话按既有生命周期继续。
