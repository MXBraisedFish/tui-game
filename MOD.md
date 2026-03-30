# TUI-GAME 模组编写条目

本文是 TUI-GAME 模组系统的正式说明文档。内容按条目结构组织，用于说明模组文件结构、字段标准、可用类型、图片参数、资源路径、语言文件、动作映射与示例写法。

## 一、模组目录结构

### 1. 根目录位置

模组固定放置于：

```text
tui-game-data/
  mod/
    list/
      <namespace>/
```

其中：

- `tui-game-data/`：游戏运行数据目录。
- `mod/`：模组系统总目录。
- `list/`：模组包实际扫描目录。
- `<namespace>/`：单个模组包目录。

### 2. 标准目录结构

```text
tui-game-data/
  mod/
    list/
      <namespace>/
        meta.json
        scripts/
          main.lua
          extra.lua
          function/
            util.lua
            render.lua
        assets/
          lang/
            en_us.json
            zh_cn.json
          image/
            thumbnail.png
            banner.webp
          data/
            custom.json
```

### 3. 各目录与文件作用

- `meta.json`
  - 模组包元信息文件。
  - 缺少或格式错误时，整包拒绝加载。

- `scripts/`
  - 游戏主脚本目录。
  - 仅扫描该目录根层的 `.lua` 文件作为游戏入口脚本。

- `scripts/function/`
  - 辅助脚本目录。
  - 不会被扫描为独立游戏。
  - 仅供主脚本 `require` 或宿主自动加载使用。

- `assets/`
  - 模组资源目录。
  - 图片、语言文件、附加数据均放于此目录。

- `assets/lang/`
  - 模组语言文件目录。
  - `en_us.json` 为必需文件。
  - `zh_cn.json` 为可选文件。
  - 其它语言根据所需编写。

## 二、命名空间规范

### 1. 命名空间用途

命名空间用于：

- 映射模组目录位置；
- 作为资源引用前缀；
- 参与生成稳定的游戏唯一 ID；
- 作为日志、存档、缓存的隔离单位。

### 2. 命名规则

命名空间只允许：

- 大写字母 `A-Z`
- 小写字母 `a-z`
- 数字 `0-9`

不允许：

- 下划线 `_`
- 连字符 `-`
- 空格
- 点号 `.`
- 其他符号

合法示例：

```text
examplepack
Demo01
mod123
```

非法示例：

```text
example_pack
demo-pack
demo.pack
demo pack
```

### 3. 目录名一致性

目录名必须与 `meta.json` 中的 `namespace` 完全一致。若不一致，整包拒绝加载。

## 三、meta.json 条目

### 1. 文件作用

`meta.json` 用于定义模组包级别的元数据，包括包名、作者、版本、命名空间、API 兼容范围以及子图、宣传图等内容。

### 2. 必填字段

| 字段名 | 类型 | 说明 |
|---|---|---|
| `package_name` | `string` | 模组包名称，可直接写文本或语言键 |
| `author` | `string` | 模组作者 |
| `version` | `string` | 模组版本号 |
| `namespace` | `string` | 命名空间，必须与目录名一致 |
| `api_version` | `integer` 或 `[integer, integer]` | API 版本要求 |

### 3. 选填字段

| 字段名 | 类型 | 说明 |
|---|---|---|
| `description` | `string` | 包简介，可直接写文本或语言键 |
| `thumbnail` | `string` 或 `array` | 游戏列表子图 |
| `banner` | `string` 或 `array` | 模组详情宣传图 |

### 4. 类型说明

#### `package_name`

- 可写普通字符串；
- 可写语言键；
- 不允许为空字符串或纯空白。

示例：

```json
"package_name": "Example Pack"
```

```json
"package_name": "example_mod.package_name"
```

#### `description`

- 可缺省；
- 缺省时宿主会显示默认简介；
- 可写普通字符串；
- 可写语言键。

#### `api_version`

支持两种写法：

1. 单一整数：

```json
"api_version": 1
```

表示仅支持 API 版本 `1`。

2. 双元素数组：

```json
"api_version": [1, 3]
```

表示支持 API `1` 到 `3`，包含端点。

### 5. `thumbnail` 与 `banner`

这两个字段都支持两种来源：

1. 路径字符串
2. 二维数组字符图

详见后文“图片与字符图参数条目”。

### 6. 完整示例

```json
{
  "package_name": "example_mod.package_name",
  "description": "example_mod.package_description",
  "author": "TUI-GAME",
  "version": "1.0.0",
  "namespace": "examplepack",
  "api_version": [1, 1],
  "thumbnail": "color:math:examplepack:image/thumbnail.png",
  "banner": [
    "f%{tc:cyan}Example Pack{tc:clear}",
    "A minimal mod package."
  ]
}
```

## 四、游戏主脚本条目

### 1. 扫描规则

宿主只扫描：

```text
scripts/*.lua
```

不会扫描：

```text
scripts/function/*.lua
```

### 2. 每个主脚本必须提供的内容

每个主脚本必须定义：

- `init_game()`
- `game_loop()`
- `best_score()`
- `GAME_META`

### 3. `GAME_META` 结构

```lua
GAME_META = {
  name = "example_mod.game_name",
  description = "example_mod.game_description",
  detail = "example_mod.game_detail",
  save = true
}
```

### 4. 字段含义

| 字段名 | 类型 | 说明 |
|---|---|---|
| `name` | `string` | 游戏名，可写文本或语言键，不能为空 |
| `description` | `string` | 游戏简介，可写文本或语言键 |
| `detail` | `string` | 游戏详情，可写文本或语言键 |
| `save` | `boolean` | 是否允许存档 |

### 5. 兼容规则

- `name` 若为空、缺失或仅包含空白，则该游戏拒绝加载。
- `description` 和 `detail` 可为空。
- `save` 必须为布尔值。
- 允许额外字段，宿主会忽略。

### 6. 函数职责

#### `init_game()`

用于初始化游戏状态、注册动作、准备资源。

#### `game_loop()`

用于执行游戏主循环。

#### `best_score()`

用于返回当前游戏最佳记录。

## 五、游戏唯一 ID 条目

### 1. 生成方式

游戏唯一 ID 由宿主自动生成，模组不能手动指定。

生成规则：

```text
game_id = namespace + ":" + short_hash(package_name|namespace|author) + ":" + script_name
```

### 2. 作用

此 ID 用于绑定：

- 存档
- 最佳记录
- 动作映射

### 3. 设计结果

在以下信息不变时，ID 可保持稳定：

- `package_name`
- `namespace`
- `author`
- 主脚本文件名

## 六、图片与字符图参数条目

### 1. 基础写法

图片来源可以写成字符串：

```text
namespace:path/to/file
```

或：

```text
参数:参数:namespace:path/to/file
```

### 2. 可用参数

#### 渲染模式参数

这些参数互相冲突，若同时出现，以最后一个生效：

- `math`
- `number`
- `block`

若都不写，则默认使用盲文字符模式。

#### 颜色参数

这些参数互相冲突，若同时出现，以最后一个生效：

- `color`
- `white`

若都不写，则默认使用灰度图渲染。

### 3. 参数语义

#### 默认模式

不写任何参数时：

```text
namespace:image/banner.png
```

表示：

- 使用盲文字符；
- 使用灰度颜色；
- 不是纯白；
- 不是原彩图。

#### `color`

```text
color:namespace:image/banner.png
```

表示：

- 按原图颜色染色；
- 字符本身仍由当前渲染模式决定。

#### `white`

```text
white:namespace:image/banner.png
```

表示：

- 强制使用纯白字符；
- 不使用灰度；
- 不使用原图颜色。

若同时存在 `white` 与 `color`，以后出现的为准。

#### `math`

```text
math:namespace:image/banner.png
```

表示使用以下字符制作像素画：

```text
@%#*+=-:.
```

由暗到亮映射。

#### `number`

```text
number:namespace:image/banner.png
```

表示使用数字字符：

```text
0123456789
```

由暗到亮映射。

#### `block`

```text
block:namespace:image/banner.png
```

表示使用块字符：

```text
█▓▒
```

由暗到亮映射。

### 4. 冲突优先级示例

示例：

```text
white:color:number:math:examplepack:image/banner.png
```

解析结果：

- `white` 与 `color` 冲突，后者 `color` 生效；
- `number` 与 `math` 冲突，后者 `math` 生效；
- 最终只等价于：

```text
color:math:examplepack:image/banner.png
```

### 5. 路径支持的图片格式

当前支持：

- `png`
- `jpg`
- `jpeg`
- `webp`

不支持：

- `svg`
- `gif`

### 6. 子图与宣传图尺寸

| 图像类型 | 目标字符尺寸 |
|---|---|
| 子图 `thumbnail` | `4 x 8` |
| 宣传图 `banner` | `13 x 86` |

### 7. 图片裁切规则

- 先按目标比例居中裁切；
- 再缩放到目标尺寸；
- 最后转换成字符图。

若终端比宣传图显示区域更窄：

- 优先保留中间区域；
- 左右裁切。

## 七、数组字符图条目

### 1. 可写类型

数组字符图允许：

- 字符串
- 数组
- 字符串与数组混写

例如：

```json
[
  ["xx", "xxxx"],
  "yyyyyy"
]
```

### 2. 处理方式

宿主会逐行递归拍平数组内容，将同一行的片段拼接为一个完整字符串。

### 3. 长度补齐规则

数组图中的每一行都会检查长度。

若某一行长度小于目标宽度：

- 优先向左补空格；
- 然后左右交替补空格；
- 直到达到规定宽度。

### 4. 富文本支持

数组中的字符串允许直接书写 `{}` 指令。

例如：

```json
[
  "f%{tc:yellow}Example{tc:clear}",
  ["{tc:cyan}A", "{tc:clear}BC"]
]
```

### 5. 示例

#### `thumbnail` 数组示例

```json
"thumbnail": [
  "████████",
  "██ ██ ██",
  "   ██   ",
  "  ████  "
]
```

#### `banner` 嵌套数组示例

```json
"banner": [
  ["f%{tc:yellow}Example ", "{tc:cyan}Pack{tc:clear}"],
  "A minimal mod package."
]
```

## 八、默认图条目

### 1. 默认子图

当 `thumbnail` 缺失、格式错误、文件损坏或加载失败时，宿主使用：

```text
████████
██ ██ ██
   ██   
  ████  
```

### 2. 默认宣传图

当 `banner` 缺失、格式错误、文件损坏或加载失败时，宿主使用内置 ASCII 图，并在可用区域中居中显示。

## 九、语言文件条目

### 1. 目录

语言文件固定放在：

```text
assets/lang/
```

### 2. 文件要求

必须：

- `en_us.json`

可选：

- `zh_cn.json`

### 3. 键名规范

建议统一使用点分隔风格，例如：

```json
{
  "example_mod.package_name": "Example Pack",
  "example_mod.package_description": "A simple demo mod.",
  "example_mod.game_name": "Example Demo",
  "example_mod.game_description": "Press Enter to exit.",
  "example_mod.game_detail": "This is a minimal playable mod example."
}
```

### 4. 解析顺序

语言解析顺序为：

1. 当前语言文件
2. `en_us.json`
3. `[missing-i18n-key:namespace:key]`

## 十、资源路径条目

### 1. 引用格式

资源可以通过以下方式引用：

- `namespace:key`
- `namespace:path/to/file`
- `color:namespace:path/to/file`
- `math:namespace:path/to/file`
- `number:namespace:path/to/file`
- `block:namespace:path/to/file`

### 2. 安全限制

资源路径必须满足：

- 路径相对 `assets/`
- 不允许 `/` 或 `\` 开头
- 不允许 `..`
- 不允许单独的 `.`
- 不允许目录跳转

若路径越界或尝试访问其他目录，宿主会拒绝访问。

### 3. 命名空间隔离

- 模组不能读取其他模组的资源；
- 模组不能读取宿主内置 `assets/`；
- 首版不支持跨模组依赖。

## 十一、动作映射条目

### 1. 设计原则

模组应优先使用“动作”，而不是直接绑定物理按键。这样用户后续修改快捷键时，模组无需改代码。

### 2. 注册接口

```lua
register_action(name, default_keys, description)
```

参数说明：

| 参数 | 类型 | 说明 |
|---|---|---|
| `name` | `string` | 动作名 |
| `default_keys` | `string` 或 `string[]` | 默认按键 |
| `description` | `string` | 动作描述 |

### 3. 使用限制

- 仅允许在 `init_game()` 阶段注册；
- `game_loop()` 运行中动态注册不支持；
- 同名动作若重复注册且内容不同，会报错。

### 4. 读取接口

#### 阻塞读取

```lua
local action = get_action_blocking()
```

#### 非阻塞轮询

```lua
local action = poll_action()
```

#### 查询动作是否刚刚触发

```lua
if is_action_pressed("confirm") then
  -- ...
end
```

### 5. 原始按键接口

```lua
local key = get_raw_key(true)
```

该接口保留给：

- 文本输入
- 调试
- 少量特殊编辑场景

不建议将核心移动、确认、菜单逻辑写死在原始按键上。

### 6. 按键写法

允许：

- `"left"`
- `"enter"`
- `"space"`
- `"a"`
- `"1"`
- `{"left", "a"}`
- `"ctrl+a"`
- `"alt+x"`
- `"shift+tab"`

其中常规按键支持最稳定，组合键是否能被终端完整识别，取决于系统和终端环境。

### 7. 冲突规则

若一个物理键绑定到多个动作：

- 宿主只返回一个动作；
- 按动作注册顺序优先；
- 第一个命中的动作生效。

## 十二、存档条目

### 1. 存档路径

每个模组游戏的存档统一写入：

```text
tui-game-data/mod/save/<namespace>/<game_id>.json
```

### 2. 使用方式

宿主提供：

```lua
save_data(key, value)
load_data(key)
save_game_slot(game_id, value)
load_game_slot(game_id)
```

模组作者无需自己拼接文件路径。

### 3. `save` 的含义

- `save = true`
  - 允许该游戏作为模组继续游戏入口的一部分。

- `save = false`
  - 即使存在旧存档，也不会在全局继续游戏入口中显示。

## 十三、最佳记录条目

### 1. 返回字符串

```lua
function best_score()
  return "Best Time 00:00:30"
end
```

### 2. 返回结构表

```lua
function best_score()
  return {
    label = "Best Time",
    value = "00:00:30",
    score = 120,
    time_sec = 30,
    extra = {
      mode = "hard",
      streak = 4
    }
  }
end
```

### 3. `extra`

`extra` 是额外键值对，宿主会原样保存，并在需要时用于展示附加信息。

### 4. 非法返回

若 `best_score()` 返回：

- `nil`
- 非法类型
- 非法结构表

宿主不会崩溃，而是：

- 记录 warning
- 游戏列表中显示 `--`

## 十四、日志条目

### 1. 接口

```lua
mod_log(level, message)
```

### 2. 可用级别

- `debug`
- `info`
- `warn`
- `error`

### 3. 写入路径

```text
tui-game-data/mod/logs/<namespace>.log
```

## 十五、安全与崩溃保护条目

### 1. 环境隔离

每个模组游戏拥有独立 Lua 环境，不共享全局变量。

### 2. 危险接口禁用

宿主默认禁用：

- `os.execute`
- `os.remove`
- `os.rename`
- `os.exit`
- `io.*`
- `package.loadlib`
- `debug.*`

### 3. 超时与预算保护

宿主会为模组脚本挂接执行预算保护。

若脚本长时间不返回、不触达宿主 API 或陷入死循环：

- 宿主会终止该游戏运行；
- 记录 `timeout`；
- 将玩家退回游戏列表；
- 不影响其他模组或内置游戏。

### 4. 崩溃降级

若模组运行时崩溃：

- 该游戏退出；
- 宿主显示降级错误提示；
- 然后返回游戏列表；
- 错误写入日志与模组状态文件。

## 十六、常见错误条目

### 1. 整包拒绝加载

以下情况会导致整包拒绝加载：

- `meta.json` 非法 JSON
- 缺少 `namespace`
- `namespace` 与目录名不一致
- `api_version` 不兼容

### 2. 单游戏拒绝加载

以下情况会导致脚本对应游戏拒绝加载：

- Lua 语法错误
- 缺失 `GAME_META`
- 缺失 `init_game()`、`game_loop()` 或 `best_score()`
- `GAME_META.name` 为空

### 3. 资源回退

以下情况不会整包崩溃，而是退回默认图：

- 图片不存在
- 图片格式不支持
- 图片损坏
- 数组图格式不合法

## 十七、示例条目

仓库中提供完整示例模组：

```text
examples/mod/example_pack/
```

示例内包含：

- `meta.json`
- 一个主脚本
- 一个辅助脚本
- 英文语言文件
- 中文语言文件
- 子图或宣传图示例

建议直接从该示例复制并修改，而不是从零手写。
