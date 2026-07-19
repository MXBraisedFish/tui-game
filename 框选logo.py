"""
blueprint.py — LOGO 截图框动画（最小版）

仅含：幽灵态 LOGO + 截图框对角展开/收回
"""

import sys, time, random, shutil

template_logo = [
    "▟                                                                    ▙",
    "   ████████  ██    ██  ██     ██████    █████   ███    ███  ███████   ",
    "      ██     ██    ██  ██    ██        ██   ██  ████  ████  ██        ",
    "      ██     ██    ██  ██    ██   ███  ███████  ██ ████ ██  █████     ",
    "      ██     ██    ██  ██    ██    ██  ██   ██  ██  ██  ██  ██        ",
    "      ██      ██████   ██     ██████   ██   ██  ██      ██  ███████   ",
    "▜                                                                    ▛",
]

R  = "\033[0m"
Bd = "\033[2m"
Bo = "\033[1m"

# truecolor RGB 颜色
GHOST   = (80, 80, 90)    # 幽灵态：暗灰蓝
REVEAL  = (60, 170, 255)  # 显影：鲜艳亮蓝
WHITE   = (240, 240, 250) # 框线：亮白
DOT     = (110, 110, 125) # 点阵：中灰
CORNER_TL = (255, 55, 55)   # ╋ 左上角：鲜红
CORNER_BR = (70, 185, 255)  # ╋ 右下角：亮蓝

LOGO = [
    "▟                                                                    ▙",
    "   ████████  ██    ██  ██     ██████    █████   ███    ███  ███████   ",
    "      ██     ██    ██  ██    ██        ██   ██  ████  ████  ██        ",
    "      ██     ██    ██  ██    ██   ███  ███████  ██ ████ ██  █████     ",
    "      ██     ██    ██  ██    ██    ██  ██   ██  ██  ██  ██  ██        ",
    "      ██      ██████   ██     ██████   ██   ██  ██      ██  ███████   ",
    "▜                                                                    ▛",
]
ROWS = len(LOGO)
COLS = max(len(r) for r in LOGO)

def _rgb_fg(c): return f"\033[38;2;{c[0]};{c[1]};{c[2]}m"
def _rgb_bg(c): return f"\033[48;2;{c[0]};{c[1]};{c[2]}m"

class Cell:
    __slots__ = ("ch","f","b","B","d")
    def __init__(self,ch=" ",f=None,b=None,B=False,d=False):
        self.ch=ch;self.f=f;self.b=b;self.B=B;self.d=d
    def pfx(self):
        p=[]
        if self.b is not None: p.append(_rgb_bg(self.b))
        if self.f is not None: p.append(_rgb_fg(self.f))
        if self.B: p.append(Bo)
        if self.d: p.append(Bd)
        return "".join(p)
    def __eq__(self,o):
        return isinstance(o,Cell) and self.f==o.f and self.b==o.b and self.B==o.B and self.d==o.d

class Screen:
    def __init__(self,w,h):
        self.w,self.h=w,h;self.e=Cell();self.clear()
    def clear(self):
        self.g=[[self.e for _ in range(self.w)]for _ in range(self.h)]
    def ok(self,x,y):return 0<=x<self.w and 0<=y<self.h
    def put(self,x,y,c):
        if self.ok(x,y):self.g[y][x]=c
    def render(self):
        ls=[]
        for row in self.g:
            ps=[];run=[];pv=Cell()
            for c in row:
                if c==pv:run.append(c.ch)
                else:
                    if run:ps.append("".join(run));run.clear()
                    if c.f is not None or c.b is not None or c.B or c.d:
                        ps.append(R);ps.append(c.pfx())
                    elif pv.f is not None or pv.b is not None or pv.B or pv.d:
                        ps.append(R)
                    run.append(c.ch)
                pv=c
            if run:ps.append("".join(run))
            if pv.f is not None or pv.b is not None or pv.B or pv.d:ps.append(R)
            ls.append("".join(ps))
        return "\n".join(ls)

# ─── 截图框 ───────────────────────────────────────────────

class ScreenshotFrame:
    """╋ 锚点 → 对角展开 → 停留显影 → 收回"""
    LIFE = 110; APPEAR = 10; HOLD = 70; RETREAT = 8

    def __init__(self, x, y, w, h):
        self.x = x; self.y = y
        self.w = w; self.h = h
        self.tick = 0

    @property
    def alive(self): return self.tick < self.LIFE

    @property
    def cw(self):
        if self.tick < self.APPEAR:
            t = self.tick/self.APPEAR
            return max(1, int(1+(1-(1-t)**3)*(self.w-1)))
        elif self.tick < self.APPEAR+self.HOLD:
            return self.w
        else:
            t = (self.tick-self.APPEAR-self.HOLD)/self.RETREAT
            return max(1, self.w-int((t**2)*(self.w-1)))

    @property
    def ch(self):
        if self.tick < self.APPEAR:
            t = self.tick/self.APPEAR
            return max(1, int(1+(1-(1-t)**3)*(self.h-1)))
        elif self.tick < self.APPEAR+self.HOLD:
            return self.h
        else:
            t = (self.tick-self.APPEAR-self.HOLD)/self.RETREAT
            return max(1, self.h-int((t**2)*(self.h-1)))

    @property
    def opacity(self):
        if self.tick < self.APPEAR: return self.tick/self.APPEAR
        if self.tick < self.APPEAR+self.HOLD: return 1.0
        t = (self.tick-self.APPEAR-self.HOLD)/self.RETREAT
        return max(0,1-t)

    def step(self): self.tick += 1

    def draw(self, buf):
        w, h, op = self.cw, self.ch, self.opacity
        if op < 0.05 or w < 1 or h < 1: return
        x, y = self.x, self.y

        # 锚点 ╋ 左上角：红色（不参与文字背景混合）
        buf[y][x] = Cell("╋", f=CORNER_TL, B=True)
        # 右上角
        if w >= 2:
            bg = GHOST if LOGO[y][x + w - 1] == "█" else None
            buf[y][x + w - 1] = Cell("┓", f=WHITE, B=True, b=bg)
        # 左下角
        if h >= 2:
            bg = GHOST if LOGO[y + h - 1][x] == "█" else None
            buf[y + h - 1][x] = Cell("┗", f=WHITE, B=True, b=bg)
        # 右下角 ╋：蓝色（不参与文字背景混合）
        if w >= 2 and h >= 2: buf[y + h - 1][x + w - 1] = Cell("╋", f=CORNER_BR, B=True)
        # 水平虚线
        dash_c = WHITE if op > 0.7 else GHOST
        for dx in range(1, w - 1):
            bg_t = GHOST if LOGO[y][x + dx] == "█" else None
            buf[y][x + dx] = Cell("╍", f=dash_c, b=bg_t)
            if h >= 2:
                bg_b = GHOST if LOGO[y + h - 1][x + dx] == "█" else None
                buf[y + h - 1][x + dx] = Cell("╍", f=dash_c, b=bg_b)
        # 垂直虚线
        for dy in range(1, h - 1):
            bg_l = GHOST if LOGO[y + dy][x] == "█" else None
            buf[y + dy][x] = Cell("┇", f=dash_c, b=bg_l)
            if w >= 2:
                bg_r = GHOST if LOGO[y + dy][x + w - 1] == "█" else None
                buf[y + dy][x + w - 1] = Cell("┇", f=dash_c, b=bg_r)

    def inside(self, rx, ry):
        return (self.x < rx < self.x + self.cw - 1 and
                self.y < ry < self.y + self.ch - 1)

# ─── 引擎 ──────────────────────────────────────────────────

class Blueprint:
    def __init__(self):
        self._tsize()
        self.frames = []
        self.tick = 0
        self.next_frame = random.randint(30, 70)

    def _tsize(self):
        try: c, r = shutil.get_terminal_size((90, 32))
        except: c, r = 90, 32
        self.cols = max(COLS+10, c)
        self.rows = max(ROWS+10, r)
        self.scr = Screen(self.cols, self.rows)

    @property
    def lx(self): return max(2, (self.cols-COLS)//2)
    @property
    def ly(self): return max(2, (self.rows-ROWS)//2)

    def draw_ghosted(self, buf):
        for ry, line in enumerate(LOGO):
            for rx, ch in enumerate(line):
                if ch != " ":
                    buf[ry][rx] = Cell(ch, f=GHOST, d=True)

    def draw_grid(self, buf):
        """静态点阵：外围边框 + 中间行列 + 均匀纵列"""
        # 顶行 / 底行：填满 ▪
        for rx in range(1, COLS - 1):
            buf[0][rx] = Cell("▪", f=DOT, d=True)
            buf[ROWS - 1][rx] = Cell("▪", f=DOT, d=True)
        # 最左列 / 最右列
        for ry in range(1, ROWS - 1):
            buf[ry][0] = Cell("▪", f=DOT, d=True)
            buf[ry][COLS - 1] = Cell("▪", f=DOT, d=True)
        # 中间行：整行填满 ▪
        mid_row = ROWS // 2
        for rx in range(1, COLS - 1):
            if buf[mid_row][rx].ch == " ":
                buf[mid_row][rx] = Cell("▪", f=DOT, d=True)
        # 中间列：均匀布行
        mid_col = COLS // 2
        for ry in range(0, ROWS, 5):
            if buf[ry][mid_col].ch == " ":
                buf[ry][mid_col] = Cell("▪", f=DOT, d=True)
        # 均匀分布的纵列
        for rx in range(5, COLS - 1, 5):
            for ry in range(0, ROWS):
                if buf[ry][rx].ch == " ":
                    buf[ry][rx] = Cell("▪", f=DOT, d=True)

    def reveal(self, buf):
        for f in self.frames:
            if f.opacity < 0.3 or f.cw < 3 or f.ch < 2: continue
            for ry in range(ROWS):
                for rx in range(len(LOGO[ry])):
                    if not f.inside(rx, ry): continue
                    if LOGO[ry][rx] != " ":
                        buf[ry][rx] = Cell(LOGO[ry][rx], f=REVEAL, B=True)
                    elif buf[ry][rx].ch == "▪":
                        buf[ry][rx] = Cell()

    def draw_frames(self, buf):
        for f in self.frames: f.draw(buf)

    def spawn_frame(self):
        w = random.randint(15, 30)
        h = random.randint(5, 7)
        if w > COLS or h > ROWS: return
        # 随机采样候选位置，最多尝试 200 次
        for _ in range(200):
            x = random.randint(0, COLS - w)
            y = random.randint(0, ROWS - h)
            if not self._overlaps_any(x, y, w, h):
                self.frames.append(ScreenshotFrame(x, y, w, h))
                return

    def _overlaps_any(self, x, y, w, h):
        gap = 2
        ax1 = x - gap; ay1 = y - gap
        ax2 = x + w + gap - 1; ay2 = y + h + gap - 1
        for f in self.frames:
            bx1 = f.x; by1 = f.y
            bx2 = f.x + f.w - 1; by2 = f.y + f.h - 1
            if ax1 <= bx2 and ax2 >= bx1 and ay1 <= by2 and ay2 >= by1:
                return True
        return False

    def step(self):
        self.tick += 1
        if self.tick >= self.next_frame and len(self.frames) < 2:
            self.spawn_frame()
            self.next_frame = self.tick + random.randint(30, 80)
        for f in self.frames: f.step()
        self.frames = [f for f in self.frames if f.alive]

    def render(self):
        s = self.scr; s.clear()
        buf = [[Cell() for _ in range(COLS)] for _ in range(ROWS)]
        self.draw_ghosted(buf)
        self.draw_grid(buf)
        self.reveal(buf)
        self.draw_frames(buf)
        for ry in range(ROWS):
            for rx in range(COLS):
                c = buf[ry][rx]
                if c.ch != " " or c.f is not None:
                    s.put(self.lx+rx, self.ly+ry, c)
        return s.render()

# ─── Main ────────────────────────────────────────────────

def main():
    bp = Blueprint()
    out = sys.stdout.buffer if hasattr(sys.stdout, "buffer") else sys.stdout
    out.write(b"\033[?25l\033[?1049h"); out.flush()
    try:
        while True:
            bp.step()
            out.write(("\033[H"+bp.render()).encode("utf-8"))
            out.flush()
            time.sleep(0.06)
    except KeyboardInterrupt: pass
    finally:
        out.write(b"\033[?25h\033[?1049l"+R.encode("utf-8")+b"\033[2J\033[H")
        out.flush()

if __name__ == "__main__": main()
