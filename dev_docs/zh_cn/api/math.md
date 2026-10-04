# math 库

`math` 提供数学常量和计算方法。浮点数计算可能产生精度误差。

---

# 目录

## 常量

| 常量                  | 说明                        | 定位                                      |
| ------------------- | ------------------------- | --------------------------------------- |
| `POSITIVE_INFINITE` | 正无穷                       | [POSITIVE_INFINITE](#positive_infinite) |
| `PI`                | 圆周率 π                     | [PI](#pi)                               |
| `E`                 | 自然常数 e                    | [E](#e)                                 |
| `NEGATIVE_INFINITE` | 负无穷                       | [NEGATIVE_INFINITE](#negative_infinite) |
| `DEG`               | 弧度转角度系数                   | [DEG](#deg)                             |
| `RAD`               | 角度转弧度系数                   | [RAD](#rad)                             |
| `MAX_INTEGER`       | 最大整数 `2^63-1`             | [MAX_INTEGER](#max_integer)             |
| `MIN_INTEGER`       | 最小整数 `-2^63`              | [MIN_INTEGER](#min_integer)             |
| `INFINITE`          | 正无穷，POSITIVE_INFINITE 的别名 | [INFINITE](#infinite)                   |

## 方法

| 方法 | 说明 | 定位 |
| --- | --- | --- |
| `abs` | 计算绝对值 | [abs](#abs) |
| `ceil` | 向上取整 | [ceil](#ceil) |
| `floor` | 向下取整 | [floor](#floor) |
| `round` | 四舍五入到最近的整数 | [round](#round) |
| `round_to` | 按指定位数四舍五入 | [round_to](#round_to) |
| `fmod` | 计算取模（余数） | [fmod](#fmod) |
| `pow` | 计算幂运算 $x^y$ | [pow](#pow) |
| `exp` | 计算 $e^{value}$ | [exp](#exp) |
| `log` | 计算指定底数的对数 | [log](#log) |
| `lg` | 计算以 10 为底的对数 | [lg](#lg) |
| `ln` | 计算以 e 为底的对数 | [ln](#ln) |
| `sqrt` | 计算平方根 | [sqrt](#sqrt) |
| `ldexp` | 计算 $x \times 2^{exp}$ | [ldexp](#ldexp) |
| `frexp` | 将数值分解为尾数与二进制指数 | [frexp](#frexp) |
| `sin` | 计算正弦（弧度制） | [sin](#sin) |
| `cos` | 计算余弦（弧度制） | [cos](#cos) |
| `tan` | 计算正切（弧度制） | [tan](#tan) |
| `asin` | 计算反正弦（弧度制） | [asin](#asin) |
| `acos` | 计算反余弦（弧度制） | [acos](#acos) |
| `atan` | 计算反正切（弧度制） | [atan](#atan) |
| `atan2` | 计算反正切（弧度制） | [atan2](#atan2) |
| `deg` | 将弧度转换为角度 | [deg](#deg) |
| `rad` | 将角度转换为弧度 | [rad](#rad) |
| `normalize_angle` | 将角度归一化到 `[0, 360)` 区间 | [normalize_angle](#normalize_angle) |
| `max` | 返回一组数中的最大值 | [max](#max) |
| `min` | 返回一组数中的最小值 | [min](#min) |
| `modf` | 分离数值的整数部分与小数部分 | [modf](#modf) |
| `tointeger` | 将数值精确转换为整数 | [tointeger](#tointeger) |
| `type` | 返回数值的类型名 | [type](#type) |
| `ult` | 以无符号整数比较两个整数 | [ult](#ult) |
| `approx_equal` | 以指定误差比较两个数字是否相等 | [approx_equal](#approx_equal) |
| `percent` | 计算百分比 $\frac{value}{total}$ | [percent](#percent) |
| `factorial` | 计算阶乘 $n!$ | [factorial](#factorial) |
| `combination` | 计算组合数 $C^n_k$ | [combination](#combination) |


---

# 常量

## `PI`

圆周率 π。

### 调用

```lua
math.PI
```

### 可用于

- 任意

### 示例

```lua
debug.print(tostring(math.PI))
```

**输出：**

```lua

```

### 等值

```text
3.141592653589793
```

---

## `E`

自然常数 e。

### 调用

```lua
math.E
```

### 可用于

- 数学比较。

### 示例

```lua
debug.print(tostring(math.E))
```

**输出：**

```lua

```

### 等值

```text
2.718281828459045
```

---

## `POSITIVE_INFINITE`

正无穷。

### 调用

```lua
math.POSITIVE_INFINITE
```

### 可用于

- 数学比较。

### 示例

```lua
debug.print(tostring(math.POSITIVE_INFINITE > math.MAX_INTEGER))
```

**输出：**

```lua

```

### 等值

```text
1 / 0
```

### 额外说明

- 该值大于所有有限数。
- 不可用于计算。

---

## `NEGATIVE_INFINITE`

负无穷。

### 调用

```lua
math.NEGATIVE_INFINITE
```

### 可用于

- 任意

### 示例

```lua
debug.print(tostring(math.NEGATIVE_INFINITE < math.MIN_INTEGER))
```

**输出：**

```lua

```

### 等值

```text
-1 / 0
```

### 额外说明

- 该值小于所有有限数。
- 不可用于计算。

---

## `DEG`

弧度转角度系数，`180 / π`。

### 调用

```lua
math.DEG
```

### 可用于

- 任意

### 示例

```lua
debug.print(tostring(math.DEG))
```

**输出：**

```lua

```

### 等值

```text
57.29577951308232
```

---

## `RAD`

角度转弧度系数，`π / 180`。

### 调用

```lua
math.RAD
```

### 可用于

- 任意

### 示例

```lua
debug.print(tostring(math.RAD))
```

**输出：**

```lua

```

### 等值

```text
0.017453292519943295
```

---

## `MAX_INTEGER`

最大可表示的整数 `2^63-1`。

### 调用

```lua
math.MAX_INTEGER
```

### 可用于

- 任意

### 示例

```lua
debug.print(tostring(math.MAX_INTEGER))
```

**输出：**

```lua

```

### 等值

```text
9223372036854775807
```

---

## `MIN_INTEGER`

最小可表示的整数 `-2^63`。

### 调用

```lua
math.MIN_INTEGER
```

### 可用于

- 任意

### 示例

```lua
debug.print(tostring(math.MIN_INTEGER))
```

**输出：**

```lua

```

### 等值

```text
-9223372036854775808
```

---

## `INFINITE`

正无穷，POSITIVE_INFINITE 的别名。

### 调用

```lua
math.INFINITE
```

### 可用于

- 任意

### 示例

```lua
debug.print(math.INFINITE == math.POSITIVE_INFINITE)
```

**输出：**

```lua
```

### 等值

```text
math.POSITIVE_INFINITE
```

### 额外说明

- 与 `math.POSITIVE_INFINITE` 的值相同。

---

# 方法

## `abs`

计算绝对值。

### 调用

```lua
math.abs
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明             |
| ------- | ----- | ---------------- |
| `value` | float | 要取绝对值的数值 |

## 返回值

返回一个值。

| 类型  | 说明   |
| ----- | ------ |
| float | 绝对值 |

### 示例

```lua
local n = math.abs(-5.20)
debug.print(tostring(n))
```

**输出：**

```lua

```

### 额外说明

运行时数值参数和计算结果必须是有限数；无穷常量只可用于比较，不能传入受限运算。

---

## `ceil`

向上取整。

### 调用

```lua
math.ceil
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明         |
| ------- | ----- | ------------ |
| `value` | float | 要取整的数值 |

## 返回值

返回一个值。

| 类型    | 说明                      |
| ------- | ------------------------- |
| integer | 不小于 `value` 的最小整数 |

### 示例

```lua
local n = math.ceil(3.14)
debug.print(tostring(n))
```

**输出：**

```lua

```

---

## `floor`

向下取整。

### 调用

```lua
math.floor
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明         |
| ------- | ----- | ------------ |
| `value` | float | 要取整的数值 |

## 返回值

返回一个值。

| 类型    | 说明                      |
| ------- | ------------------------- |
| integer | 不大于 `value` 的最大整数 |

### 示例

```lua
local n = math.floor(3.14)
debug.print(tostring(n))
```

**输出：**

```lua

```

---

## `round`

四舍五入到最近的整数。

### 调用

```lua
math.round
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明         |
| ------- | ----- | ------------ |
| `value` | float | 要取整的数值 |

## 返回值

返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| integer | 四舍五入后的整数 |

### 示例

```lua
local n1 = math.round(3.5)
local n2 = math.round(-3.5)
debug.print(tostring(n1) .. ", " .. tostring(n2))
```

**输出：**

```lua

```

---

## `round_to`

按指定位数四舍五入。

### 调用

```lua
math.round_to
```

## 参数

### 必填参数

| 参数名   | 类型    | 说明         |
| -------- | ------- | ------------ |
| `value`  | float   | 要取整的数值 |
| `digits` | integer | 小数位数     |

## 返回值

返回一个值。

| 类型  | 说明               |
| ----- | ------------------ |
| float | 保留指定位数的结果 |

### 示例

```lua
local r1 = math.round_to(3.14159, 2)
local r2 = math.round_to(12345, -2)
debug.print(tostring(r1) .. ", " .. tostring(r2))
```

**输出：**

```lua

```

### 额外说明

- 参数 `digits` 范围为 $[-308, 308]$。

---

## `fmod`

计算取模（余数）。

### 调用

```lua
math.fmod
```

## 参数

### 必填参数

| 参数名 | 类型    | 说明   |
| ------ | ------- | ------ |
| `x`    | integer | 被除数 |
| `y`    | integer | 除数   |

## 返回值

返回一个值。

| 类型    | 说明 |
| ------- | ---- |
| integer | 余数 |

### 示例

```lua
local r = math.fmod(7, 3)
debug.print(tostring(r))
```

**输出：**

```lua

```

### 额外说明

- `x`、`y` 必须是整数，`y` 不能为 0；本接口不接受带小数的余数运算。

- 结果符号与被除数一致。

---

## `pow`

计算幂运算 $x^y$。

### 调用

```lua
math.pow
```

## 参数

### 必填参数

| 参数名 | 类型  | 说明 |
| ------ | ----- | ---- |
| `x`    | float | 底数 |
| `y`    | float | 指数 |

## 返回值

返回一个值。

| 类型  | 说明       |
| ----- | ---------- |
| float | 幂运算结果 |

### 示例

```lua
local p = math.pow(2, 10)
debug.print(tostring(p))
```

**输出：**

```lua

```

---

## `exp`

计算 $e^{value}$。

### 调用

```lua
math.exp
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明   |
| ------- | ----- | ------ |
| `value` | float | 指数值 |

## 返回值

返回一个值。

| 类型  | 说明                 |
| ----- | -------------------- |
| float | $e^{value}$ 的计算值 |

### 示例

```lua
local e2 = math.exp(2)
debug.print(tostring(e2))
```

**输出：**

```lua

```

---

## `log`

计算指定底数的对数。

### 调用

```lua
math.log
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明 |
| ------- | ----- | ---- |
| `value` | float | 真数 |
| `base`  | float | 底数 |

## 返回值

返回一个值。

| 类型  | 说明   |
| ----- | ------ |
| float | 对数值 |

### 示例

```lua
local log = math.log(8, 2)
debug.print(tostring(log))
```

**输出：**

```lua

```

### 额外说明

- `value` 必须大于 0；`base` 必须大于 0 且不等于 1，不能省略。

---

## `lg`

计算以 10 为底的对数。

### 调用

```lua
math.lg
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明 |
| ------- | ----- | ---- |
| `value` | float | 真数 |

## 返回值

返回一个值。

| 类型  | 说明       |
| ----- | ---------- |
| float | 常用对数值 |

### 示例

```lua
local lg = math.lg(100)
debug.print(tostring(lg))
```

**输出：**

```lua

```

---

## `ln`

计算以 e 为底的对数。

### 调用

```lua
math.ln
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明 |
| ------- | ----- | ---- |
| `value` | float | 真数 |

## 返回值

返回一个值。

| 类型  | 说明       |
| ----- | ---------- |
| float | 常用对数值 |

### 示例

```lua
local ln = math.ln(math.E)
debug.print(tostring(ln))
```

**输出：**

```lua

```

---

## `sqrt`

计算平方根。

### 调用

```lua
math.sqrt
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明     |
| ------- | ----- | -------- |
| `value` | float | 被开方数 |

## 返回值

返回一个值。

| 类型  | 说明     |
| ----- | -------- |
| float | 平方根值 |

### 示例

```lua
local r = math.sqrt(16)
debug.print(tostring(r))
```

**输出：**

```lua

```

---

## `ldexp`

计算 $x \times 2^{exp}$。

### 调用

```lua
math.ldexp
```

## 参数

### 必填参数

| 参数名 | 类型    | 说明 |
| ------ | ------- | ---- |
| `x`    | float   | 尾数 |
| `exp`  | integer | 指数 |

## 返回值

返回一个值。

| 类型  | 说明     |
| ----- | -------- |
| float | 计算结果 |

### 示例

```lua
local v = math.ldexp(3, 2)
debug.print(tostring(v))
```

**输出：**

```lua

```

---

## `frexp`

将数值分解为尾数与二进制指数。

### 调用

```lua
math.frexp
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明         |
| ------- | ----- | ------------ |
| `value` | float | 要分解的数值 |

## 返回值

返回两个值，依次为尾数和二进制指数。

| 值名       | 类型    | 说明   |
| ---------- | ------- | ------ |
| `mantissa` | float   | 尾数 |
| `exponent` | integer | 指数 |

### 示例

```lua
local mantissa, exponent = math.frexp(12.8)
debug.print(tostring(mantissa) .. ", " .. tostring(exponent))
```

**输出：**

```lua

```

### 额外说明

- 参数 `value`、返回值 `mantissa` 和 `exponent` 满足公式 $value = mantissa \times 2^{exponent}$；`math.ldexp` 可进行逆运算。

---

## `sin`

计算正弦（弧度制）。

### 调用

```lua
math.sin
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明   |
| ------- | ----- | ------ |
| `value` | float | 弧度值 |

## 返回值

返回一个值。

| 类型  | 说明   |
| ----- | ------ |
| float | 正弦值 |

### 示例

```lua
local s = math.sin(math.PI / 2)
debug.print(tostring(s))
```

**输出：**

```lua

```

---

## `cos`

计算余弦（弧度制）。

### 调用

```lua
math.cos
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明   |
| ------- | ----- | ------ |
| `value` | float | 弧度值 |

## 返回值

返回一个值。

| 类型  | 说明   |
| ----- | ------ |
| float | 余弦值 |

### 示例

```lua
local c = math.cos(math.PI)
debug.print(tostring(c))
```

**输出：**

```lua

```

---

## `tan`

计算正切（弧度制）。

### 调用

```lua
math.tan
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明   |
| ------- | ----- | ------ |
| `value` | float | 弧度值 |

## 返回值

返回一个值。

| 类型  | 说明   |
| ----- | ------ |
| float | 正切值 |

### 示例

```lua
local t = math.tan(math.PI / 4) -- 可能会有浮点数精度问题
debug.print(tostring(t))
```

**输出：**

```lua

```

---

## `asin`

计算反正弦（弧度制）。

### 调用

```lua
math.asin
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明   |
| ------- | ----- | ------ |
| `value` | float | 正弦值 |

## 返回值

返回一个值。

| 类型  | 说明   |
| ----- | ------ |
| float | 弧度值 |

### 示例

```lua
local r = math.asin(0.5)
debug.print(tostring(r))
```

**输出：**

```lua

```

### 额外说明

- 参数 `value` 范围为 $[-1, 1]$

---

## `acos`

计算反余弦（弧度制）。

### 调用

```lua
math.acos
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明   |
| ------- | ----- | ------ |
| `value` | float | 余弦值 |

## 返回值

返回一个值。

| 类型  | 说明   |
| ----- | ------ |
| float | 弧度值 |

### 示例

```lua
local r = math.acos(0.5)
debug.print(tostring(r))
```

**输出：**

```lua

```

### 额外说明

- 参数 `value` 范围为 $[-1, 1]$

---

## `atan`

计算反正切（弧度制）。

### 调用

```lua
math.atan
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明   |
| ------- | ----- | ------ |
| `value` | float | 正切值 |

## 返回值

返回一个值。

| 类型  | 说明   |
| ----- | ------ |
| float | 弧度值 |

### 示例

```lua
local r = math.atan(1)
debug.print(tostring(r))
```

**输出：**

```lua

```

---

## `atan2`

计算反正切（弧度制）。

### 调用

```lua
math.atan2
```

## 参数

### 必填参数

| 参数名 | 类型  | 说明   |
| ------ | ----- | ------ |
| `y`    | float | 纵坐标 |
| `x`    | float | 横坐标 |

## 返回值

返回一个值。

| 类型  | 说明   |
| ----- | ------ |
| float | 弧度值 |

### 示例

```lua
local a = math.atan2(1, 1)
debug.print(tostring(a))
```

**输出：**

```lua

```

---

## `deg`

将弧度转换为角度。

### 调用

```lua
math.deg
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明   |
| ------- | ----- | ------ |
| `value` | float | 弧度值 |

## 返回值

返回一个值。

| 类型  | 说明   |
| ----- | ------ |
| float | 角度值 |

### 示例

```lua
local d = math.deg(math.PI)
debug.print(tostring(d))
```

**输出：**

```lua

```

---

## `rad`

将角度转换为弧度。

### 调用

```lua
math.rad
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明   |
| ------- | ----- | ------ |
| `value` | float | 角度值 |

## 返回值

返回一个值。

| 类型  | 说明   |
| ----- | ------ |
| float | 弧度值 |

### 示例

```lua
local r = math.rad(180)
debug.print(tostring(r))
```

**输出：**

```lua

```

---

## `normalize_angle`

将角度归一化到 `[0, 360)` 区间。

### 调用

```lua
math.normalize_angle
```

## 参数

### 必填参数

| 参数名  | 类型    | 说明   |
| ------- | ------- | ------ |
| `value` | integer | 角度值 |

## 返回值

返回一个值。

| 类型  | 说明           |
| ----- | -------------- |
| float | 归一化后的角度 |

### 示例

```lua
local a1 = math.normalize_angle(450)
local a2 = math.normalize_angle(-90)
debug.print(tostring(a1) .. ", " .. tostring(a2))
```

**输出：**

```lua

```

---

## `max`

返回一组数中的最大值。

### 调用

```lua
math.max
```

## 参数

### 必填参数

| 参数名   | 类型  | 说明       |
| -------- | ----- | ---------- |
| `values` | table | 数值数组表 |

## 返回值

返回一个值。

| 类型  | 说明   |
| ----- | ------ |
| float | 最大值 |

### 示例

```lua
local m = math.max({ 1, 5, 3, 9, 2 })
debug.print(tostring(m))
```

**输出：**

```lua

```

### 额外说明

只接受最多 16,384 项的稠密数值数组。可提供 `n` 字段表示长度，但其值必须与数组项一致，数组中不能有空洞。

---

## `min`

返回一组数中的最小值。

### 调用

```lua
math.min
```

## 参数

### 必填参数

| 参数名   | 类型  | 说明       |
| -------- | ----- | ---------- |
| `values` | table | 数值数组表 |

## 返回值

返回一个值。

| 类型  | 说明   |
| ----- | ------ |
| float | 最小值 |

### 示例

```lua
local m = math.min({ 1, 5, 3, 9, 2 })
debug.print(tostring(m))
```

**输出：**

```lua

```

### 额外说明

只接受最多 16,384 项的稠密数值数组。可提供 `n` 字段表示长度，但其值必须与数组项一致，数组中不能有空洞。

---

## `modf`

分离数值的整数部分与小数部分。

### 调用

```lua
math.modf
```

## 参数

### 必填参数

| 参数名  | 类型    | 说明         |
| ------- | ------- | ------------ |
| `value` | number | 要分解的数值 |

## 返回值

返回两个值，依次为整数部分和小数部分。

| 值名              | 类型    | 说明       |
| ----------------- | ------- | ---------- |
| `integer_part`    | integer | 整数部分 |
| `fractional_part` | float   | 小数部分 |

### 示例

```lua
local integer_part, fractional_part = math.modf(2.5)
debug.print(tostring(integer_part) .. ", " .. tostring(fractional_part))
```

**输出：**

```lua

```

---

## `tointeger`

将数值精确转换为整数。

### 调用

```lua
math.tointeger
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明         |
| ------- | ----- | ------------ |
| `value` | float | 要转换的数值 |

## 返回值

返回一个值。

| 类型          | 说明             |
| ------------- | ---------------- |
| integer / nil | 精确转换后的整数 |

### 示例

```lua
local i1 = math.tointeger(3.0)
local i2 = math.tointeger(3.14)
debug.print(tostring(i1) .. ", " .. tostring(i2))
```

**输出：**

```lua

```

### 额外说明

- 若转换失败时返回 `nil`。

---

## `type`

返回数值的类型名。

### 调用

```lua
math.type
```

## 参数

### 必填参数

| 参数名  | 类型 | 说明       |
| ------- | ---- | ---------- |
| `value` | any  | 要判断的值 |

## 返回值

返回一个值。

| 类型         | 说明     |
| ------------ | -------- |
| string / nil | 数值类型 |

### 示例

```lua
local t1 = math.type(3)
local t2 = math.type(3.14)
local t3 = math.type("3")
debug.print(tostring(t1) .. ", " .. tostring(t2) .. ", " .. tostring(t3))
```

**输出：**

```lua

```

### 额外说明

- 若参数传递不为数值时返回 `nil`。

---

## `ult`

以无符号整数比较两个整数。

### 调用

```lua
math.ult
```

## 参数

### 必填参数

| 参数名  | 类型    | 说明       |
| ------- | ------- | ---------- |
| `left`  | integer | 左侧操作数 |
| `right` | integer | 右侧操作数 |

## 返回值

返回一个值。

| 类型    | 说明     |
| ------- | -------- |
| boolean | 比较结果 |

### 示例

```lua
local b1 = math.ult(-1, 1)  -- -1 二进制码在无符号整数为 2^64-1
local b2 = math.ult(1, -1)
debug.print(tostring(b1) .. ", " .. tostring(b2))
```

**输出：**

```lua

```

### 额外说明

- 该 API 等价于两个无符号数使用操作符 `<`。
- 负数会直接以二进制码进行比较，而非取绝对值。

---

## `approx_equal`

以指定误差比较两个数字是否相等。

### 调用

```lua
math.approx_equal
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明       |
| ------- | ----- | ---------- |
| `left`  | float | 左侧操作数 |
| `right` | float | 右侧操作数 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --------- | ----- | --- | ----------------------- |
| `epsilon` | float | `1e-10` | 误差范围 |

## 返回值

返回一个值。

| 类型    | 说明     |
| ------- | -------- |
| boolean | 比较结果 |

### 示例

```lua
local ae1 = math.approx_equal(0.1 + 0.2, 0.3)
local ae2 = math.approx_equal(1000000.0, 1000000.0000001, {epsilon = 1e-10})

debug.print(tostring(ae1))
debug.print(tostring(ae2))
```

**输出：**

```lua

```

---

## `percent`

计算百分比 $\frac{value}{total}$。

### 调用

```lua
math.percent
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明 |
| ------- | ----- | ---- |
| `value` | float | 分子 |
| `total` | float | 分母 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| ------------ | ------- | --- | ------------------------- |
| `as_percent` | boolean | `false` | 百分比输出 |

## 返回值

返回一个值。

| 类型  | 说明       |
| ----- | ---------- |
| float | 百分比数值 |

### 示例

```lua
local p1 = math.percent(25, 80)
debug.print(tostring(p1))

local p2 = math.percent(25, 80, {as_percent = true})
debug.print(tostring(p2))
```

**输出：**

```lua

```

### 额外说明

- `total` 不能为 0。默认返回 `value / total`；`as_percent = true` 时再乘以 100。

---

## `factorial`

计算阶乘 $n!$。

### 调用

```lua
math.factorial
```

## 参数

### 必填参数

| 参数名 | 类型    | 说明   |
| ------ | ------- | ------ |
| `n`    | integer | 阶乘数 |

## 返回值

返回一个值。

| 类型  | 说明     |
| ----- | -------- |
| float | 阶乘结果 |

### 示例

```lua
local f = math.factorial(5)
debug.print(tostring(f))
```

**输出：**

```lua

```

### 额外说明

- 参数 `n` 范围为 $[0, 170]$。

---

## `combination`

计算组合数 $C^n_k$。

### 调用

```lua
math.combination
```

## 参数

### 必填参数

| 参数名 | 类型    | 说明   |
| ------ | ------- | ------ |
| `n`    | integer | 总数   |
| `k`    | integer | 选取数 |

## 返回值

返回一个值。

| 类型    | 说明     |
| ------- | -------- |
| integer | 组合数值 |

### 示例

```lua
local c = math.combination(5, 2)
debug.print(tostring(c))
```

**输出：**

```lua

```

### 额外说明

- 参数 `k` 范围为 $[0, n]$。

参数 `k` 必须在 `0..=n` 范围内；组合数超出整数范围时会报错。
