# timer 库

`timer` 用于等待一段时间后通知脚本，也可以按间隔重复通知。

---

# 目录

## 方法

| 方法       | 说明                         | 定位                  |
| ---------- | ---------------------------- | --------------------- |
| `create`   | 创建一个计时器               | [create](#create)     |
| `list`     | 查看所有计时器的信息         | [list](#list)         |
| `count`    | 查看计时器总数               | [count](#count)       |
| `delete`   | 删除一个计时器               | [delete](#delete)     |
| `clear`    | 删除所有计时器               | [clear](#clear)       |
| `exists`   | 检查一个计时器是否还在       | [exists](#exists)     |
| `set`      | 修改一个计时器的参数         | [set](#set)           |
| `get_info` | 查看一个计时器的参数和状态   | [get_info](#get_info) |
| `start`    | 开始或继续计时               | [start](#start)       |
| `pause`    | 暂停计时，保留已经走过的时间 | [pause](#pause)       |
| `reset`    | 清零计时进度，回到未启动状态 | [reset](#reset)       |
| `restart`  | 清零计时进度，然后重新开始   | [restart](#restart)   |

---

# 方法

## `create`

创建一个计时器。

### 调用

```lua
timer.create
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `duration` | integer / float | 每次计时的时长，单位为秒 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `delay` | integer / float | `0` | 首次启动前的等待时间；每轮之间也会加上它，单位为秒 |
| `loop` | boolean | `false` | 是否循环 |
| `interval` | integer / float | `0` | 每轮之间额外等待的时间，允许负数，单位为秒 |
| `repeat` | integer / nil / `false` | `nil` | 循环时的总触发次数；`false` 表示不限次数；`loop` 为 `false` 时该值不生效，实际只触发一次 |
| `callback` | function / nil / `false` | `nil` | 接收事件的函数；`false` 表示清空 |
| `tip` | string / nil / `false` | `nil` | 随事件返回的自定义文字；`false` 表示清空 |

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| string / nil | 计时器 ID，达到数量上限或无法创建时为 `nil` |

### 示例

```lua
local id = timer.create(1.5, {delay = 0.2, loop = true, ["repeat"] = 3})
timer.start(id)
```

**输出：**

```lua
```

## 额外说明

- 必填参数按顺序传入，选填参数放在最后的表里。未知字段会抛出错误。
- 时间单位为秒，允许小数。duration、delay 必须是非负有限数，interval 必须是有限数；delay + interval 必须大于或等于 0，超过可表示范围也会抛出错误。
- 首次触发需要等待 delay + duration；两次触发之间需要等待 delay + interval + duration。loop 为 `false` 时只触发一次；为 `true` 时，repeat 限制总触发次数，省略或设为 `false` 表示不限次数。
- repeat 是 Lua 关键字，必须写成 `["repeat"] = 3`，不能写 `repeat = 3`。`repeat` 的取值须为 1 到 4294967295 的整数。
- 创建后状态为 idle，需要调用 start 或 restart 才开始计时。结束后的对象仍可查询、修改和重启，不会自动删除。
- 每帧每个计时器最多触发一次。长帧未消耗完的计时进度留到后续帧继续处理；零时长循环也遵守这个限制。
- callback 收到与 HandleEvent 相同的事件表。提供 callback 时只调用它；不提供时交给 HandleEvent。两者不会重复接收同一事件。
- 非末次触发的 kind 为 tick，末次触发为 finished；不限次数的循环只产生 tick。事件包含 ID、触发次数和 tip。查看⌊[计时器事件](../EVENT.md#41-timer)⌉。
- 回调里可以创建、修改或删除计时器；新计时器最早在下一帧触发。回调发生错误或超过执行限额时，会停止对应脚本。
- 游戏和屏保分别管理自己的计时器，每个脚本最多创建 1024 个。事件投递上限查看⌊[计时器事件](../EVENT.md#41-timer)⌉。
- 覆盖屏不会自动暂停计时器；游戏被遮挡时仍会计时。需要暂停时主动调用 pause。
- 计时按纳秒精度执行，更小的数值会按纳秒舍入；get_info 返回的 duration、delay 和 interval 是传入的原始数值，不参与舍入。

---

## `list`

查看所有计时器的信息。

### 调用

```lua
timer.list
```

## 参数

无。

## 返回值

返回一个表。

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `1..n` | table | 每个计时器的信息表 |
| `n` | integer | 计时器总数 |

### 示例

```lua
timer.create(1)
timer.create(2)
local timers = timer.list()
debug.assert(timers.n == 2 and timers[1].duration == 1)
```

**输出：**

```lua
```

## 额外说明

- 返回的数字下标从 1 开始，按创建顺序排列。
- 每项的字段与 get_info 一致。返回的信息是当前快照，修改表不会修改计时器。查看⌊[get_info](#get_info)⌉。

---

## `count`

查看计时器总数。

### 调用

```lua
timer.count
```

## 参数

无。

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| integer | 计时器总数 |

### 示例

```lua
timer.create(1)
debug.assert(timer.count() == 1)
```

**输出：**

```lua
```

## 额外说明

- 已结束但尚未删除的计时器也计入总数。

---

## `delete`

删除一个计时器。

### 调用

```lua
timer.delete
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 计时器 ID |

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 操作是否成功 |

### 示例

```lua
local id = timer.create(1)
debug.assert(timer.delete(id))
debug.assert(not timer.exists(id))
```

**输出：**

```lua
```

## 额外说明

- ID 格式错误会抛出错误；格式正确但对象不存在时返回 `false`。删除后，尚未投递的旧事件也不再触发。

---

## `clear`

删除所有计时器。

### 调用

```lua
timer.clear
```

## 参数

无。

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 操作是否成功 |

### 示例

```lua
timer.create(1)
debug.assert(timer.clear() and timer.count() == 0)
```

**输出：**

```lua
```

## 额外说明

- 删除后，尚未投递的旧事件也不再触发；重复清空仍返回 `true`。不会删除其他脚本的计时器。

---

## `exists`

检查一个计时器是否还在。

### 调用

```lua
timer.exists
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 计时器 ID |

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 是否存在 |

### 示例

```lua
local id = timer.create(1)
debug.assert(timer.exists(id))
```

**输出：**

```lua
```

## 额外说明

- ID 格式错误会抛出错误；格式正确但对象不存在时返回 `false`。

---

## `set`

修改一个计时器的参数。

### 调用

```lua
timer.set
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 计时器 ID |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `duration` | integer / float | 保持原值 | 每次计时的时长，单位为秒 |
| `delay` | integer / float | 保持原值 | 首次启动前的等待时间；每轮之间也会加上它，单位为秒 |
| `loop` | boolean | 保持原值 | 是否循环 |
| `interval` | integer / float | 保持原值 | 每轮之间额外等待的时间，允许负数，单位为秒 |
| `repeat` | integer / nil / `false` | 保持原值 | 循环时的总触发次数；`false` 表示不限次数；`loop` 为 `false` 时该值不生效，实际只触发一次 |
| `callback` | function / nil / `false` | 保持原值 | 接收事件的函数；`false` 表示清空 |
| `tip` | string / nil / `false` | 保持原值 | 随事件返回的自定义文字；`false` 表示清空 |

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 操作是否成功 |

### 示例

```lua
local id = timer.create(1, {tip = "提醒", loop = true, ["repeat"] = 3})
timer.start(id)
debug.assert(timer.set(id, {duration = 2, tip = false, ["repeat"] = false}))
local info = timer.get_info(id)
debug.assert(info.state == "idle" and info.duration == 2 and info.tip == nil)
timer.start(id)
```

**输出：**

```lua
```

## 额外说明

- 选填字段省略或写 `nil` 时保持原值。callback、tip 写 `false` 会清空；repeat 写 `false` 会取消次数限制。Lua 表中 `nil` 会移除字段，因此不能用 `nil` 表示清空旧值。
- 修改成功后清零进度和触发次数，状态变为 idle，需要再调用 start。即使传入空表或省略选填表，也会重置。
- 参数无效时会抛出错误，原配置和进度保持不变。未知字段会抛出错误。
- 时间、循环次数的限制与 create 相同，repeat 仍须用 `["repeat"]` 写法。查看⌊[create](#create)⌉。
- 修改后，尚未投递的旧事件不再触发。ID 格式错误会抛出错误；格式正确但对象不存在时返回 `false`。

---

## `get_info`

查看一个计时器的参数和状态。

### 调用

```lua
timer.get_info
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 计时器 ID |

## 返回值

**对象存在时**，返回一个信息表；**不存在时**，返回 `nil`。

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 计时器 ID |
| `duration` | number | 每次计时的时长，单位为秒 |
| `delay` | number | 首次和后续循环的延时，单位为秒 |
| `loop` | boolean | 是否循环 |
| `interval` | number | 循环额外等待时间，单位为秒 |
| `repeat` | integer / nil | 循环时的总触发次数；`nil` 表示不限次数；`loop` 为 `false` 时该字段只回显配置值，不代表实际触发次数 |
| `callback` | function / nil | 接收计时器事件的函数 |
| `tip` | string / nil | 随事件返回的自定义文字 |
| `state` | string | idle、running、paused 或 finished |
| `elapsed` | number | 本轮累计时间，包含等待时间，单位为秒 |
| `remaining` | number | 距离下次触发还要多久，单位为秒；结束后为 0 |
| `executed_count` | integer | 已经触发的次数 |

### 示例

```lua
local id = timer.create(1)
local info = timer.get_info(id)
debug.assert(info.duration == 1 and info.state == "idle")
debug.assert(info.executed_count == 0)
```

**输出：**

```lua
```

## 额外说明

- 没有配置 callback、tip 或循环次数时，对应字段为 `nil`。
- idle 表示还没开始或已重置，running 表示正在计时，paused 表示已暂停，finished 表示全部触发完成。
- elapsed 包含本轮等待和计时时间；长帧积累的余量可能让它暂时超过本轮时长。remaining 为距离下一次触发的剩余时间，不会为负数。
- 返回的信息是快照，修改表不会改变计时器。ID 格式错误会抛出错误；格式正确但对象不存在时返回 `nil`。

---

## `start`

开始或继续计时。

### 调用

```lua
timer.start
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 计时器 ID |

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 操作是否成功 |

### 示例

```lua
local id = timer.create(1)
debug.assert(timer.start(id))
timer.pause(id)
debug.assert(timer.start(id))
```

**输出：**

```lua
```

## 额外说明

- idle 状态开始计时；paused 状态从原进度继续；finished 状态从头开始并清零次数。已在运行时返回 `false`。ID 格式错误会抛出错误，对象不存在时返回 `false`。

---

## `pause`

暂停计时，保留已经走过的时间。

### 调用

```lua
timer.pause
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 计时器 ID |

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 操作是否成功 |

### 示例

```lua
local id = timer.create(1)
timer.start(id)
debug.assert(timer.pause(id))
debug.assert(timer.get_info(id).state == "paused")
```

**输出：**

```lua
```

## 额外说明

- 只有 running 状态能成功暂停。再次暂停或暂停其他状态返回 `false`；暂停会使尚未投递的旧事件失效。用 start 从原进度继续。ID 格式错误会抛出错误，对象不存在时返回 `false`。

---

## `reset`

清零计时进度，回到未启动状态。

### 调用

```lua
timer.reset
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 计时器 ID |

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 操作是否成功 |

### 示例

```lua
local id = timer.create(1)
timer.start(id)
debug.assert(timer.reset(id))
debug.assert(timer.get_info(id).state == "idle")
```

**输出：**

```lua
```

## 额外说明

- 清零计时进度和触发次数，状态变为 idle，参数保持原值；尚未投递的旧事件失效。ID 格式错误会抛出错误，对象不存在时返回 `false`。

---

## `restart`

清零计时进度，然后重新开始。

### 调用

```lua
timer.restart
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 计时器 ID |

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 操作是否成功 |

### 示例

```lua
local id = timer.create(1)
debug.assert(timer.restart(id))
debug.assert(timer.get_info(id).state == "running")
```

**输出：**

```lua
```

## 额外说明

- 清零计时进度和触发次数，立即变为 running，参数保持原值；首次延时重新计算，尚未投递的旧事件失效。ID 格式错误会抛出错误，对象不存在时返回 `false`。

---
