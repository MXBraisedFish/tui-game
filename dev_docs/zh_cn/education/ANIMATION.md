track 轨道
{
  [1] = {
    timestamp -> float 时间点
    key = { -> 当前时间点记录的参数
      "参数" = "值" -> 参数名和参数内容
    }
  }
  [2] = {
    timestamp -> float 时间点
    key = { -> 当前时间点记录的参数
      "参数" = "值" -> 参数名和参数内容
    }
  }
}

easing 插值函数（也可以直接写function / const-animation）
{
  "参数1" = function / const-animation, -> 表示所有插值区段共用一个插值函数
  "参数2" = {
    [1] = function / const-animation -> 表示哪个区间使用哪个插值函数，插值区段取决于该值的时间点闭合段数
    [2] = ...
  }
}

default_function 默认插值函数（也可以直接写function / const-animation）
{
  "参数1" = function / const-animation
  "参数2" = function / const-animation
}

函数形式：

function(current_time, start_time, end_time, start_value, end_value)
  return 任意值
end

current_time -> 当前已经持续到的时间
start_time -> 当前插值区段开始时间
end_time -> 当前插值区段结束时间
start_value -> 当前插值区段开始值
end_value -> 当前插值区段结束值

返回值格式：
{
  "{key}" = 当前时间点插值函数计算出来的任意值
}
单独获取指定参数：
直接返回当前时间点插值函数计算出来的任意值

特殊处理：
1. 对于参数 `track` 中的数组，应当按照时间点的递增顺序排列
  - 遇到顺序错误（例如大时间点在上，小时间点在下），程序会自动重排序
  - 遇到时间点相等的，保留首个被扫的轨道内容，其他的全部丢弃
  - 如果时间点值非法（例如小于0，超过持续时间，类型错误等）会直接抛出错误
  - 对于存在键未闭合阶段节点缺少的值，优先使用前者补齐，否则使用后者
    例：现在有这样一个轨道
      {
        [1] = {
          timestamp = 0
          key = {

            b = 3

          }
        },
        [2] = {
          timestamp = 1
          key = {
            a = 1

            c = 4
          }
        },
        [3] = {
          timestamp = 2
          key = {
            a = 2
            b = 1

          }
        },
        [4] = {
          timestamp = 3
          key = {



          }
        }
      }
      当前轨道中三个值abc分别出现以下情况：
      a -> 仅在2-3时间段插值闭合
      b -> 仅在1-3时间段插值闭合
      c -> 仅在2时间点定义值
      按照缺少节点的值采用使用前者补齐否则使用后者原则
      a -> 在1时间点补全为2时间点的值（无前者，使用后者补全），在4时间点补全为3时间点的值（有前者使用前者补全）
      b -> 在4时间点补全为3时间点的值（有前者使用前者补全），2时间点被1时间点和3时间点闭合，因此无需补全
      c -> 在1时间点（无前者，使用后者补全），3时间点（有前者使用前者补全）和4时间点（有前者使用前者补全）补全为2时间点的值
      最终轨道会被补全为：
      {
        [1] = {
          timestamp = 0
          key = {
            a = 1
            b = 3
            c = 4
          }
        },
        [2] = {
          timestamp = 1
          key = {
            a = 1
            （此处b不补全，由1和3闭合为一个插值区段）
            c = 4
          }
        },
        [3] = {
          timestamp = 2
          key = {
            a = 2
            b = 1
            c = 4（注意，c不会因为4时间点补全后和2时间点闭合就不补全3时间点，缺少尾包裹会被全补全，而且实际顺序是3时间点先被补全，因此4时间点实际补全的是3时间点补全后的值）
          }
        },
        [4] = {
          timestamp = 3
          key = {
            a = 2
            b = 1
            c = 4
          }
        }
      }
      在区间上
      a具有三个插值区段：1-2,2-3,3-4
      b具有两个插值区段：1-3,3-4
      c具有三个插值区段：1-2,2-3,3-4

2. 对于参数 `easing` 中的对象插值函数，是一个选填选项或者直接填写插值函数
  - 如果不传递任何值，则默认使用`animation.STEP`作为插值函数
  - 如果直接传递一个函数或者常量，则表示所有字段共用一个插值函数
  - 如果传递表，则表示要求字段使用指定的插值函数
  - 对于传递的表方式，如果有注册的字段没有注册插值函数，或插值区段缺少对应的插值函数，则均默认使用`animation.STEP`，可在参数`default_function`来指定缺少插值函数时使用的默认函数
  - 对于插值函数出现错误依旧会抛出错误
  - 对于不存在的区段所设置的插值函数会被忽略
  - 存在对相同区段设置的插值函数会保留第一个，其余的均被忽略


常量所对应的函数：

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
