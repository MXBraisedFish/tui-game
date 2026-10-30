着色器函数

默认：

```lua
function(char)
  return char
end
```

传递参数 `char` 格式：

```lua
{
  char = "" -- string 当前字符格的字符
  x = 0 -- integer 当前字符起始x坐标
  y = 0 -- integer 当前字符起始y坐标
  relative_x = 0 -- integer 当前字符相对起始x坐标
  relative_y = 0 -- integer 当前字符相对起始x坐标
  fg = "" -- string 当前字符的前景色
  bg = "" -- string 当前字符的背景色
  bold = false -- boolean 当前字符是否加粗
  italic = false -- boolean 当前字符是否斜体
  underline = false -- boolean 当前字符是否有下划线
  strike = false -- boolean 当前字符是否有删除线
  blink = false -- boolean 当前字符是否闪烁
  reverse = false -- boolean 当前字符是否反转
  hidden = false -- boolean 当前字符是否隐藏
  dim = false -- boolean 当前字符是否暗淡
}
```

返回参数 `char` 格式：

```lua
{
  char = "" -- string 当前字符格的字符
  x = 0 -- integer 当前字符起始x坐标
  y = 0 -- integer 当前字符起始y坐标
  relative_x = 0 -- integer 当前字符相对起始x坐标
  relative_y = 0 -- integer 当前字符相对起始x坐标
  fg = "" -- string 当前字符的前景色
  bg = "" -- string 当前字符的背景色
  bold = false -- boolean 当前字符是否加粗
  italic = false -- boolean 当前字符是否斜体
  underline = false -- boolean 当前字符是否有下划线
  strike = false -- boolean 当前字符是否有删除线
  blink = false -- boolean 当前字符是否闪烁
  reverse = false -- boolean 当前字符是否反转
  hidden = false -- boolean 当前字符是否隐藏
  dim = false -- boolean 当前字符是否暗淡
}
```

特殊处理：
返回的值中如有缺少，不会报错而是会自动补充为该单元格原始值，但如果返回的值和字段的类型或格式错误会报错
x和y是相对于当前切片图层的绝对坐标（CANVAS类型的着色器为"base"画布）
relative_x和relative_y是相对于当前着色器作用区域的相对坐标
最后对字符处理的实际绘制位置取决于字段x/y/relative_x/relative_y，若当前绝对坐标和相对坐标偏移量不同，优先以绝对坐标为准
0 宽标记字符会和当前字符一起伴随输入为 `char` 字段内
多宽字符会正常输出，但其后方的占位单元格不会输出，例：
123
我a
在批处理中
1. {char = "我", x = 1}
2. {char = "a", x = 3} -> 中文的"我"后的x=2的占位符不会参与着色器的批处理

着色器的优先级取决于入队先后顺序，早入队的会比后入队的着色器优先参与对字符的处理
effect.effect和effect.apply对待的等级一致，不会因为effect.apply是对象优先级就高于effect.effect
但canvas类型的着色器一定在slice类型的着色器后处理（要等图层切片合成完整）

渲染流程：

Render -> 开始绘制

draw -> 绘制到缓存区

effect -> 入队

draw -> 绘制到缓存区

Render -> 结束

（以下为程序内部大致流程，并非完整实现展示）

effect_B（SLICE） -> 着色器批处理（比A晚入队，但是因为是第一个图层切片层级，先被处理）

effect_C（SLICE） -> 着色器批处理（比B晚入队）

（图层合成）

effect_A（CANVAS） -> 着色器批处理（虽然比A和B都早入队，但是因为是合成画布作用域，因此在所有图层切片合并后开始作用）

effect_D（CANVAS） -> 着色器批处理（同上）

（对比、输出）

注意：

着色器处理后的字符属于重新绘制，且处理顺序从左上角开始，依次按照行逐个处理字符，例：
 123
1@ #
2& *
当前着色器效果为将所有的字符都绘制到第二列的位置
最终的绘制效果为
 123
1 #
2 *
因为处理完成后，@会被先绘制到1,2位置，然后#被绘制到1,2位置，因此#会盖住原本的@

字符的最终的绘制坐标不影响处理顺序，仅按照原始的获取顺序来，例
12345
@#$%^
当前着色器效果为将五个字符位置随机互换，假设本轮随机互换完成后的顺序为$^%#@
最后的实际绘制依旧为（以下为模拟程序帧，不代表实际内部运行机制）
12345
    @
12345
   #@
12345
\$  #@
12345
\$ %#@
12345
\$^%#@
绘制顺序依旧按照@#\$%^来绘制，并不会按照批处理后的坐标\$^%#@处理
这对字符的绘制顺序覆盖有所影响

完全空白区域会以""空字符传递（不是" "）

在着色器处理完整片区域前其未处理的字符不会被覆盖，例：
1&
2@
3
当前着色器效果为将所有字符向下移动一行
&被处理完成后并不会被立刻绘制为
1
2&
3
此时@并没有被处理，接下来的着色器依旧会接收到@字符的信息继续处理，当整片区域被处理完成后才会绘制到指定位置
1
2&
3@

宽字符处理：
1. 若该宽字符为占位符部分包含在着色器区域内，根据可选参数 `include_continuation` 处理
2. 若该宽字符为起始部分包含在着色器区域内，会参与到着色器处理内
例：
123
我a
若当前着色器可选参数 `include_continuation` 为 false，则选中1,2时，"我"并不会包含在批处理内，这个占位符单元格会被直接跳过，首个处理字符为3-a
若当前着色器可选参数 `include_continuation` 为 true，则选中1,2时，"我"包含在批处理内，首个处理字符为1-我
若当前着色器选中1,1时，"我"包含在批处理内，首个处理字符为1-我