local M = {}

function M.tr(key)
  return translate(key)
end

function M.draw_text(x, y, text, fg, bg)
  canvas_draw_text(math.max(0, x - 1), math.max(0, y - 1), text, fg, bg)
end

function M.fill_rect(x, y, w, h, bg)
  if w <= 0 or h <= 0 then
    return
  end
  canvas_fill_rect(math.max(0, x - 1), math.max(0, y - 1), w, h, " ", nil, bg or "black")
end

function M.random_index(count)
  if count <= 1 then
    return 1
  end
  return random(count - 1) + 1
end

function M.format_duration(sec)
  sec = math.max(0, math.floor(tonumber(sec) or 0))
  local h = math.floor(sec / 3600)
  local m = math.floor((sec % 3600) / 60)
  local s = sec % 60
  return string.format("%02d:%02d:%02d", h, m, s)
end

function M.text_width(text)
  local ok, w = pcall(get_text_width, tostring(text or ""))
  if ok and type(w) == "number" then
    return w
  end
  return #tostring(text or "")
end

function M.wrap_words(text, max_width)
  text = tostring(text or "")
  if max_width <= 1 then
    return { text }
  end
  local lines = {}
  local current = ""
  local had_token = false
  for token in string.gmatch(text, "%S+") do
    had_token = true
    if current == "" then
      current = token
    else
      local candidate = current .. " " .. token
      if M.text_width(candidate) <= max_width then
        current = candidate
      else
        lines[#lines + 1] = current
        current = token
      end
    end
  end
  if not had_token then
    return { "" }
  end
  if current ~= "" then
    lines[#lines + 1] = current
  end
  return lines
end

local KEY_DISPLAY = {
  up = "↑",
  down = "↓",
  left = "←",
  right = "→",
  enter = "Enter",
  esc = "Esc",
  space = "Space",
  backspace = "Bksp",
  del = "Del",
  tab = "Tab",
  back_tab = "BTab",
}

local function display_key_name(key)
  key = tostring(key or "")
  if key == "" then return "" end
  local mapped = KEY_DISPLAY[key]
  if mapped ~= nil then return mapped end
  if #key == 1 then return string.upper(key) end
  if string.sub(key, 1, 1) == "f" and tonumber(string.sub(key, 2)) ~= nil then
    return string.upper(key)
  end
  return key
end

function M.key_label(action)
  if type(get_key) ~= "function" then
    return "[]"
  end
  local ok, info = pcall(get_key, action)
  if not ok or type(info) ~= "table" then
    return "[]"
  end
  if info[action] ~= nil and type(info[action]) == "table" then
    info = info[action]
  end
  local keys = info.key_user or info.key
  if type(keys) ~= "table" then
    keys = { keys }
  end
  local out = {}
  for i = 1, #keys do
    local label = display_key_name(keys[i])
    if label ~= "" then
      out[#out + 1] = "[" .. label .. "]"
    end
  end
  if #out == 0 then
    return "[]"
  end
  return table.concat(out, "/")
end

function M.replace_prompt_keys(text)
  text = tostring(text or "")
  text = string.gsub(text, "%[Y%]", M.key_label("confirm_yes"))
  text = string.gsub(text, "%[N%]", M.key_label("confirm_no"))
  text = string.gsub(text, "%[Q%]/%[ESC%]", M.key_label("quit_action"))
  return text
end

return M
