# 终端自截图合成实验

目标：同一帧在当前 Windows Terminal 和离屏合成中尽量接近，逐项隔离误差。首轮结论见 [B7_REPORT](../../docs/B7_REPORT.md)。这是实验程序，不替代 dev 的 ScreenshotService。

## 一次完整实验

在 test workspace 中运行：

```powershell
./experiments/terminal-raster/run.ps1 `
  -OutputDirectory E:\Code\tg-test\b7-render-lab\my-run `
  -DevWorktree E:\Code\tui-game
```

需要 Windows Terminal、已安装的 Maple Mono NF CN 和当前命令提示符 profile。脚本使用该 profile 的现有设置，临时打开专用窗口，等待样本呈现，捕获窗口并关闭；不会截图桌面或改变用户的 settings.json。终端至少应有 80×20 格。不同机器用 `-TerminalProfile` 指定 profile 名称/GUID，并用 `-FontProfile` 传入自己的 JSON。

`profiles/current.json` 对应本机当前配置：Maple Mono NF CN、请求字重 600、12 pt / 144 DPI 即 24 px、实测 15×36 px、基线候选 27 px、Tango Dark、intenseTextStyle=bright。不是跨平台通用默认值。换字号/DPI/行距/主题后必须调整配置并重测。标尺只校准格宽和行高，不测字体像素大小或基线。

输出目录包含可直接打开的 `index.html`，可切换后端并用透明度滑块将合成图叠加到终端图上。原始像素展示，横向可滚动。还包含两个独立 exe、输入/实测配置、frame fixture、窗口截图、严格裁剪的参考图、各后端 PNG/元数据、逐行误差 JSON、差异热图和 SHA-256 摘要。

## 独立命令

```powershell
terminal-raster.exe emit TG-B7-LAB-manual 120 ready.json
terminal-raster.exe capture TG-B7-LAB-manual window.png
image-compare.exe calibrate window.png input-profile.json output
terminal-raster.exe render output/measured-profile.json output/measured
image-compare.exe compare output/reference.png output/measured/native-cluster-gray.png output/comparison
```

`emit` 在当前交互终端内工作；Q/Esc/Ctrl+C 结束并恢复终端。窗口 resize 会重新绘制，缩小到不能容纳样本时明确报错。`capture` 在另一个命令行调用，标题必须唯一。路径由命令行显式传入，因此 exe 可部署到测试目录运行。

## 六组对照的意义

| 文件前缀 | 控制内容 | 明确限制 |
|---|---|---|
| software-natural | fontdb 主字体 + HarfRust + fontdue，自然 advance | 无字体回退；不模拟粗斜体；不是 dev 完整实现 |
| software-fit | 同上，将整行 advance 分配到终端格宽 | 重现整行间距缩放思路，缺字会影响整行布局 |
| native-run-gray | DirectWrite 系统字体回退，自然行布局 | 没有终端格宽约束，说明仅换后端不够 |
| native-cell-gray | 每 grapheme 固定落格，灰度 AA | 跨 grapheme 连字/上下文塑形丢失 |
| native-cell-cleartype | 同上，仅换成 ClearType | 用来验证抗锯齿模式，不能默认更接近终端 |
| native-cluster-gray | 行级塑形，按 shaped cluster 调整字距 | 保留连字；TextLayout 双向布局仍不同于终端屏幕顺序 |

原生后端目前不开启彩色 emoji。几何字符仍按字体绘制，没有假装满足 terminal 自绘 box/block 规则。不要因 ASCII 很接近就宣称全 Unicode 对齐。

## 后续优先级

1. 将颜色、字体及几何参数显式冻结；本次实验足以否定固定 14.4 px 分数格和忽略字重模拟的假设。
2. 在 Windows 原生后端实现终端的“屏幕顺序 + 字体/脚本分段 + cluster 最后一个 glyph 补 advance”方式；避免将 TextLayout 段落双向重排用于已经排好格子的屏幕内容。
3. 单独实现/复用几何 box/block，固定取整规则；当前多种线条存在缝隙或厚度差异。
4. 为系统回退记录实际 font face 和 scale；按字形簇映射缺字。不要把主字体最相似候选当作已确认回退策略。
5. 彩色 emoji 若解除现有单色范围限制，再新增原生彩色字体层对照；先保留当前失败样本。
6. 最后接入真实 ComposedFrame 和 PNG/视频编码前帧，共享不可变配置；不先改编码器。
