"""
wave_logo.py — 水波纹 LOGO 动画

基于距离场 + 正弦波的液体波纹效果：
- 计算每个点到最近文字块的棋盘距离
- sin(distance * freq - time) 产生同心波纹
- 灰度字符梯度从暗到亮均匀分布
"""

template_logo = [
    "                                                                      ",
    "   ████████  ██    ██  ██     ██████    █████   ███    ███  ███████   ",
    "      ██     ██    ██  ██    ██        ██   ██  ████  ████  ██        ",
    "      ██     ██    ██  ██    ██   ███  ███████  ██ ████ ██  █████     ",
    "      ██     ██    ██  ██    ██    ██  ██   ██  ██  ██  ██  ██        ",
    "      ██      ██████   ██     ██████   ██   ██  ██      ██  ███████   ",
    "                                                                      ",
]

import sys, time, math, random, shutil
from tg_rich_tect import RichTextService

svc = RichTextService()

LOGO = [
    "                                                                      ",
    "   ████████  ██    ██  ██     ██████    █████   ███    ███  ███████   ",
    "      ██     ██    ██  ██    ██        ██   ██  ████  ████  ██        ",
    "      ██     ██    ██  ██    ██   ███  ███████  ██ ████ ██  █████     ",
    "      ██     ██    ██  ██    ██    ██  ██   ██  ██  ██  ██  ██        ",
    "      ██      ██████   ██     ██████   ██   ██  ██      ██  ███████   ",
    "                                                                      ",
]
ROWS = len(LOGO)
COLS = max(len(r) for r in LOGO)

# 距离场：每个格点到最近文字块(█)的棋盘距离
DIST = [[COLS + ROWS for _ in range(COLS)] for _ in range(ROWS)]
_text_pts = [(rx, ry) for ry in range(ROWS) for rx in range(len(LOGO[ry])) if LOGO[ry][rx] == "█"]
for ry in range(ROWS):
    for rx in range(COLS):
        for tx, ty in _text_pts:
            d = abs(rx - tx) + abs(ry - ty)
            if d < DIST[ry][rx]:
                DIST[ry][rx] = d

# 字符梯度
HEIGHT_CHARS = [
    " ",
    "·",
    ".",
    "-",
    "+",
    "*",
    "#",
    "@",
]


def height_char(v):
    v = max(0, min(1, v))
    idx = int(v * (len(HEIGHT_CHARS) - 1))
    return HEIGHT_CHARS[idx]

def height_color(v):
    """
    黑 -> 深蓝 -> 蓝 -> 浅蓝
    """

    v = max(0, min(1, v))

    colors = [
        (10, 25, 55),        # 黑
        (30, 60, 120),      # 深蓝
        (65, 105, 195),     # 蓝
        (120, 190, 235),   # 浅蓝
        (200, 230, 255),  # 高亮蓝
    ]

    pos = v * (len(colors) - 1)

    idx = int(pos)
    t = pos - idx

    if idx >= len(colors)-1:
        return colors[-1]

    c1 = colors[idx]
    c2 = colors[idx+1]

    return (
        int(c1[0] + (c2[0]-c1[0]) * t),
        int(c1[1] + (c2[1]-c1[1]) * t),
        int(c1[2] + (c2[2]-c1[2]) * t),
    )


class WaveLogo:
    def __init__(self):
        self._tsize()
        self.tick = 0

    def _tsize(self):
        try: c, r = shutil.get_terminal_size((90, 32))
        except: c, r = 90, 32
        self.cols = max(COLS+10, c)
        self.rows = max(ROWS+10, r)

    @property
    def lx(self): return max(2, (self.cols-COLS)//2)
    @property
    def ly(self): return max(2, (self.rows-ROWS)//2)

    def step(self):
        self.tick += 1

    def wave_height(self, x, y):
        """基于距离场的正弦水波"""
        d = DIST[y][x]
        t = self.tick * 0.04
        # 多频叠加
        h = math.sin(d * 0.30 - t * 1.2) * 0.6
        h += math.sin(d * 0.55 - t * 0.8) * 0.25
        h += math.sin((x + y) * 0.08 - t * 0.3) * 0.15
        return (h + 1) / 2

    def render(self):
        pad_h = " " * self.lx
        buf = ["\033[H"]
        for _ in range(self.ly):
            buf.append("")

        for ry in range(ROWS):
            segments = []
            for rx in range(COLS):
                h = self.wave_height(rx, ry)
                is_text = rx < len(LOGO[ry]) and LOGO[ry][rx] == "█"
                if is_text:
                    h = min(1.0, h + 0.45)
                ch = height_char(h)

                if ch == " ":
                    segments.append(" ")
                else:
                    r, g, b = height_color(h)

                    segments.append(
                        f"<fg:rgb({r},{g},{b})>{ch}</fg>"
                    )

            line = pad_h + "".join(segments)
            buf.append(svc.render_ansi(f"f%{line}"))

        return "\n".join(buf)


def main():
    w = WaveLogo()
    out = sys.stdout.buffer if hasattr(sys.stdout,"buffer") else sys.stdout
    out.write(b"\033[?25l\033[?1049h"); out.flush()
    try:
        while True:
            w.step()
            out.write(w.render().encode("utf-8"))
            out.flush()
            time.sleep(0.10)
    except KeyboardInterrupt: pass
    finally:
        out.write(b"\033[?25h\033[?1049l\033[0m\033[2J\033[H")
        out.flush()

if __name__=="__main__": main()
