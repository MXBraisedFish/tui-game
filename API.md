# UI API quick map

只给当前 UI 检查用。原则：UI 不自己实现文本测量、换行、裁切、命中、组件状态。

## Text / rich text

- `DrawTextParams`
  - `text`, `params`, `fg/bg`, `line_align`
  - `wrap_mode: None | Normal | Auto`
  - `non_truncate_word_wrap` 默认 `true`
  - `max_width`, `max_height`, `overflow_marker`
- `RenderService`
  - `draw_host_text`, `draw_text`, `draw_text_on`, `draw_text_in_scroll_box`
  - `draw_host_filled_rect`, `draw_host_border_rect`
  - `draw_filled_rect*`, `draw_border_rect*`
- `CanvasService`
  - 原始写入：`text*`, `styled_text*`, `host_styled_text`
  - 查询：`base_*`, `prepared_slice_*`, `prepared_scroll_box_*`
- `LayoutService`
  - 宽高：`physical_*`, `developer_*`
  - 测量：`get_text_width`, `get_draw_text_size`
  - 坐标：`resolve_*`, `resolve_base_*`, `resolve_slice_*`, `resolve_scroll_box_*`
- `RichTextService`
  - `parse`, `visible_text`
  - `{key}` 用 `RichTextParams`

UI 禁止：

- 自己按 char/grapheme 写换行。
- 自己忽略富文本标签算宽度。
- 自己做 `...` 截断；用 `DrawTextParams.max_width + overflow_marker`。

## UI objects

- 每个页面：`UiObjectPool`, `RuntimeObjectPool`
- 页面事件：`UiEvent::{Action, HitArea, TextInput, ScrollBox}`

## Components

- `HitAreaService`
  - `create/remove/exists`
  - `render`, `render_on`, `render_host`
  - 状态：`is_hovered`, `is_pressed`, `pointer_position`, `local_pointer_position`
- `TextInputService`
  - `create/remove/render/render_on/render_host`
  - `focus/blur/is_active/is_focused`
  - `get_text/set_text/clear/cursor/selection`
- `ScrollBoxService`
  - `create/remove/set_rect/set_content_size`
  - `scroll_to/scroll_by/top/bottom`
  - `viewport_*`, `visible_content_*`, `content_*`
- `SliceService`
  - `create/remove/set_rect/visible/order`
  - `configured_rect`, `resolved_*`
- `ProgressBarService`
  - `create/remove/set_progress/render/render_on/render_host`

UI 禁止：

- 自己维护鼠标 hover/click/drag。
- 自己裁剪滚动内容；用 ScrollBox。
- 自己绘制组件命中区域；用 HitArea。

## Runtime objects

- `TimeService + RuntimeObjectPool`
  - Timer / DelayTimer / RepeatTimer
- `RandomService + RuntimeObjectPool`

UI 禁止：

- 用本地 elapsed 状态模拟已有 timer，除非只是临时简单 UI 状态。

## Audit notes

- 通用手写换行已清掉：`rg "wrap_plain|wrapped_plain|\.graphemes\(" src/host_engine/ui` 无结果。
- `terminal_check.rs` 保留原始鼠标读取：它本身就是终端能力检测页。
- 包资源 icon/banner 的补齐/裁切是包资源规范化，不当作通用文本布局。
- storage path marquee 是该页面指定动画，不当作 ScrollBox 替代。
