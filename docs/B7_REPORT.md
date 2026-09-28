# B7 终端自截图合成实验报告

日期：2026-09-28。实验在独立 `test` worktree 内完成，未更改或合并 dev 的截图实现。

## 结论

在本机当前 Windows Terminal 配置下，**可以让普通文字、组合字符、编程连字和颜色非常接近终端实机输出**。关键不是仅调整字号，而是同时匹配字体选择与模拟字重、整数格定位、塑形簇间距、默认色和抗锯齿模式。

这不等于全部字符已对齐。复杂脚本、双向文字、彩色 emoji、box/block 几何字符仍有明确差异；本实验不宣称完成 B7，也不承诺跨终端逐像素一致。软件探针刻意只加载主字体，**不是 dev 完整渲染器**；下述数值不能直接当作 dev 修改前后的性能或质量提升比例。

## 当前基准与可复现输入

- 用户确认采用当前终端配置，不修改 settings.json。命令提示符 profile：Maple Mono NF CN、semi-bold（600）、Tango Dark、`intenseTextStyle=bright`；继承 defaults 中的 `cellHeight=1.5`。
- 当前窗口 DPI 为 144，字号按默认 12 pt 换算为 24 px。当前配置与早先 B7 的“无显式行距覆盖”记录不同，旧图不能作为本轮同条件参考。
- 固定 80×20 格 fixture；ASCII、CJK、组合字符、连字、Arabic/Hebrew/Indic、emoji、box/block、样式、ANSI 和 truecolor 分行放置。
- 原始窗口 1872×1176 px；依据色块标尺自动定位参考区域 `(23,73)`，裁剪后 **1200×720 px**。未对图像缩放或平移配准。
- 80 个连续交替背景色格，以及左侧纵向标尺，实测每格 **15×36 px**。27 px 基线是输入候选值，**不是标尺测出来的基线**。
- 系统 Regular 字体与原部署目录 `assets/fonts/mmo.ttf` 的 SHA-256 相同：`8B4F149BEAEAD3EAC78FFB84ADF31513EBF06FE6DBC6E181CCD9DEBAD3C70F1A`。没有复制或分发字体文件。
- 详细输入、字体匹配与文件摘要见 [terminal-reference.json](terminal-reference.json)、[b7-results.json](b7-results.json)。dev 的工作区基线另存于 [dev-baseline.json](dev-baseline.json)，包含未提交状态的摘要，不能只按其中 HEAD 重建。

## 已定位的差异

### 1. 真实字格与字体自然 advance 不同

Regular 字体在 24 px 的 `M` advance 是约 14.4 px，终端真实格宽却是 15 px。若按 14.4 px 累积列位置，80 列会少 48 px。由一行文字的最左／最右墨迹推算宽度也不可靠，因为墨迹边距不等于字格边界。

因此应记录终端格几何，按格子定位，再在格子／塑形簇内部放置字形；不能将字体自然 advance 直接当成终端格宽。

### 2. 同名同文件字体仍可能有不同字重

同样请求 Maple Mono NF CN、600 字重：

- fontdb 查询返回 Regular 400，不提供模拟粗体。
- DirectWrite 匹配返回 700，`GetSimulations()` 为 1，即 `DWRITE_FONT_SIMULATIONS_BOLD`。

这解释了“字体文件相同但终端更厚”的重要来源。导出只检查字体路径或 family 不足以证明字体选择一致。原生实验直接使用 DirectWrite 的匹配与模拟结果。

### 3. 按整行缩放字距、逐 grapheme 绘制都各有损失

- 自然行布局没有终端格宽约束，越往右偏差越大；仅更换 DirectWrite 后端并不能解决定位。
- 逐 grapheme 落格能对齐普通文字，但拆散跨 grapheme 的编程连字。
- 对整行先塑形，再按 shaped cluster 的宽度补齐字距，可以保留本例 `->`、`=>`、`!=`、`===` 等连字；组合／连字行误差从 38.43 降到 0.51。
- 本实验的 `SetCharacterSpacing` 仍不是 Windows Terminal 的 Atlas cluster 绘制实现，不能据此推断复杂脚本已经等价。

### 4. 当前目标更接近灰度 AA，不是 ClearType

相同字体、格子和颜色下，仅将原生逐格后端从灰度改成 ClearType，ASCII 有效像素 MAE 从 0.43 上升到 18.12。当前截图应以灰度候选为基准；此观察仅适用于本轮终端和捕获条件。

### 5. 主题色与 intenseTextStyle 必须参与解析

当前默认前景为 `#D3D7CF`，背景为黑色；bold 样式按 profile 的 bright 行为处理。按这些配置解析后，ANSI 色块和 truecolor 色块逐像素一致，bold 文字行误差从 26.69 降到 0.55。不能把所有终端的默认前景硬编码为白色，也不能默认 bold 总是再加一层描粗。

## 数值对照

下表为**有效内容像素并集**的 RGB 平均绝对误差，范围 0–255，越低越接近；没有将大量空白像素计入来稀释误差。行号从 0 开始。相同原始终端截图用于全部候选，完整逐行结果与超阈值像素比例已归档。

| 样本 | software-fit（单主字体） | native-cell-gray | native-cluster-gray |
|---|---:|---:|---:|
| ASCII，行 1 | 41.79 | 0.43 | 0.43 |
| CJK，行 3 | 136.04 | 8.40 | 8.40 |
| 组合字符／连字，行 4 | 43.94 | 38.43 | 0.51 |
| RTL 混排，行 5 | 132.16 | 59.71 | 104.24 |
| Indic，行 6 | 129.14 | 91.87 | 91.87 |
| emoji，行 7 | 126.31 | 88.25 | 88.59 |
| block，行 11 | 50.61 | 36.89 | 36.89 |
| bold，行 13 | 51.46 | 0.55 | 0.55 |
| italic，行 14 | 119.12 | 1.33 | 1.33 |
| ANSI 色块，行 16 | 0 | 0 | 0 |
| truecolor 色块，行 17 | 0 | 0 | 0 |

CJK 的残差仍明显高于 ASCII，不能说 CJK 全部精确一致。软件对照的缺字、无模拟粗斜体、无系统回退也是误差来源；上表比较的是整条实验路径，不是只改变一个栅格库的受控实验。

小型证据随分支保存：[终端参考](evidence/reference.png)、[原生簇间距候选](evidence/native-cluster-gray.png)、[软件对照](evidence/software-fit.png)。本机完整交互对照页为 `E:\Code\tg-test\b7-render-lab\verified-run\index.html`。

## 尚未解决及下一步

1. **屏幕顺序与双向文字**：终端已经按格子维护屏幕顺序，不能假设对导出的整行再做段落 bidi 重排就是正确的。本例 native-cluster 的 RTL 行反而比 native-cell 更差。下一步应按照 terminal 的脚本／字体分段和 cluster advance 规则实现，分别核对 Arabic、Hebrew 与混排；不能用一条总分掩盖退步。
2. **回退的 face 与 scale**：DirectWrite 默认回退显著改善缺字，但现在只记录了主字体匹配，尚未逐 glyph 记录回退字体与缩放。下一轮先增加这些诊断，再评估 dev 的 cluster 缺字策略。
3. **几何字符**：终端对部分 box/block 自绘或调整尺寸。当前实验按字体绘制，存在边界、线粗和填充差异。应复用／对照 dev 的几何绘制，加入奇数格宽、分割比例、线条连续性的单独样本。
4. **emoji**：本实验维持当前单色范围；终端实图有彩色 emoji，因此这部分不可能在当前范围内视觉等价。若以后启用彩色层，应作为独立候选，不能拿单色像素误差要求满分彩色匹配。
5. **真实引擎接入**：先在实验内接收实际 ComposedFrame，保留 profile/字体/能力元数据，再比较 PNG 与视频编码前帧；最后才处理有损编码误差。本轮没有修改视频编码器。

推荐继续以 `native-cluster-gray` 作为 Windows 候选研究，但保持逐格候选作为复杂脚本对照。不要直接用当前实验替换生产截图服务，不要把本机参数设为所有用户的固定默认值。

## 分支环境与验证

- 活动项目统一为根 workspace：`terminal-raster`、`image-compare`、`xtask`。原有 8 个 Python logo 文件原样移到 `archive/logo_animation/`。
- Rust 1.93.0、edition 2024、resolver 3；唯一 Cargo.lock。根依赖精确锁定为 dev 实际版本，子项目通过 workspace 继承。
- xtask 已验证 **101 个 registry 包**的名称／版本／来源均存在于归档 dev 基线，也通过当前 dev Cargo.lock 的只读核对。未添加 dev 锁文件之外的库版本。
- 7 项自动测试通过，涵盖完整／缺失／截断标尺、fixture 占格、ANSI 边界、bright+reverse、无效几何。
- `cargo fmt --all -- --check` 与 workspace 全目标 `clippy -D warnings` 通过。
- 一次手工链路及一次 `run.ps1` 完整复跑均得到 15×36 px 字格和同样的逐行数值。实测发现并修复了新 Terminal 窗口初始化 resize 导致样本提前退出的问题；最终脚本只关闭唯一标题的实验窗口。
- Windows 已实机验证；Linux/macOS 只保留可移植代码结构，本轮未在其上运行。GitHub Pages、dev 与 main 均未改动。

## 参考实现

- [Windows Terminal AtlasEngine 字体几何](https://github.com/microsoft/terminal/blob/main/src/renderer/atlas/AtlasEngine.api.cpp)：字体单位、DPI、格尺寸与基线。
- [Windows Terminal AtlasEngine 塑形与 cluster advance](https://github.com/microsoft/terminal/blob/main/src/renderer/atlas/AtlasEngine.cpp)：以屏幕列宽修正 cluster 末 glyph 的 advance。
- [Windows Terminal 默认主题配置](https://github.com/microsoft/terminal/blob/main/src/cascadia/TerminalSettingsModel/defaults.json)：Tango Dark 调色板。
- [DirectWrite bitmap rendering](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nn-dwrite-idwritebitmaprendertarget)：原生离屏文字渲染能力。

上述上游 main 源码用于理解和设计实验，不声称与本机安装版本逐行相同；有关本机一致性的结论以本轮实测文件为准。
