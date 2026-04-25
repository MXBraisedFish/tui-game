function clone_items(items)
  local out = {}
  for i = 1, #items do
    out[i] = items[i]
  end
  return out
end

function clone_columns(columns)
  local out = {}
  for i = 1, #columns do
    out[i] = clone_items(columns[i])
  end
  return out
end
