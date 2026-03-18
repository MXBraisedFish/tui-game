# 富文本指令使用说明
## 声明
字符串以非转义的`f%`开头声明该字符串将使用富文本方式渲染。
## 语法
`{command:param1>param2|command:param}`
- `command` 指令
- `paramX` X级参数参数
- `:` 分割指令和参数
- `>` 分割多级参数
- `|` 分割多指令
## 阅读说明
指令
- 一级参数1
  - 二级参数1
    - 三级参数1
- 一级参数2

`无包裹` 填写特定的常量
`[XXX]` 代替为指定的变量
`!XXX!` 选填参数

## 指令列表
### tc 修改文字颜色
- [color] 可填写对应的颜色字符串、rgb(R,G,B)、#RGB、#RRGGBB
  - [!count!] 可填写整数，应用的字符数(该参数未填写时必须使用clear参数结束样式)
- clear 结束样式(有[!count!]参数时该参数可省略)

### bg 修改背景颜色
- [color] 可填写对应的颜色字符串、rgb(R,G,B)、#RGB、#RRGGBB
  - [!count!] 可填写整数，应用的字符数(该参数未填写时必须使用clear参数结束样式)
- clear 结束样式(有[!count!]参数时该参数可省略)

## 示例
1. `{tc:green}你好！这是绿色文字！{tc:clear}`
<span style="color: green;">你好！这是绿色文字！</span>

---

2. `{tc:green>3}你好！这是指定字符数量绿色文字！`
<span style="color: green;">你好！</span>这是指定字符数量绿色文字！

---

3. `{bg:black|tc:white>3}你好！{tc:yellow}这是混合样式！{bg:clear|tc:clear}`
<span style="color: white;background-color:black">你好！</span><span style="color: yellow;background-color:black">这是混合样式！</span>

## 抛出异常
|异常提醒|触发条件|
|:---|:---:|
|<span style="color:red;">样式未终止</span>|tc和bg指令未使用clear参数结束|
|<span style="color:red;">空指令</span>|出现空的`{}`|
|<span style="color:red;">指令未闭合</span>|出现未成对且未转义的`{`或`}`|
|<span style="color:red;">指令无效</span>|使用未知指令|
|<span style="color:red;">参数无效</span>|使用未知参数|

## 符号转义
可使用`\`符号对内容进行转义避免解析异常

## [color]参数可用指定字符串：
- black
- white
- red
- green
- yellow
- blue
- magenta
- cyan
- gray / grey
- dark_gray / dark_grey
- light_red
- light_green
- light_yellow
- light_blue
- light_magenta
- light_cyan