from tg_rich_tect import RichTextService

svc = RichTextService()

LOGO = [
  "f%                                                                      ",
  "f%   ████████  ██    ██  ██     ██████    █████   ███    ███  ███████   ",
  "f%      ██     ██    ██  ██    ██        ██   ██  ████  ████  ██        ",
  "f%      ██     ██    ██  ██    ██   ███  ███████  ██ ████ ██  █████     ",
  "f%      ██     ██    ██  ██    ██    ██  ██   ██  ██  ██  ██  ██        ",
  "f%      ██      ██████   ██     ██████   ██   ██  ██      ██  ███████   ",
  "f%                                                                      ",
]

for i in LOGO:
  i = svc.render_ansi(i)
  print(i)