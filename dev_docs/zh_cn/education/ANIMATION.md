track 轨道
{
["0.3"] = {
"参数" = "值"
},
["1"] = { (数组是时间值而不是百分比值，并且必须使用字符串键)
"参数" = "值"
}
}

easing 插值函数
{
"参数" = function(now, start_time, end_time, start_value, end_value)
return 任意值
end （或者填写官方提供的插值函数）
}

常量：

STEP 步进

```lua
function(current_time, start_time, end_time, start_value, end_value)
	if current_time < end_time then
		return start_value
	else
		return end_value
	end
end
```

LINEAR 线性

```lua
function(current_time, start_time, end_time, start_value, end_value)
	local denom = end_time - start_time
  local p
  if denom == 0 then
    p = 1
  else
    p = math.min(1, math.max({ 0, (current_time - start_time) / denom }))
  end
	return start_value + ( end_value - start_value ) * p
end
```

EASE_IN_QUAD 缓入快出

```lua
function(current_time, start_time, end_time, start_value, end_value)
	local denom = end_time - start_time
  local p
  if denom == 0 then
    p = 1
  else
    p = math.min(1, math.max({ 0, (current_time - start_time) / denom }))
  end
	local e = p * p
	return start_value + ( end_value - start_value ) * e
end
```

EASE_OUT_QUAD 快入缓出

```lua
function(current_time, start_time, end_time, start_value, end_value)
	local denom = end_time - start_time
  local p
  if denom == 0 then
    p = 1
  else
    p = math.min(1, math.max({ 0, (current_time - start_time) / denom }))
  end
	local e = 1 - (1 - p) * (1 - p)
	return start_value + ( end_value - start_value ) * e
end
```

EASE_IN_OUT_CUBIC 慢快慢

```lua
function(current_time, start_time, end_time, start_value, end_value)
	local denom = end_time - start_time
  local p
  if denom == 0 then
    p = 1
  else
    p = math.min(1, math.max({ 0, (current_time - start_time) / denom }))
  end
	local e
	if p < 0.5 then
		e = 4 * p * p * p
	else
		e = 1 - ((-2 * p + 2) ^ 3) / 2
	end
	return start_value + ( end_value - start_value ) * e
end
```

EASE_IN_OUT_REVERSE_CUBIC 快慢快

```lua
function(current_time, start_time, end_time, start_value, end_value)
	local denom = end_time - start_time
  local p
  if denom == 0 then
    p = 1
  else
    p = math.min(1, math.max({ 0, (current_time - start_time) / denom }))
  end
	local e
    if p < 0.5 then
        local u = 2 * p
        e = 0.5 * (1 - (1 - u) * (1 - u) * (1 - u))
    else
        local u = 2 * p - 1
        e = 0.5 + 0.5 * u * u * u
    end
    return start_value + ( end_value - start_value ) * e
end
```
