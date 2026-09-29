import random
import sys
import time
from tg_rich_tect import RichTextService

template_logo = [
    "f%   ████████  ██    ██  ██     ██████    █████   ███    ███  ███████   ",
    "f%      ██     ██    ██  ██    ██        ██   ██  ████  ████  ██        ",
    "f%      ██     ██    ██  ██    ██   ███  ███████  ██ ████ ██  █████     ",
    "f%      ██     ██    ██  ██    ██    ██  ██   ██  ██  ██  ██  ██        ",
    "f%      ██      ██████   ██     ██████   ██   ██  ██      ██  ███████   ",
]

svc = RichTextService()

try:
    sys.stdout.reconfigure(encoding='utf-8')
except Exception:
    pass

LOGO_ROWS = [
    "f%   ████████  ██    ██  ██     ██████    █████   ███    ███  ███████   ",
    "f%      ██     ██    ██  ██    ██        ██   ██  ████  ████  ██        ",
    "f%      ██     ██    ██  ██    ██   ███  ███████  ██ ████ ██  █████     ",
    "f%      ██     ██    ██  ██    ██    ██  ██   ██  ██  ██  ██  ██        ",
    "f%      ██      ██████   ██     ██████   ██   ██  ██      ██  ███████   ",
]

CONTENT_WIDTH = max(len(t) - 2 for t in LOGO_ROWS)
BLANK = "f%" + " " * CONTENT_WIDTH
TEMPLATES = [BLANK] + LOGO_ROWS + [BLANK]
NUM_ROWS = len(TEMPLATES)
LOGO_START = 1
LOGO_END = 1 + len(LOGO_ROWS)

GLITCH_CHARS = ["#", "@", "$", "%", "&", "!", "?", "*", "=", "+", "~", "x", "X", "O", "0",
                "a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z"]
BLOCK_FRAGS = ["▀", "▄", "▌", "▐", "▖", "▗", "▘", "▙", "▚", "▛", "▜", "▝", "▞", "▟"]
NOISE_CHARS = ["░", "▒", "▓", "▏", "▎", "▍", "▌", "▋", "▊", "▉", "╶", "╴", "╵", "╷", "╸", "╹", "╺", "╻", "·", "∙"]
NOISE_MAX = 6
FLICKER_MAX = 1
FRAG_MAX = 4
MAX_CORR = 5
MAX_CROSS = 4
MAX_BLUE = 6

corruptions = []
crosses = []
blues = []
noises = []
flickers = []
frags = []


def rand_hex():
    return "#{:02x}{:02x}{:02x}".format(
        random.randint(0, 255), random.randint(0, 255), random.randint(0, 255)
    )


def rand_gray():
    v = random.randint(90, 160)
    return "#{:02x}{:02x}{:02x}".format(v, v, v)


def try_spawn_noise():
    if len(noises) >= NOISE_MAX:
        return
    if random.random() < 0.12:
        row = random.randint(0, NUM_ROWS - 1)
        count = random.randint(5, 15)
        for _ in range(count):
            col = random.randint(0, CONTENT_WIDTH - 1)
            noises.append({
                "row": row, "col": col,
                "ch": random.choice(NOISE_CHARS),
                "remaining": random.randint(1, 4),
            })


def try_spawn_flicker():
    if len(flickers) >= FLICKER_MAX:
        return
    if random.random() < 0.08:
        row = random.randint(0, NUM_ROWS - 1)
        kind = random.choice(["dim", "reverse"])
        flickers.append({
            "row": row, "kind": kind,
            "remaining": random.randint(1, 3),
        })


def try_spawn_frag():
    if len(frags) >= FRAG_MAX:
        return
    if random.random() < 0.20:
        row = random.randint(LOGO_START, LOGO_END - 1)
        body = TEMPLATES[row][2:]
        positions = [i for i, ch in enumerate(body) if ch == "█"]
        if not positions:
            return
        count = min(random.randint(1, 5), len(positions))
        for col in random.sample(positions, count):
            frags.append({
                "row": row, "col": col,
                "ch": random.choice(BLOCK_FRAGS),
                "remaining": random.randint(2, 10),
            })


def try_spawn_corr():
    if len(corruptions) >= MAX_CORR:
        return
    if random.random() < 0.4:
        row = random.randint(LOGO_START, LOGO_END - 1)
        body = TEMPLATES[row][2:]
        n = len(body)
        if n < 2:
            return
        a = random.randint(0, n - 2)
        b = random.randint(a + 1, min(a + 15, n))
        corruptions.append({
            "row": row, "a": a, "b": b,
            "color": rand_hex(),
            "remaining": random.randint(5, 25),
        })


def try_spawn_cross():
    if len(crosses) >= MAX_CROSS:
        return
    if random.random() < 0.45:
        row = random.randint(0, NUM_ROWS - 1)
        col = random.randint(0, CONTENT_WIDTH - 1)
        crosses.append({
            "row": row, "col": col, "ch": random.choice(GLITCH_CHARS),
            "bg": rand_hex(),
            "remaining": random.randint(8, 30),
        })


def try_spawn_blue():
    if len(blues) >= MAX_BLUE:
        return
    if random.random() < 0.15:
        h = random.randint(1, 3)
        w = random.randint(3, 6)
        row = random.randint(0, NUM_ROWS - h)
        col = random.randint(0, CONTENT_WIDTH - w)
        blues.append({
            "row": row, "col": col, "h": h, "w": w,
            "gray": rand_gray(),
            "remaining": random.randint(20, 55),
        })


def update_all():
    for lst in (corruptions, crosses, blues, noises, flickers, frags):
        for item in lst:
            item["remaining"] -= 1
    corruptions[:] = [c for c in corruptions if c["remaining"] > 0]
    crosses[:] = [c for c in crosses if c["remaining"] > 0]
    blues[:] = [c for c in blues if c["remaining"] > 0]
    noises[:] = [c for c in noises if c["remaining"] > 0]
    flickers[:] = [c for c in flickers if c["remaining"] > 0]
    frags[:] = [c for c in frags if c["remaining"] > 0]


def compute_canvas():
    canvas = []
    for r in range(NUM_ROWS):
        body = TEMPLATES[r][2:]
        row = [(ch, None, None, False, False) for ch in body]
        canvas.append(row)

    for fl in flickers:
        r = fl["row"]
        for c in range(CONTENT_WIDTH):
            ch, fg, bg, rev, dim = canvas[r][c]
            if fl["kind"] == "dim":
                canvas[r][c] = (ch, fg, bg, rev, True)
            else:
                canvas[r][c] = (ch, fg, bg, not rev, dim)

    for corr in corruptions:
        r = corr["row"]
        for i in range(corr["a"], corr["b"]):
            if i < CONTENT_WIDTH:
                ch, _, bg, rev, dim = canvas[r][i]
                canvas[r][i] = (ch, corr["color"], bg, rev, dim)

    for bb in blues:
        gray = bb["gray"]
        for ri in range(bb["h"]):
            r = bb["row"] + ri
            for ci in range(bb["w"]):
                c = bb["col"] + ci
                if ri == 0:
                    bg = gray if ci < bb["w"] - 1 else "#ff4444"
                    canvas[r][c] = ("▅", "#0066ff", bg, False, False)
                else:
                    canvas[r][c] = ("█", "#0066ff", None, False, False)

    for n in noises:
        r, c = n["row"], n["col"]
        if r < NUM_ROWS and c < CONTENT_WIDTH:
            _, fg, bg, rev, _ = canvas[r][c]
            canvas[r][c] = (n["ch"], fg, bg, rev, True)

    for cx in crosses:
        r, c = cx["row"], cx["col"]
        if r < NUM_ROWS and c < CONTENT_WIDTH:
            _, corr_fg, _, _, dim = canvas[r][c]
            canvas[r][c] = (cx["ch"], corr_fg, cx["bg"], True, dim)

    for fg in frags:
        r, c = fg["row"], fg["col"]
        if r < NUM_ROWS and c < CONTENT_WIDTH:
            _, prev_fg, bg, rev, dim = canvas[r][c]
            canvas[r][c] = (fg["ch"], prev_fg, bg, rev, dim)

    return canvas


def render_canvas(canvas):
    lines = []
    for r in range(NUM_ROWS):
        parts = ["f%"]
        c = 0
        while c < CONTENT_WIDTH:
            ch, fg, bg, rev, dim = canvas[r][c]
            run_end = c + 1
            while run_end < CONTENT_WIDTH:
                _, fg2, bg2, rev2, dim2 = canvas[r][run_end]
                if (fg2, bg2, rev2, dim2) != (fg, bg, rev, dim):
                    break
                run_end += 1
            run_text = "".join(canvas[r][ci][0] for ci in range(c, run_end))

            if fg is None and bg is None and not rev and not dim:
                parts.append(run_text)
            elif rev and fg and bg:
                parts.append(f"<fg:{fg}><bg:{bg}><reverse>{run_text}</reverse></bg></fg>")
            elif rev and bg:
                parts.append(f"<bg:{bg}><reverse>{run_text}</reverse></bg>")
            elif rev:
                parts.append(f"<reverse>{run_text}</reverse>")
            elif dim and fg:
                parts.append(f"<fg:{fg}><dim>{run_text}</dim></fg>")
            elif dim:
                parts.append(f"<dim>{run_text}</dim>")
            elif fg and bg:
                parts.append(f"<fg:{fg}><bg:{bg}>{run_text}<reset>")
            elif bg:
                parts.append(f"<bg:{bg}>{run_text}<reset>")
            else:
                parts.append(f"<fg:{fg}>{run_text}<reset>")
            c = run_end
        lines.append("".join(parts))
    return lines


def main():
    try:
        while True:
            for _ in range(random.randint(0, 2)):
                try_spawn_corr()
            for _ in range(random.randint(0, 2)):
                try_spawn_cross()
            for _ in range(random.randint(0, 1)):
                try_spawn_blue()
            try_spawn_noise()
            try_spawn_flicker()
            for _ in range(random.randint(0, 1)):
                try_spawn_frag()

            update_all()

            canvas = compute_canvas()
            lines = [svc.render_ansi(line) for line in render_canvas(canvas)]
            sys.stdout.write("\x1b[2J\x1b[H" + "\n".join(lines))
            sys.stdout.flush()
            time.sleep(0.16)
    except KeyboardInterrupt:
        sys.stdout.write("\x1b[2J\x1b[H")
        sys.stdout.flush()


if __name__ == "__main__":
    main()
