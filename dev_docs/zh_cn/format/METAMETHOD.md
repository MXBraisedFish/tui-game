## `__index`

访问表里不存在的键时，去哪里继续找这个值。

### 触发条件

用 `table[key]` 或 `table.key` 读取表中不存在的键时触发。

### 后继值

- **函数**：将 `table` 和 `key` 作为参数顺序传入，用返回值作为结果。
- **表**：去另一个表继续查 `key`。

### 示例

```lua
local base = { x = 10 }

local obj1 = setmetatable { table = {}, metatable = { __index = base } }

debug.print { message = obj1.x }

local obj2 = setmetatable { table = {}, metatable = {
  __index = function(table, key)
    return "Don't have '" .. key .. "'"
  end
} }

debug.print { message = obj2.y }
```

输出：

```text
10
Don't have 'y'
```

---

## `__newindex`

当给表里不存在的键赋值时，把这个值实际写到哪去。

### 触发条件

用 `table[key] = value` 或 `table.key = value` 给表中不存在的键赋值时触发。

### 后继值

- **函数**：将 `table`、`key` 和 `value` 作为参数顺序传入。
- **表**：去另一个表给指定的键赋值。

### 示例

```lua
local base = {}

local obj1 = setmetatable { table = {}, metatable = { __newindex = base } }

obj1.a = 1
debug.print { message = base.a }

local obj2 = setmetatable { table = {}, metatable = { 
  __newindex = function(table, key, value)
    debug.print { message = "Don't have '" .. key .. "'" }
  end
} }

obj2.b = "test"
```

输出：

```text
1
Don't have 'b'
```

---

## `__call`

让表能像函数一样被调用，调用时执行指定的处理逻辑。

### 触发条件

把表当成函数 `table(value...)` 时触发。

### 后继值

- **函数**：将 `tabel` 和 `value...` 作为参数顺序传入。

### 示例

```lua
local add = setmetatable { table = {},  metatable = { 
  __call = function(self, a, b) 
    return a + b 
  end 
} }

debug.print { message = add(3, 4)}
```

输出：

```text
7
```

---

## `__add`

定义两个值用 `+` 相加时的行为。

### 触发条件

对两个值使用 `+` 运算符时触发。

### 后继值

- **函数**：`value1 + value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __add = function(value1, value2) 
    return value1.v + value2.v 
  end 
}

local x = setmetatable { table = { v = 1 }, metatable = mt }
local y = { v = 2 }

debug.print { message = x + y }
```

输出：

```text
3
```

### 额外补充

- 左操作数和右操作数中，只要其一的元表包含 `__add` 即可。
- `__add` 调用顺序为先查左、后查右。

---

## `__sub`

定义两个值用 `-` 相减时的行为。

### 触发条件

对两个值使用 `-` 运算符时触发。

### 后继值

- **函数**：`value1 - value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __sub = function(value1, value2) 
    return value1.v - value2.v 
  end 
}

local x = setmetatable { table = { v = 1 }, metatable = mt }
local y = { v = 2 }

debug.print { message = x - y }
```

输出：

```text
-1
```

### 额外补充

- 左操作数和右操作数中，只要其一的元表包含 `__sub` 即可。
- `__sub` 调用顺序为先查左、后查右。

---

## `__mul`

定义两个值用 `*` 相乘时的行为。

### 触发条件

对两个值使用 `*` 运算符时触发。

### 后继值

- **函数**：`value1 * value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __mul = function(value1, value2) 
    return value1.v * value2.v 
  end 
}

local x = setmetatable { table = { v = 1 }, metatable = mt }
local y = { v = 2 }

debug.print { message = x * y }
```

输出：

```text
2
```

### 额外补充

- 左操作数和右操作数中，只要其一的元表包含 `__mul` 即可。
- `__mul` 调用顺序为先查左、后查右。

---

## `__div`

定义两个值用 `/` 相除时的行为。

### 触发条件

对两个值使用 `/` 运算符时触发。

### 后继值

- **函数**：`value1 / value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __div = function(value1, value2) 
    return value1.v / value2.v 
  end 
}

local x = setmetatable { table = { v = 10 }, metatable = mt }
local y = { v = 4 }

debug.print { message = x / y }
```

输出：

```text
2.5
```

### 额外补充

- 左操作数和右操作数中，只要其一的元表包含 `__div` 即可。
- `__div` 调用顺序为先查左、后查右。

---

## `__mod`

定义两个值用 `%` 取模时的行为。

### 触发条件

对两个值使用 `%` 运算符时触发。

### 后继值

- **函数**：`value1 % value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __mod = function(value1, value2) 
    return value1.v % value2.v 
  end 
}

local x = setmetatable { table = { v = 10 }, metatable = mt }
local y = { v = 3 }

debug.print { message = x % y }
```

输出：

```text
1
```

### 额外补充

- 左操作数和右操作数中，只要其一的元表包含 `__mod` 即可。
- `__mod` 调用顺序为先查左、后查右。

---

## `__pow`

定义两个值用 `^` 求幂时的行为。

### 触发条件

对两个值使用 `^` 运算符时触发。

### 后继值

- **函数**：`value1 ^ value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __pow = function(value1, value2) 
    return value1.v ^ value2.v 
  end 
}

local x = setmetatable { table = { v = 2 }, metatable = mt }
local y = { v = 3 }

debug.print { message = x ^ y }
```

输出：

```text
8.0
```

### 额外补充

- 左操作数和右操作数中，只要其一的元表包含 `__pow` 即可。
- `__pow` 调用顺序为先查左、后查右。

---

## `__idiv`

定义两个值用 `//` 整除时的行为。

### 触发条件

对两个值使用 `//` 运算符时触发。

### 后继值

- **函数**：`value1 // value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __idiv = function(value1, value2) 
    return value1.v // value2.v 
  end 
}

local x = setmetatable { table = { v = 10 }, metatable = mt }
local y = { v = 3 }

debug.print { message = x // y }
```

输出：

```text
3
```

### 额外补充

- 左操作数和右操作数中，只要其一的元表包含 `__idiv` 即可。
- `__idiv` 调用顺序为先查左、后查右。

---

## `__band`

定义两个值用 `&` 进行按位与运算时的行为。

### 触发条件

对两个值使用 `&` 运算符时触发。

### 后继值

- **函数**：`value1 & value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __band = function(value1, value2) 
    return value1.v & value2.v 
  end 
}

local x = setmetatable { table = { v = 6 }, metatable = mt }
local y = { v = 3 }

debug.print { message = x & y }
```

输出：

```text
2
```

### 额外补充

- 左操作数和右操作数中，只要其一的元表包含 `__band` 即可。
- `__band` 调用顺序为先查左、后查右。

---

## `__bor`

定义两个值用 `|` 进行按位或运算时的行为。

### 触发条件

对两个值使用 `|` 运算符时触发。

### 后继值

- **函数**：`value1 | value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __bor = function(value1, value2) 
    return value1.v | value2.v 
  end 
}

local x = setmetatable { table = { v = 6 }, metatable = mt }
local y = { v = 3 }

debug.print { message = x | y }
```

输出：

```text
7
```

### 额外补充

- 左操作数和右操作数中，只要其一的元表包含 `__bor` 即可。
- `__bor` 调用顺序为先查左、后查右。

---

## `__bxor`

定义两个值用 `~` 进行按位异或运算时的行为。

### 触发条件

对两个值使用 `~` 运算符时触发。

### 后继值

- **函数**：`value1 ~ value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __bxor = function(value1, value2) 
    return value1.v ~ value2.v 
  end 
}

local x = setmetatable { table = { v = 6 }, metatable = mt }
local y = { v = 3 }

debug.print { message = x ~ y }
```

输出：

```text
5
```

### 额外补充

- 左操作数和右操作数中，只要其一的元表包含 `__bxor` 即可。
- `__bxor` 调用顺序为先查左、后查右。

---

## `__shl`

定义两个值用 `<<` 进行左移运算时的行为。

### 触发条件

对两个值使用 `<<` 运算符时触发。

### 后继值

- **函数**：`value1 << value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __shl = function(value1, value2) 
    return value1.v << value2.v 
  end 
}

local x = setmetatable { table = { v = 3 }, metatable = mt }
local y = { v = 2 }

debug.print { message = x << y }
```

输出：

```text
12
```

### 额外补充

- 左操作数和右操作数中，只要其一的元表包含 `__shl` 即可。
- `__shl` 调用顺序为先查左、后查右。

---

## `__shr`

定义两个值用 `>>` 进行右移运算时的行为。

### 触发条件

对两个值使用 `>>` 运算符时触发。

### 后继值

- **函数**：`value1 >> value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __shr = function(value1, value2) 
    return value1.v >> value2.v 
  end 
}

local x = setmetatable { table = { v = 12 }, metatable = mt }
local y = { v = 2 }

debug.print { message = x >> y }
```

输出：

```text
3
```

### 额外补充

- 左操作数和右操作数中，只要其一的元表包含 `__shr` 即可。
- `__shr` 调用顺序为先查左、后查右。

---

## `__unm`

定义一个值用 `-` 取负时的行为。

### 触发条件

对一个值使用 `-` 运算符时触发。

### 后继值

- **函数**：`-value`，将 `value` 作为参数传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __unm = function(value) 
    return -value.v 
  end 
}

local x = setmetatable { table = { v = 5 }, metatable = mt }

debug.print { message = -x }
```

输出：

```text
-5
```

---

## `__bnot`

定义一个值用 `~` 进行按位取反运算时的行为。

### 触发条件

对一个值使用 `~` 运算符时触发。

### 后继值

- **函数**：`~value`，将 `value` 作为参数传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __bnot = function(value) 
    return ~value.v 
  end 
}

local x = setmetatable { table = { v = 0 }, metatable = mt }

debug.print { message = ~x }
```

输出：

```text
-1
```

---

## `__len`

定义一个值用 `#` 求长度时的行为。

### 触发条件

对一个值使用 `#` 运算符时触发。

### 后继值

- **函数**：`#value`，将 `value` 作为参数传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __len = function(value) 
    return value.n 
  end 
}

local x = setmetatable { table = { n = 42 }, metatable = mt }

debug.print { message = #x }
```

输出：

```text
42
```

---

## `__concat`

定义两个值用 `..` 连接时的行为。

### 触发条件

对两个值使用 `..` 运算符时触发。

### 后继值

- **函数**：`value1 .. value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __concat = function(value1, value2) 
    return value1.v .. value2.v 
  end 
}

local x = setmetatable { table = { v = "a" }, metatable = mt }
local y = { v = "b" }

debug.print { message = x .. y }
```

输出：

```text
ab
```

### 额外补充

- 左操作数和右操作数中，只要其一的元表包含 `__concat` 即可。
- `__concat` 调用顺序为先查左、后查右。

---

## `__eq`

定义两个表用 `==` 比较是否相等时的行为。

### 触发条件

对两个表使用 `==` 运算符时触发。

### 后继值

- **函数**：`table1 == table2`，将 `table1` 和 `table2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __eq = function(table1, table2) 
    return table1.v == table2.v 
  end 
}

local x = setmetatable { table = { v = 1 }, metatable = mt }
local y = { v = 1 }

debug.print { message = x == y }
```

输出：

```text
true
```

### 额外补充

- 返回值会被转换为布尔值。
- 左操作数和右操作数中，只要其一的元表包含 `__eq` 即可。
- `__eq` 调用顺序为先查左、后查右。

---

## `__lt`

定义两个值用 `<` 比较是否小于时的行为。

### 触发条件

对两个值使用 `<` 运算符时触发。

### 后继值

- **函数**：`value1 < value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __lt = function(value1, value2) 
    return value1.v < value2.v 
  end 
}

local x = setmetatable { table = { v = 1 }, metatable = mt }
local y = { v = 2 }

debug.print { message = x < y }
```

输出：

```text
true
```

### 额外补充

- 返回值会被转换为布尔值。
- 左操作数和右操作数中，只要其一的元表包含 `__lt` 即可。
- `__lt` 调用顺序为先查左、后查右。

---

## `__le`

定义两个值用 `<=` 比较是否小于等于时的行为。

### 触发条件

对两个值使用 `<=` 运算符时触发。

### 后继值

- **函数**：`value1 <= value2`，将 `value1` 和 `value2` 作为参数顺序传入，用返回值作为结果。

### 示例

```lua
local mt = { 
  __le = function(value1, value2) 
    return value1.v <= value2.v 
  end 
}

local x = setmetatable { table = { v = 2 }, metatable = mt }
local y = { v = 2 }

debug.print { message = x <= y }
```

输出：

```text
true
```

### 额外补充

- 返回值会被转换为布尔值。
- 左操作数和右操作数中，只要其一的元表包含 `__le` 即可。
- `__le` 调用顺序为先查左、后查右。

---

## `__gc`

在对象被 GC 回收时执行指定的逻辑。

### 触发条件

当这个对象变成被 GC 回收时触发。

### 后继值

- **函数**：将对象自己作为参数传入。

### 示例

```lua
local mt = { 
  __gc = function(obj) 
    debug.print { message = "collected" } 
  end 
}

do 
  local x = setmetatable { table = {}, metatable = mt } -- 离开作用于被 GC 回收
end
```

输出：

```text
collected
```

---

## `__close`

在变量离开作用域时自动执行清理逻辑。

### 触发条件

把一个值声明为 `<close>` 变量后，当它离开作用域时触发。

### 后继值

- **函数**：如果该作用域正常执行，将对象自己作为参数传入；如果该作用域抛出异常，将对象自己和错误信息作为参数顺序传入。。

### 示例

```lua
local mt = { 
  __close = function(obj, err)
    if err == nil then
      debug.print { message = "closed" }
    else
      debug.print { message = "error!!!" }
    end
  end 
}

do
  local x <close> = setmetatable { table = {}, metatable = mt }
end

do
  local y <close> = setmetatable { table = {}, metatable = mt }
  debug.assert { value = false }
end
```

输出：

```text
closed
error!!!
[运行][Lua][yyyy-mm-dd hh:mm:ss.ms][错误] Lua game session 'test.package' failed during Callback (Update): runtime error: assertion failed
stack traceback: [Error Message]
【脚本终止运行】
```

---

## `__mode`

把表变成弱表，让表里的键或值在不被其他地方引用时可以被垃圾回收掉。

### 触发条件

设置元表的 `__mode` 字段后，由 GC 处理器按照弱表规则回收表内容。

### 后继值

- **字符串**：`"k"` 表示弱键，`"v"` 表示弱值，`"kv"` 表示键值均弱。

### 示例

```lua
local i = 0

function Update(dt)
  if i == 0 then
    local t = setmetatable { table = {}, metatable = { __mode = "v" } }
    t["name"] = {}
    i = 1
  elseif i == 1 then
    debug.print { message = type(t["name"]) }
    i = 2
  end
end

```

输出：

```text
nil
```

---

## `__metatable`

保护元表。

### 触发条件

表被调用 `getmetatable` 或 `setmetatable` 时触发。

### 后继值

- **任意值**：`getmetatable` 返回该值；`setmetatable` 会报错阻止修改。

### 示例

```lua
local t = setmetatable { table = {}, metatable = { __metatable = "locked" } }
debug.print { message = getmetatable(t) }
```

输出：

```text
locked
```

---

## `__name`

给表设置一个自定义类型名。

### 触发条件

表被调用 `tostring` 或抛出异常时使用该值。

### 后继值

- **字符串**：作为表的可读类型名。

### 示例

```lua
local t = setmetatable { table = {}, metatable = { __name = "Type" } }
debug.print { message = tostring(t) }
```

输出：

```text
Type: 0x...
```
