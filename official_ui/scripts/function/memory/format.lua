local M = {}

local UNITS = {"B", "KB", "MB", "GB", "TB", "PB"}

function M.size(bytes)
  local value = tonumber(bytes) or 0
  local unit_index = 1
  while value >= 1024 and unit_index < #UNITS do
    value = value / 1024
    unit_index = unit_index + 1
  end

  if unit_index == 1 then
    return tostring(math.floor(value)) .. " " .. UNITS[unit_index]
  end
  return string.format("%.2f %s", value, UNITS[unit_index])
end

return M
