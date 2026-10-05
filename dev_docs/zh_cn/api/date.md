# date 库

获取当前时间，并在毫秒时间戳与年月日、时分秒之间转换。

---

# 目录

## 常量

| 常量           | 说明                                    | 定位                          |
| -------------- | --------------------------------------- | ----------------------------- |
| `LOCAL`        | 使用程序运行时的本机时区                | [LOCAL](#local)               |
| `UTC`          | 使用 UTC 固定时区                       | [UTC](#utc)                   |
| `UTC_MINUS_12` | 使用在 UTC 时间上减去 12 小时的固定时区 | [UTC_MINUS_12](#utc_minus_12) |
| `UTC_MINUS_11` | 使用在 UTC 时间上减去 11 小时的固定时区 | [UTC_MINUS_11](#utc_minus_11) |
| `UTC_MINUS_10` | 使用在 UTC 时间上减去 10 小时的固定时区 | [UTC_MINUS_10](#utc_minus_10) |
| `UTC_MINUS_9`  | 使用在 UTC 时间上减去 9 小时的固定时区  | [UTC_MINUS_9](#utc_minus_9)   |
| `UTC_MINUS_8`  | 使用在 UTC 时间上减去 8 小时的固定时区  | [UTC_MINUS_8](#utc_minus_8)   |
| `UTC_MINUS_7`  | 使用在 UTC 时间上减去 7 小时的固定时区  | [UTC_MINUS_7](#utc_minus_7)   |
| `UTC_MINUS_6`  | 使用在 UTC 时间上减去 6 小时的固定时区  | [UTC_MINUS_6](#utc_minus_6)   |
| `UTC_MINUS_5`  | 使用在 UTC 时间上减去 5 小时的固定时区  | [UTC_MINUS_5](#utc_minus_5)   |
| `UTC_MINUS_4`  | 使用在 UTC 时间上减去 4 小时的固定时区  | [UTC_MINUS_4](#utc_minus_4)   |
| `UTC_MINUS_3`  | 使用在 UTC 时间上减去 3 小时的固定时区  | [UTC_MINUS_3](#utc_minus_3)   |
| `UTC_MINUS_2`  | 使用在 UTC 时间上减去 2 小时的固定时区  | [UTC_MINUS_2](#utc_minus_2)   |
| `UTC_MINUS_1`  | 使用在 UTC 时间上减去 1 小时的固定时区  | [UTC_MINUS_1](#utc_minus_1)   |
| `UTC_PLUS_1`   | 使用在 UTC 时间上加上 1 小时的固定时区  | [UTC_PLUS_1](#utc_plus_1)     |
| `UTC_PLUS_2`   | 使用在 UTC 时间上加上 2 小时的固定时区  | [UTC_PLUS_2](#utc_plus_2)     |
| `UTC_PLUS_3`   | 使用在 UTC 时间上加上 3 小时的固定时区  | [UTC_PLUS_3](#utc_plus_3)     |
| `UTC_PLUS_4`   | 使用在 UTC 时间上加上 4 小时的固定时区  | [UTC_PLUS_4](#utc_plus_4)     |
| `UTC_PLUS_5`   | 使用在 UTC 时间上加上 5 小时的固定时区  | [UTC_PLUS_5](#utc_plus_5)     |
| `UTC_PLUS_6`   | 使用在 UTC 时间上加上 6 小时的固定时区  | [UTC_PLUS_6](#utc_plus_6)     |
| `UTC_PLUS_7`   | 使用在 UTC 时间上加上 7 小时的固定时区  | [UTC_PLUS_7](#utc_plus_7)     |
| `UTC_PLUS_8`   | 使用在 UTC 时间上加上 8 小时的固定时区  | [UTC_PLUS_8](#utc_plus_8)     |
| `UTC_PLUS_9`   | 使用在 UTC 时间上加上 9 小时的固定时区  | [UTC_PLUS_9](#utc_plus_9)     |
| `UTC_PLUS_10`  | 使用在 UTC 时间上加上 10 小时的固定时区 | [UTC_PLUS_10](#utc_plus_10)   |
| `UTC_PLUS_11`  | 使用在 UTC 时间上加上 11 小时的固定时区 | [UTC_PLUS_11](#utc_plus_11)   |
| `UTC_PLUS_12`  | 使用在 UTC 时间上加上 12 小时的固定时区 | [UTC_PLUS_12](#utc_plus_12)   |
| `UTC_PLUS_13`  | 使用在 UTC 时间上加上 13 小时的固定时区 | [UTC_PLUS_13](#utc_plus_13)   |
| `UTC_PLUS_14`  | 使用在 UTC 时间上加上 14 小时的固定时区 | [UTC_PLUS_14](#utc_plus_14)   |
| `TIMESTAMP`    | 返回毫秒时间戳                          | [TIMESTAMP](#timestamp)       |
| `DATE`         | 返回包含年月日和时间的表                | [DATE](#date)                 |

## 方法

| 方法                | 说明                                                             | 定位                                    |
| ------------------- | ---------------------------------------------------------------- | --------------------------------------- |
| `now`               | 获取当前时间                                                     | [now](#now)                             |
| `date_to_timestamp` | 把年月日、时分秒和毫秒转为时间戳                                 | [date_to_timestamp](#date_to_timestamp) |
| `timestamp_to_date` | 把毫秒时间戳转为指定时区的年月日、时分秒和毫秒                   | [timestamp_to_date](#timestamp_to_date) |
| `timestamp_diff`    | 计算两个时间点之间相差多少毫秒，可传入时间戳、日期表，或混合使用 | [timestamp_diff](#timestamp_diff)       |

---

# 常量

## `LOCAL`

使用程序运行时的本机时区。

### 调用

```lua
date.LOCAL
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.LOCAL, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 0,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"local"
```

## 额外说明

- 本地时间跟随程序运行系统所设时区，且按照夏令规则做时间转换。

---

## `UTC`

使用 UTC 固定时区。

### 调用

```lua
date.UTC
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 0,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_MINUS_12`

使用在 UTC 时间上减去 12 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_12
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_MINUS_12, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 1999,
  month = 12,
  day = 31,
  hour = 12,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc-12"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_MINUS_11`

使用在 UTC 时间上减去 11 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_11
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_MINUS_11, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 1999,
  month = 12,
  day = 31,
  hour = 13,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc-11"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_MINUS_10`

使用在 UTC 时间上减去 10 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_10
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_MINUS_10, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 1999,
  month = 12,
  day = 31,
  hour = 14,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc-10"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_MINUS_9`

使用在 UTC 时间上减去 9 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_9
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_MINUS_9, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 1999,
  month = 12,
  day = 31,
  hour = 15,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc-9"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_MINUS_8`

使用在 UTC 时间上减去 8 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_8
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_MINUS_8, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 1999,
  month = 12,
  day = 31,
  hour = 16,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc-8"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_MINUS_7`

使用在 UTC 时间上减去 7 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_7
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_MINUS_7, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 1999,
  month = 12,
  day = 31,
  hour = 17,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc-7"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_MINUS_6`

使用在 UTC 时间上减去 6 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_6
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_MINUS_6, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 1999,
  month = 12,
  day = 31,
  hour = 18,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc-6"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_MINUS_5`

使用在 UTC 时间上减去 5 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_5
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_MINUS_5, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 1999,
  month = 12,
  day = 31,
  hour = 19,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc-5"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_MINUS_4`

使用在 UTC 时间上减去 4 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_4
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_MINUS_4, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 1999,
  month = 12,
  day = 31,
  hour = 20,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc-4"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_MINUS_3`

使用在 UTC 时间上减去 3 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_3
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_MINUS_3, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 1999,
  month = 12,
  day = 31,
  hour = 21,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc-3"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_MINUS_2`

使用在 UTC 时间上减去 2 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_2
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_MINUS_2, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 1999,
  month = 12,
  day = 31,
  hour = 22,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc-2"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_MINUS_1`

使用在 UTC 时间上减去 1 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_1
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_MINUS_1, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 1999,
  month = 12,
  day = 31,
  hour = 23,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc-1"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_PLUS_1`

使用在 UTC 时间上加上 1 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_1
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_PLUS_1, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 1,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc+1"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_PLUS_2`

使用在 UTC 时间上加上 2 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_2
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_PLUS_2, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 2,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc+2"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_PLUS_3`

使用在 UTC 时间上加上 3 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_3
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_PLUS_3, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 3,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc+3"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_PLUS_4`

使用在 UTC 时间上加上 4 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_4
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_PLUS_4, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 4,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc+4"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_PLUS_5`

使用在 UTC 时间上加上 5 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_5
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_PLUS_5, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 5,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc+5"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_PLUS_6`

使用在 UTC 时间上加上 6 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_6
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_PLUS_6, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 6,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc+6"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_PLUS_7`

使用在 UTC 时间上加上 7 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_7
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_PLUS_7, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 7,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc+7"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_PLUS_8`

使用在 UTC 时间上加上 8 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_8
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_PLUS_8, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 8,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc+8"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_PLUS_9`

使用在 UTC 时间上加上 9 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_9
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_PLUS_9, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 9,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc+9"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_PLUS_10`

使用在 UTC 时间上加上 10 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_10
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_PLUS_10, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 10,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc+10"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_PLUS_11`

使用在 UTC 时间上加上 11 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_11
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_PLUS_11, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 11,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc+11"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_PLUS_12`

使用在 UTC 时间上加上 12 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_12
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_PLUS_12, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 12,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc+12"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_PLUS_13`

使用在 UTC 时间上加上 13 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_13
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_PLUS_13, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 13,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc+13"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `UTC_PLUS_14`

使用在 UTC 时间上加上 14 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_14
```

### 可用于

- 参数 `timezone`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC_PLUS_14, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 14,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"utc+14"
```

## 额外说明

- 该常量无夏令规则转换。

---

## `TIMESTAMP`

返回毫秒时间戳。

### 调用

```lua
date.TIMESTAMP
```

### 可用于

- 参数 `time_type`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(date.now({ timezone = date.UTC, time_type = date.TIMESTAMP }))
```

**输出：**

```lua
946684800000
```

### 等值

```text
"timestamp"
```

---

## `DATE`

返回包含年月日和时间的表。

### 调用

```lua
date.DATE
```

### 可用于

- 参数 `time_type`

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.now({ timezone = date.UTC, time_type = date.DATE })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 0,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

### 等值

```text
"date"
```

---

# 方法

## `now`

获取当前时间。

### 调用

```lua
date.now
```

## 参数

### 选填参数

| 参数名      | 类型       | 默认值           | 说明     |
| ----------- | ---------- | ---------------- | -------- |
| `timezone`  | const-date | `date.LOCAL`     | 时区     |
| `time_type` | const-date | `date.TIMESTAMP` | 返回格式 |

## 返回值

**选填参数 `time_type` 为 `date.TIMESTAMP` 时**，返回一个值。

| 类型    | 说明       |
| ------- | ---------- |
| integer | 毫秒时间戳 |

**选填参数 `time_type` 为 `date.DATE` 时**，返回一个对象表。

| 字段          | 类型    | 说明 |
| ------------- | ------- | ---- |
| `year`        | integer | 年   |
| `month`       | integer | 月   |
| `day`         | integer | 日   |
| `hour`        | integer | 时   |
| `minute`      | integer | 分   |
| `second`      | integer | 秒   |
| `millisecond` | integer | 毫秒 |

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(date.now({ timezone = date.UTC, time_type = date.TIMESTAMP }))
debug.print(table.pretty(date.now({ timezone = date.UTC, time_type = date.DATE })))
```

**输出：**

```lua
946684800000
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 0,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

## 额外说明

- 返回对象表字段 `month` 取值范围为 $[1,12]$ 。
- 返回对象表字段 `day` 取值范围为 $[1,x]$，$x$ 为当前月份总日数。
- 返回对象表字段 `hour` 取值范围为 $[0,23]$ 。
- 返回对象表字段 `minute` 取值范围为 $[0,59]$ 。
- 返回对象表字段 `second` 取值范围为 $[0,59]$ 。
- 返回对象表字段 `millisecond` 取值范围为 $[0,999]$ 。

---

## `date_to_timestamp`

把年月日、时分秒和毫秒转为时间戳。

### 调用

```lua
date.date_to_timestamp
```

## 参数

### 必填参数

| 参数名        | 类型    | 说明 |
| ------------- | ------- | ---- |
| `year`        | integer | 年   |
| `month`       | integer | 月   |
| `day`         | integer | 日   |
| `hour`        | integer | 时   |
| `minute`      | integer | 分   |
| `second`      | integer | 秒   |
| `millisecond` | integer | 毫秒 |

### 选填参数

| 参数名     | 类型       | 默认值       | 说明 |
| ---------- | ---------- | ------------ | ---- |
| `timezone` | const-date | `date.LOCAL` | 时区 |

## 返回值

返回一个值。

| 类型    | 说明       |
| ------- | ---------- |
| integer | 毫秒时间戳 |

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(date.date_to_timestamp(2000, 1, 1, 0, 0, 0, 0, { timezone = date.UTC }))
```

**输出：**

```lua
946684800000
```

## 额外说明

- 必填参数 `month` 取值范围为 $[1,12]$ 。
- 必填参数 `day` 取值范围为 $[1,x]$，$x$ 为当前月份总日数。
- 必填参数 `hour` 取值范围为 $[0,23]$ 。
- 必填参数 `minute` 取值范围为 $[0,59]$ 。
- 必填参数 `second` 取值范围为 $[0,59]$ 。
- 必填参数 `millisecond` 取值范围为 $[0,999]$ 。

---

## `timestamp_to_date`

把毫秒时间戳转为指定时区的年月日、时分秒和毫秒。

### 调用

```lua
date.timestamp_to_date
```

## 参数

### 必填参数

| 参数名      | 类型    | 说明       |
| ----------- | ------- | ---------- |
| `timestamp` | integer | 毫秒时间戳 |

### 选填参数

| 参数名     | 类型       | 默认值       | 说明 |
| ---------- | ---------- | ------------ | ---- |
| `timezone` | const-date | `date.LOCAL` | 时区 |

## 返回值

返回一个对象表。

| 字段          | 类型    | 说明 |
| ------------- | ------- | ---- |
| `year`        | integer | 年   |
| `month`       | integer | 月   |
| `day`         | integer | 日   |
| `hour`        | integer | 时   |
| `minute`      | integer | 分   |
| `second`      | integer | 秒   |
| `millisecond` | integer | 毫秒 |

### 示例

> 2000/1/1 00:00:00 UTC

```lua
debug.print(table.pretty(date.timestamp_to_date(946684800000, { timezone = date.UTC })))
```

**输出：**

```lua
{
  year = 2000,
  month = 1,
  day = 1,
  hour = 0,
  minute = 0,
  second = 0,
  millisecond = 0
}
```

## 额外说明

- 返回对象表字段 `month` 取值范围为 $[1,12]$ 。
- 返回对象表字段 `day` 取值范围为 $[1,x]$，$x$ 为当前月份总日数。
- 返回对象表字段 `hour` 取值范围为 $[0,23]$ 。
- 返回对象表字段 `minute` 取值范围为 $[0,59]$ 。
- 返回对象表字段 `second` 取值范围为 $[0,59]$ 。
- 返回对象表字段 `millisecond` 取值范围为 $[0,999]$ 。

---

## `timestamp_diff`

计算两个时间点之间相差多少毫秒，可传入时间戳、日期表，或混合使用。

### 调用

```lua
date.timestamp_diff
```

## 参数

### 必填参数

| 参数名            | 类型            | 说明                     |
| ----------------- | --------------- | ------------------------ |
| `early_timestamp` | integer / table | 较早的毫秒时间戳或日期表 |
| `later_timestamp` | integer / table | 较晚的毫秒时间戳或日期表 |

### 选填参数

| 参数名     | 类型       | 默认值       | 说明                                       |
| ---------- | ---------- | ------------ | ------------------------------------------ |
| `timezone` | const-date | `date.LOCAL` | 日期表所属的时区；整数时间戳不受该选项影响 |

## 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| integer | 相差的毫秒数 |

### 示例

> 2000/1/1 00:00:00 UTC 至 2001/1/1 00:00:00 UTC

```lua
local diff = date.timestamp_diff(946684800000, {
  year = 2001,
  month = 1,
  day = 1,
  hour = 0,
  minute = 0,
  second = 0,
  millisecond = 0
}, { timezone = date.UTC })

debug.print(diff)
```

**输出：**

```lua
31622400000
```

## 额外说明

- 必填参数 `early_timestamp` 和必填参数 `later_timestamp` 为日期表时必须包含以下字段：

| 字段名        | 类型    | 说明 | 取值范围                      |
| ------------- | ------- | ---- | ----------------------------- |
| `year`        | integer | 年   | -                             |
| `month`       | integer | 月   | $[1,12]$                      |
| `day`         | integer | 日   | $[1,x]$，$x$ 为当前月份总日数 |
| `hour`        | integer | 时   | $[0,23]$                      |
| `minute`      | integer | 分   | $[0,59]$                      |
| `second`      | integer | 秒   | $[0,59]$                      |
| `millisecond` | integer | 毫秒 | $[0,999]$                     |

- 必填参数 `early_timestamp` 必须小于等于必填参数 `later_timestamp`。
- 返回值取绝对值。
