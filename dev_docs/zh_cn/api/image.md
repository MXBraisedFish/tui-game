# image 库

`image` 将包内图片转换为终端方块字形富文本，并通过事件交付结果。

---

# 目录

## 方法

| 方法   | 说明                                       | 定位          |
| ------ | ------------------------------------------ | ------------- |
| `load` | 读取包内图片，并转换为可以绘制的终端字符画 | [load](#load) |

---

# 方法

## `load`

读取包内图片，并转换为可以绘制的终端字符画。

### 调用

```lua
image.load
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `path` | string | 相对 `assets/` 的路径；不接受绝对路径、父级路径或越出包目录的符号链接；扩展名可省略 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `block_width` | integer | `max(floor(原图宽度 / 100), 1)` | 输出字符格宽度；必须大于 0 |
| `block_height` | integer | `max(floor(原图高度 / 200), 1)` | 输出字符格高度；必须大于 0 |
| `crop_x` | integer | `0` | 像素裁剪起点 x |
| `crop_y` | integer | `0` | 像素裁剪起点 y |
| `crop_width` | integer | 从 `crop_x` 到图片右侧的剩余区域 | 像素裁剪宽度 |
| `crop_height` | integer | 从 `crop_y` 到图片底部的剩余区域 | 像素裁剪高度 |
| `scale` | number | `1.0` | 裁剪区域的缩放比例；必须是有限正数 |
| `cache` | boolean | `true` | 是否读写图像缓存 |
| `mode` | string | `"half_block"` | 转换模式；也可用 `"mix_block"` 使用更多方块字形 |
| `background` | string | `"#000000"` | RGBA 透明像素的混合背景；只接受精确的 `"#rrggbb"` 或 `"rgb(r,g,b)"` 字符串，也可传 `color.hex`、`color.rgb` 的返回值 |

## 返回值

**成功入队时**，返回当前会话内的整数 `request_id`。完成或失败事件会在 `data.request_id` 中回传相同 ID；成功事件的 `data.output` 是可传给 `draw.text` 的富文本字符串。

| 类型 | 说明 |
| --- | --- |
| integer | 已入队图片请求的 ID |

### 示例

```lua
local image_text = nil
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
    image_text = event.data.output
    draw.render()
  end
end

function Render()
  if image_text then
    draw.text(2, 2, image_text)
  end
end
```

**输出：**

```lua
```

## 额外说明

- 示例需要包内 `assets/ui/title.png`（或 `.jpg`、`.jpeg`）。路径不能越出 `assets/`；省略扩展名时按上述顺序查找。
- 每个会话同时最多有 4 个待完成的图片请求；达到上限时该调用会报错；事件交付后释放名额。
- 输出单边最多 2,048 格、总计最多 16,384 格；源文件最多 32 MiB，源图最多 16,000,000 像素、单边最多 16,384 像素。
- `crop_x`、`crop_y`、`crop_width` 和 `crop_height` 使用像素单位。负偏移、空区域或越界尺寸会使请求失败；省略裁剪尺寸时取剩余区域。
- 透明像素先与 `background` 指定的颜色混合，再转换成字符画。
- 事件结构见⌞[事件文档](../EVENT.md)⌝。
