"""
scramble_logo.py — 模板反转扫描 LOGO 动画

模板 LOGO 仅用于确定初始 0/1 布局（█→0, 空格→1）。
0 = 空白, 1 = 随机字符。
对角线扫描逐帧翻转子：在 0 和 1 之间切换。
随机字符由独立计时器控制刷新（间隔 12~36 tick）。
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

import sys, time, random, shutil
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

POOL = (
    [chr(c) for c in range(ord('a'), ord('z')+1)]
    + [chr(c) for c in range(ord('A'), ord('Z')+1)]
    + [chr(c) for c in range(ord('0'), ord('9')+1)]
)

class Scramble:
    def __init__(self):
        self._tsize()
        # grid: 0=空白, 1=随机字符。初始由模板决定
        self.grid = [[0 for _ in range(COLS)] for _ in range(ROWS)]
        for ry in range(ROWS):
            for rx in range(len(LOGO[ry])):
                self.grid[ry][rx] = 0 if LOGO[ry][rx] == "█" else 1
        # 实际显示的字符
        self.chars = [[" " for _ in range(COLS)] for _ in range(ROWS)]
        self._fill_chars()
        # 独立计时器，控制 1 号格子的随机字符刷新
        self.timers = [[random.randint(0, 36) for _ in range(COLS)] for _ in range(ROWS)]

        self.tick = 0
        self.sweep_diag = -1
        self.hold = 0

    def _tsize(self):
        try: c, r = shutil.get_terminal_size((90, 32))
        except: c, r = 90, 32
        self.cols = max(COLS+10, c)
        self.rows = max(ROWS+10, r)

    @property
    def lx(self): return max(2, (self.cols-COLS)//2)
    @property
    def ly(self): return max(2, (self.rows-ROWS)//2)

    def _fill_chars(self):
        """根据 grid 填充显示字符"""
        for ry in range(ROWS):
            for rx in range(COLS):
                self.chars[ry][rx] = random.choice(POOL) if self.grid[ry][rx] == 1 else " "

    def _refresh_timers(self):
        """独立计时器：1 号格子按期刷新字符"""
        for ry in range(ROWS):
            for rx in range(COLS):
                if self.grid[ry][rx] == 1:
                    self.timers[ry][rx] -= 1
                    if self.timers[ry][rx] <= 0:
                        self.chars[ry][rx] = random.choice(POOL)
                        self.timers[ry][rx] = random.randint(12, 36)

    def step(self):
        self.tick += 1
        max_diag = ROWS + COLS - 2  # 最大对角线编号

        if self.hold > 0:
            self._refresh_timers()
            self.hold -= 1
            if self.hold <= 0:
                self.sweep_diag = -1
            return

        self.sweep_diag += 1
        diag = self.sweep_diag

        if diag <= max_diag:
            for ry in range(ROWS):
                rx = diag - ry
                if 0 <= rx < COLS:
                    self.grid[ry][rx] = 1 - self.grid[ry][rx]
                    self.chars[ry][rx] = random.choice(POOL) if self.grid[ry][rx] == 1 else " "
        else:
            self.hold = 80

        self._refresh_timers()

    def render(self):
        pad_h = " " * self.lx
        buf = ["\033[H"]
        for _ in range(self.ly):
            buf.append("")
        for ry in range(ROWS):
            line = pad_h + "".join(self.chars[ry])
            buf.append(svc.render_ansi(f"f%<fg:#51D16B>{line}"))
        return "\n".join(buf)


def main():
    s = Scramble()
    out = sys.stdout.buffer if hasattr(sys.stdout,"buffer") else sys.stdout
    out.write(b"\033[?25l\033[?1049h"); out.flush()
    try:
        while True:
            s.step()
            out.write(s.render().encode("utf-8"))
            out.flush()
            time.sleep(0.06)
    except KeyboardInterrupt: pass
    finally:
        out.write(b"\033[?25h\033[?1049l\033[0m\033[2J\033[H")
        out.flush()

if __name__=="__main__": main()
