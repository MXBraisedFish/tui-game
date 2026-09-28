import random
import time
import sys

from tg_rich_tect import RichTextService

template_logo = [
    "f%  ██████████ ████  ████ ████     ████████   ███████  █████  █████ █████████  ",
    "f%     ████    ████  ████ ████    ████       ████ ████ ████████████ ████       ",
    "f%     ████    ████  ████ ████    ████ █████ █████████ ████████████ ███████    ",
    "f%     ████    ████  ████ ████    ████  ████ ████ ████ ████████████ ████       ",
    "f%     ████     ████████  ████     ████████  ████ ████ ████    ████ █████████  ",
]

_rich = RichTextService()

# 原始字符艺术数据
ART = [
    "f%  <fg:cyan>█<fg:white>████████<fg:bright_red>█<reset> <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>  <fg:cyan>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>     <fg:cyan>█<fg:white>██████<fg:bright_red>█<reset>   <fg:cyan>█<fg:white>█████<fg:bright_red>█<reset>  <fg:cyan>█<fg:white>███<fg:bright_red>█<reset>  <fg:cyan>█<fg:white>███<fg:bright_red>█<reset> <fg:cyan>█<fg:white>███████<fg:bright_red>█<reset>  ",
    "f%     <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>    <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>  <fg:cyan>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>    <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>       <fg:cyan>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>████<fg:bright_red>█<reset><fg:cyan>█<fg:white>████<fg:bright_red>█<reset> <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>       ",
    "f%     <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>    <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>  <fg:cyan>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>    <fg:cyan>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>███<fg:bright_red>█<reset> <fg:cyan>█<fg:white>███████<fg:bright_red>█<reset> <fg:cyan>█<fg:white>██<fg:magenta>█<fg:white>████<fg:magenta>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>█████<fg:bright_red>█<reset>    ",
    "f%     <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>    <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>  <fg:cyan>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>    <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>  <fg:cyan>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>██<fg:bright_red>█<reset><fg:cyan>█<fg:white>██<fg:bright_red>█<reset><fg:cyan>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>       ",
    "f%     <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>     <fg:cyan>█<fg:white>██████<fg:bright_red>█<reset>  <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>     <fg:cyan>█<fg:white>██████<fg:bright_red>█<reset>  <fg:cyan>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>██<fg:bright_red>█<reset>    <fg:cyan>█<fg:white>██<fg:bright_red>█<reset> <fg:cyan>█<fg:white>███████<fg:bright_red>█<reset>  ",
]

ART_ORIGINAL = [line.rstrip("\n") for line in ART]


def parse_line(line):
    rest = line[2:]
    i = 0
    while i < len(rest) and rest[i] == " ":
        i += 1
    return rest[:i], rest[i:]


def any_glitching(lines):
    return any(l["steps"] > 0 for l in lines)


def trigger_glitch(lines):
    normal_indices = [i for i, l in enumerate(lines) if l["steps"] == 0]
    if len(normal_indices) < 5:
        return

    targets = random.sample(normal_indices, 5)
    for idx in targets:
        delta = random.choice([-2, -1, 1, 2])
        lines[idx]["init_offset"] = float(delta)
        lines[idx]["offset"] = float(delta)
        lines[idx]["total_steps"] = random.randint(1, 8)
        lines[idx]["steps"] = lines[idx]["total_steps"]


def update_lines(lines):
    for l in lines:
        if l["steps"] > 0:
            step_size = l["init_offset"] / l["total_steps"]
            l["offset"] -= step_size
            l["steps"] -= 1
            if l["steps"] == 0:
                l["offset"] = 0.0


def render(lines):
    buf = []
    for l in lines:
        space_count = len(l["orig_spaces"]) + int(round(l["offset"]))
        space_count = max(0, space_count)
        raw = "f%" + " " * space_count + l["content"]
        buf.append(_rich.render_ansi(raw))
    return "\n".join(buf)


def init_lines():
    lines = []
    for raw in ART_ORIGINAL:
        orig_spaces, content = parse_line(raw)
        lines.append({
            "orig_spaces": orig_spaces,
            "content": content,
            "offset": 0.0,
            "init_offset": 0.0,
            "steps": 0,
            "total_steps": 0,
        })
    return lines


def main():
    lines = init_lines()
    cooldown_until = 0.0
    has_triggered = False
    try:
        while True:
            now = time.time()

            if has_triggered and not any_glitching(lines):
                cooldown_until = now + random.uniform(1.0, 3.0)
                has_triggered = False

            if not has_triggered and not any_glitching(lines) and now >= cooldown_until:
                if random.random() < 0.4:
                    trigger_glitch(lines)
                    has_triggered = True

            update_lines(lines)
            sys.stdout.write("\x1b[2J\x1b[H" + render(lines) + "\n")
            sys.stdout.flush()
            time.sleep(random.uniform(0.08, 0.15))
    except KeyboardInterrupt:
        sys.stdout.write("\x1b[2J\x1b[H")
        sys.stdout.flush()


if __name__ == "__main__":
    main()

[
    "f%   ████████  ██    ██  ██     ██████    █████   ███    ███  ███████  ",
    "f%      ██     ██    ██  ██    ██        ██   ██  ████  ████  ██       ",
    "f%      ██     ██    ██  ██    ██   ███  ███████  ██ ████ ██  █████    ",
    "f%      ██     ██    ██  ██    ██    ██  ██   ██  ██  ██  ██  ██       ",
    "f%      ██      ██████   ██     ██████   ██   ██  ██      ██  ███████  ",
]