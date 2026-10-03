# date 库

获取当前时间，并在毫秒时间戳与年月日、时分秒之间转换。

---

# 目录

## 常量

| 常量 | 说明 | 定位 |
| --- | --- | --- |
| `LOCAL` | 使用程序运行时的本机时区。 | [LOCAL](#local) |
| `UTC` | 使用 UTC 时间，不加时差。 | [UTC](#utc) |
| `UTC_MINUS_12` | 使用在 UTC 时间上减去 12 小时的固定时区。 | [UTC_MINUS_12](#utc_minus_12) |
| `UTC_MINUS_11` | 使用在 UTC 时间上减去 11 小时的固定时区。 | [UTC_MINUS_11](#utc_minus_11) |
| `UTC_MINUS_10` | 使用在 UTC 时间上减去 10 小时的固定时区。 | [UTC_MINUS_10](#utc_minus_10) |
| `UTC_MINUS_9` | 使用在 UTC 时间上减去 9 小时的固定时区。 | [UTC_MINUS_9](#utc_minus_9) |
| `UTC_MINUS_8` | 使用在 UTC 时间上减去 8 小时的固定时区。 | [UTC_MINUS_8](#utc_minus_8) |
| `UTC_MINUS_7` | 使用在 UTC 时间上减去 7 小时的固定时区。 | [UTC_MINUS_7](#utc_minus_7) |
| `UTC_MINUS_6` | 使用在 UTC 时间上减去 6 小时的固定时区。 | [UTC_MINUS_6](#utc_minus_6) |
| `UTC_MINUS_5` | 使用在 UTC 时间上减去 5 小时的固定时区。 | [UTC_MINUS_5](#utc_minus_5) |
| `UTC_MINUS_4` | 使用在 UTC 时间上减去 4 小时的固定时区。 | [UTC_MINUS_4](#utc_minus_4) |
| `UTC_MINUS_3` | 使用在 UTC 时间上减去 3 小时的固定时区。 | [UTC_MINUS_3](#utc_minus_3) |
| `UTC_MINUS_2` | 使用在 UTC 时间上减去 2 小时的固定时区。 | [UTC_MINUS_2](#utc_minus_2) |
| `UTC_MINUS_1` | 使用在 UTC 时间上减去 1 小时的固定时区。 | [UTC_MINUS_1](#utc_minus_1) |
| `UTC_PLUS_1` | 使用在 UTC 时间上加上 1 小时的固定时区。 | [UTC_PLUS_1](#utc_plus_1) |
| `UTC_PLUS_2` | 使用在 UTC 时间上加上 2 小时的固定时区。 | [UTC_PLUS_2](#utc_plus_2) |
| `UTC_PLUS_3` | 使用在 UTC 时间上加上 3 小时的固定时区。 | [UTC_PLUS_3](#utc_plus_3) |
| `UTC_PLUS_4` | 使用在 UTC 时间上加上 4 小时的固定时区。 | [UTC_PLUS_4](#utc_plus_4) |
| `UTC_PLUS_5` | 使用在 UTC 时间上加上 5 小时的固定时区。 | [UTC_PLUS_5](#utc_plus_5) |
| `UTC_PLUS_6` | 使用在 UTC 时间上加上 6 小时的固定时区。 | [UTC_PLUS_6](#utc_plus_6) |
| `UTC_PLUS_7` | 使用在 UTC 时间上加上 7 小时的固定时区。 | [UTC_PLUS_7](#utc_plus_7) |
| `UTC_PLUS_8` | 使用在 UTC 时间上加上 8 小时的固定时区。 | [UTC_PLUS_8](#utc_plus_8) |
| `UTC_PLUS_9` | 使用在 UTC 时间上加上 9 小时的固定时区。 | [UTC_PLUS_9](#utc_plus_9) |
| `UTC_PLUS_10` | 使用在 UTC 时间上加上 10 小时的固定时区。 | [UTC_PLUS_10](#utc_plus_10) |
| `UTC_PLUS_11` | 使用在 UTC 时间上加上 11 小时的固定时区。 | [UTC_PLUS_11](#utc_plus_11) |
| `UTC_PLUS_12` | 使用在 UTC 时间上加上 12 小时的固定时区。 | [UTC_PLUS_12](#utc_plus_12) |
| `UTC_PLUS_13` | 使用在 UTC 时间上加上 13 小时的固定时区。 | [UTC_PLUS_13](#utc_plus_13) |
| `UTC_PLUS_14` | 使用在 UTC 时间上加上 14 小时的固定时区。 | [UTC_PLUS_14](#utc_plus_14) |
| `TIMESTAMP` | 让 now 返回一个毫秒时间戳。 | [TIMESTAMP](#timestamp) |
| `DATE` | 让 now 返回包含年月日和时间的表。 | [DATE](#date) |

## 方法

| 方法                  | 说明            | 定位                                      |
| ------------------- | ------------- | --------------------------------------- |
| `now`               | 获取当前时间        | [now](#now)                             |
| `date_to_timestamp` | 日期转为毫秒时间戳     | [date_to_timestamp](#date_to_timestamp) |
| `timestamp_to_date` | 毫秒时间戳转为日期     | [timestamp_to_date](#timestamp_to_date) |
| `timestamp_diff`    | 计算两个时间戳相差的毫秒数 | [timestamp_diff](#timestamp_diff)       |

---

# 常量

## `LOCAL`

使用程序运行时的本机时区。

### 调用

```lua
date.LOCAL
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.LOCAL)
```

**输出：**

```lua
```

### 等值

```text
"local"
```

### 额外说明

跟随运行程序的电脑所设置的时区，与脚本语言、包语言和界面语言无关；本机采用夏令时时按该日期的规则转换。

---

## `UTC`

使用 UTC 时间，不加时差。

### 调用

```lua
date.UTC
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC)
```

**输出：**

```lua
```

### 等值

```text
"utc"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_MINUS_12`

使用在 UTC 时间上减去 12 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_12
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_MINUS_12)
```

**输出：**

```lua
```

### 等值

```text
"utc-12"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_MINUS_11`

使用在 UTC 时间上减去 11 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_11
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_MINUS_11)
```

**输出：**

```lua
```

### 等值

```text
"utc-11"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_MINUS_10`

使用在 UTC 时间上减去 10 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_10
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_MINUS_10)
```

**输出：**

```lua
```

### 等值

```text
"utc-10"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_MINUS_9`

使用在 UTC 时间上减去 9 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_9
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_MINUS_9)
```

**输出：**

```lua
```

### 等值

```text
"utc-9"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_MINUS_8`

使用在 UTC 时间上减去 8 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_8
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_MINUS_8)
```

**输出：**

```lua
```

### 等值

```text
"utc-8"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_MINUS_7`

使用在 UTC 时间上减去 7 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_7
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_MINUS_7)
```

**输出：**

```lua
```

### 等值

```text
"utc-7"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_MINUS_6`

使用在 UTC 时间上减去 6 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_6
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_MINUS_6)
```

**输出：**

```lua
```

### 等值

```text
"utc-6"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_MINUS_5`

使用在 UTC 时间上减去 5 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_5
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_MINUS_5)
```

**输出：**

```lua
```

### 等值

```text
"utc-5"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_MINUS_4`

使用在 UTC 时间上减去 4 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_4
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_MINUS_4)
```

**输出：**

```lua
```

### 等值

```text
"utc-4"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_MINUS_3`

使用在 UTC 时间上减去 3 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_3
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_MINUS_3)
```

**输出：**

```lua
```

### 等值

```text
"utc-3"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_MINUS_2`

使用在 UTC 时间上减去 2 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_2
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_MINUS_2)
```

**输出：**

```lua
```

### 等值

```text
"utc-2"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_MINUS_1`

使用在 UTC 时间上减去 1 小时的固定时区。

### 调用

```lua
date.UTC_MINUS_1
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_MINUS_1)
```

**输出：**

```lua
```

### 等值

```text
"utc-1"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_PLUS_1`

使用在 UTC 时间上加上 1 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_1
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_PLUS_1)
```

**输出：**

```lua
```

### 等值

```text
"utc+1"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_PLUS_2`

使用在 UTC 时间上加上 2 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_2
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_PLUS_2)
```

**输出：**

```lua
```

### 等值

```text
"utc+2"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_PLUS_3`

使用在 UTC 时间上加上 3 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_3
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_PLUS_3)
```

**输出：**

```lua
```

### 等值

```text
"utc+3"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_PLUS_4`

使用在 UTC 时间上加上 4 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_4
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_PLUS_4)
```

**输出：**

```lua
```

### 等值

```text
"utc+4"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_PLUS_5`

使用在 UTC 时间上加上 5 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_5
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_PLUS_5)
```

**输出：**

```lua
```

### 等值

```text
"utc+5"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_PLUS_6`

使用在 UTC 时间上加上 6 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_6
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_PLUS_6)
```

**输出：**

```lua
```

### 等值

```text
"utc+6"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_PLUS_7`

使用在 UTC 时间上加上 7 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_7
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_PLUS_7)
```

**输出：**

```lua
```

### 等值

```text
"utc+7"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_PLUS_8`

使用在 UTC 时间上加上 8 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_8
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_PLUS_8)
```

**输出：**

```lua
```

### 等值

```text
"utc+8"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_PLUS_9`

使用在 UTC 时间上加上 9 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_9
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_PLUS_9)
```

**输出：**

```lua
```

### 等值

```text
"utc+9"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_PLUS_10`

使用在 UTC 时间上加上 10 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_10
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_PLUS_10)
```

**输出：**

```lua
```

### 等值

```text
"utc+10"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_PLUS_11`

使用在 UTC 时间上加上 11 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_11
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_PLUS_11)
```

**输出：**

```lua
```

### 等值

```text
"utc+11"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_PLUS_12`

使用在 UTC 时间上加上 12 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_12
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_PLUS_12)
```

**输出：**

```lua
```

### 等值

```text
"utc+12"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_PLUS_13`

使用在 UTC 时间上加上 13 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_13
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_PLUS_13)
```

**输出：**

```lua
```

### 等值

```text
"utc+13"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `UTC_PLUS_14`

使用在 UTC 时间上加上 14 小时的固定时区。

### 调用

```lua
date.UTC_PLUS_14
```

### 可用于

- 参数 `now.timezone`
- 参数 `date_to_timestamp.timezone`
- 参数 `timestamp_to_date.timezone`

### 示例

```lua
debug.print(date.UTC_PLUS_14)
```

**输出：**

```lua
```

### 等值

```text
"utc+14"
```

### 额外说明

固定时区没有夏令时切换。`date.UTC` 表示零时差。

---

## `TIMESTAMP`

让 now 返回一个毫秒时间戳。

### 调用

```lua
date.TIMESTAMP
```

### 可用于

- 参数 `now.time_type`

### 示例

```lua
debug.print(date.TIMESTAMP)
```

**输出：**

```lua
```

### 等值

```text
"timestamp"
```

---

## `DATE`

让 now 返回包含年月日和时间的表。

### 调用

```lua
date.DATE
```

### 可用于

- 参数 `now.time_type`

### 示例

```lua
debug.print(date.DATE)
```

**输出：**

```lua
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

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `timezone` | const-date | 时区，默认 `date.LOCAL` |
| `time_type` | const-date | 返回内容，默认 `date.TIMESTAMP` |

## 返回值

`time_type` 为 `date.TIMESTAMP` 时，返回一个值。

| 类型 | 说明 |
| --- | --- |
| integer | 从 1970 年 1 月 1 日 UTC 零点起经过的毫秒数 |

`time_type` 为 `date.DATE` 时，返回一个对象表。

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `year` | integer | 年 |
| `month` | integer | 月，1～12 |
| `day` | integer | 日，从 1 开始 |
| `hour` | integer | 时，0～23 |
| `minute` | integer | 分，0～59 |
| `second` | integer | 秒，0～59 |
| `millisecond` | integer | 毫秒，0～999 |

### 示例

```lua
local timestamp = date.now()
debug.print(timestamp)
local current = date.now({timezone = date.UTC_PLUS_8, time_type = date.DATE})
debug.print(current)
```

**输出：**

```lua
```

### 额外说明

没有必填参数；选填参数用末尾的表显式填写，例如 `date.now({timezone = date.UTC, time_type = date.DATE})`。使用默认设置时调用 `date.now()` 即可。未知选项、错误类型或多余参数会报错。

时间戳表示一个确定的时刻，因此切换时区不会给时间戳加减时差；`timezone` 只改变日期表中的年月日、时分秒。调用间隔中时间仍会流逝，不保证两次调用返回完全相同的时间戳。电脑校时可能使当前时间前进或后退。

---

## `date_to_timestamp`

把年月日、时分秒和毫秒转为时间戳。

### 调用

```lua
date.date_to_timestamp
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `year` | integer | 年 |
| `month` | integer | 月，1～12 |
| `day` | integer | 日，从 1 开始，必须是该月份实际存在的日期 |
| `hour` | integer | 时，0～23 |
| `minute` | integer | 分，0～59 |
| `second` | integer | 秒，0～59 |
| `millisecond` | integer | 毫秒，0～999；没有毫秒部分时明确填写 0 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `timezone` | const-date | 输入日期所属的时区，默认 `date.LOCAL` |

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| integer | Unix 毫秒时间戳 |

### 示例

```lua
local timestamp = date.date_to_timestamp(
  1970, 1, 1, 8, 0, 0, 123,
  {timezone = date.UTC_PLUS_8}
)
debug.assert(timestamp == 123)
debug.print(timestamp)
```

**输出：**

```lua
```

### 额外说明

必填参数按 `year, month, day, hour, minute, second, millisecond` 的顺序传入，不能省略；选填 `timezone` 写在最后的选项表中。不传选项表时使用本机时区。未知选项、错误类型、多余参数、不存在的日期、超出范围的时间或无法转换的年份会报错；不会自动把 2 月 30 日纠正成 3 月的日期。秒只能为 0～59，不接收闰秒。年份按公历计算，0 年表示公元前 1 年。

`date.LOCAL` 下遇到夏令时切换，某些时间可能不存在或对应两个时刻；此时会报错，请明确选择对应的固定 UTC 偏移。固定时区不随夏令时变化。

早于 1970 年 UTC 零点的时刻返回负数。转换使用填写的时区，与界面语言无关。

---

## `timestamp_to_date`

把毫秒时间戳转为指定时区的年月日、时分秒和毫秒。

### 调用

```lua
date.timestamp_to_date
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `timestamp` | integer | Unix 毫秒时间戳，可以是负数 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `timezone` | const-date | 输出日期的时区，默认 `date.LOCAL` |

## 返回值

返回一个对象表。

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `year` | integer | 年 |
| `month` | integer | 月，1～12 |
| `day` | integer | 日，从 1 开始 |
| `hour` | integer | 时，0～23 |
| `minute` | integer | 分，0～59 |
| `second` | integer | 秒，0～59 |
| `millisecond` | integer | 毫秒，0～999 |

### 示例

```lua
local result = date.timestamp_to_date(-1, {timezone = date.UTC})
debug.assert(result.year == 1969 and result.month == 12 and result.day == 31)
debug.assert(result.hour == 23 and result.minute == 59 and result.second == 59)
debug.assert(result.millisecond == 999)
debug.print(result)
```

**输出：**

```lua
```

### 额外说明

时间戳作为第一个参数传入；选填 `timezone` 写在最后的选项表中，例如 `date.timestamp_to_date(-1, {timezone = date.UTC})`。不传选项表时使用本机时区。未知选项、错误类型、多余参数或超出可转换范围的时间戳会报错。输出日期使用公历；年份 0 表示公元前 1 年。

结果本身是一个表，所以返回日期表，而不是把七个字段拆成七个返回值。可修改这个表；需要转回原时间戳时，按顺序把七个字段传给 `date.date_to_timestamp`，并在末尾选项表中指定转换时使用的同一个 `timezone`。`date.LOCAL` 下的夏令时重复时间可能无法唯一转回时间戳。

---

## `timestamp_diff`

计算两个时间戳之间相差多少毫秒，两侧相等时返回 0。

### 调用

```lua
date.timestamp_diff
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `left_timestamp` | integer | 较早的 Unix 毫秒时间戳，可以是负数 |
| `right_timestamp` | integer | 右侧的 Unix 毫秒时间戳，必须大于或等于 `left_timestamp` |

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| integer | 两个时间戳相差的毫秒数，大于或等于 0 |

### 示例

```lua
local difference = date.timestamp_diff(1000, 2500)
debug.assert(difference == 1500)
debug.assert(date.timestamp_diff(1000, 1000) == 0)
debug.print(difference)
```

**输出：**

```lua
```

### 额外说明

两个必填时间戳按左、右的顺序传入，此方法没有选填参数，不接受额外的选项表。右时间戳必须大于或等于左时间戳；相等时返回 0，右侧更小时报错，不会自动交换两侧。结果是 `right_timestamp - left_timestamp` 的非负值，也就是差值的绝对值，单位始终是毫秒。

两侧都可以是负数，也可以跨过 Unix 零点。不需要时区参数，因为时间戳已经表示一个确定的时刻。此方法直接计算整数差值，无需先转成年月日。

缺少参数、错误类型、多余参数或差值超出整数范围都会报错。可返回的最大差值为 9223372036854775807 毫秒；超出时不会返回负数或不准确的小数。
