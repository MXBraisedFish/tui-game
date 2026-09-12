## `__index`

当访问表中不存在的键时提供查找机制。

### 触发条件

执行 `t[k]`，且 `t` 中不存在键 `k` 时触发。

### 后继值

- **函数**：以 `t` 和 `k` 为参数调用，返回结果作为 `t[k]` 的值。
- **表（或其他有 `__index` 的值）**：对该后继值继续进行常规索引 `后继值[k]`。

### 示例

```lua
local base = { x = 10 }
local obj = setmetatable({}, { __index = base })
print(obj.x)
print(obj.y)
```

输出：

```text
10
nil
```

---

## `__newindex`

当向表中不存在的键赋值时提供写入机制。

### 触发条件

执行 `t[k] = v`，且 `t` 中不存在键 `k` 时触发。

### 后继值

- **函数**：以 `t`、`k`、`v` 为参数调用，返回值被忽略。
- **表**：对该后继值继续进行常规赋值 `后继值[k] = v`。

### 示例

```lua
local log = {}
local obj = setmetatable({}, { __newindex = function(t, k, v) log[#log+1] = k .. "=" .. v end })
obj.a = 1
print(obj.a, log[1])
```

输出：

```text
nil a=1
```

---

## `__call`

使非函数值可以像函数一样被调用。

### 触发条件

执行 `f(...)`，且 `f` 不是函数时触发。

### 后继值

- **函数**：以 `f` 为首个参数，后接原本调用的全部参数进行调用；返回值全部作为调用结果返回。

### 示例

```lua
local t = setmetatable({}, { __call = function(self, a, b) return a + b end })
print(t(3, 4))
```

输出：

```text
7
```

---

## `__add`

重载加法运算符。

### 触发条件

执行 `a + b`，且 `a` 和 `b` 不是同时为数字时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值作为运算结果。

### 示例

```lua
local mt = { __add = function(a, b) return a.v + b.v end }
local x = setmetatable({ v = 1 }, mt)
local y = setmetatable({ v = 2 }, mt)
print(x + y)
```

输出：

```text
3
```

---

## `__sub`

重载减法运算符。

### 触发条件

执行 `a - b`，且 `a` 和 `b` 不是同时为数字时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值作为运算结果。

### 示例

```lua
local mt = { __sub = function(a, b) return a.v - b.v end }
local x = setmetatable({ v = 5 }, mt)
local y = setmetatable({ v = 3 }, mt)
print(x - y)
```

输出：

```text
2
```

---

## `__mul`

重载乘法运算符。

### 触发条件

执行 `a * b`，且 `a` 和 `b` 不是同时为数字时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值作为运算结果。

### 示例

```lua
local mt = { __mul = function(a, b) return a.v * b.v end }
local x = setmetatable({ v = 3 }, mt)
local y = setmetatable({ v = 4 }, mt)
print(x * y)
```

输出：

```text
12
```

---

## `__div`

重载除法运算符。

### 触发条件

执行 `a / b`，且 `a` 和 `b` 不是同时为数字时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值作为运算结果。

### 示例

```lua
local mt = { __div = function(a, b) return a.v / b.v end }
local x = setmetatable({ v = 10 }, mt)
local y = setmetatable({ v = 4 }, mt)
print(x / y)
```

输出：

```text
2.5
```

---

## `__mod`

重载取模运算符。

### 触发条件

执行 `a % b`，且 `a` 和 `b` 不是同时为数字时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值作为运算结果。

### 示例

```lua
local mt = { __mod = function(a, b) return a.v % b.v end }
local x = setmetatable({ v = 10 }, mt)
local y = setmetatable({ v = 3 }, mt)
print(x % y)
```

输出：

```text
1
```

---

## `__pow`

重载幂运算符。

### 触发条件

执行 `a ^ b`，且 `a` 和 `b` 不是同时为数字时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值作为运算结果。

### 示例

```lua
local mt = { __pow = function(a, b) return a.v ^ b.v end }
local x = setmetatable({ v = 2 }, mt)
local y = setmetatable({ v = 3 }, mt)
print(x ^ y)
```

输出：

```text
8.0
```

---

## `__idiv`

重载整除运算符。

### 触发条件

执行 `a // b`，且 `a` 和 `b` 不是同时为数字时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值作为运算结果。

### 示例

```lua
local mt = { __idiv = function(a, b) return a.v // b.v end }
local x = setmetatable({ v = 10 }, mt)
local y = setmetatable({ v = 3 }, mt)
print(x // y)
```

输出：

```text
3
```

---

## `__band`

重载按位与运算符。

### 触发条件

执行 `a & b`，且 `a` 和 `b` 不是同时为整数时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值作为运算结果。

### 示例

```lua
local mt = { __band = function(a, b) return a.v & b.v end }
local x = setmetatable({ v = 6 }, mt)
local y = setmetatable({ v = 3 }, mt)
print(x & y)
```

输出：

```text
2
```

---

## `__bor`

重载按位或运算符。

### 触发条件

执行 `a | b`，且 `a` 和 `b` 不是同时为整数时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值作为运算结果。

### 示例

```lua
local mt = { __bor = function(a, b) return a.v | b.v end }
local x = setmetatable({ v = 6 }, mt)
local y = setmetatable({ v = 3 }, mt)
print(x | y)
```

输出：

```text
7
```

---

## `__bxor`

重载按位异或运算符。

### 触发条件

执行 `a ~ b`，且 `a` 和 `b` 不是同时为整数时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值作为运算结果。

### 示例

```lua
local mt = { __bxor = function(a, b) return a.v ~ b.v end }
local x = setmetatable({ v = 6 }, mt)
local y = setmetatable({ v = 3 }, mt)
print(x ~ y)
```

输出：

```text
5
```

---

## `__shl`

重载左移运算符。

### 触发条件

执行 `a << b`，且 `a` 和 `b` 不是同时为整数时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值作为运算结果。

### 示例

```lua
local mt = { __shl = function(a, b) return a.v << b.v end }
local x = setmetatable({ v = 3 }, mt)
local y = setmetatable({ v = 2 }, mt)
print(x << y)
```

输出：

```text
12
```

---

## `__shr`

重载右移运算符。

### 触发条件

执行 `a >> b`，且 `a` 和 `b` 不是同时为整数时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值作为运算结果。

### 示例

```lua
local mt = { __shr = function(a, b) return a.v >> b.v end }
local x = setmetatable({ v = 12 }, mt)
local y = setmetatable({ v = 2 }, mt)
print(x >> y)
```

输出：

```text
3
```

---

## `__unm`

重载一元负运算符。

### 触发条件

执行 `-a`，且 `a` 不是数字时触发。

### 后继值

- **函数**：以 `a` 为参数调用，返回值作为运算结果。

### 示例

```lua
local mt = { __unm = function(a) return -a.v end }
local x = setmetatable({ v = 5 }, mt)
print(-x)
```

输出：

```text
-5
```

---

## `__bnot`

重载按位取反运算符。

### 触发条件

执行 `~a`，且 `a` 不是整数时触发。

### 后继值

- **函数**：以 `a` 为参数调用，返回值作为运算结果。

### 示例

```lua
local mt = { __bnot = function(a) return ~a.v end }
local x = setmetatable({ v = 0 }, mt)
print(~x)
```

输出：

```text
-1
```

---

## `__len`

重载长度运算符。

### 触发条件

执行 `#a`，且 `a` 不是字符串时触发。

### 后继值

- **函数**：以 `a` 为参数调用，返回值作为长度。

### 示例

```lua
local mt = { __len = function(a) return a.n end }
local x = setmetatable({ n = 42 }, mt)
print(#x)
```

输出：

```text
42
```

---

## `__concat`

重载连接运算符。

### 触发条件

执行 `a .. b`，且 `a` 和 `b` 不是同时为字符串或数字时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值作为连接结果。

### 示例

```lua
local mt = { __concat = function(a, b) return a.v .. b.v end }
local x = setmetatable({ v = "a" }, mt)
local y = setmetatable({ v = "b" }, mt)
print(x .. y)
```

输出：

```text
ab
```

---

## `__eq`

重载相等比较运算符。

### 触发条件

执行 `a == b`，且 `a` 和 `b` 都是表或都是完整 userdata，且不是原始相等时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值被转换为布尔值。

### 示例

```lua
local mt = { __eq = function(a, b) return a.v == b.v end }
local x = setmetatable({ v = 1 }, mt)
local y = setmetatable({ v = 1 }, mt)
print(x == y)
```

输出：

```text
true
```

---

## `__lt`

重载小于比较运算符。

### 触发条件

执行 `a < b`，且 `a` 和 `b` 不是同时为数字或同时为字符串时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值被转换为布尔值。

### 示例

```lua
local mt = { __lt = function(a, b) return a.v < b.v end }
local x = setmetatable({ v = 1 }, mt)
local y = setmetatable({ v = 2 }, mt)
print(x < y)
```

输出：

```text
true
```

---

## `__le`

重载小于等于比较运算符。

### 触发条件

执行 `a <= b`，且 `a` 和 `b` 不是同时为数字或同时为字符串时触发。

### 后继值

- **函数**：以 `a`、`b` 为参数调用，返回值被转换为布尔值。

### 示例

```lua
local mt = { __le = function(a, b) return a.v <= b.v end }
local x = setmetatable({ v = 2 }, mt)
local y = setmetatable({ v = 2 }, mt)
print(x <= y)
```

输出：

```text
true
```

---

## `__gc`

在对象被垃圾回收时作为终结器调用。

### 触发条件

带 `__gc` 元方法的表或 userdata 被垃圾回收器判定为死亡时触发。

### 后继值

- **函数**：以对象自身为唯一参数调用。

### 示例

```lua
local mt = { __gc = function(o) print("collected") end }
do local x = setmetatable({}, mt) end
collectgarbage()
```

输出：

```text
collected
```

---

## `__close`

在 to-be-closed 变量离开作用域时调用。

### 触发条件

变量声明时带有 `<close>` 标记，其作用域退出时触发。

### 后继值

- **函数**：以对象和错误信息（或 nil）为参数调用。

### 示例

```lua
local mt = { __close = function(o, err) print("closed") end }
do
  local x <close> = setmetatable({}, mt)
end
```

输出：

```text
closed
```

---

## `__mode`

控制表的弱引用行为。

### 触发条件

设置元表的 `__mode` 字段时，不是“触发”而是告知垃圾回收器该表的弱引用模式。

### 后继值

- **字符串**：`"k"` 表示弱键，`"v"` 表示弱值，`"kv"` 表示键值均弱。

### 示例

```lua
local weak = setmetatable({}, { __mode = "v" })
local obj = {}
weak[1] = obj
obj = nil
collectgarbage()
print(weak[1])
```

输出：

```text
nil
```

---

## `__metatable`

保护元表不被访问或修改。

### 触发条件

对设置了 `__metatable` 字段的对象调用 `getmetatable` 或 `setmetatable` 时触发保护行为。

### 后继值

- **任意值**：`getmetatable` 返回该值；`setmetatable` 会报错阻止修改。

### 示例

```lua
local t = setmetatable({}, { __metatable = "locked" })
print(getmetatable(t))
```

输出：

```text
locked
```

---

## `__name`

供 `tostring` 和错误信息使用的类型名称。

### 触发条件

元表中包含字符串类型的 `__name` 字段时，`tostring` 或错误消息可能使用该名称。

### 后继值

- **字符串**：作为对象的可读类型名。

### 示例

```lua
local t = setmetatable({}, { __name = "MyType" })
print(tostring(t))
```

输出：

```text
MyType: 0x...
```