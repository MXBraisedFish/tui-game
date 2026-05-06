local M = {}

function M.size(bytes)
  local value = tonumber(bytes) or 0
  local units = {"B", "KB", "MB", "GB", "TB"}
  local unit = 1

  while value >= 1024 and unit < #units do
    value = value / 1024
    unit = unit + 1
  end

  if unit == 1 then
    return tostring(math.floor(value)) .. " " .. units[unit]
  end
  return string.format("%.2f %s", value, units[unit])
end

return M
