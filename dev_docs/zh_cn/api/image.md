# image 库

## 基本库说明

`image` 提供将图片渲染在终端的方法。

> 在使用图片加载时请时刻关注参数单位。

---

## 目录

---

## 方法

## `load`

异步读取 `assets/` 目录下的图片并转换为半方块字符像素画。

### 调用

```lua
-- 表参数
image.load{}
```

### 参数

| 参数名         | 类型    | 必填 | 默认值                      | 说明                      |
| -------------- | ------- | ---- | --------------------------- | ------------------------- |
| `path`         | string  | 是   | -                           | 相对 `assets/` 的文件路径 |
| `block_width`  | integer | 否   | `floor(图像原始宽度 / 100)` | 输出宽度（字符格宽度）    |
| `block_height` | integer | 否   | `floor(图像原始高度 / 200)` | 输出高度（字符格高度）    |
| `crop_x`       | integer | 否   | 0                           | x 轴裁剪起始位置（像素）  |
| `crop_y`       | integer | 否   | 0                           | y 轴裁剪起始位置（像素）  |
| `crop_width`   | integer | 否   | 图像原始宽度                | x 轴裁剪宽度（像素）      |
| `crop_height`  | integer | 否   | 图像原始高度                | y 轴裁剪高度（像素）      |
| `scale`        | number  | 否   | `1.0`                       | 裁剪图像缩放比例          |
| `cache`        | boolean | 否   | `true`                      | 图像缓存                  |

### 返回

事件返回，请查看⌊[事件结构](../EVENT.md)⌉文档⌊image⌉部分。

### 示例

```lua

```

输出：

```

```

### 额外补充

- 参数 `block_width` 单位为**字符格宽度**。
- 参数 `block_height` 单位为**字符格高度**。
- 参数 `crop_x`、参数 `crop_y`、参数 `crop_width`和参数 `crop_height` 单位为**像素**。
- 若经过裁剪、缩放后的图片比例与参数 `block_width` 和参数 `block_height` 比例不同，会被强制拉伸至相同比例。
