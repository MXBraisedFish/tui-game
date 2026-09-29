# image 库

`image` 将包内图片转换为终端方块字形富文本，并通过事件交付结果。

---

# 目录

## 方法

| 方法 | 说明 | 定位 |
| --- | --- | --- |
| `load` | 异步读取并转换图片 | [load](#load) |

---

# 方法

## `load`

异步读取 Session 包 `assets/` 目录中的 PNG、JPG 或 JPEG 图片并转换为终端字符画。路径必须位于该目录内；省略扩展名时依次查找 `png`、`jpg`、`jpeg`。

### 调用

```lua
image.load
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `path` | string | 相对 `assets/` 的路径；不接受绝对路径、父级路径或越出包目录的符号链接。扩展名可省略。 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `block_width` | integer | 输出字符格宽度；默认 `max(floor(图片宽度 / 100), 1)`，必须大于 0。 |
| `block_height` | integer | 输出字符格高度；默认 `max(floor(图片高度 / 200), 1)`，必须大于 0。 |
| `crop_x` | integer | 像素裁剪起点 x；默认 `0`。 |
| `crop_y` | integer | 像素裁剪起点 y；默认 `0`。 |
| `crop_width` | integer | 像素裁剪宽度；省略时取从 `crop_x` 到图片右侧的剩余区域。 |
| `crop_height` | integer | 像素裁剪高度；省略时取从 `crop_y` 到图片底部的剩余区域。 |
| `scale` | number | 裁剪区域的缩放比例；默认 `1.0`，必须是有限正数。 |
| `cache` | boolean | 是否读写图像缓存；默认 `true`。 |
| `mode` | string | 转换模式；默认 `"half_block"`，也可用 `"mix_block"` 匹配宿主支持的方块字形。 |
| `background` | string | RGBA 透明像素的混合背景；默认 `"#000000"`。只接受精确 RGB 字符串 `"#rrggbb"` 或 `"rgb(r,g,b)"`，也可传 `color.hex`、`color.rgb` 的返回值。 |

## 返回值

立即返回当前 Session 内的整数 request id。完成或失败事件会在 `data.request_id` 中回传相同 ID；成功事件的 `data.output` 是可传给 `draw.text` 的富文本字符串。

| 类型 | 说明 |
| --- | --- |
| integer | 已入队图片请求的 ID。 |

### 示例

```lua
local image_request_id = image.load("ui/title", {
  block_width = 40,
  block_height = 10,
  mode = "mix_block",
  background = color.hex(0, 0, 0),
})

function HandleEvent(event)
  if event.type == "image"
    and event.data.request_id == image_request_id
    and event.data.ok then
    draw.text(2, 2, event.data.output)
  end
end
```

**输出：**

```lua
```

### 额外说明

- 每个 Session 同时最多有 4 个待完成的图片请求；事件交付后释放名额。
- 输出单边最多 2,048 格、总计最多 16,384 格；源文件最多 32 MiB，源图最多 16,000,000 像素。
- `crop_x`、`crop_y`、`crop_width` 和 `crop_height` 使用像素单位。负偏移、空区域或越界尺寸会使请求失败；省略裁剪尺寸时取剩余区域。
- `background` 按每个 8-bit RGB 通道 `(src*a + bg*(255-a) + 127)/255` 混合后再缩放，并纳入缓存键。
- 事件结构见⌞[事件文档](../EVENT.md)⌝。
