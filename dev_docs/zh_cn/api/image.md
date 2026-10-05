# image 库

`image` 将包内图片转换为终端方块字形富文本，并通过事件交付结果。

---

# 目录

## 方法

| 方法   | 说明                                       | 定位          |
| ------ | ------------------------------------------ | ------------- |
| `load` | 读取包内图片，并转换为可以绘制的终端字符画 | [load](#load) |

---

# 常量

## `HALF_BLOCK`

半方块渲染方式。

### 调用

```lua
file.HALF_BLOCK`
```

### 可用于

- 参数 `mode`

### 示例

> test.png

```lua
local request_id = image.load("test.png", { mode = image.HALF_BLOCK, block_width = 44, block_height = 20 })
local get_image = false
local image

function HandleEvent(event)
  if event.type == "image" then
    get_image = true
    image = event.data.output
  end
end

function Render()
  if get_image then
    draw.text(0, 0, image)
  end
end
```

**输出：**

![image_HALF_BLOCK_example](../image/image_HALF_BLOCK_example.png)

### 等值

```text
"half_block"
```

---

## `MIX_BLOCK`

半方块渲染方式。

### 调用

```lua
file.MIX_BLOCK`
```

### 可用于

- 参数 `mode`

### 示例

> test.png

```lua
local request_id = image.load("test.png", { mode = image.MIX_BLOCK, block_width = 44, block_height = 20 })
local get_image = false
local image

function HandleEvent(event)
  if event.type == "image" then
    get_image = true
    image = event.data.output
  end
end

function Render()
  if get_image then
    draw.text(0, 0, image)
  end
end
```

**输出：**

![image_MIX_BLOCK_example](../image/image_MIX_BLOCK_example.png)

### 等值

```text
"mix_block"
```

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

| 参数名 | 类型   | 说明                  |
| ------ | ------ | --------------------- |
| `path` | string | 相对 `assets/` 的路径 |

### 选填参数

| 参数名         | 类型            | 默认值                           | 说明                    |
| -------------- | --------------- | -------------------------------- | ----------------------- |
| `block_width`  | integer         | `max(floor(原图宽度 / 100), 1)`  | 输出字符格宽度         |
| `block_height` | integer         | `max(floor(原图高度 / 200), 1)`  | 输出字符格高度          |
| `crop_x`       | integer         | `0`                              | 像素裁剪起点 x          |
| `crop_y`       | integer         | `0`                              | 像素裁剪起点 y          |
| `crop_width`   | integer         | 从 `crop_x` 到图片右侧的剩余区域 | 像素裁剪宽度            |
| `crop_height`  | integer         | 从 `crop_y` 到图片底部的剩余区域 | 像素裁剪高度            |
| `scale`        | integer / float | `1.0`                            | 裁剪区域的缩放比例      |
| `anti_alias`   | integer / float | `0.8`                           | 平滑像素边缘 |
| `cache`        | boolean         | `true`                           | 是否读写图像缓存        |
| `mode`         | const-image     | `image.HALF_BLOCK`               | 转换模式                |
| `background`   | string          | `"#000000"`                      | RGBA 透明像素的混合背景 |

## 返回值

**请求提交成功时**，返回一个值。

| 类型   | 说明    |
| ------ | ------- |
| string | 事件 ID |

**请求提交失败时**，返回一个值。

| 类型 | 说明         |
| ---- | ------------ |
| nil  | 请求提交失败 |

### 示例

> test.png

```lua
local request_id = image.load("test.png", { mode = image.MIX_BLOCK, block_width = 44, block_height = 20 })
local get_image = false
local image

function HandleEvent(event)
  if event.type == "image" then
    get_image = true
    image = event.data.output
  end
end

function Render()
  if get_image then
    draw.text(0, 0, image)
  end
end
```

**输出：**

![image_MIX_BLOCK_example](../image/image_MIX_BLOCK_example.png)

## 额外说明

- 事件返回值见⌊[事件协议](../EVENT.md)⌉
- 仅支持 `.png`/`.jpg`/`.jpeg` 格式。
- 选填参数 `anti_alias` 取值范围为 $[0, 32]$。
- 选填参数单位如下：

| 参数名         | 单位       |
| -------------- | ---------- |
| `block_width`  | 字符格宽度 |
| `block_height` | 字符格高度 |
| `crop_x`       | 像素       |
| `crop_y`       | 像素       |
| `crop_width`   | 像素       |
| `crop_height`  | 像素       |
| `anti_alias`   | 缩放后的图像像素 |
