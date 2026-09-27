# image 库

## 基本库说明

`image` 提供将图片转换为终端方块字形富文本的异步方法。

> 在使用图片加载时请时刻关注参数单位。

---

## 目录

---

## 方法

## `load`

异步读取 Session 包 `assets/` 目录下的 PNG、JPG 或 JPEG 图片并转换为终端字符画。路径必须在 `assets/` 内；省略扩展名时按 `png`、`jpg`、`jpeg` 顺序查找。

### 调用

```lua
-- 表参数
image.load{}
```

### 参数

| 参数名         | 类型    | 必填 | 默认值 | 说明 |
| -------------- | ------- | ---- | ------ | ---- |
| `path`         | string  | 是   | - | 相对 `assets/` 的路径；禁止绝对路径、父级路径和越出包目录的符号链接。可省略 `.png`、`.jpg` 或 `.jpeg` 扩展名。 |
| `block_width`  | integer | 否   | `max(floor(图像原始宽度 / 100), 1)` | 输出宽度，单位为字符格；必须大于 0。 |
| `block_height` | integer | 否   | `max(floor(图像原始高度 / 200), 1)` | 输出高度，单位为字符格；必须大于 0。 |
| `crop_x`       | integer | 否   | `0` | 裁剪起点 x，单位为像素；不得为负数。 |
| `crop_y`       | integer | 否   | `0` | 裁剪起点 y，单位为像素；不得为负数。 |
| `crop_width`   | integer | 否   | 从 `crop_x` 到图片右边缘的剩余宽度 | 裁剪宽度，单位为像素；显式尺寸必须大于 0 且不得越界。 |
| `crop_height`  | integer | 否   | 从 `crop_y` 到图片下边缘的剩余高度 | 裁剪高度，单位为像素；显式尺寸必须大于 0 且不得越界。 |
| `scale`        | number  | 否   | `1.0` | 裁剪图像缩放比例；必须是有限正数。 |
| `cache`        | boolean | 否   | `true` | 是否读写图像缓存；`false` 时跳过内存与磁盘缓存。 |
| `mode`         | string  | 否   | `"half_block"` | 转换模式：`"half_block"` 保持半块基线，`"mix_block"` 匹配宿主支持的方块字形。 |
| `background`   | string  | 否   | `"#000000"` | 透明像素混合背景；仅接受 `"#rrggbb"` 或 `"rgb(r,g,b)"`，也可传 `color.hex{...}` / `color.rgb{...}` 的返回值。 |

### 返回

调用立即返回当前 Session 内的 `request_id`。转换完成后，`image` 事件会携带相同 ID；成功时 `output` 是可直接传给 `draw.text` 的富文本字符串，失败时带有安全化的错误信息。事件结构见[事件文档](../EVENT.md)的 `image` 部分。

### 示例

```lua
local image_request_id

function Init(ctx)
  image_request_id = image.load{
    path = "ui/title", -- 自动查找 title.png / title.jpg / title.jpeg
    block_width = 40,
    block_height = 10,
    mode = "mix_block",
    background = color.hex{ r = 0, g = 0, b = 0 },
  }
end

function HandleEvent(event)
  if event.type == "image"
    and event.data.request_id == image_request_id
    and event.data.ok then
    draw.text{ x = 2, y = 2, text = event.data.output }
  end
end
```

### 额外补充

- 参数 `block_width` 单位为**字符格宽度**。
- 参数 `block_height` 单位为**字符格高度**。
- `crop_x`、`crop_y`、`crop_width` 和 `crop_height` 的单位为**像素**；缺省裁剪宽高各自延伸到图片对应边缘。
- 显式裁剪区域为负偏移、空区域或越出图片边界时，请求会失败。
- 若经过裁剪、缩放后的图片比例与输出格数比例不同，会被强制拉伸到指定格数。
- 输出单边最多 2,048 格、总计最多 16,384 格；源文件最多 32 MiB，源图最多 16,000,000 像素。
- 每个 Session 同时最多有 4 个待完成的图片请求；上一个请求事件投递后会释放名额。
- `background` 仅参与 RGBA 透明度合成，按每个 8-bit RGB 通道 `(src*a + bg*(255-a) + 127)/255` 计算后再缩放。
