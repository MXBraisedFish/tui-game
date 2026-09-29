"""
tg_rich_tect.py — 富文本解析库 (Python 移植版)

将 Rust 项目 tui-game 的 rich_text 服务完整移植为 Python 实现。
支持解析 f% 前缀的富文本字符串，将 <tag> 标签转换为样式段、{param} 替换为实际值。

使用示例:
    from tg_rich_tect import RichTextService, RichTextParams

    svc = RichTextService()
    result = svc.parse("f%<fg:red>Hello</fg> World")
    for seg in result.segments:
        print(seg.text, seg.style)

    text = svc.visible_text("f%<fg:red>Hello</fg> World")
    print(text)  # "Hello World"
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum
from typing import Iterator

# ============================================================
# 终端颜色枚举
# ============================================================

class TerminalColor(Enum):
    Black = "black"
    Red = "red"
    Green = "green"
    Yellow = "yellow"
    Blue = "blue"
    Magenta = "magenta"
    Cyan = "cyan"
    White = "white"
    BrightBlack = "bright_black"
    BrightRed = "bright_red"
    BrightGreen = "bright_green"
    BrightYellow = "bright_yellow"
    BrightBlue = "bright_blue"
    BrightMagenta = "bright_magenta"
    BrightCyan = "bright_cyan"
    BrightWhite = "bright_white"


# ============================================================
# 文本颜色：终端色 / RGB 真彩色 / 透明
# ============================================================

@dataclass
class _Rgb:
    r: int
    g: int
    b: int


class TextColor:
    __match_args__ = ("_variant", "_rgb")

    def __init__(self) -> None:
        raise TypeError("Use TextColor.create_terminal(), .create_rgb(), "
                        ".create_force_rgb(), or .create_transparent()")

    @staticmethod
    def create_terminal(c: TerminalColor) -> TextColor:
        inst = object.__new__(TextColor)
        inst._variant = "terminal"
        inst._terminal = c
        return inst

    @staticmethod
    def create_rgb(r: int, g: int, b: int) -> TextColor:
        inst = object.__new__(TextColor)
        inst._variant = "rgb"
        inst._rgb = _Rgb(r, g, b)
        return inst

    @staticmethod
    def create_force_rgb(r: int, g: int, b: int) -> TextColor:
        inst = object.__new__(TextColor)
        inst._variant = "force_rgb"
        inst._rgb = _Rgb(r, g, b)
        return inst

    @staticmethod
    def create_transparent() -> TextColor:
        inst = object.__new__(TextColor)
        inst._variant = "transparent"
        return inst

    @property
    def variant(self) -> str:
        return self._variant

    @property
    def terminal_color(self) -> TerminalColor | None:
        if self._variant == "terminal":
            return self._terminal
        return None

    @property
    def rgb(self) -> _Rgb | None:
        if self._variant in ("rgb", "force_rgb"):
            return self._rgb
        return None

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, TextColor):
            return NotImplemented
        if self._variant != other._variant:
            return False
        if self._variant in ("rgb", "force_rgb"):
            return self._rgb == other._rgb
        if self._variant == "terminal":
            return self._terminal == other._terminal
        return True

    def __hash__(self) -> int:
        if self._variant in ("rgb", "force_rgb"):
            rgb = self._rgb
            return hash((self._variant, rgb.r, rgb.g, rgb.b))
        if self._variant == "terminal":
            return hash((self._variant, self._terminal))
        return hash(self._variant)

    def __repr__(self) -> str:
        if self._variant == "transparent":
            return "TextColor.create_transparent()"
        if self._variant == "terminal":
            return f"TextColor.create_terminal({self._terminal!r})"
        rgb = self._rgb
        if self._variant == "rgb":
            return f"TextColor.create_rgb({rgb.r}, {rgb.g}, {rgb.b})"
        return f"TextColor.create_force_rgb({rgb.r}, {rgb.g}, {rgb.b})"


# ============================================================
# 解析颜色字符串
# ============================================================

TERMINAL_COLOR_MAP: dict[str, TerminalColor] = {
    "black":            TerminalColor.Black,
    "red":              TerminalColor.Red,
    "green":            TerminalColor.Green,
    "yellow":           TerminalColor.Yellow,
    "blue":             TerminalColor.Blue,
    "magenta":          TerminalColor.Magenta,
    "cyan":             TerminalColor.Cyan,
    "white":            TerminalColor.White,
    "bright_black":     TerminalColor.BrightBlack,
    "bright_red":       TerminalColor.BrightRed,
    "bright_green":     TerminalColor.BrightGreen,
    "bright_yellow":    TerminalColor.BrightYellow,
    "bright_blue":      TerminalColor.BrightBlue,
    "bright_magenta":   TerminalColor.BrightMagenta,
    "bright_cyan":      TerminalColor.BrightCyan,
    "bright_white":     TerminalColor.BrightWhite,
}


def parse_text_color(value: str) -> TextColor | None:
    value = value.strip()

    c = TERMINAL_COLOR_MAP.get(value)
    if c is not None:
        return TextColor.create_terminal(c)

    c = _parse_hex_color(value)
    if c is not None:
        return c

    c = _parse_rgb_color(value)
    if c is not None:
        return c

    return None


def _parse_hex_color(value: str) -> TextColor | None:
    if not value.startswith("#"):
        return None
    hex_part = value[1:]
    if len(hex_part) != 6:
        return None
    try:
        r = int(hex_part[0:2], 16)
        g = int(hex_part[2:4], 16)
        b = int(hex_part[4:6], 16)
    except ValueError:
        return None
    return TextColor.create_rgb(r, g, b)


def _parse_rgb_color(value: str) -> TextColor | None:
    if not value.startswith("rgb(") or not value.endswith(")"):
        return None
    inner = value[4:-1]
    parts = [p.strip() for p in inner.split(",")]
    if len(parts) != 3:
        return None
    try:
        r = int(parts[0])
        g = int(parts[1])
        b = int(parts[2])
    except ValueError:
        return None
    return TextColor.create_rgb(r, g, b)


# ============================================================
# 文本样式
# ============================================================

@dataclass
class TextStyle:
    foreground: TextColor | None = None
    background: TextColor | None = None
    bold: bool = False
    italic: bool = False
    underline: bool = False
    strike: bool = False
    blink: bool = False
    reverse: bool = False
    hidden: bool = False
    dim: bool = False

    def enable_style(self, tag: str) -> bool:
        """按标签名启用文本修饰，返回是否识别成功。"""
        match tag:
            case "bold" | "b":
                self.bold = True
            case "italic" | "i":
                self.italic = True
            case "underline" | "u":
                self.underline = True
            case "strike" | "s":
                self.strike = True
            case "blink" | "l":
                self.blink = True
            case "reverse" | "r":
                self.reverse = True
            case "hidden" | "h":
                self.hidden = True
            case "dim" | "d":
                self.dim = True
            case _:
                return False
        return True

    def disable_style(self, tag: str) -> bool:
        """按标签名禁用文本修饰，返回是否识别成功。"""
        match tag:
            case "bold" | "b":
                self.bold = False
            case "italic" | "i":
                self.italic = False
            case "underline" | "u":
                self.underline = False
            case "strike" | "s":
                self.strike = False
            case "blink" | "l":
                self.blink = False
            case "reverse" | "r":
                self.reverse = False
            case "hidden" | "h":
                self.hidden = False
            case "dim" | "d":
                self.dim = False
            case _:
                return False
        return True

    def set_foreground(self, color: TextColor) -> None:
        self.foreground = color

    def clear_foreground(self) -> None:
        self.foreground = None

    def set_background(self, color: TextColor) -> None:
        self.background = color

    def clear_background(self) -> None:
        self.background = None

    def clone(self) -> TextStyle:
        return TextStyle(
            foreground=self.foreground,
            background=self.background,
            bold=self.bold,
            italic=self.italic,
            underline=self.underline,
            strike=self.strike,
            blink=self.blink,
            reverse=self.reverse,
            hidden=self.hidden,
            dim=self.dim,
        )

    def reset(self) -> None:
        self.foreground = None
        self.background = None
        self.bold = False
        self.italic = False
        self.underline = False
        self.strike = False
        self.blink = False
        self.reverse = False
        self.hidden = False
        self.dim = False


# ============================================================
# 富文本类型
# ============================================================

@dataclass
class RichTextSegment:
    text: str
    style: TextStyle


@dataclass
class RichText:
    segments: list[RichTextSegment] = field(default_factory=list)


# ============================================================
# 富文本参数
# ============================================================

class RichTextParams:
    values: dict[str, str]
    key_actions: dict[str, list[list[str]]]

    def __init__(self, values: dict[str, str] | None = None,
                 key_actions: dict[str, list[list[str]]] | None = None) -> None:
        self.values = values or {}
        self.key_actions = key_actions or {}


# ============================================================
# 按键显示格式化
# ============================================================

_KEY_DISPLAY: dict[str, str] = {
    "esc": "Esc", "enter": "Enter", "tab": "Tab", "backspace": "Bksp",
    "space": "Space", "up": "\u2191", "down": "\u2193", "left": "\u2190",
    "right": "\u2192", "home": "Home", "end": "End", "pageup": "PgUp",
    "pagedown": "PgDn", "ins": "Ins", "del": "Del", "capslock": "Caps",
    "numlock": "Num", "scrolllock": "Scrl", "printscreen": "Prtsc",
    "pause": "Pause",
    "left_ctrl": "Ctrl", "ctrl": "Ctrl", "right_ctrl": "Ctrl",
    "left_shift": "Shift", "shift": "Shift", "right_shift": "Shift",
    "left_alt": "Alt", "alt": "Alt", "right_alt": "Alt",
    "left_meta": "Meta", "meta": "Meta", "right_meta": "Meta",
    "`": "`", "-": "-", "=": "=", "[": "[", "]": "]",
    "\\": "\\", ";": ";", "'": "'", ",": ",", ".": ".", "/": "/",
    "k+": "K+", "k-": "K-", "k*": "K*", "k/": "K/",
    "kenter": "KEnter", "kdel": "KDel",
}

_MODIFIER_KEY_TOKENS = {"ctrl", "shift", "alt", "meta",
                         "left_ctrl", "right_ctrl", "left_shift",
                         "right_shift", "left_alt", "right_alt",
                         "left_meta", "right_meta"}

for _ch in "abcdefghijklmnopqrstuvwxyz":
    _KEY_DISPLAY[_ch] = _ch.upper()


def _display_key_token(token: str) -> str:
    t = token.lower()
    if t in _KEY_DISPLAY:
        return _KEY_DISPLAY[t]
    if t.startswith("f") and len(t) > 1 and t[1:].isdigit():
        return t.upper()
    if t.startswith("k") and len(t) > 1 and t[1:].isdigit():
        return f"K{t[1:]}"
    if t.isdigit():
        return t
    return token


def _key_display_order(token: str) -> int:
    t = token.lower()
    if t in _MODIFIER_KEY_TOKENS:
        return 0
    if len(t) == 1 and t.isalpha():
        return 10
    if t.isdigit():
        return 20
    if t.startswith("k") and len(t) > 1 and t[1:].isdigit():
        return 30
    syms = {"`", "-", "=", "[", "]", "\\", ";", "'", ",", ".", "/",
            "k+", "k-", "k*", "k/", "kenter", "kdel"}
    if t in syms:
        return 50
    return 60


def format_key_display(patterns: list[list[str]]) -> str:
    results: list[str] = []
    for pattern in patterns:
        keys = sorted(pattern, key=_key_display_order)
        display_parts = [_display_key_token(k) for k in keys]
        if display_parts:
            results.append(f"[{' + '.join(display_parts)}]")
        else:
            results.append(f"[{' + '.join(pattern)}]")
    return "/".join(results)


# ============================================================
# 核心解析器
# ============================================================

RICH_TEXT_PREFIX = "f%"
_ESCAPABLE_CHARS = frozenset({'{', '}', '<', '>', '\\'})


class _ParameterReadResult:
    __slots__ = ("closed", "content")
    def __init__(self, closed: bool, content: str):
        self.closed = closed
        self.content = content


class _TagReadResult:
    __slots__ = ("closed", "content")
    def __init__(self, closed: bool, content: str):
        self.closed = closed
        self.content = content


def _read_escaped_char(chars: Iterator[tuple[int, str]]) -> str | None:
    try:
        _, ch = next(chars)
    except StopIteration:
        return None
    if ch in _ESCAPABLE_CHARS:
        return ch
    return None


def _read_parameter_name(chars: Iterator[tuple[int, str]]) -> _ParameterReadResult:
    name: list[str] = []
    for _, ch in chars:
        if ch == '{':
            return _ParameterReadResult(False, ''.join(name))
        if ch == '\\':
            esc = _read_escaped_char(chars)
            name.append(esc if esc is not None else ch)
            continue
        if ch == '}':
            return _ParameterReadResult(True, ''.join(name))
        name.append(ch)
    return _ParameterReadResult(False, ''.join(name))


def _read_tag(chars: Iterator[tuple[int, str]]) -> _TagReadResult:
    tag: list[str] = []
    for _, ch in chars:
        if ch == '<':
            return _TagReadResult(False, ''.join(tag))
        if ch == '\\':
            esc = _read_escaped_char(chars)
            tag.append(esc if esc is not None else ch)
            continue
        if ch == '>':
            return _TagReadResult(True, ''.join(tag))
        tag.append(ch)
    return _TagReadResult(False, ''.join(tag))


def _apply_tag(tag: str, current_style: TextStyle) -> bool:
    tag = tag.strip()

    if tag == "reset":
        current_style.reset()
        return True

    if tag == "/fg":
        current_style.clear_foreground()
        return True

    if tag == "/bg":
        current_style.clear_background()
        return True

    if tag.startswith("/"):
        return current_style.disable_style(tag[1:].strip())

    if tag.startswith("fg:"):
        color = parse_text_color(tag[3:])
        if color is not None:
            current_style.set_foreground(color)
            return True
        return False

    if tag.startswith("bg:"):
        color = parse_text_color(tag[3:])
        if color is not None:
            current_style.set_background(color)
            return True
        return False

    return current_style.enable_style(tag)


def _flush_segment(segments: list[RichTextSegment], output: list[str],
                   style: TextStyle) -> None:
    if not output:
        return
    segments.append(RichTextSegment(text=''.join(output), style=style.clone()))
    output.clear()


def _parse_formatted_text(text: str, params: RichTextParams | None) -> RichText:
    segments: list[RichTextSegment] = []
    output: list[str] = []
    current_style = TextStyle()
    chars = _enumerate_iter(text)

    for _, ch in chars:
        if ch == '\\':
            esc = _read_escaped_char(chars)
            output.append(esc if esc is not None else ch)
            continue

        if ch == '{':
            result = _read_parameter_name(chars)
            if result.closed:
                _write_resolved_parameter(output, result.content, params)
            else:
                output.append('{')
                output.append(result.content)
            continue

        if ch == '<':
            result = _read_tag(chars)
            if result.closed:
                _flush_segment(segments, output, current_style)
                if not _apply_tag(result.content, current_style):
                    output.append('<')
                    output.append(result.content)
                    output.append('>')
            else:
                output.append('<')
                output.append(result.content)
            continue

        output.append(ch)

    _flush_segment(segments, output, current_style)
    return RichText(segments=segments)


def _enumerate_iter(s: str) -> Iterator[tuple[int, str]]:
    yield from enumerate(s)


def _write_resolved_parameter(output: list[str], name: str,
                              params: RichTextParams | None) -> None:
    if not name:
        output.append("{}")
        return

    value = _resolve_parameter(name, params)
    if value is not None:
        output.append(value)
    else:
        output.append('{')
        output.append(name)
        output.append('}')


def _resolve_parameter(name: str, params: RichTextParams | None) -> str | None:
    if ':' in name:
        ns, key = name.split(':', 1)
        if ns == "value":
            return _resolve_value(key, params)
        if ns == "key":
            return _resolve_key(key, params)
        return None
    else:
        return _resolve_value(name, params)


def _resolve_value(key: str, params: RichTextParams | None) -> str | None:
    if params is None:
        return None
    return params.values.get(key)


def _resolve_key(action: str, params: RichTextParams | None) -> str | None:
    if params is None:
        return None
    patterns = params.key_actions.get(action)
    if patterns is None:
        return None
    return format_key_display(patterns)


def _plain_text(text: str) -> RichText:
    return RichText(segments=[RichTextSegment(text=text, style=TextStyle())])


# ============================================================
# 公共解析入口
# ============================================================

def parse(text: str, params: RichTextParams | None = None) -> RichText:
    if text.startswith(RICH_TEXT_PREFIX):
        body = text[len(RICH_TEXT_PREFIX):]
    elif params is not None:
        return _parse_formatted_text(text, params)
    else:
        return _plain_text(text)

    return _parse_formatted_text(body, params)


# ============================================================
# 富文本服务
# ============================================================

class RichTextService:
    def parse(self, text: str, params: RichTextParams | None = None) -> RichText:
        return parse(text, params)

    def visible_text(self, text: str, params: RichTextParams | None = None) -> str:
        if params is None and not text.startswith("f%"):
            return text
        rich_text = self.parse(text, params)
        return ''.join(seg.text for seg in rich_text.segments)

    def render_ansi(self, text: str, params: RichTextParams | None = None) -> str:
        rich_text = self.parse(text, params)
        return render_ansi(rich_text)


# ============================================================
# ANSI 终端渲染
# ============================================================

_ANSI_RESET = "\033[0m"

_TERMINAL_COLOR_FG: dict[TerminalColor, str] = {
    TerminalColor.Black:           "\033[30m",
    TerminalColor.Red:             "\033[31m",
    TerminalColor.Green:           "\033[32m",
    TerminalColor.Yellow:          "\033[33m",
    TerminalColor.Blue:            "\033[34m",
    TerminalColor.Magenta:         "\033[35m",
    TerminalColor.Cyan:            "\033[36m",
    TerminalColor.White:           "\033[37m",
    TerminalColor.BrightBlack:     "\033[90m",
    TerminalColor.BrightRed:       "\033[91m",
    TerminalColor.BrightGreen:     "\033[92m",
    TerminalColor.BrightYellow:    "\033[93m",
    TerminalColor.BrightBlue:      "\033[94m",
    TerminalColor.BrightMagenta:   "\033[95m",
    TerminalColor.BrightCyan:      "\033[96m",
    TerminalColor.BrightWhite:     "\033[97m",
}

_TERMINAL_COLOR_BG: dict[TerminalColor, str] = {
    TerminalColor.Black:           "\033[40m",
    TerminalColor.Red:             "\033[41m",
    TerminalColor.Green:           "\033[42m",
    TerminalColor.Yellow:          "\033[43m",
    TerminalColor.Blue:            "\033[44m",
    TerminalColor.Magenta:         "\033[45m",
    TerminalColor.Cyan:            "\033[46m",
    TerminalColor.White:           "\033[47m",
    TerminalColor.BrightBlack:     "\033[100m",
    TerminalColor.BrightRed:       "\033[101m",
    TerminalColor.BrightGreen:     "\033[102m",
    TerminalColor.BrightYellow:    "\033[103m",
    TerminalColor.BrightBlue:      "\033[104m",
    TerminalColor.BrightMagenta:   "\033[105m",
    TerminalColor.BrightCyan:      "\033[106m",
    TerminalColor.BrightWhite:     "\033[107m",
}

_STYLE_ANSI: dict[str, str] = {
    "bold":      "\033[1m",
    "dim":       "\033[2m",
    "italic":    "\033[3m",
    "underline": "\033[4m",
    "blink":     "\033[5m",
    "reverse":   "\033[7m",
    "hidden":    "\033[8m",
    "strike":    "\033[9m",
}

_STYLE_RESET: dict[str, str] = {
    "bold":      "\033[22m",
    "dim":       "\033[22m",
    "italic":    "\033[23m",
    "underline": "\033[24m",
    "blink":     "\033[25m",
    "reverse":   "\033[27m",
    "hidden":    "\033[28m",
    "strike":    "\033[29m",
}


def _style_to_ansi(style: TextStyle) -> str:
    codes: list[str] = []

    fg = style.foreground
    if fg is not None:
        if fg.variant == "terminal":
            codes.append(_TERMINAL_COLOR_FG[fg.terminal_color])
        elif fg.variant in ("rgb", "force_rgb"):
            rgb = fg.rgb
            codes.append(f"\033[38;2;{rgb.r};{rgb.g};{rgb.b}m")

    bg = style.background
    if bg is not None:
        if bg.variant == "terminal":
            codes.append(_TERMINAL_COLOR_BG[bg.terminal_color])
        elif bg.variant in ("rgb", "force_rgb"):
            rgb = bg.rgb
            codes.append(f"\033[48;2;{rgb.r};{rgb.g};{rgb.b}m")

    if style.bold:
        codes.append(_STYLE_ANSI["bold"])
    if style.dim:
        codes.append(_STYLE_ANSI["dim"])
    if style.italic:
        codes.append(_STYLE_ANSI["italic"])
    if style.underline:
        codes.append(_STYLE_ANSI["underline"])
    if style.blink:
        codes.append(_STYLE_ANSI["blink"])
    if style.reverse:
        codes.append(_STYLE_ANSI["reverse"])
    if style.hidden:
        codes.append(_STYLE_ANSI["hidden"])
    if style.strike:
        codes.append(_STYLE_ANSI["strike"])

    return ''.join(codes)


def render_ansi(rich_text: RichText) -> str:
    parts: list[str] = []
    prev: TextStyle | None = None
    for seg in rich_text.segments:
        if not seg.text:
            continue
        if prev is None or _style_neq(prev, seg.style):
            if prev is not None:
                parts.append(_ANSI_RESET)
            codes = _style_to_ansi(seg.style)
            if codes:
                parts.append(codes)
            prev = seg.style
        parts.append(seg.text)
    if prev is not None:
        parts.append(_ANSI_RESET)
    return ''.join(parts)


def _style_neq(a: TextStyle, b: TextStyle) -> bool:
    return (
        a.foreground != b.foreground
        or a.background != b.background
        or a.bold != b.bold
        or a.dim != b.dim
        or a.italic != b.italic
        or a.underline != b.underline
        or a.blink != b.blink
        or a.reverse != b.reverse
        or a.hidden != b.hidden
        or a.strike != b.strike
    )
