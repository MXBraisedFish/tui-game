# math 库

## 基本库说明

`math` 提供数学运算与数学常量。

> 在使用数学计算时请时刻关注浮点数精度问题，可能会导致意外的 bug。

---

## 目录

### 常量

| 常量名                           | 说明              | 索引                                      |
| -------------------------------- | ----------------- | ----------------------------------------- |
| `PI`                             | 圆周率 π          | [PI](#PI)                                 |
| `E`                              | 自然常数 e        | [E](#E)                                   |
| `POSITIVE_INFINITE` / `INFINITE` | 正无穷            | [POSITIVE_INFINITE / INFINITE](#INFINITE) |
| `NEGATIVE_INFINITE`              | 负无穷            | [NEGATIVE_INFINITE](#NEGATIVE_INFINITE)   |
| `DEG`                            | 弧度转角度系数    | [DEG](#DEG)                               |
| `RAD`                            | 角度转弧度系数    | [RAD](#RAD)                               |
| `MAX_INTEGER`                    | 最大整数 `2^63-1` | [MAX_INTEGER](#MAX_INTEGER)               |
| `MIN_INTEGER`                    | 最小整数 `-2^63`  | [MIN_INTEGER](#MIN_INTEGER)               |

### 方法

| 方法名            | 说明                             | 索引                                |
| ----------------- | -------------------------------- | ----------------------------------- |
| `abs`             | 计算绝对值                       | [abs](#abs)                         |
| `ceil`            | 向上取整                         | [ceil](#ceil)                       |
| `floor`           | 向下取整                         | [floor](#floor)                     |
| `round`           | 四舍五入到最近的整数             | [round](#round)                     |
| `round_to`        | 按指定位数四舍五入               | [round_to](#round_to)               |
| `fmod`            | 计算取模（余数）                 | [fmod](#fmod)                       |
| `pow`             | 计算幂运算 $x^y$                 | [pow](#pow)                         |
| `exp`             | 计算 $e^{value}$                 | [exp](#exp)                         |
| `log`             | 计算指定底数的对数               | [log](#log)                         |
| `lg`              | 计算以 10 为底的对数             | [lg](#lg)                           |
| `ln`              | 计算以 e 为底的对数              | [ln](#ln)                           |
| `sqrt`            | 计算平方根                       | [sqrt](#sqrt)                       |
| `ldexp`           | 计算 $x \times 2^{exp}$          | [ldexp](#ldexp)                     |
| `frexp`           | 将数值分解为尾数与二进制指数     | [frexp](#frexp)                     |
| `sin`             | 计算正弦（弧度制）               | [sin](#sin)                         |
| `cos`             | 计算余弦（弧度制）               | [cos](#cos)                         |
| `tan`             | 计算正切（弧度制）               | [tan](#tan)                         |
| `asin`            | 计算反正弦（弧度制）             | [asin](#asin)                       |
| `acos`            | 计算反余弦（弧度制）             | [acos](#acos)                       |
| `atan`            | 计算反正切（弧度制）             | [atan](#atan)                       |
| `atan2`           | 计算反正切（弧度制）             | [atan2](#atan2)                     |
| `deg`             | 将弧度转换为角度                 | [deg](#deg)                         |
| `rad`             | 将角度转换为弧度                 | [rad](#rad)                         |
| `normalize_angle` | 将角度归一化到 `[0, 360)` 区间   | [normalize_angle](#normalize_angle) |
| `max`             | 返回一组数中的最大值             | [max](#max)                         |
| `min`             | 返回一组数中的最小值             | [min](#min)                         |
| `modf`            | 分离数值的整数部分与小数部分     | [modf](#modf)                       |
| `tointeger`       | 将数值精确转换为整数             | [tointeger](#tointeger)             |
| `type`            | 返回数值的类型名                 | [type](#type)                        |
| `ult`             | 以无符号整数比较两个整数         | [ult](#ult)                         |
| `approx_equal`    | 以指定误差比较两个数字是否相等   | [approx_equal](#approx_equal)       |
| `percent`         | 计算百分比 $\frac{value}{total}$ | [percent](#percent)                 |
| `factorial`       | 计算阶乘 $n!$                    | [factorial](#factorial)             |
| `combination`     | 计算组合数 $C^n_k$               | [combination](#combination)         |

---

## 常量

## `PI`

圆周率 π。

**可用于**

- 任意

### 调用

```lua
math.PI
```

### 示例

```lua
debug.print { message = tostring(math.PI) }
```

输出：

```text
3.141592653589793
```

---

## `E`

自然常数 e。

**可用于**

- 数学比较。

### 调用

```lua
math.E
```

### 示例

```lua
debug.print { message = tostring(math.E) }
```

输出：

```text
2.718281828459045
```

---

## `POSITIVE_INFINITE` / `INFINITE` {#INFINITE}

正无穷。

**可用于**

- 数学比较。

### 调用

```lua
math.POSITIVE_INFINITE
```

### 示例

```lua
debug.print { message = tostring(math.POSITIVE_INFINITE > math.MAX_INTEGER) }
```

输出：

```text
true
```

### 额外补充

- 该值永远大于任何数。
- 不可用于计算。

---

## `NEGATIVE_INFINITE`

负无穷。

**可用于**

- 任意

### 调用

```lua
math.NEGATIVE_INFINITE
```

### 示例

```lua
debug.print { message = tostring(math.NEGATIVE_INFINITE < math.MIN_INTEGER) }
```

输出：

```text
true
```

### 额外补充

- 该值永远小于任何数。
- 不可用于计算。

---

## `DEG`

弧度转角度系数，`180 / π`。

**可用于**

- 任意

### 调用

```lua
math.DEG
```

### 示例

```lua
debug.print { message = tostring(math.DEG) }
```

输出：

```text
57.29577951308232
```

---

## `RAD`

角度转弧度系数，`π / 180`。

**可用于**

- 任意

### 调用

```lua
math.RAD
```

### 示例

```lua
debug.print { message = tostring(math.RAD) }
```

输出：

```text
0.017453292519943295
```

---

## `MAX_INTEGER`

最大可表示的整数 `2^63-1`。

**可用于**

- 任意

### 调用

```lua
math.MAX_INTEGER
```

### 示例

```lua
debug.print { message = tostring(math.MAX_INTEGER) }
```

输出：

```text
9223372036854775807
```

---

## `MIN_INTEGER`

最小可表示的整数 `-2^63`。

**可用于**

- 任意

### 调用

```lua
math.MIN_INTEGER
```

### 示例

```lua
debug.print { message = tostring(math.MIN_INTEGER) }
```

输出：

```text
-9223372036854775808
```

---

## 方法

## `abs`

计算绝对值。

### 调用

```lua
-- 单参数
math.abs()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明             |
| ------- | ------ | ---- | ------ | ---------------- |
| `value` | float | 是   | -      | 要取绝对值的数值 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| float | 绝对值 |

### 示例

```lua
local n = math.abs(-5.20)
debug.print { message = tostring(n) }
```

输出：

```text
5.2
```

---

## `ceil`

向上取整。

### 调用

```lua
-- 单参数
math.ceil()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明         |
| ------- | ------ | ---- | ------ | ------------ |
| `value` | float | 是   | -      | 要取整的数值 |

### 返回

直接返回一个值。

| 类型    | 说明                      |
| ------- | ------------------------- |
| integer | 不小于 `value` 的最小整数 |

### 示例

```lua
local n = math.ceil(3.14)
debug.print { message = tostring(n) }
```

输出：

```text
4
```

---

## `floor`

向下取整。

### 调用

```lua
-- 单参数
math.floor()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明         |
| ------- | ------ | ---- | ------ | ------------ |
| `value` | float | 是   | -      | 要取整的数值 |

### 返回

直接返回一个值。

| 类型    | 说明                      |
| ------- | ------------------------- |
| integer | 不大于 `value` 的最大整数 |

### 示例

```lua
local n = math.floor(3.14)
debug.print { message = tostring(n) }
```

输出：

```text
3
```

---

## `round`

四舍五入到最近的整数。

### 调用

```lua
-- 单参数
math.round()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明         |
| ------- | ------ | ---- | ------ | ------------ |
| `value` | float | 是   | -      | 要取整的数值 |

### 返回

直接返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| integer | 四舍五入后的整数 |

### 示例

```lua
local n1 = math.round(3.5)
local n2 = math.round(-3.5)
debug.print { message = tostring(n1) .. ", " .. tostring(n2) }
```

输出：

```text
4, -4
```

---

## `round_to`

按指定位数四舍五入。

### 调用

```lua
-- 表参数
math.round_to{}
```

### 参数

| 参数名   | 类型    | 必填 | 默认值 | 说明         |
| -------- | ------- | ---- | ------ | ------------ |
| `value`  | float  | 是   | -      | 要取整的数值 |
| `digits` | integer | 是   | -      | 小数位数     |

### 返回

直接返回一个值。

| 类型   | 说明               |
| ------ | ------------------ |
| float | 保留指定位数的结果 |

### 示例

```lua
local r1 = math.round_to { value = 3.14159, digits = 2 }
local r2 = math.round_to { value = 12345, digits = -2 }
debug.print { message = tostring(r1) .. ", " .. tostring(r2) }
```

输出：

```text
3.14, 12300
```

### 额外补充

- 参数 `digits` 范围为 $[-308, 308]$。

---

## `fmod`

计算取模（余数）。

### 调用

```lua
-- 表参数
math.fmod{}
```

### 参数

| 参数名 | 类型    | 必填 | 默认值 | 说明   |
| ------ | ------- | ---- | ------ | ------ |
| `x`    | integer | 是   | -      | 被除数 |
| `y`    | integer | 是   | -      | 除数   |

### 返回

直接返回一个值。

| 类型    | 说明 |
| ------- | ---- |
| integer | 余数 |

### 示例

```lua
local r = math.fmod(7, 3)
debug.print { message = tostring(r) }
```

输出：

```text
1
```

### 额外补充

- 结果符号与被除数一致。

---

## `pow`

计算幂运算 $x^y$。

### 调用

```lua
-- 表参数
math.pow{}
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明 |
| ------ | ------ | ---- | ------ | ---- |
| `x`    | float | 是   | -      | 底数 |
| `y`    | float | 是   | -      | 指数 |

### 返回

直接返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| float | 幂运算结果 |

### 示例

```lua
local p = math.pow(2, 10)
debug.print { message = tostring(p) }
```

输出：

```text
1024
```

---

## `exp`

计算 $e^{value}$。

### 调用

```lua
-- 单参数
math.exp()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明   |
| ------- | ------ | ---- | ------ | ------ |
| `value` | float | 是   | -      | 指数值 |

### 返回

直接返回一个值。

| 类型   | 说明                 |
| ------ | -------------------- |
| float | $e^{value}$ 的计算值 |

### 示例

```lua
local e2 = math.exp(2)
debug.print { message = tostring(e2) }
```

输出：

```text
7.38905609893065
```

---

## `log`

计算指定底数的对数。

### 调用

```lua
-- 表参数
math.log{}
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明 |
| ------- | ------ | ---- | ------ | ---- |
| `value` | float | 是   | -      | 真数 |
| `base`  | float | 是   | -      | 底数 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| float | 对数值 |

### 示例

```lua
local log = math.log { value = 8, base = 2 }
debug.print { message = tostring(log) }
```

输出：

```text
3
```

---

## `lg`

计算以 10 为底的对数。

### 调用

```lua
-- 单参数
math.lg()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明 |
| ------- | ------ | ---- | ------ | ---- |
| `value` | float | 是   | -      | 真数 |

### 返回

直接返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| float | 常用对数值 |

### 示例

```lua
local lg = math.lg(100)
debug.print { message = tostring(lg) }
```

输出：

```text
2
```

---

## `ln`

计算以 e 为底的对数。

### 调用

```lua
-- 单参数
math.ln()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明 |
| ------- | ------ | ---- | ------ | ---- |
| `value` | float | 是   | -      | 真数 |

### 返回

直接返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| float | 常用对数值 |

### 示例

```lua
local ln = math.ln(math.E)
debug.print { message = tostring(ln) }
```

输出：

```text
1
```

---

## `sqrt`

计算平方根。

### 调用

```lua
-- 单参数
math.sqrt()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明     |
| ------- | ------ | ---- | ------ | -------- |
| `value` | float | 是   | -      | 被开方数 |

### 返回

直接返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| float | 平方根值 |

### 示例

```lua
local r = math.sqrt(16)
debug.print { message = tostring(r) }
```

输出：

```text
4
```

---

## `ldexp`

计算 $x \times 2^{exp}$。

### 调用

```lua
-- 表参数
math.ldexp{}
```

### 参数

| 参数名 | 类型    | 必填 | 默认值 | 说明 |
| ------ | ------- | ---- | ------ | ---- |
| `x`    | float  | 是   | -      | 尾数 |
| `exp`  | integer | 是   | -      | 指数 |

### 返回

直接返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| float | 计算结果 |

### 示例

```lua
local v = math.ldexp{ x = 3, exp = 2 }
debug.print { message = tostring(v) }
```

输出：

```text
12
```

---

## `frexp`

将数值分解为尾数与二进制指数。

### 调用

```lua
-- 单参数
math.frexp()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明         |
| ------- | ------ | ---- | ------ | ------------ |
| `value` | float | 是   | -      | 要分解的数值 |

### 返回

返回一个对象表。

| 字段       | 类型    | 说明 |
| ---------- | ------- | ---- |
| `mantissa` | float  | 尾数 |
| `exponent` | integer | 指数 |

### 示例

```lua
local f = math.frexp(12.8)
debug.print { message = tostring(f.mantissa) .. ", " .. tostring(f.exponent) }
```

输出：

```text
0.8, 4
```

### 额外补充

- 参数 `value`、字段 `mantissa`和字段 `exponent` 满足公式 $value = mantissa \times 2^{exponent}$，为 API `math.ldexp` 的逆运算。

---

## `sin`

计算正弦（弧度制）。

### 调用

```lua
-- 单参数
math.sin()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明   |
| ------- | ------ | ---- | ------ | ------ |
| `value` | float | 是   | -      | 弧度值 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| float | 正弦值 |

### 示例

```lua
local s = math.sin(math.PI / 2)
debug.print { message = tostring(s) }
```

输出：

```text
1
```

---

## `cos`

计算余弦（弧度制）。

### 调用

```lua
-- 单参数
math.cos()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明   |
| ------- | ------ | ---- | ------ | ------ |
| `value` | float | 是   | -      | 弧度值 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| float | 余弦值 |

### 示例

```lua
local c = math.cos(math.PI)
debug.print { message = tostring(c) }
```

输出：

```text
-1
```

---

## `tan`

计算正切（弧度制）。

### 调用

```lua
-- 单参数
math.tan()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明   |
| ------- | ------ | ---- | ------ | ------ |
| `value` | float | 是   | -      | 弧度值 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| float | 正切值 |

### 示例

```lua
local t = math.tan(math.PI / 4) -- 可能会有浮点数精度问题
debug.print { message = tostring(t) }
```

输出：

```text
0.9999999999999999
```

---

## `asin`

计算反正弦（弧度制）。

### 调用

```lua
-- 单参数
math.asin()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明   |
| ------- | ------ | ---- | ------ | ------ |
| `value` | float | 是   | -      | 正弦值 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| float | 弧度值 |

### 示例

```lua
local r = math.asin(0.5)
debug.print { message = tostring(r) }
```

输出：

```text
0.5235987755982989
```

### 额外补充

- 参数 `value` 范围为 $[-1, 1]$

---

## `acos`

计算反余弦（弧度制）。

### 调用

```lua
-- 单参数
math.acos(value)
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明   |
| ------- | ------ | ---- | ------ | ------ |
| `value` | float | 是   | -      | 余弦值 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| float | 弧度值 |

### 示例

```lua
local r = math.acos(0.5)
debug.print { message = tostring(r) }
```

输出：

```text
1.0471975511965979
```

### 额外补充

- 参数 `value` 范围为 $[-1, 1]$

---

## `atan`

计算反正切（弧度制）。

### 调用

```lua
-- 单参数
math.atan()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明   |
| ------- | ------ | ---- | ------ | ------ |
| `value` | float | 是   | -      | 正切值 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| float | 弧度值 |

### 示例

```lua
local r = math.atan(1)
debug.print { message = tostring(r) }
```

输出：

```text
0.7853981633974483
```

---

## `atan2`

计算反正切（弧度制）。

### 调用

```lua
-- 表参数
math.atan2{}
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明   |
| ------ | ------ | ---- | ------ | ------ |
| `y`    | float | 是   | -      | 纵坐标 |
| `x`    | float | 是   | -      | 横坐标 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| float | 弧度值 |

### 示例

```lua
local a = math.atan2(1, 1)
debug.print { message = tostring(a) }
```

输出：

```text
0.7853981633974483
```

---

## `deg`

将弧度转换为角度。

### 调用

```lua
-- 单参数
math.deg()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明   |
| ------- | ------ | ---- | ------ | ------ |
| `value` | float | 是   | -      | 弧度值 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| float | 角度值 |

### 示例

```lua
local d = math.deg(math.PI)
debug.print { message = tostring(d) }
```

输出：

```text
180
```

---

## `rad`

将角度转换为弧度。

### 调用

```lua
-- 单参数
math.rad()
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明   |
| ------- | ------ | ---- | ------ | ------ |
| `value` | float | 是   | -      | 角度值 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| float | 弧度值 |

### 示例

```lua
local r = math.rad(180)
debug.print { message = tostring(r) }
```

输出：

```text
3.141592653589793
```

---

## `normalize_angle`

将角度归一化到 `[0, 360)` 区间。

### 调用

```lua
-- 单参数
math.normalize_angle()
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明   |
| ------- | ------- | ---- | ------ | ------ |
| `value` | integer | 是   | -      | 角度值 |

### 返回

直接返回一个值。

| 类型   | 说明           |
| ------ | -------------- |
| float | 归一化后的角度 |

### 示例

```lua
local a1 = math.normalize_angle(450)
local a2 = math.normalize_angle(-90)
debug.print { message = tostring(a1) .. ", " .. tostring(a2) }
```

输出：

```text
90, 270
```

---

## `max`

返回一组数中的最大值。

### 调用

```lua
-- 单参数
math.max()
```

### 参数

| 参数名   | 类型  | 必填 | 默认值 | 说明     |
| -------- | ----- | ---- | ------ | -------- |
| `values` | table | 是   | -      | 数值数组表 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| float | 最大值 |

### 示例

```lua
local m = math.max({ 1, 5, 3, 9, 2 })
debug.print { message = tostring(m) }
```

输出：

```text
9
```

---

## `min`

返回一组数中的最小值。

### 调用

```lua
-- 表参数
math.min{}
```

### 参数

| 参数名   | 类型  | 必填 | 默认值 | 说明     |
| -------- | ----- | ---- | ------ | -------- |
| `values` | table | 是   | -      | 数值数组表 |

### 返回

直接返回一个值。

| 类型   | 说明   |
| ------ | ------ |
| float | 最小值 |

### 示例

```lua
local m = math.min({ 1, 5, 3, 9, 2 })
debug.print { message = tostring(m) }
```

输出：

```text
1
```

---

## `modf`

分离数值的整数部分与小数部分。

### 调用

```lua
-- 单参数
math.modf()
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明         |
| ------- | ------- | ---- | ------ | ------------ |
| `value` | integer | 是   | -      | 要分解的数值 |

### 返回

返回一个对象表。

| 字段              | 类型    | 说明     |
| ----------------- | ------- | -------- |
| `integer_part`    | integer | 整数部分 |
| `fractional_part` | float  | 小数部分 |

### 示例

```lua
local n = math.modf(2.5)
debug.print { message = tostring(n.integer_part) .. ", " .. tostring(n.fractional_part) }
```

输出：

```text
2, 0.5
```

---

## `tointeger`

将数值精确转换为整数。

### 调用

```lua
-- 单参数
math.tointeger(value)
```

### 参数

| 参数名  | 类型   | 必填 | 默认值 | 说明         |
| ------- | ------ | ---- | ------ | ------------ |
| `value` | float | 是   | -      | 要转换的数值 |

### 返回

直接返回一个值。

| 类型          | 说明             |
| ------------- | ---------------- |
| integer / nil | 精确转换后的整数 |

### 示例

```lua
local i1 = math.tointeger(3.0)
local i2 = math.tointeger(3.14)
debug.print { message = tostring(i1) .. ", " .. tostring(i2) }
```

输出：

```text
3, nil
```

### 额外补充

- 若转换失败时返回 `nil`。

---

## `type`

返回数值的类型名。

### 调用

```lua
-- 单参数
math.type()
```

### 参数

| 参数名  | 类型 | 必填 | 默认值 | 说明                         |
| ------- | ---- | ---- | ------ | ---------------------------- |
| `value` | any  | 是   | -      | 要判断的值 |

### 返回

直接返回一个值。

| 类型         | 说明     |
| ------------ | -------- |
| string / nil | 数值类型 |

### 示例

```lua
local t1 = math.type(3)
local t2 = math.type(3.14)
local t3 = math.type("3")
debug.print { message = tostring(t1) .. ", " .. tostring(t2) .. ", " .. tostring(t3) }
```

输出：

```text
integer, float, nil
```

### 额外补充

- 若参数传递不为数值时返回 `nil`。

---

## `ult`

以无符号整数比较两个整数。

### 调用

```lua
-- 表参数
math.ult{}
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明       |
| ------- | ------- | ---- | ------ | ---------- |
| `left`  | integer | 是   | -      | 左侧操作数 |
| `right` | integer | 是   | -      | 右侧操作数 |

### 返回

直接返回一个值。

| 类型    | 说明     |
| ------- | -------- |
| boolean | 比较结果 |

### 示例

```lua
local b1 = math.ult { left = -1, right = 1 }  -- -1 二进制码在无符号整数为 2^64-1
local b2 = math.ult { left = 1, right = -1 }
debug.print { message = tostring(b1) .. ", " .. tostring(b2) }
```

输出：

```text
false, true
```

### 额外补充

- 该 API 等价于两个无符号数使用操作符 `<`。
- 负数会直接以二进制码进行比较，而非取绝对值。

---

## `approx_equal`

以指定误差比较两个数字是否相等。

### 调用

```lua
-- 表参数
math.approx_equal{}
```

### 参数

| 参数名    | 类型    | 必填 | 默认值  | 说明       |
| --------- | ------- | ---- | ------- | ---------- |
| `left`    | float   | 是   | -       | 左侧操作数 |
| `right`   | float   | 是   | -       | 右侧操作数 |
| `epsilon` | float  | 否   | `1e-10` | 误差范围   |

### 返回

直接返回一个值。

| 类型    | 说明     |
| ------- | -------- |
| boolean | 比较结果 |

### 示例

```lua
local ae1 = math.approx_equal { left = 0.1 + 0.2, right = 0.3 }
local ae2 = math.approx_equal { left = 1000000.0, right = 1000000.0000001, epsilon = 1e-10 }

debug.print { message = tostring(ae1) }
debug.print { message = tostring(ae2) }
```

输出：

```text
true
false
```

---

## `percent`

计算百分比 $\frac{value}{total}$。

### 调用

```lua
-- 表参数
math.percent{}
```

### 参数

| 参数名       | 类型    | 必填 | 默认值  | 说明       |
| ------------ | ------- | ---- | ------- | ---------- |
| `value`      | float  | 是   | -       | 分子       |
| `total`      | float  | 是   | -       | 分母       |
| `as_percent` | boolean | 否   | `false` | 百分比输出 |

### 返回

直接返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| float | 百分比数值 |

### 示例

```lua
local p1 = math.percent { value = 25, total = 80 }
debug.print { message = tostring(p1) }

local p2 = math.percent { value = 25, total = 80, as_percent = true }
debug.print { message = tostring(p2) }
```

输出：

```text
0.3125
31.25
```

---

## `factorial`

计算阶乘 $n!$。

### 调用

```lua
-- 单参数
math.factorial(n)
```

### 参数

| 参数名 | 类型    | 必填 | 默认值 | 说明   |
| ------ | ------- | ---- | ------ | ------ |
| `n`    | integer | 是   | -      | 阶乘数 |

### 返回

直接返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| float | 阶乘结果 |

### 示例

```lua
local f = math.factorial(5)
debug.print { message = tostring(f) }
```

输出：

```text
120
```

### 额外补充

- 参数 `n` 范围为 $[0, 170]$。

---

## `combination`

计算组合数 $C^n_k$。

### 调用

```lua
-- 表参数
math.combination{}
```

### 参数

| 参数名 | 类型    | 必填 | 默认值 | 说明   |
| ------ | ------- | ---- | ------ | ------ |
| `n`    | integer | 是   | -      | 总数   |
| `k`    | integer | 是   | -      | 选取数 |

### 返回

直接返回一个值。

| 类型    | 说明     |
| ------- | -------- |
| integer | 组合数值 |

### 示例

```lua
local c = math.combination { n = 5, k = 2 }
debug.print { message = tostring(c) }
```

输出：

```text
10
```

### 额外补充

- 参数 `k` 范围为 $[0, n]$。
