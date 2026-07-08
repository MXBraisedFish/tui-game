# Rich Text System Reference

## Parser Entry Point

All rich text parsing begins with the `parse` function in `src/host_engine/services/rich_text/parser.rs`. It is exposed via `RichTextService::parse(text, params)`. The `DrawTextParams.text` field is automatically parsed by the text rendering pipeline — you just pass a string with the right prefix and tags.

---

## 1. The `f%` Prefix

The string MUST start with `f%` to activate rich text parsing. Without it, the entire string is treated as plain text.

```rust
// Plain text — no parsing
"Hello World"

// Rich text — parsed for tags and parameters
"f%<fg:red>Hello</fg> World"

// EVEN IF params is Some, f% is still required for tag parsing.
// However: if params is Some and no f% prefix, {param} substitution STILL works.
```

---

## 2. Style Tags

Tags are enclosed in `< >`. They apply styles to all text after them until overridden or reset.

### 2.1 Foreground Color: `<fg:COLOR>`

```rust
"f%<fg:red>Red text</fg> back to default"
"f%<fg:bright_cyan>Cyan</fg>"
"f%<fg:#FF8800>Orange via hex</fg>"
"f%<fg:rgb(85,87,83)>Custom gray</fg>"
```

Turn off: `</fg>` — clears foreground, reverts to terminal default.

### 2.2 Background Color: `<bg:COLOR>`

```rust
"f%<bg:blue>Blue background</bg>"
"f%<bg:rgb(30,30,30)>Dark bg</bg>"
```

Turn off: `</bg>` — clears background.

### 2.3 Text Styles

| Tag | Short | Effect |
|---|---|---|
| `<b>` | `<bold>` | **Bold** |
| `<i>` | `<italic>` | *Italic* |
| `<u>` | `<underline>` | Underline |
| `<s>` | `<strike>` | ~~Strikethrough~~ |
| `<l>` | `<blink>` | Blink |
| `<r>` | `<reverse>` | Swap fg/bg |
| `<h>` | `<hidden>` | Invisible |
| `<d>` | `<dim>` | Dimmed |

Turn off with `/` prefix:
```rust
"f%<b>bold</b> not bold"
"f%<u>underlined</u> normal"
```

### 2.4 `<reset>`

Resets ALL styles (fg, bg, bold, italic, etc.) to default.

```rust
"f%<fg:red><b>Red bold<reset> plain"
```

### 2.5 Nesting

Tags can be nested — inner tags override outer:

```rust
"f%<fg:white>white <fg:red>red</fg> white again</fg>"
"f%<b>bold <i>bold-italic</i> bold</b>"
```

---

## 3. Color Formats

Three formats accepted in `<fg:...>` and `<bg:...>`:

### 3.1 Named Terminal Colors

```
black, red, green, yellow, blue, magenta, cyan, white
bright_black, bright_red, bright_green, bright_yellow,
bright_blue, bright_magenta, bright_cyan, bright_white
```

These map to ANSI 16-color palette via `TerminalColor` enum.

### 3.2 Hex: `#RRGGBB`

```rust
"f%<fg:#FF6600>Orange text</fg>"
```

Parsed as `TextColor::Rgb { r, g, b }`. Rendered as truecolor on supporting terminals, or dithered to 256-color on others.

### 3.3 Functional: `rgb(r, g, b)`

```rust
"f%<fg:rgb(85,87,83)>Custom gray</fg>"
```

Same as hex but in `rgb()` syntax. Spaces are trimmed.

### 3.4 `TextColor::ForceRgb`

When constructing `TextColor` programmatically, use `ForceRgb` to bypass the 256-color dithering and always output 24-bit escape codes:

```rust
TextColor::ForceRgb { r: 85, g: 87, b: 83 }
```

---

## 4. Parameter Substitution: `{...}`

Parameters are resolved from `RichTextParams`. The `f%` prefix is required for this (or `params` must be `Some`).

### 4.1 Value Parameters: `{value:NAME}` or `{NAME}`

```rust
let mut values = HashMap::new();
values.insert("type".to_string(), "cache".to_string());
let params = RichTextParams { values, key_actions: HashMap::new() };

// With params:
"f%Exporting {value:type} data"  // → "Exporting cache data"
"f%Exporting {type} data"        // → same (bare = value by default)
```

### 4.2 Key Display Parameters: `{key:ACTION}`

Resolves to human-readable key binding display — e.g., `{key:confirm}` → `"[Enter]"`.

The `key_actions` map has action names → key patterns (from action map):

```rust
let params = RichTextParams::from_action_map(&action_map_entries, "export_settings.");
// This registers both "export_settings.confirm" AND "confirm" (prefix stripped) as keys.
// "{key:confirm}" → "[Enter]"
// "{key:export_settings.back}" → "[Esc]"
```

Key patterns use `format_key_display()` which produces strings like:
- `[W]` — single key
- `[Ctrl + S]` — combo
- `[W]/[↑]` — alternatives

### 4.3 Missing Parameters

If a `{value:...}` or `{key:...}` cannot be resolved, the raw text is kept verbatim in the output:

```rust
"{key:unknown_action}"  // stays as literal text
"{value:missing}"       // stays as literal text
```

### 4.4 Escaping Inside Parameters

Use `\` to escape `{`, `}`, `<`, `>`, `\`:

```rust
"f%The set is \\{x, y, z\\}"
```

---

## 5. `RichTextParams`

```rust
pub struct RichTextParams {
    pub values: HashMap<String, String>,
    pub key_actions: HashMap<String, Vec<Vec<String>>>,
}
```

### 5.1 `from_action_map()`

The recommended way to create params for UI code:

```rust
fn action_map() -> Vec<ActionMapEntry> {
    vec![
        ActionMapEntry {
            action: "my_ui.confirm".to_string(),
            description: "Confirm".to_string(),
            keys: vec![vec!["enter".to_string()]],
        },
        ActionMapEntry {
            action: "my_ui.back".to_string(),
            description: "Back".to_string(),
            keys: vec![vec!["esc".to_string()]],
        },
    ]
}

let params = RichTextParams::from_action_map(&Self::action_map(), "my_ui.");
// Now you can use {key:confirm} or {key:my_ui.confirm}
```

### 5.2 Direct Construction

```rust
let mut values = HashMap::new();
values.insert("name".to_string(), "Alice".to_string());

let mut key_actions = HashMap::new();
key_actions.insert("jump".to_string(), vec![vec!["j".to_string()]]);

let params = RichTextParams { values, key_actions };
```

---

## 6. `RichTextService`

Two main methods:

```rust
let rt = RichTextService::new();

// Parse to structured segments
let rich: RichText = rt.parse("f%<fg:red>hi</fg>", Some(&params));
// rich.segments[0] = RichTextSegment { text: "hi", style: TextStyle { fg: Red, ... } }

// Extract plain text only (strips all tags)
let plain: String = rt.visible_text("f%<fg:red>hi</fg>", Some(&params));
// plain = "hi"
```

`visible_text()` short-circuits for plain text (no `f%` prefix, no params) — returns the input unchanged.

---

## 7. Integration with `DrawTextParams`

The rendering pipeline automatically parses rich text from `DrawTextParams.text`:

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

If you DON'T want rich text parsing, just omit `f%` prefix and pass `params: None`:

```rust
render.draw_host_text(
    canvas,
    &DrawTextParams {
        text: "Plain text with no parsing".to_string(),
        ..Default::default()
    },
);
```

---

## 8. Common Patterns

### 8.1 Title with color + bold

```rust
format!("f%<fg:bright_yellow><b>{}</b></fg>", title)
```

### 8.2 Gray hint text (custom RGB)

```rust
format!("f%<fg:rgb(85,87,83)>{}</fg>", hint)
```

### 8.3 Red error message

```rust
format!("f%<fg:bright_red>{}</fg>", error_text)
```

### 8.4 Key binding display in hints

```rust
let params = RichTextParams::from_action_map(&Self::action_map(), "my_ui.");
let hint = format!(
    "f%{key:confirm} Confirm  {key:back} Cancel",
);
// Renders as: "[Enter] Confirm  [Esc] Cancel"
```

### 8.5 Mixed value params + key params

```rust
let mut params = RichTextParams::from_action_map(&Self::action_map(), "my_ui.");
params.values.insert("target".to_string(), "cache".to_string());

"f%Clear {target}: press {key:confirm}"
// → "Clear cache: press [Enter]"
```

### 8.6 Placeholder text with gray color

```rust
let placeholder = format!("<fg:rgb(85,87,83)>{}</fg>", default_text);
// Note: NO f% prefix — this fragment is embedded in a larger f% string
format!("f%{} {}", indicator, placeholder)
```

---

## 9. Output Data Types

### `RichText`

```rust
pub struct RichText {
    pub segments: Vec<RichTextSegment>,
}
```

### `RichTextSegment`

```rust
pub struct RichTextSegment {
    pub text: String,        // Visible text content
    pub style: TextStyle,    // The active style for this segment
}
```

### `TextStyle`

```rust
pub struct TextStyle {
    pub foreground: Option<TextColor>,
    pub background: Option<TextColor>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub dim: bool,
}
```

### `TextColor`

```rust
pub enum TextColor {
    Terminal(TerminalColor),          // ANSI 16-color
    Rgb { r: u8, g: u8, b: u8 },     // Truecolor (may be dithered)
    ForceRgb { r: u8, g: u8, b: u8 }, // Always truecolor
    Transparent,
}
```

---

## 10. Key Rules

1. **`f%` prefix required** for tag parsing. Without it, `<tags>` are rendered as literal text.
2. **One `f%` per text string** — on the outermost `format!()` call. Fragments embedded inside should NOT have their own `f%`.
3. **Unclosed tags** (`<b` without `>`) are rendered as literal text — graceful degradation, no panic.
4. **Unknown color names** in `<fg:X>` / `<bg:X>` cause the tag to be rendered literally.
5. **Missing params** `{name}` stay as literal `{name}` in the output — safe fallback.
6. **Escaping**: `\<` → `<`, `\{` → `{`, `\}` → `}`, `\>` → `>`, `\\` → `\`.
7. **Nesting**: Inner tag overrides outer for same property (e.g., fg color).
