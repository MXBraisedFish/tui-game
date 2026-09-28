# 固定终端帧图像校准与比较

该项目不依赖窗口 API，可独立处理 PNG；依赖继承根 workspace。

```text
image-compare calibrate <window.png> <profile.json> <out-dir>
image-compare compare <reference.png> <candidate.png> <out-dir>
```

校准器针对 terminal-raster 的 80×20 fixture：定位完整的 80 格洋红/青色水平标尺，再核对蓝/黄纵向标尺，得到整数格尺寸与 crop。缺标尺、裁剪不全或异常行边界时报错；不根据 glyph 墨迹宽度猜测。

比较器拒绝尺寸不同的图片，不缩放、不平移。按 20 行输出 RGB MAE、任一通道差值超过 16 的像素比例，以及有效内容像素的 RGB MAE。有效内容是两张图中相对本行背景非空白像素的并集，排除左侧两格标尺区；不是 OCR，也不是感知质量评分。背景颜色错误会进入误差统计。

色板行与文字行必须分别分析。差异热图用红色强度显示最大通道差（放大 4 倍并封顶），仅用于定位差异。没有一个全图阈值可证明字体、复杂文字或终端观感完全一致。
