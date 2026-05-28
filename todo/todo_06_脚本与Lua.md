# 脚本与 Lua 层

## 当前阶段目标

从裸 Lua VM 升级为完整的脚本运行时：注册引擎 API、加载和执行游戏脚本、管理脚本沙箱和安全策略。Lua 是游戏逻辑的执行环境，本层决定了游戏开发者能使用哪些能力。

---

## 服务项目清单

### 6-1 LuaService 核心

- **职责**：管理 Lua VM 的生命周期，包括创建 VM、配置沙箱、注册 API 模块、加载和执行脚本、处理脚本错误。
- **当前状态**：骨架存在。`LuaService` 包装了 `mlua::Lua`，仅提供 `new()`、`lua()` 和 `eval()` 三个方法。无任何 API 注册、无脚本加载、无安全限制。
- **待完善**：
  - VM 初始化配置：设置内存限制、指令数限制、执行超时。
  - 沙箱环境：移除或限制危险的标准库函数（如 `os.execute`、`io.open` 等），仅保留安全子集。
  - 脚本加载：按文件路径加载 Lua 脚本，支持多文件项目（`require` 机制）。
  - 脚本缓存：将加载过的 Lua 脚本编译为字节码缓存，加速后续加载。
  - 错误处理：脚本执行出错时，捕获错误信息（含 Lua 调用栈），返回给引擎而非直接 panic。
  - 全局状态隔离：不同游戏包运行在独立的 Lua 环境中（或至少独立的作用域），防止跨包数据污染。

### 6-2 Lua API 模块集

- **职责**：向 Lua 脚本暴露引擎能力的 API 模块集合。每个模块是一个 Lua table，包含一组函数，游戏脚本通过 `require("module_name")` 或全局变量访问。
- **当前状态**：不存在。
- **待完善**：

  **绘制 API（drawing）**：
  - 画布大小查询：`get_width()`、`get_height()`
  - 像素级绘制：`set_pixel(x, y, char, fg, bg)`
  - 图形绘制：`draw_rect(x, y, w, h, char, fg, bg)`、`draw_line(x1, y1, x2, y2, char, fg, bg)`、`draw_circle(cx, cy, r, char, fg, bg)`
  - 文本绘制：`draw_text(x, y, text, fg, bg)`
  - 裁剪区域：`set_clip(x, y, w, h)`、`reset_clip()`
  - 清屏：`clear(fg, bg)`
  - 颜色常量：提供预定义的 ANSI 颜色名称常量表

  **文件读取 API（file_reader）**：
  - 读取包内文本文件：`read_text(path) -> string`
  - 读取包内数据文件：`read_data(path) -> string`（二进制安全的字符串）
  - 限制：只能读取包自身目录内的文件，不能读取其他包或引擎系统文件。
  - 读取范围限制在包目录内，路径不能包含 `..` 越界。

  **文件写入 API（file_writer）**：
  - 仅在 `package.json` 中声明 `runtime.write = true` 时可用。
  - 写入文本文件：`write_text(path, content)`
  - 写入数据文件：`write_data(path, data)`
  - 仅能写入包自身的 save 目录，路径受限。

  **布局计算 API（layout）**：
  - 辅助游戏开发者进行 UI 布局计算。
  - 文本居中计算：给定文本和宽度，返回起始 x 坐标。
  - 弹性布局辅助：等分空间、对齐计算。

  **文字测量 API（measurement）**：
  - 测量文本的显示宽度（考虑 Unicode 全角/半角）：`measure_width(text) -> int`
  - 测量文本行数（给定最大宽度后的自动换行行数）：`measure_lines(text, max_width) -> int`

  **随机数 API（random）**：
  - `random_int(min, max) -> int`
  - `random_float() -> float`
  - 可设置种子用于可复现的随机序列。

  **计时器 API（timer）**：
  - 获取当前帧的 delta time。
  - 获取自游戏启动以来的总运行时间。
  - 创建倒计时器（回调或查询式）。

  **表格工具 API（table_utils）**：
  - Lua 表格的辅助操作：深拷贝、合并、序列化为字符串等。

  **文本支持 API（text_support）**：
  - 字符串处理辅助：截断、填充、大小写转换、Unicode 字符判断。

  **模块加载 API（module_loader）**：
  - 允许游戏包内部使用 `require` 加载同目录下的其他 Lua 文件。

### 6-3 Lua 宿主桥接（Host Bridge）

- **职责**：定义 Rust 和 Lua 之间的通信协议。游戏脚本通过桥接向引擎发送请求（如"保存游戏状态"、"查询玩家最佳成绩"、"结束游戏"），引擎通过桥接调用脚本的生命周期函数。
- **当前状态**：不存在。
- **待完善**：
  - 请求格式定义：Lua 侧向 Rust 侧发送的请求消息结构（JSON 或 Lua table 约定）。
  - 四个生命周期入口：`init()`、`handle_event(event)`、`update(dt)`、`render()`
  - 引擎到脚本的调用：引擎在帧循环中按序调用脚本的上述四个函数。
  - 脚本到引擎的请求队列：脚本调用 API 后，请求加入队列，引擎在帧循环中处理。
  - 存档请求：脚本请求保存游戏数据，引擎将数据写入 StorageService。
  - 成绩提交：脚本提交玩家成绩，引擎更新最佳成绩记录。
  - 退出请求：脚本请求结束当前游戏会话。

### 6-4 Lua 安全沙箱

- **职责**：限制 Lua 脚本的执行权限，防止恶意或错误的脚本影响引擎稳定性或访问用户敏感数据。
- **当前状态**：不存在。
- **待完善**：
  - 禁用危险标准库函数：`os.execute()`、`os.exit()`、`os.remove()`、`os.rename()`、`io.popen()` 等。
  - 内存限制：设置 Lua VM 的最大可用内存。
  - 指令数限制：设置单次脚本执行的指令上限，防止无限循环挂起引擎。
  - 执行超时：设置脚本执行的最大时间，超时后中断。
  - 文件访问限制：脚本只能访问自身包目录和自身 save 目录。
  - 网络访问：默认禁止，未来如需联网功能则提供受限的网络 API。

---

## 旧架构参考

### Lua 运行时（`old_src/boot/preload/lua_runtime/`）
旧架构的 Lua 集成非常完整，包含以下模块：
- **host_bridge**：Rust↔Lua 桥接，定义了完整的通信协议和数据结构。桥接的数据格式经过了实战验证。
- **sandbox**：沙箱配置，定义了安全策略和函数白名单。
- **environment**：Lua 全局环境配置，注入 API 模块到全局作用域。
- **10+ API 模块**：drawing、file_reader、file_writer、layout、measurement、module_loader、random、table_utils、text_support、timer_support。
- **debug**：Lua 侧的调试支持（如打印调试信息到引擎日志）。

旧架构的 Lua API 经过了实际游戏的验证（18 个可玩游戏），其 API 函数签名和参数设计具有很高的参考价值，可以直接沿用。

旧架构中 Lua 通过 `Canvas::Bridge` 访问画布进行绘制。新架构应简化为：Lua 调用绘制 API → API 将命令写入队列 → 引擎在渲染阶段处理队列 → 命令执行到 Canvas。

---

## 完成后可验证的可用项

1. 启动引擎后 Lua VM 创建成功，`eval("return 1+1")` 返回 `"2"`。
2. 危险函数（如 `os.execute`）在脚本中不可用。
3. 脚本调用 `draw_rect(5, 5, 10, 3, "#", "white", "black")` 在终端正确绘制矩形。
4. 脚本尝试读取自身包目录外的文件时返回错误。
5. 脚本中的无限循环在超时后被中断，引擎不挂起。
6. 脚本调用存档请求后，数据被正确写入 StorageService。
7. 多文件 Lua 项目通过 `require` 正确加载依赖。
8. 游戏结束时脚本调用退出请求，GameService 正常关闭会话。
