import colorsys
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

RAW_TEMPLATES = [
    "f%   <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset>    <fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset>     <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset>    <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset>   <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset>    <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset>   ",
    "f%      <fg:{}>█<reset><fg:{}>█<reset>     <fg:{}>█<reset><fg:{}>█<reset>    <fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset>    <fg:{}>█<reset><fg:{}>█<reset>        <fg:{}>█<reset><fg:{}>█<reset>   <fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset>        ",
    "f%      <fg:{}>█<reset><fg:{}>█<reset>     <fg:{}>█<reset><fg:{}>█<reset>    <fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset>    <fg:{}>█<reset><fg:{}>█<reset>   <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset> <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset> <fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset>     ",
    "f%      <fg:{}>█<reset><fg:{}>█<reset>     <fg:{}>█<reset><fg:{}>█<reset>    <fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset>    <fg:{}>█<reset><fg:{}>█<reset>    <fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset>   <fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset>        ",
    "f%      <fg:{}>█<reset><fg:{}>█<reset>      <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset>   <fg:{}>█<reset><fg:{}>█<reset>     <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset>   <fg:{}>█<reset><fg:{}>█<reset>   <fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset>      <fg:{}>█<reset><fg:{}>█<reset>  <fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset><fg:{}>█<reset>   ",
]

TEMPLATES = [t.rstrip("\n") for t in RAW_TEMPLATES]
BLOCKS_PER_ROW = [t.count("{}") for t in TEMPLATES]

PERIOD = 22.0
DIAG_FACTOR = 1.8
SPEED = -0.5


def hsv_to_hex(h, s, v):
    r, g, b = colorsys.hsv_to_rgb(h % 1.0, s, v)
    return "#{:02x}{:02x}{:02x}".format(int(r * 255), int(g * 255), int(b * 255))


def main():
    offset = 0.0
    frame_interval = 0.05
    try:
        while True:
            all_colors = []
            for row in range(len(TEMPLATES)):
                n = BLOCKS_PER_ROW[row]
                row_colors = []
                for col in range(n):
                    hue = ((col + row * DIAG_FACTOR) / PERIOD + offset) % 1.0
                    row_colors.append(hsv_to_hex(hue, 0.5, 1.0))
                all_colors.append(row_colors)

            buf = []
            for row in range(len(TEMPLATES)):
                formatted = TEMPLATES[row].format(*all_colors[row])
                buf.append(svc.render_ansi(formatted))

            sys.stdout.write("\x1b[2J\x1b[H" + "\n".join(buf))
            sys.stdout.flush()

            offset = (offset + SPEED * frame_interval) % 1.0
            time.sleep(frame_interval)
    except KeyboardInterrupt:
        sys.stdout.write("\x1b[2J\x1b[H")
        sys.stdout.flush()


if __name__ == "__main__":
    main()
