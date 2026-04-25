# 富文本指令

# 文档信息

1. 更新日期：2026年4月23日
2. 本文档旨在说明模组包的字符串中如何使用富文本样式。

---

# 目录
- [使用范围](#使用范围)
- [富文本声明](#富文本声明)
- [语法](#语法)
- [指令列表](#指令列表)
- [公共参数](#公共参数)
- [抛出异常](#抛出异常)
- [符号转义](#符号转义)
- [附录](#附录)
  - [颜色 color](#颜色-color)

---

# 适用范围

**`game.json` 文件以下字段：**

`mad_name`
`introduction`
`author`
`game_name`
`description`
`detail`
`version`
`icon`
`banner`

**`package.json` 文件以下字段：**

`best_none`

**传递值 `best` 表以下字段：**

`best_string`

---

# 富文本声明

字符串以非转义的 `f%` 开头，声明该字符串将使用富文本方式渲染。

---

# 语法

```
{command:param1>param2|command:param1+param2}
```

**符号说明**

| 符号 | 含义           |
| ---- | -------------- |
| `:`  | 分割指令和参数 |
| `>`  | 分割多级参数   |
| `+`  | 合并同级参数   |
| `\|` | 分割多指令     |

**参数标记说明**

| 标记     | 含义         |
| ------ | ---------- |
| `无包裹`  | 填写特定参数     |
| `[XXX` | 选填参数       |
| `!XXX` | 替换为指定的类型参数 |
| `^XXX` | 同级不可合并参数   |

---

# 指令列表

## `tc` — 修改文字颜色

| 参数等级 | 参数        | 说明                                         |
| ---- | --------- | ------------------------------------------ |
| 一级参数 | `^!color` | 可填写对应的颜色字符串，详细见『附录-[颜色 color](#颜色-color)』。 |

## `ts` — 修改文字样式

| 参数等级 | 参数          | 说明  |
| ---- | ----------- | --- |
| 一级参数 | `bold`      | 加粗  |
| 一级参数 | `italic`    | 斜体  |
| 一级参数 | `underline` | 下划线 |
| 一级参数 | `strike`    | 删除线 |
| 一级参数 | `blink`     | 闪烁  |
| 一级参数 | `reverse`   | 反转  |
| 一级参数 | `hidden`    | 隐藏  |
| 一级参数 | `dim`       | 暗淡  |

## `bg` — 修改背景颜色

| 参数等级 | 参数        | 说明                                         |
| ---- | --------- | ------------------------------------------ |
| 一级参数 | `^!color` | 可填写对应的颜色字符串，详细见『附录-[颜色 color](#颜色-color)』。 |

## `key` — 展示动作按键信息

| 参数等级 | 参数           | 说明                      |
| ---- | ------------ | ----------------------- |
| 一级参数 | `key`        | 按键动作注册表动作               |
| 二级参数 | `^[user`     | 映射用户自定义物理按键（不填写时默认值）    |
| 二级参数 | `^[original` | 映射原始物理按键（不填写时默认 `user`） |

---

# 公共参数

| 参数等级 | 参数         | 说明                                       |
| ---- | ---------- | ---------------------------------------- |
| 二级参数 | `^[!count` | 可填写整数，应用的字符数（该参数未填写时必须使用 `clear` 参数结束样式） |
| 一级参数 | `^clear`   | 结束样式（有 `[!count` 参数时该参数可省略）              |

---

# 示例

## 示例 1：修改文字颜色

**输入**

```
{tc:green}你好！这是绿色文字！{tc:clear}
```

**效果**
<span style="color: green;">你好！这是绿色文字！</span>

## 示例 2：指定字符数量的颜色修改

**输入**

```
{tc:green>3}你好！这是指定字符数量绿色文字！
```

**效果**
<span style="color: green;">你好！</span>这是指定字符数量绿色文字！

## 示例 3：修改文字样式（加粗、斜体、下划线）

**输入**

```
{ts:bold}加粗文字{ts:clear} 和 {ts:italic}斜体文字{ts:clear} 以及 {ts:underline}下划线文字{ts:clear}
```

**效果**
<span style="font-weight: bold;">加粗文字</span> 和 <span style="font-style: italic;">斜体文字</span> 以及 <span style="text-decoration: underline;">下划线文字</span>

## 示例 4：混合使用（背景色 + 文字颜色 + 文字样式）

**输入**

```
{bg:black|tc:white>3}你好！{ts:bold|tc:yellow}这是混合样式!{bg:clear|tc:clear|ts:clear}
```

**效果**
<span style="color: white; background-color: black;">你好！</span><span style="color: yellow; font-weight: bold; background-color: black;">这是混合样式！</span>

## 示例 5：按键展示

**信息**

*原始物理按键：*
```json
"case_sensitive": false,
"actions": {
  "jump": {
    "key": "space",
    "key_name": "跳跃"
  },
  "move": {
    "key": ["left", "right", "up", "down"],
    "key_name": "移动"
  }
}
```

*用户自定义物理按键：*
```text
动作    [1] 按键    [2] 按键    [3] 按键    [4] 按键    [5] 按键
—————————————————————————————————————————————————————————————
跳跃    [ Enter ]
移动    [ W ]       [ A ]      [ S ]       [ D ]
```

**输入**

```
{tc:orange|key:jump} 跳跃\n{key:move>original} 移动{tc:clear}
```

**效果**
<span style="color: orange;">[Enter] 跳跃<br />[↑]/[↓]/[←]/[→] 移动</span>

---

# 抛出异常

| 异常提醒                                        | 触发条件                                |
| ------------------------------------------- | ----------------------------------- |
| <span style="color: red;">样式未终止</span>      | `tc` , `bg`和`ts` 指令未使用 `clear` 参数结束 |
| <span style="color: red;">空指令</span>        | 出现空的 `{}`                           |
| <span style="color: red;">指令未闭合</span>      | 出现未成对且未转义的 `{` 或 `}`                |
| <span style="color: red;">指令无效</span>       | 使用未知指令                              |
| <span style="color: red;">参数无效</span>       | 使用未知参数                              |
| <span style="color: red;">未匹配到对应的语义键</span> | `key` 指令未找到匹配的动作                    |

---

# 符号转义

可使用 `\` 符号对内容进行转义，避免解析异常。

示例：`\{` 表示普通左花括号，不会被解析为指令起始。

---

# 附录

## 颜色 `color`

### 预定义颜色名称

> 注：以下颜色值为逻辑名称，实际显示效果取决于终端的颜色映射。

| 颜色值          | 映射的终端颜色 |
| --------------- | -------------- |
| `black`         | Black          |
| `white`         | White          |
| `red`           | Red            |
| `light_red`     | Red            |
| `dark_red`      | DarkRed        |
| `yellow`        | Yellow         |
| `light_yellow`  | Yellow         |
| `dark_yellow`   | DarkYellow     |
| `orange`        | DarkYellow     |
| `green`         | Green          |
| `light_green`   | Green          |
| `blue`          | Blue           |
| `light_blue`    | Blue           |
| `cyan`          | Cyan           |
| `light_cyan`    | Cyan           |
| `magenta`       | Magenta        |
| `light_magenta` | Magenta        |
| `grey`          | Grey           |
| `gray`          | Grey           |
| `dark_grey`     | DarkGrey       |
| `dark_gray`     | DarkGrey       |

### 自定义颜色格式

> 注：值以字符串的形式传递。

| 格式           | 示例                | 注意事项                                          |
| ------------ | ----------------- | --------------------------------------------- |
| `rgb(r,g,b)` | `rgb(255,128,64)` | 标准 RGB 颜色，括号内为 0–255 的整数值。**请勿在字母与括号之间添加空格**。 |
| `#rrggbb`    | `#ff8040`         | 十六进制颜色表示（6 位）。**不支持 `#rgb` 缩写格式**。            |
