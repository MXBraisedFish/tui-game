# 富文本指令使用说明

## 适用范围



---

## 声明

字符串以非转义的 `f%` 开头，声明该字符串将使用富文本方式渲染。

---

## 语法

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

| 标记     | 含义                 |
| -------- | -------------------- |
| `无包裹` | 填写特定参数         |
| `[XXX`   | 替换为指定的类型参数 |
| `!XXX`   | 选填参数             |
| `^XXX`   | 同级不可合并参数     |

---

## 指令列表

### `tc` — 修改文字颜色

| 参数      | 说明                                                    |
| --------- | ------------------------------------------------------- |
| `^[color` | 可填写对应的颜色字符串、`rgb(R,G,B)`、`#RGB`、`#RRGGBB` |

### `ts` — 修改文字样式

| 参数        | 说明   |
| ----------- | ------ |
| `bold`      | 加粗   |
| `italic`    | 斜体   |
| `underline` | 下划线 |
| `strike`    | 删除线 |
| `blink`     | 闪烁   |
| `reverse`   | 反转   |
| `hidden`    | 隐藏   |
| `dim`       | 暗淡   |

### `bg` — 修改背景颜色

| 参数       | 说明                                                                    |
| ---------- | ----------------------------------------------------------------------- |
| `^[color`  | 可填写对应的颜色字符串、`rgb(R,G,B)`、`#RGB`、`#RRGGBB`                 |
| `^[!count` | 可填写整数，应用的字符数（该参数未填写时必须使用 `clear` 参数结束样式） |
| `^clear`   | 结束样式（有 `[!count` 参数时该参数可省略）                             |

---

## 公共参数

| 参数等级 | 参数       | 说明                                                                    |
| -------- | ---------- | ----------------------------------------------------------------------- |
| 二级参数 | `^[!count` | 可填写整数，应用的字符数（该参数未填写时必须使用 `clear` 参数结束样式） |
| 一级参数 | `^clear`   | 结束样式（有 `[!count` 参数时该参数可省略）                             |

---

## 示例

### 示例 1：修改文字颜色

**输入**

```
{tc:green}你好！这是绿色文字！{tc:clear}
```

**效果**
<span style="color: green;">你好！这是绿色文字！</span>

---

### 示例 2：指定字符数量的颜色修改

**输入**

```
{tc:green>3}你好！这是指定字符数量绿色文字！
```

**效果**
<span style="color: green;">你好！</span>这是指定字符数量绿色文字！

---

### 示例 3：修改文字样式（加粗、斜体、下划线）

**输入**

```
{ts:bold}加粗文字{ts:clear} 和 {ts:italic}斜体文字{ts:clear} 以及 {ts:underline}下划线文字{ts:clear}
```

**效果**
<span style="font-weight: bold;">加粗文字</span> 和 <span style="font-style: italic;">斜体文字</span> 以及 <span style="text-decoration: underline;">下划线文字</span>

---

### 示例 4：混合使用（背景色 + 文字颜色 + 文字样式）

**输入**

```
{bg:black|tc:white>3}你好！{ts:bold|tc:yellow}这是混合样式！{bg:clear|tc:clear|ts:clear}
```

**效果**
<span style="color: white; background-color: black;">你好！</span><span style="color: yellow; font-weight: bold; background-color: black;">这是混合样式！</span>

---

## 抛出异常

| 异常提醒                                    | 触发条件                                 |
| ------------------------------------------- | ---------------------------------------- |
| <span style="color: red;">样式未终止</span> | `tc` 和 `bg` 指令未使用 `clear` 参数结束 |
| <span style="color: red;">空指令</span>     | 出现空的 `{}`                            |
| <span style="color: red;">指令未闭合</span> | 出现未成对且未转义的 `{` 或 `}`          |
| <span style="color: red;">指令无效</span>   | 使用未知指令                             |
| <span style="color: red;">参数无效</span>   | 使用未知参数                             |

---

## 符号转义

可使用 `\` 符号对内容进行转义，避免解析异常。

示例：`\{` 表示普通左花括号，不会被解析为指令起始。

---

## 可用颜色列表

`[color` 参数可使用以下字符串：

| 颜色值                    | 说明     |
| ------------------------- | -------- |
| `black`                   | 黑色     |
| `white`                   | 白色     |
| `red`                     | 红色     |
| `green`                   | 绿色     |
| `yellow`                  | 黄色     |
| `blue`                    | 蓝色     |
| `magenta`                 | 品红色   |
| `cyan`                    | 青色     |
| `gray` / `grey`           | 灰色     |
| `dark_gray` / `dark_grey` | 深灰色   |
| `light_red`               | 亮红色   |
| `light_green`             | 亮绿色   |
| `light_yellow`            | 亮黄色   |
| `light_blue`              | 亮蓝色   |
| `light_magenta`           | 亮品红色 |
| `light_cyan`              | 亮青色   |
