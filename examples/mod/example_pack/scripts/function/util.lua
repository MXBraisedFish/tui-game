example_util = example_util or {}

function example_util.center_x(text)
  local term_w = select(1, get_terminal_size())
  local width = get_text_width(text)
  return math.max(1, math.floor((term_w - width) / 2) + 1)
end
