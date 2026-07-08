# 富文本系统参考手册

## 解析入口

所有富文本解析始于 `src/host_engine/services/rich_text/parser.rs` 中的 `parse` 函数，通过 `RichTextService::parse(text, params)` 对外暴露。`DrawTextParams.text` 字段会被文本渲染管线自动解析——你只需传入带有正确前缀和标签的字符串即可。

---

## 1. `f%` 前缀

字符串**必须**以 `f%` 开头才能激活富文本解析。没有此前缀，整个字符串将被视为纯文本。

```rust
// 纯文本——不做解析
"Hello World"

// 富文本——标签和参数会被解析
"f%<fg:red>Hello</fg> World"

// 即使传入了 params，标签解析仍然需要 f% 前缀。
// 但：如果传入了 params 且无 f% 前缀，{param} 替换仍然生效。
```

---

## 2. 样式标签

标签用 `< >` 包裹，对其后所有文本生效，直到被覆盖或重置。

### 2.1 前景色：`<fg:颜色>`

```rust
"f%<fg:red>红色文字</fg> 恢复默认"
"f%<fg:bright_cyan>青色</fg>"
"f%<fg:#FF8800>橙色（十六进制）</fg>"
"f%<fg:rgb(85,87,83)>自定义灰色</fg>"
```

关闭：`</fg>` — 清除前景色，恢复终端默认。

### 2.2 背景色：`<bg:颜色>`

```rust
"f%<bg:blue>蓝色背景</bg>"
"f%<bg:rgb(30,30,30)>深色背景</bg>"
```

关闭：`</bg>` — 清除背景色。

### 2.3 文本样式

| 标签 | 简写 | 效果 |
|---|---|---|
| `<b>` | `<bold>` | **粗体** |
| `<i>` | `<italic>` | *斜体* |
| `<u>` | `<underline>` | 下划线 |
| `<s>` | `<strike>` | ~~删除线~~ |
| `<l>` | `<blink>` | 闪烁 |
| `<r>` | `<reverse>` | 反转前景/背景 |
| `<h>` | `<hidden>` | 隐藏（不可见） |
| `<d>` | `<dim>` | 暗淡 |

用 `/` 前缀关闭：

```rust
"f%<b>粗体</b> 非粗体"
"f%<u>下划线</u> 正常"
```

### 2.4 `<reset>`

重置**所有**样式（前景色、背景色、粗体、斜体等）为默认值。

```rust
"f%<fg:red><b>红粗<reset> 纯文本"
```

### 2.5 嵌套

标签可嵌套——内层覆盖外层：

```rust
"f%<fg:white>白色 <fg:red>红色</fg> 白色</fg>"
"f%<b>粗体 <i>粗斜体</i> 粗体</b>"
```

---

## 3. 颜色格式

`<fg:...>` 和 `<bg:...>` 中支持三种格式：

### 3.1 命名终端色

```
black, red, green, yellow, blue, magenta, cyan, white
bright_black, bright_red, bright_green, bright_yellow,
bright_blue, bright_magenta, bright_cyan, bright_white
```

中文对照：黑、红、绿、黄、蓝、品红、青、白，以及对应的亮色变体。

它们映射到 ANSI 16 色调色板的 `TerminalColor` 枚举。

### 3.2 十六进制：`#RRGGBB`

```rust
"f%<fg:#FF6600>橙色文字</fg>"
```

解析为 `TextColor::Rgb { r, g, b }`。在支持真彩的终端上直接渲染，在不支持的终端上抖动到 256 色。

### 3.3 函数式：`rgb(r, g, b)`

```rust
"f%<fg:rgb(85,87,83)>自定义灰色</fg>"
```

与十六进制等效，但使用 `rgb()` 语法。空格会自动 trim。

### 3.4 `TextColor::ForceRgb`

在代码中构造 `TextColor` 时，使用 `ForceRgb` 可绕过 256 色抖动，始终输出 24 位转义码：

```rust
TextColor::ForceRgb { r: 85, g: 87, b: 83 }
```

---

## 4. 参数替换：`{...}`

参数从 `RichTextParams` 中解析，需要 `f%` 前缀（或 `params` 为 `Some`）。

### 4.1 值参数：`{value:名称}` 或 `{名称}`

```rust
let mut values = HashMap::new();
values.insert("type".to_string(), "cache".to_string());
let params = RichTextParams { values, key_actions: HashMap::new() };

"f%正在导出 {value:type} 数据"  // → "正在导出 cache 数据"
"f%正在导出 {type} 数据"        // → 同上（裸名称默认 = value）
```

### 4.2 按键显示参数：`{key:动作名}`

解析为人类可读的按键绑定显示——如 `{key:confirm}` → `"[Enter]"`。

`key_actions` 映射为 动作名 → 按键模式（来自 action map）：

```rust
let params = RichTextParams::from_action_map(&action_map_entries, "export_settings.");
// 同时注册 "export_settings.confirm" 和 "confirm"（去除前缀）作为键。
// "{key:confirm}" → "[Enter]"
// "{key:export_settings.back}" → "[Esc]"
```

按键模式使用 `format_key_display()` 生成：
- `[W]` — 单键
- `[Ctrl + S]` — 组合键
- `[W]/[↑]` — 多方案

### 4.3 缺失参数

如果 `{value:...}` 或 `{key:...}` 无法解析，原始文本会原样保留在输出中：

```rust
"{key:unknown_action}"  // 保持为原文
"{value:missing}"       // 保持为原文
```

### 4.4 参数中的转义

用 `\` 转义 `{`、`}`、`<`、`>`、`\`：

```rust
"f%集合为 \\{x, y, z\\}"
```

---

## 5. `RichTextParams`

```rust
pub struct RichTextParams {
    pub values: HashMap<String, String>,              // 值映射
    pub key_actions: HashMap<String, Vec<Vec<String>>>, // 按键动作映射
}
```

### 5.1 `from_action_map()`

推荐用于 UI 代码创建参数的工厂方法：

```rust
fn action_map() -> Vec<ActionMapEntry> {
    vec![
        ActionMapEntry {
            action: "my_ui.confirm".to_string(),
            description: "确认".to_string(),
            keys: vec![vec!["enter".to_string()]],
        },
        ActionMapEntry {
            action: "my_ui.back".to_string(),
            description: "返回".to_string(),
            keys: vec![vec!["esc".to_string()]],
        },
    ]
}

let params = RichTextParams::from_action_map(&Self::action_map(), "my_ui.");
// 现在可以使用 {key:confirm} 或 {key:my_ui.confirm}
```

### 5.2 直接构造

```rust
let mut values = HashMap::new();
values.insert("name".to_string(), "Alice".to_string());

let mut key_actions = HashMap::new();
key_actions.insert("jump".to_string(), vec![vec!["j".to_string()]]);

let params = RichTextParams { values, key_actions };
```

---

## 6. `RichTextService`

两个主要方法：

```rust
let rt = RichTextService::new();

// 解析为结构化分段
let rich: RichText = rt.parse("f%<fg:red>你好</fg>", Some(&params));
// rich.segments[0] = RichTextSegment { text: "你好", style: TextStyle { fg: Red, ... } }

// 仅提取纯文本（去除所有标签）
let plain: String = rt.visible_text("f%<fg:red>你好</fg>", Some(&params));
// plain = "你好"
```

`visible_text()` 对纯文本（无 `f%` 前缀、无 params）会**短路返回**——直接返回原始字符串不做解析。

---

## 7. 与 `DrawTextParams` 集成

渲染管线自动从 `DrawTextParams.text` 解析富文本：

```rust
render.draw_host_text(
    canvas,
    &DrawTextParams {
        x: 0,
        y: 0,
        text: format!("f%<fg:bright_magenta><b>{}</b></fg>", title),
        params: Some(params.clone()),
        ..Default::default()
    },
);
```

如果你**不需要**富文本解析，省略 `f%` 前缀并传 `params: None`：

```rust
render.draw_host_text(
    canvas,
    &DrawTextParams {
        text: "不解析的纯文本".to_string(),
        ..Default::default()
    },
);
```

---

## 8. 常见模式

### 8.1 带颜色+粗体的标题

```rust
format!("f%<fg:bright_yellow><b>{}</b></fg>", title)
```

### 8.2 灰色提示文字（自定义 RGB）

```rust
format!("f%<fg:rgb(85,87,83)>{}</fg>", hint)
```

### 8.3 红色错误信息

```rust
format!("f%<fg:bright_red>{}</fg>", error_text)
```

### 8.4 hint 中显示按键绑定

```rust
let params = RichTextParams::from_action_map(&Self::action_map(), "my_ui.");
let hint = format!(
    "f%{key:confirm} 确认  {key:back} 返回",
);
// 渲染为："[Enter] 确认  [Esc] 返回"
```

### 8.5 值参数 + 按键参数混合

```rust
let mut params = RichTextParams::from_action_map(&Self::action_map(), "my_ui.");
params.values.insert("target".to_string(), "缓存".to_string());

"f%清除 {target}：按 {key:confirm}"
// → "清除 缓存：按 [Enter]"
```

### 8.6 灰色占位符文本

```rust
let placeholder = format!("<fg:rgb(85,87,83)>{}</fg>", default_text);
// 注意：没有 f% 前缀——此片段嵌入在外部 f% 字符串中
format!("f%{} {}", indicator, placeholder)
```

---

## 9. 输出数据类型

### `RichText` — 解析结果

```rust
pub struct RichText {
    pub segments: Vec<RichTextSegment>,
}
```

### `RichTextSegment` — 样式段

```rust
pub struct RichTextSegment {
    pub text: String,        // 可见文本内容
    pub style: TextStyle,    // 该段的活动样式
}
```

### `TextStyle` — 文本样式

```rust
pub struct TextStyle {
    pub foreground: Option<TextColor>,  // 前景色
    pub background: Option<TextColor>,  // 背景色
    pub bold: bool,                     // 粗体
    pub italic: bool,                   // 斜体
    pub underline: bool,                // 下划线
    pub strike: bool,                   // 删除线
    pub blink: bool,                    // 闪烁
    pub reverse: bool,                  // 反转
    pub hidden: bool,                   // 隐藏
    pub dim: bool,                      // 暗淡
}
```

### `TextColor` — 颜色枚举

```rust
pub enum TextColor {
    Terminal(TerminalColor),            // ANSI 16 色
    Rgb { r: u8, g: u8, b: u8 },       // 真彩（可能抖动到 256 色）
    ForceRgb { r: u8, g: u8, b: u8 },  // 强制真彩（不抖动）
    Transparent,                         // 透明
}
```

---

## 10. 关键规则

1. **`f%` 前缀必不可少**——没有它，`<标签>` 会被当作普通文字渲染。
2. **每个文本字符串只有一个 `f%`**——放在最外层 `format!()` 调用上。嵌入的片段**不应**带有自己的 `f%`。
3. **未闭合标签**（如 `<b` 没有 `>`）会被当作普通文字渲染——平和降级，不会 panic。
4. **未知颜色名**在 `<fg:X>` / `<bg:X>` 中会导致整个标签被当作普通文字渲染。
5. **缺失参数**`{name}` 会在输出中保留原文 `{name}`——安全回退。
6. **转义规则**：`\<` → `<`，`\{` → `{`，`\}` → `}`，`\>` → `>`，`\\` → `\`。
7. **嵌套规则**：内层标签对相同属性（如前/背景色）覆盖外层。
