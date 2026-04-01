example_util = example_util or {}

function example_util.center_x(text)
  return resolve_x(ANCHOR_CENTER, get_text_width(text), 0)
end

function example_util.clamp(value, min_value, max_value)
  if value < min_value then
    return min_value
  end
  if value > max_value then
    return max_value
  end
  return value
end

function example_util.field_origin(field_width, field_height)
  local origin_x = resolve_x(ANCHOR_CENTER, field_width, 0)
  local origin_y = resolve_y(ANCHOR_MIDDLE, field_height, -1)
  origin_x = math.max(2, origin_x)
  origin_y = math.max(6, origin_y)
  return origin_x, origin_y
end

function example_util.draw_center(y, text, fg, bg)
  draw_text(example_util.center_x(text), y, text, fg, bg)
end

function example_util.draw_anchor(anchor_x, anchor_y, text, fg, bg, offset_x, offset_y)
  local width, height = get_text_size(text)
  local x, y = resolve_rect(anchor_x, anchor_y, width, height, offset_x or 0, offset_y or 0)
  draw_text(x, y, text, fg, bg)
end
