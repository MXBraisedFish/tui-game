# 模组包设置界面 Rust 实现与 Lua 参考实现差异报告

本文档对比 `official_ui/scripts/` 中的 Lua 参考实现与 `src/host_engine/runtime/ui/pages/` 中的 Rust 实现，标记所有不符合原版 Lua 界面渲染的问题。每个问题包含：Lua 参考行为、Rust 当前行为、影响类别（布局/颜色/UX动态）。

---

## 一、模组包设置中转页 (ModHubPage / setting_mods)

### 1.1 菜单项按键提示的动态切换

- **Lua 参考**: 当前选中项显示 `[Enter]`（确认键），非选中项显示各自的数字快捷键 `[1]` / `[2]` / `[3]`。选中项变化时，按键提示随之切换。
- **Rust 当前**: `draw_center_menu` 中第 0 行永远显示 confirm 键，第 1 行永远显示 option2 键，第 2 行永远显示 option3 键。无论哪行被选中，按键提示都不变。
- **影响类别**: UX动态显示
- **根因**: `mod_hub.rs` 传入的 `ACTIONS = ["confirm", "option2", "option3"]` 按行索引固定映射，缺少 `option1`。`draw_center_menu`（`common.rs:76-108`）没有根据 `selected_index` 替换选中行的 action 为 confirm。
- **修复方向**: 在 `draw_center_menu` 中，当 `index == selected_index` 时，应使用 "confirm" action 获取按键提示；当 `index != selected_index` 时，才使用 `actions[index]`。同时 `ACTIONS` 应补充 `option1`。

### 1.2 选中项与非选中项的标记符号间距

- **Lua 参考**: 选中项前缀 `"▶ "`（三角+空格），非选中项前缀 `"  "`（两个空格）。标记、按键、标签三者紧密排列在同一行。
- **Rust 当前**: 选中项标记 `"▶"`（无尾随空格），非选中项 `" "`。标记绘制在 x 位置，按键绘制在 `x+2`，标签绘制在 `x+2+hint.len()+3`。由于标记仅占 1 字符宽度但 cursor 偏移为 2，非选中项会产生额外空位。
- **影响类别**: 布局问题
- **根因**: `draw_center_menu`（`common.rs:79-106`）使用固定偏移量组合绘制，与 Lua 的流式 `cursor_x` 累加策略不一致。

---

## 二、模组包列表页 (ModGameListPage / ModScreensaverListPage / ModBossListPage)

### 2.1 左面板列表项仅显示包名——缺失完整信息行

- **Lua 参考（full 模式）**: 每个列表项占据 5 行高度，包含：
  - 第 0 行: `[D]` 调试标记（红色，仅 debug=true 时显示）+ 包名（加粗）
  - 第 1 行: `Author: ` 标签 + 作者名（富文本）
  - 第 2 行: `Version: ` 标签 + 版本号（富文本）
  - 第 3 行: `Status: ` 标签 + 启用/禁用状态（绿色/红色加粗）
  - 左侧 8×4 区域绘制图标
  - 右侧边缘 safe_mode=false 时绘制红色竖条
- **Rust 当前**: 每个列表项仅绘制一行包名文本（`render_game_row` / `render_overlay_row`），无图标、无调试标记、无作者、无版本、无状态、无 safe_mode 指示条。
- **影响类别**: 布局问题、颜色使用问题、UX动态显示
- **根因**: `mod_package_list.rs` 的 `render_game_row` 和 `render_overlay_row` 仅调用 `canvas.draw_text_styled` 绘制包名，未实现 Lua 中的 `draw_full_item` 逻辑。

### 2.2 缺失 brief 列表模式

- **Lua 参考**: 支持 full/brief 两种列表模式切换。brief 模式每项仅 1 行高，紧凑显示 `[D]` + 包名 + `[On]`/`[Off]` 状态标签。
- **Rust 当前**: 无 brief 模式，始终为单行展示，且单行展示中也缺少状态标签和调试标记。
- **影响类别**: UX动态显示
- **根因**: `mod_package_list.rs` 未实现 list_mode 切换逻辑和 `draw_brief_item`。

### 2.3 左面板缺失排序/排序方向彩色标题头

- **Lua 参考**: 左面板顶部显示 ` Mods *[Asc] Name ` 格式的彩色标题头。`*` 为白色加粗，`[Asc]` 中 "Asc"/"Desc" 为绿色加粗，"Name"/"Author"/"Safe Mode"/"Enabled" 为黄色加粗。
- **Rust 当前**: 左面板仅显示普通标题文本 `ctx.i18n.mod_list.list_title`，无排序方向和排序字段展示。
- **影响类别**: 布局问题、颜色使用问题、UX动态显示

### 2.4 缺失分页导航行

- **Lua 参考**: 左面板底部（`content_height - 2` 行）显示分页信息：
  - 左侧：`◀ [PgUp]`（仅当 page>1 时显示）
  - 右侧：`[PgDn] ▶`（仅当 page < total_pages 时显示）
  - 中央：`{current_page}/{total_pages}`（正常为灰色，跳转模式时输入数字高亮黄底黑字）
- **Rust 当前**: 无分页行渲染。`render_game_packages` / `render_overlay_packages` 使用 `.take(panel.height - 4)` 硬截断，超出可视区的项目直接丢弃。
- **影响类别**: 布局问题、UX动态显示

### 2.5 缺失跳转页码输入模式

- **Lua 参考**: 按 jump 键后进入跳转模式，action_line 切换为数字输入提示，用户可输入数字后按确认跳转。页码输入区用黄底黑字高亮。
- **Rust 当前**: 未实现跳转模式。
- **影响类别**: UX动态显示

### 2.6 选中项背景色不一致

- **Lua 参考**: 选中项背景色为 `DARK_GRAY`，前景色为 `white`。
- **Rust 当前**: 选中项背景色使用主题色 `background.selected`（fallback `"#78a8da"` 蓝色），前景色使用 `text.on_selected`（fallback `"black"`）。
- **影响类别**: 颜色使用问题
- **根因**: `render_game_row` / `render_overlay_row` 使用 theme_color 查找，但 Lua 明确指定 `DARK_GRAY` 背景 + `white` 前景。

### 2.7 右面板信息区简陋——缺失结构化信息展示

- **Lua 参考**: 右面板信息区包含：
  - Banner 横幅图像（富文本居中，最多 13 行高度）
  - 分隔空行
  - `Basic Info:` 节标题（黄色）
  - 包名、作者、版本
  - `Security Info:` 节标题（黄色）
  - 启用状态（绿色/红色）、调试状态（红色/灰色）、写入请求状态（红色/灰色）
  - 安全模式状态（绿色 On / 红色 Off(Session) / 红色 Off(Permanent)）
  - `Introduction:` 节标题（黄色）+ 富文本介绍
  - 右侧滚动条（↑↓箭头+滑块轨道）
- **Rust 当前**: `render_info_lines` 仅逐行输出 `[package_name, author_line, version_line, introduction, description]`。无节标题、无颜色区分、无 Banner、无滚动条、无安全信息段。
- **影响类别**: 布局问题、颜色使用问题、UX动态显示

### 2.8 Boss/Screensaver 列表页未区分渲染

- **Lua 参考**:
  - Game 列表: 包含 safe_mode、write 安全信息字段，排序包含 safe_mode 选项
  - Boss 列表: 无 safe_mode 和 write 字段，排序为 name→author→toggle→debug
  - Screensaver 列表: 无 safe_mode 和 write 字段，排序为 name→author→toggle→debug
  - Boss/Screensaver 的 action_line 不含 safe_mode 操作按键
- **Rust 当前**: 三种列表使用相同的 `render_packages` → `render_game_packages` / `render_overlay_packages` 路径，渲染逻辑无差异。
- **影响类别**: UX动态显示

### 2.9 底部操作提示行信息严重缺失

- **Lua 参考**: action_line 动态包含以下所有操作提示（支持多行折行）：
  - `[↑]/[↓] Select`
  - `[Enter] Toggle / Confirm`
  - `[D] Debug`
  - `[S] Safe Mode`（仅 Game 列表有）
  - `[L] List`（切换 full/brief 模式）
  - `[W]/[S] Scroll`（信息面板滚动）
  - `[O] Order`（切换升序/降序）
  - `[T] Sort`（切换排序字段）
  - `[J] Jump`（仅多页时显示）
  - `[PgUp]/[PgDn] Flip`（仅多页时显示）
  - `[Esc] Back`
- **Rust 当前**: 硬编码 `"[↑]/[↓] Select   [Enter] Toggle   [Esc] Back"`。缺少 Debug、List、Scroll、Order、Sort、Safe Mode、Jump、Flip 操作提示。
- **影响类别**: UX动态显示
- **根因**: `mod_package_list.rs` 的 `render` 方法使用固定字符串调用 `draw_footer`。

### 2.10 缺失信息面板滚动功能

- **Lua 参考**: 右面板支持垂直滚动，通过 scroll_up/scroll_down 操作。右侧显示滚动条（↑/↓箭头 + 滑块轨道 + 按键提示）。
- **Rust 当前**: 无滚动支持，信息行超出面板高度时直接截断（`if y >= panel.height - 1 { break }`）。
- **影响类别**: UX动态显示

---

## 三、安全模式警告页 (WarningModPage / warning_mod)

### 3.1 缺失模组包名称显示

- **Lua 参考**: 警告文本下方显示 `Mod: {实际包名称}`，使用白色文本，使玩家明确知道正在操作哪个模组。
- **Rust 当前**: `render_security_warning` 的 `info_lines` 参数为 `vec![ctx.i18n.mod_security.mod_label.clone()]`，其中 `mod_label` 仅为 `"Mod: "` 标签文本，未拼接实际的包名称。
- **影响类别**: UX动态显示
- **根因**: `warnings.rs:239` 传入的 info_lines 只包含标签前缀，没有在调用侧获取 `mod_list_state.package_name(uid)` 并拼接。

### 3.2 操作按钮颜色语义

- **Lua 参考**:
  - 取消按钮: 绿色（`CANCEL_COLOR = "green"`）
  - 临时关闭按钮（倒计时中）: 灰色（`DISABLED_COLOR = DARK_GRAY`）
  - 临时关闭按钮（就绪）: 红色（`CONFIRM_COLOR = "red"`）
  - 永久关闭按钮（倒计时中）: 灰色
  - 永久关闭按钮（就绪）: 红色
- **Rust 当前**: 取消按钮绿色（`state.success`）、倒计时按钮灰色/就绪红色（`text.muted`/`state.danger`）。颜色语义基本一致。
- **影响类别**: 颜色使用问题（**无问题**，已正确实现）

---

## 问题汇总统计

| 类别 | 问题数 |
|------|--------|
| 布局问题 | 7 个（1.2, 2.1, 2.3, 2.4, 2.7, 2.8, 2.10） |
| 颜色使用问题 | 4 个（2.1, 2.3, 2.6, 2.7） |
| UX动态显示 | 10 个（1.1, 2.1, 2.2, 2.3, 2.4, 2.5, 2.7, 2.8, 2.9, 3.1） |

**总计: 14 个独立问题**（部分问题同时涉及多个类别）

### 严重程度分级

- **P0（阻断级——核心交互缺失）**:
  - 2.9 底部操作提示缺失导致用户无法发现 Debug/Sort/Order/List/Jump 等功能入口
  - 2.4/2.5 无分页和跳转导致超过可视区的模组包完全无法访问
- **P1（主要——显著体验降级）**:
  - 2.1 列表项仅显示包名，缺失图标、作者、版本、状态、safe_mode 指示等完整信息
  - 2.7 信息面板无 Banner、无颜色区分的安全信息段、无节标题
  - 2.3 排序状态不可见，用户无法感知当前排序依据
  - 2.10 信息过长时无滚动机制，内容被截断
- **P2（次要——功能弱化或视觉不一致）**:
  - 1.1 菜单按键提示不随选中动态切换
  - 2.2 缺少 brief 紧凑列表模式
  - 2.6 选中项背景色使用蓝色而非 Lua 规范的深灰色
  - 2.8 Boss/Screensaver 列表未做差异化渲染
  - 3.1 安全警告页未显示目标模组包名称

---

## 五、尺寸提醒界面 (WarningNeededSizePage / warning_needed_size)

### 5.1 缺失实际/需求终端尺寸数值显示

- **Lua 参考**: "Required terminal size: {width} x {height}" 和 "Current terminal size: {width} x {height}" 两行都包含具体的宽度×高度数值。数值由 `size_text(root_state.needed)` 和 `size_text(root_state.actual)` 从传入的 root_state 中实时提取。
- **Rust 当前**: `warning_page!` 宏的 body 闭包只返回三条 i18n 纯文本标签 `[size_actual, size_needed, size_hint]`。`size_actual` 的值是 `"Current terminal size: "`（仅标签前缀），`size_needed` 是 `"Required terminal size: "`（仅标签前缀）。**实际像素数值从未被拼接到渲染文本中**，用户看不到当前和所需的具体尺寸。
- **影响类别**: 布局问题、UX动态显示
- **根因**: `warnings.rs:89-93` 的 body 闭包直接返回 i18n 字符串，未从 `ctx.terminal_size` 或 `NeededSizeRootState` 中获取实际尺寸数值并拼接。
- **修复方向**: `WarningNeededSizePage` 不能使用通用的 `warning_page!` 宏。需要自定义 `render` 方法，在渲染时读取 `ctx.terminal_size`（实际尺寸）和硬编码的 98×26（需求尺寸），拼接成完整文本行。

### 5.2 缺失操作/返回按键提示行

- **Lua 参考**: 第四行显示 `[Esc/Q] Exit the program`（root 模式）或 `[Esc/Q] Return to game list`（game 模式）。按键 `[Esc/Q]` 通过 `get_key("return")` 动态获取用户绑定的返回键。
- **Rust 当前**: body 仅有三行文本（actual / needed / hint），**完全没有操作行**。用户不知道该按什么键退出或返回。
- **影响类别**: 布局问题、UX动态显示
- **根因**: `warning_page!` 宏只渲染 body 中的行列表，不支持额外的操作提示行。`WarningNeededSizePage` 需要脱离宏自行实现。

### 5.3 信息行排列顺序颠倒

- **Lua 参考**: 从上到下依次为：① 需求尺寸（needed）→ ② 实际尺寸（actual）→ ③ 提示文字（hint）→ ④ 操作按键（action）。需求尺寸是主要告知信息，放在最前面。
- **Rust 当前**: body 顺序为 `[size_actual, size_needed, size_hint]`。实际尺寸排在需求尺寸之前，与 Lua 相反。
- **影响类别**: 布局问题
- **根因**: body 闭包中数组元素的排列顺序错误。

### 5.4 行间距不一致

- **Lua 参考**: 需求尺寸行在 `y`，实际尺寸行在 `y + 2`（中间空 1 行），提示行在 `y + 3`，操作行在 `y + 5`（与提示行之间空 1 行）。总共 content_height = 6 行（4 行文本 + 2 行空隙），整体垂直居中。
- **Rust 当前**: 所有行连续排列 `top + index`，行间距为统一的 1 行，无分组空隙。
- **影响类别**: 布局问题
- **根因**: `warning_page!` 宏的渲染循环使用 `index as u16` 作为行偏移，无空隙逻辑。

### 5.5 颜色使用不一致

- **Lua 参考**:
  - 需求尺寸行：`WARNING_COLOR = "yellow"`，BOLD
  - 实际尺寸标签："white"，BOLD；数值：`VALUE_COLOR = CYAN`，BOLD（同一行双色）
  - 提示行：`HINT_COLOR = DARK_GRAY`，普通字体
  - 操作行：`HINT_COLOR = DARK_GRAY`，普通字体
- **Rust 当前**: 所有 body 行统一使用 `theme_color(ctx, "text.warning", "yellow")`，非粗体（`draw_centered` 的 `bold: false`）。且标题 `"Terminal Size"` 使用 `text.primary / white` BOLD。
- **影响类别**: 颜色使用问题
- **根因**: `warning_page!` 宏对所有 body 行使用统一的颜色和样式参数，不支持逐行差异化。

### 5.6 标题多出不必要

- **Lua 参考**: 页面无标题，直接显示内容。
- **Rust 当前**: 通过 `draw_title` 在 y=1 渲染硬编码标题 `"Terminal Size"`（无 i18n）。此标题在 Lua 原版中不存在。
- **影响类别**: 布局问题
- **根因**: `warning_page!` 宏对所有 warning 页面统一调用 `draw_title`，但尺寸提醒页在原设计中不需要标题。

### 5.7 缺失模式区分（root vs game）

- **Lua 参考**: `root_state.mode == "game"` 时，操作行显示 `"Return to game list"`；`mode == "root"` 时显示 `"Exit the program"`。两种模式下的操作文本和行为不同。
- **Rust 当前**: `warning_page!` 宏的 `$body` 闭包无法获取 mode 信息。`handle_event` 中任何按键都导航到 `$back`（固定为 `UiPageKey::Home`），无模式区分。
- **影响类别**: UX动态显示
- **根因**: `warning_page!` 宏参数化不足，body 闭包只接收 `&UiContext`，无 mode 参数。`NeededSizeMode` 的状态虽在 `event_loop.rs` 中通过 `set_needed_size_mode` 设置到 `active_ui_page`，但从未传递到页面渲染上下文中。

### 5.8 事件处理过于宽松

- **Lua 参考**: 仅 `"return"` 动作触发 `exit = true`。其他事件不响应。
- **Rust 当前**: `"confirm" | "enter" | "back" | "return" | "esc" | "q"` 全部触发导航回 `$back`。过于宽松，与 Lua 的精确定义不一致。
- **影响类别**: UX动态显示
- **根因**: `warning_page!` 宏对所有 warning 页使用统一的事件处理逻辑。

### 5.9 i18n 动作文本未被使用

- **Lua 参考**: 使用 `WARNING_SIZE_ACTION_EXIT`（"Exit the program"）和 `WARNING_SIZE_ACTION_RETURN`（"Return to game list"）i18n 键。
- **Rust 当前**: `WarningText` 结构体已定义 `size_action_exit` 和 `size_action_return` 字段并正确加载 i18n 值，但在 `WarningNeededSizePage` 的 body 闭包中**完全未被引用**。
- **影响类别**: UX动态显示
- **根因**: body 闭包未使用这两个已加载的字段。

---

## 问题汇总（更新）

| 新增问题 | 类别 |
|---------|------|
| 5.1 缺失尺寸数值 | 布局, UX动态 |
| 5.2 缺失操作按键行 | 布局, UX动态 |
| 5.3 行排列顺序颠倒 | 布局 |
| 5.4 行间距不一致 | 布局 |
| 5.5 颜色使用不一致 | 颜色 |
| 5.6 标题多余 | 布局 |
| 5.7 缺失模式区分 | UX动态 |
| 5.8 事件处理过于宽松 | UX动态 |
| 5.9 动作文本未被使用 | UX动态 |

**本节新增: 9 个问题**

### 严重程度分级

- **P0（阻断级）**: 5.1（用户看不到具体尺寸，不知要调多大窗口）、5.2（用户不知道按什么键退出）
- **P1（主要）**: 5.5（全部黄色无区分）、5.7（game/root 模式不区分）、5.3（顺序颠倒）
- **P2（次要）**: 5.4（间距不同）、5.6（多余标题）、5.8（按键过于宽松）、5.9（i18n 未用）
