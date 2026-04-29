// 设置模块的通用工具库，提供所有子页面共享的渲染辅助函数、比较器、布局计算、文本处理、分页工具、数据加载等。是 settings 模块中最大的文件，作为各页面的基础依赖

use std::cmp::Ordering; // 排序比较（升序/降序）

use ratatui::buffer::Buffer; // 直接操作终端缓冲区进行像素级渲染
use ratatui::layout::{Alignment, Rect}; // 对齐方式和矩形区域
use ratatui::style::{Color, Modifier, Style}; // 颜色、样式修饰（粗体等）
use ratatui::text::{Line, Span}; // 富文本行和片段
use ratatui::widgets::{Block, Borders, Paragraph}; // 绘制带边框的块、段落、自动换行
use unicode_width::UnicodeWidthStr; // Unicode 字符串宽度计算（CJK 字符占2列）
use crate::app::settings::types::GridMetrics; // 网格布局参数类型

use crate::app::i18n; // 国际化文本
use crate::app::rich_text; // 富文本解析（f% 格式）
use crate::game::registry::{GameDescriptor, GameSourceKind}; // 游戏描述符和来源类型（用于排序）
use crate::mods::{self, ModPackage}; // Mod 包类型和模块（用于排序、列表渲染）

pub const TRIANGLE: &str = "\u{25B6} "; // 选中项的标记前缀
pub const H_GAP: u16 = 1; // 网格布局中列之间的间距
pub const MAX_COLS: usize = 12; // 语言选择网格的最大列数限制

// 获取国际化文本，封装了 i18n::t_or，为设置页面提供统一的文本获取方式
pub fn text(key: &str, fallback: &str) -> String {
    i18n::t_or(key, fallback)
}

// 不区分大小写的字符串比较，返回 Ordering
pub fn cmp_lowercase(left: &str, right: &str) -> Ordering {
    left.to_lowercase().cmp(&right.to_lowercase())
}

// 布尔值排序：true 排在 false 前面（right.cmp(&left)）
pub fn bool_true_first(left: bool, right: bool) -> Ordering {
    right.cmp(&left)
}

// 返回游戏来源的排序优先级：官方为 0，Mod 为 1（官方优先）
pub fn source_rank(source: &GameSourceKind) -> u8 {
    match source {
        GameSourceKind::Official => 0,
        GameSourceKind::Mod => 1,
    }
}

// Mod 包的四种排序模式：名称、启用状态、作者、安全模式。每种模式包含多级回退排序链
pub fn compare_mod_packages(left: &ModPackage, right: &ModPackage, mode: crate::app::settings::ModSortMode) -> Ordering {
    // 注意：ModSortMode 现在在 types 模块中，因此路径应为 crate::app::settings::types::ModSortMode
    // 但因 settings.rs 有 pub use types::*;，所以类型在 settings 空间也可见。
    // 这里我们用完整路径以确保编译。
    use crate::app::settings::types::ModSortMode;
    match mode {
        ModSortMode::Name => cmp_lowercase(&left.package_name, &right.package_name)
            .then_with(|| cmp_lowercase(&left.author, &right.author))
            .then_with(|| left.namespace.cmp(&right.namespace)),
        ModSortMode::Enabled => bool_true_first(left.enabled, right.enabled)
            .then_with(|| cmp_lowercase(&left.package_name, &right.package_name))
            .then_with(|| left.namespace.cmp(&right.namespace)),
        ModSortMode::Author => cmp_lowercase(&left.author, &right.author)
            .then_with(|| cmp_lowercase(&left.package_name, &right.package_name))
            .then_with(|| left.namespace.cmp(&right.namespace)),
        ModSortMode::SafeMode => bool_true_first(left.safe_mode_enabled, right.safe_mode_enabled)
            .then_with(|| cmp_lowercase(&left.package_name, &right.package_name))
            .then_with(|| left.namespace.cmp(&right.namespace)),
    }
}

// 键位页面游戏的三种排序：来源、名称、作者，多级回退
pub fn compare_keybind_games(
    left: &GameDescriptor,
    right: &GameDescriptor,
    mode: crate::app::settings::types::KeybindGameSortMode,
) -> Ordering {
    use crate::app::settings::types::KeybindGameSortMode;
    match mode {
        KeybindGameSortMode::Source => source_rank(&left.source)
            .cmp(&source_rank(&right.source))
            .then_with(|| cmp_lowercase(&left.display_name, &right.display_name))
            .then_with(|| left.id.cmp(&right.id)),
        KeybindGameSortMode::Name => cmp_lowercase(&left.display_name, &right.display_name)
            .then_with(|| source_rank(&left.source).cmp(&source_rank(&right.source)))
            .then_with(|| left.id.cmp(&right.id)),
        KeybindGameSortMode::Author => cmp_lowercase(&left.display_author, &right.display_author)
            .then_with(|| cmp_lowercase(&left.display_name, &right.display_name))
            .then_with(|| left.id.cmp(&right.id)),
    }
}

// 在指定区域内创建水平和垂直居中的矩形。广泛用于对话框和设置框
pub fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    Rect {
        x: area.x + area.width.saturating_sub(width.min(area.width)) / 2,
        y: area.y + area.height.saturating_sub(height.min(area.height)) / 2,
        width: width.min(area.width).max(1),
        height: height.min(area.height).max(1),
    }
}

// 创建黄色粗体的分区标题行，用于 Mod 详情中的"Basic Info"等
pub fn section_title_line(title: String) -> Line<'static> {
    Line::from(Span::styled(
        title,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
}

// 创建"标签: 值"格式的单行，值以粗体显示
pub fn label_value_line(label: String, value: String, value_style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(Color::White)),
        Span::raw(" "),
        Span::styled(value, value_style.add_modifier(Modifier::BOLD)),
    ])
}

// 富文本版本的 label_value_line，支持 f% 富文本和多行续行缩进
pub fn label_value_lines(
    label: String,
    value: String,
    allow_rich: bool,
    width: usize,
    value_style: Style,
) -> Vec<Line<'static>> {
    if !allow_rich || !value.starts_with("f%") {
        return vec![label_value_line(label, value, value_style)];
    }
    let mut parsed = rich_text::parse_rich_text_wrapped(&value, usize::MAX / 8, value_style);
    if parsed.is_empty() {
        return vec![label_value_line(label, String::new(), value_style)];
    }
    let mut first_spans = vec![
        Span::styled(label.clone(), Style::default().fg(Color::White)),
        Span::raw(" "),
    ];
    first_spans.extend(parsed.remove(0).spans);
    let mut lines = vec![Line::from(first_spans)];
    let indent = " ".repeat(UnicodeWidthStr::width(label.as_str()) + 1);
    let continuation_width = width.saturating_sub(indent.len()).max(1);
    for line in parsed {
        let mut spans = vec![Span::styled(indent.clone(), Style::default().fg(Color::White))];
        let wrapped = crop_line_center_to_width(&line, continuation_width);
        spans.extend(wrapped.spans);
        lines.push(Line::from(spans));
    }
    lines
}

// 渲染可选择的操作项：选中时显示 ▶ Enter 前缀和青色高亮，未选中显示序号
pub fn selection_action_line(index: usize, selected: usize, label: String) -> Line<'static> {
    let selected_row = index == selected;
    let marker_style = Style::default()
        .fg(Color::LightCyan)
        .add_modifier(Modifier::BOLD);
    let text_style = if selected_row {
        Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let key = if selected_row {
        i18n::t("menu.enter_shortcut")
    } else {
        format!("[{}]", index + 1)
    };
    let key_style = Style::default().fg(Color::DarkGray);
    Line::from(vec![
        Span::raw(" "),
        Span::styled(if selected_row { "▶ " } else { "  " }, marker_style),
        Span::styled(key, key_style),
        Span::raw(" "),
        Span::styled(label, text_style),
    ])
}

// 与 selection_action_line 类似，但增加了一个 [ 状态 ] 指示器（绿/红）
pub fn selection_option_with_value_line(
    index: usize,
    selected: usize,
    label: String,
    enabled: bool,
    enabled_key: &str,
    disabled_key: &str,
    enabled_fallback: &str,
    disabled_fallback: &str,
) -> Line<'static> {
    let selected_row = index == selected;
    let marker_style = Style::default()
        .fg(Color::LightCyan)
        .add_modifier(Modifier::BOLD);
    let text_style = if selected_row {
        Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let key = if selected_row {
        i18n::t("menu.enter_shortcut")
    } else {
        format!("[{}]", index + 1)
    };
    let key_style = Style::default().fg(Color::DarkGray);
    let status = if enabled {
        text(enabled_key, enabled_fallback)
    } else {
        text(disabled_key, disabled_fallback)
    };
    let status_style = Style::default()
        .fg(if enabled { Color::Green } else { Color::Red })
        .add_modifier(Modifier::BOLD);
    Line::from(vec![
        Span::raw(" "),
        Span::styled(if selected_row { "▶ " } else { "  " }, marker_style),
        Span::styled(key, key_style),
        Span::raw(" "),
        Span::styled(label, text_style),
        Span::raw(" "),
        Span::styled("[", Style::default().fg(Color::White)),
        Span::raw(" "),
        Span::styled(status, status_style),
        Span::raw(" "),
        Span::styled("]", Style::default().fg(Color::White)),
    ])
}

// 将文本截断到指定宽度并添加 ... 省略号。保持 Unicode 宽度感知
pub fn truncate_with_ellipsis_plain(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }
    if max_width <= 3 {
        return ".".repeat(max_width);
    }
    let mut result = String::new();
    for ch in text.chars() {
        let next = format!("{result}{ch}");
        if UnicodeWidthStr::width(next.as_str()) + 3 > max_width {
            break;
        }
        result.push(ch);
    }
    result.push_str("...");
    result
}

// 将富文本行从两端交替裁剪到指定宽度，保持样式信息。用于窄列显示 Banner 等
pub fn crop_line_center_to_width(line: &Line<'static>, width: usize) -> Line<'static> {
    if width == 0 {
        return Line::from("");
    }
    let mut cells = Vec::<(char, Style, usize)>::new();
    for span in &line.spans {
        for ch in span.content.chars() {
            let ch_width = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]));
            if ch_width == 0 {
                continue;
            }
            cells.push((ch, span.style, ch_width));
        }
    }
    let mut total_width: usize = cells.iter().map(|(_, _, w)| *w).sum();
    if total_width <= width {
        return line.clone();
    }
    let mut trim_left = true;
    while total_width > width && !cells.is_empty() {
        if trim_left {
            if let Some((_, _, w)) = cells.first().copied() {
                total_width = total_width.saturating_sub(w);
            }
            cells.remove(0);
        } else if let Some((_, _, w)) = cells.pop() {
            total_width = total_width.saturating_sub(w);
        }
        trim_left = !trim_left;
    }
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current_style: Option<Style> = None;
    let mut current_text = String::new();
    for (ch, style, _) in cells {
        match current_style {
            Some(existing) if existing == style => current_text.push(ch),
            Some(existing) => {
                spans.push(Span::styled(current_text.clone(), existing));
                current_text.clear();
                current_text.push(ch);
                current_style = Some(style);
            }
            None => {
                current_text.push(ch);
                current_style = Some(style);
            }
        }
    }
    if let Some(style) = current_style {
        spans.push(Span::styled(current_text, style));
    }
    Line::from(spans)
}

// 解析富文本并渲染到缓冲区中的指定位置
pub fn render_rich_line_to_buffer(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: usize,
    text: &str,
    base: Style,
) {
    let lines = rich_text::parse_rich_text_wrapped(text, width.max(1), base);
    let Some(first_line) = lines.first() else { return };
    let line = crop_line_center_to_width(first_line, width.max(1));
    let mut cursor_x = x;
    for span in &line.spans {
        let content = span.content.as_ref();
        if content.is_empty() {
            continue;
        }
        let remaining = width.saturating_sub(cursor_x.saturating_sub(x) as usize);
        if remaining == 0 {
            break;
        }
        buffer.set_stringn(cursor_x, y, content, remaining, span.style);
        cursor_x = cursor_x.saturating_add(UnicodeWidthStr::width(content) as u16);
        if cursor_x >= x.saturating_add(width as u16) {
            break;
        }
    }
}

// 将预编译的 Line 居中裁剪后渲染到缓冲区
pub fn render_compiled_line_to_buffer(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: usize,
    line: &Line<'static>,
) {
    let line = crop_line_center_to_width(line, width.max(1));
    let mut cursor_x = x;
    for span in &line.spans {
        let content = span.content.as_ref();
        if content.is_empty() {
            continue;
        }
        let remaining = width.saturating_sub(cursor_x.saturating_sub(x) as usize);
        if remaining == 0 {
            break;
        }
        buffer.set_stringn(cursor_x, y, content, remaining, span.style);
        cursor_x = cursor_x.saturating_add(UnicodeWidthStr::width(content) as u16);
        if cursor_x >= x.saturating_add(width as u16) {
            break;
        }
    }
}

// 根据是否允许富文本，选择富文本渲染或纯文本截断渲染
pub fn render_manifest_text_to_buffer(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: usize,
    text: &str,
    allow_rich: bool,
    base: Style,
) {
    if allow_rich && text.starts_with("f%") {
        render_rich_line_to_buffer(buffer, x, y, width, text, base);
        return;
    }
    let line = truncate_with_ellipsis_plain(text, width);
    buffer.set_stringn(x, y, line, width, base);
}

// 渲染"标签: 值"格式到缓冲区，支持富文本
pub fn render_label_manifest_value_to_buffer(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: usize,
    label: &str,
    value: &str,
    allow_rich: bool,
    base: Style,
) {
    let prefix = format!("{label} ");
    let prefix_width = UnicodeWidthStr::width(prefix.as_str());
    buffer.set_stringn(x, y, &prefix, width, base);
    let value_x = x.saturating_add(prefix_width as u16);
    let value_width = width.saturating_sub(prefix_width);
    render_manifest_text_to_buffer(buffer, value_x, y, value_width, value, allow_rich, base);
}

// 	构建 Mod 列表的标题栏，显示当前排序模式和升降序标记（↑/↓）
pub fn mod_list_title(state: &crate::app::settings::types::SettingsState) -> Line<'static> {
    let order_text = if state.mod_sort_descending {
        format!("\u{2191}{}", text("settings.mods.order.desc", "Descending"))
    } else {
        format!("\u{2193}{}", text("settings.mods.order.asc", "Ascending"))
    };
    Line::from(vec![
        Span::raw(" "),
        Span::styled(text("settings.mods.title", "Mods"), Style::default().fg(Color::White)),
        Span::styled(" *", Style::default().fg(Color::White)),
        Span::styled(
            mod_sort_label(state.mod_sort_mode),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default().fg(Color::White)),
        Span::styled("[", Style::default().fg(Color::White)),
        Span::styled(order_text, Style::default().fg(Color::DarkGray)),
        Span::styled("]", Style::default().fg(Color::White)),
        Span::raw(" "),
    ])
}

// 返回排序模式的本地化标签
pub fn mod_sort_label(mode: crate::app::settings::types::ModSortMode) -> String {
    use crate::app::settings::types::ModSortMode;
    match mode {
        ModSortMode::Name => text("settings.mods.sort.name", "Name"),
        ModSortMode::Enabled => text("settings.mods.sort.enabled", "Enabled"),
        ModSortMode::Author => text("settings.mods.sort.author", "Author"),
        ModSortMode::SafeMode => text("settings.mods.sort.safe_mode", "Safe Mode"),
    }
}

// 返回列表项高度：详细视图 5 行，简单视图 1 行
pub fn mod_item_height(list_view: crate::app::settings::types::ModListView) -> u16 {
    use crate::app::settings::types::ModListView;
    match list_view {
        ModListView::Detailed => 5,
        ModListView::Simple => 1,
    }
}

// 渲染 [D] 调试模式标记（蓝色）
pub fn render_mod_debug_prefix(buffer: &mut Buffer, x: u16, y: u16, enabled: bool, selected: bool) {
    if !enabled {
        return;
    }
    let bg = if selected { Color::DarkGray } else { Color::Reset };
    buffer.set_string(x, y, "[", Style::default().fg(Color::White).bg(bg));
    buffer.set_string(x + 1, y, "D", Style::default().fg(Color::LightBlue).bg(bg));
    buffer.set_string(x + 2, y, "]", Style::default().fg(Color::White).bg(bg));
}

// 渲染 [On]/[Off] 启用状态标签（绿/红）
pub fn render_enabled_tag(buffer: &mut Buffer, x: u16, y: u16, enabled: bool, selected: bool) {
    let bg = if selected { Color::DarkGray } else { Color::Reset };
    let value = if enabled {
        text("settings.mods.simple_enabled", "On")
    } else {
        text("settings.mods.simple_disabled", "Off")
    };
    let value_style = Style::default()
        .fg(if enabled { Color::Green } else { Color::Red })
        .bg(bg)
        .add_modifier(Modifier::BOLD);
    let bracket_style = Style::default().fg(Color::White).bg(bg);
    buffer.set_string(x, y, "[", bracket_style);
    buffer.set_string(x + 1, y, &value, value_style);
    let value_width = UnicodeWidthStr::width(value.as_str()) as u16;
    buffer.set_string(x + 1 + value_width, y, "]", bracket_style);
}

// 渲染 Mod 状态行（"State: Enabled/Disabled"）
pub fn render_mod_status_line(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    width: usize,
    package: &ModPackage,
    selected: bool,
) {
    let bg = if selected { Color::DarkGray } else { Color::Reset };
    let label = format!("{} ", text("settings.mods.state", "State:"));
    let label_style = Style::default().fg(Color::Gray).bg(bg);
    let value_style = Style::default()
        .fg(if package.enabled { Color::Green } else { Color::Red })
        .bg(bg)
        .add_modifier(Modifier::BOLD);
    buffer.set_stringn(x, y, &label, width, label_style);
    let value_x = x + UnicodeWidthStr::width(label.as_str()) as u16;
    buffer.set_stringn(
        value_x,
        y,
        if package.enabled {
            text("settings.mods.enabled", "Enabled")
        } else {
            text("settings.mods.disabled", "Disabled")
        },
        width.saturating_sub(UnicodeWidthStr::width(label.as_str())),
        value_style,
    );
}

// 渲染安全模式关闭的红色标记列（1 列宽）
pub fn render_safe_mode_marker_column(buffer: &mut Buffer, area: Rect, content_height: u16) {
    let marker_x = area.x + area.width - 1;
    let style = Style::default().bg(Color::Red);
    for dy in 0..content_height {
        buffer.set_string(marker_x, area.y + dy, " ", style);
    }
}

// 从 Mod 图像获取前 13 行预渲染行，并居中裁剪
pub fn rich_lines_from_image(image: &mods::ModImage, width: usize, base: Style) -> Vec<Line<'static>> {
    image
        .rendered_lines
        .iter()
        .take(13)
        .map(|line| center_or_crop_line_to_width(line, width, base))
        .collect()
}

// 居中或裁剪行，优先居中填充空格
pub fn center_or_crop_line_to_width(line: &Line<'static>, width: usize, base: Style) -> Line<'static> {
    let line_width = line_width(line);
    if width == 0 {
        return Line::from("");
    }
    if line_width > width {
        return crop_line_center_to_width(line, width);
    }
    if line_width == width {
        return line.clone();
    }
    let pad = width.saturating_sub(line_width);
    let left = pad / 2;
    let right = pad.saturating_sub(left);
    let mut spans = Vec::new();
    if left > 0 {
        spans.push(Span::styled(" ".repeat(left), base));
    }
    spans.extend(line.spans.iter().cloned());
    if right > 0 {
        spans.push(Span::styled(" ".repeat(right), base));
    }
    Line::from(spans)
}

// 计算 ratatui Line 的 Unicode 总宽度
pub fn line_width(line: &Line<'static>) -> usize {
    line.spans.iter().map(|span| UnicodeWidthStr::width(span.content.as_ref())).sum()
}

// 构建 Mod 管理页面的操作提示文本列表，可选包含滚动提示
pub fn build_mod_hint_segments(include_scroll: bool) -> Vec<String> {
    let mut segments = vec![
        text("settings.mods.hint.toggle", "[Enter] Toggle"),
        text("settings.mods.hint.debug", "[D] Debug"),
        text("settings.mods.hint.safe_mode", "[R] Safe Mode"),
        text("settings.mods.hint.hot_reload", "[H] Hot Reload"),
        text("settings.mods.hint.view", "[L] View"),
        text("settings.mods.hint.jump", "[P] Jump"),
        text("settings.mods.hint.sort_mode", "[Z] Sort"),
        text("settings.mods.hint.sort_order", "[X] Order"),
        text("settings.mods.hint.move", "[\u{2191}]/[\u{2193}] Move"),
        text("settings.mods.hint.page", "[Q]/[E] Page"),
        text("settings.hub.back_hint", "[ESC] Return to main menu"),
    ];
    if include_scroll {
        segments.push(text("settings.mods.hint.scroll", "[W]/[S] Scroll Details"));
    }
    segments
}

// 将提示片段按宽度自动换行为多行
pub fn wrap_mod_hint_lines(segments: &[String], width: usize) -> Vec<Line<'static>> {
    wrap_hint_lines_generic(segments, width)
}

// 同上，为按键绑定页面
pub fn wrap_keybind_hint_lines(segments: &[String], width: usize) -> Vec<Line<'static>> {
    wrap_hint_lines_generic(segments, width)
}

// 同上，为语言选择页面
pub fn wrap_language_hint_lines(segments: &[String], width: usize) -> Vec<Line<'static>> {
    wrap_hint_lines_generic(segments, width)
}

// 根据终端高度计算当前可显示的 Mod 列表项数量
fn wrap_hint_lines_generic(segments: &[String], width: usize) -> Vec<Line<'static>> {
    if width == 0 || segments.is_empty() {
        return vec![Line::from("")];
    }
    let mut lines = Vec::new();
    let mut current_segments: Vec<Span<'static>> = Vec::new();
    let mut current_width = 0usize;
    for segment in segments {
        let segment_width = UnicodeWidthStr::width(segment.as_str());
        let separator_width = if current_segments.is_empty() { 0 } else { 2 };
        if !current_segments.is_empty() && current_width + separator_width + segment_width > width {
            lines.push(Line::from(std::mem::take(&mut current_segments)));
            current_width = 0;
        }
        if !current_segments.is_empty() {
            current_segments.push(Span::raw("  "));
            current_width += 2;
        }
        current_segments.push(Span::raw(segment.clone()));
        current_width += segment_width;
    }
    if !current_segments.is_empty() {
        lines.push(Line::from(current_segments));
    }
    if lines.is_empty() {
        lines.push(Line::from(""));
    }
    lines
}

// 	计算键位列表的页面大小
pub fn current_mod_page_size(list_view: crate::app::settings::types::ModListView) -> usize {
    let (_, term_height) = crossterm::terminal::size().unwrap_or((90, 26));
    let root_height = term_height.saturating_sub(1);
    let inner_height = root_height.saturating_sub(2);
    let content_height = inner_height.saturating_sub(1);
    (content_height / mod_item_height(list_view)).max(1) as usize
}

// 计算总页数，防零除
pub fn total_mod_pages(total_items: usize, page_size: usize) -> usize {
    if total_items == 0 { 1 } else { ((total_items + page_size.saturating_sub(1)) / page_size).max(1) }
}

// 从内容缓存加载 Mod 包列表
pub fn load_mod_packages() -> Vec<ModPackage> {
    crate::app::content_cache::mods()
}

// 从内容缓存加载可用于键位绑定的游戏列表
pub fn load_keybind_games() -> Vec<GameDescriptor> {
    crate::app::content_cache::games()
}

// 构建键位页面的游戏列表标题
pub fn current_keybind_page_size() -> usize {
    let (_, height) = crossterm::terminal::size().unwrap_or((100, 24));
    height.saturating_sub(5).max(1) as usize
}

// 返回键位排序模式的本地化标签
pub fn total_keybind_pages(total_items: usize, page_size: usize) -> usize {
    if total_items == 0 { 1 } else { ((total_items + page_size.saturating_sub(1)) / page_size).max(1) }
}

// 渲染键位页面的游戏列表行，Mod 游戏显示 MOD 徽章
pub fn keybind_game_list_title(state: &crate::app::settings::types::SettingsState) -> Line<'static> {
    let order_text = if state.keybind_sort_descending {
        format!("\u{2191}{}", text("settings.mods.order.desc", "Descending"))
    } else {
        format!("\u{2193}{}", text("settings.mods.order.asc", "Ascending"))
    };
    Line::from(vec![
        Span::raw(" "),
        Span::styled(text("settings.keybind.games_title", "Game Selection"), Style::default().fg(Color::White)),
        Span::styled(" *", Style::default().fg(Color::White)),
        Span::styled(
            keybind_game_sort_label(state.keybind_sort_mode),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default().fg(Color::White)),
        Span::styled("[", Style::default().fg(Color::White)),
        Span::styled(order_text, Style::default().fg(Color::DarkGray)),
        Span::styled("]", Style::default().fg(Color::White)),
        Span::raw(" "),
    ])
}

// 根据终端宽度和语言包列表计算网格布局参数（列数、内宽、外宽）
pub fn keybind_game_sort_label(mode: crate::app::settings::types::KeybindGameSortMode) -> String {
    use crate::app::settings::types::KeybindGameSortMode;
    match mode {
        KeybindGameSortMode::Source => text("game_selection.sort.source", "Official & Mods"),
        KeybindGameSortMode::Name => text("game_selection.sort.name", "Name"),
        KeybindGameSortMode::Author => text("game_selection.sort.author", "Author"),
    }
}

// 在二维网格中处理方向键移动，计算新的选中索引
pub fn keybind_game_list_line(game: &GameDescriptor, width: usize, style: Style) -> Line<'static> {
    if width == 0 {
        return Line::from("");
    }
    let name = game.display_name.clone();
    if !game.is_mod_game() {
        return Line::from(Span::styled(truncate_with_ellipsis_plain(&name, width), style));
    }
    let badge = text("mods.badge", "MOD");
    let badge_width = UnicodeWidthStr::width(badge.as_str());
    if width <= badge_width + 1 {
        return Line::from(Span::styled(truncate_with_ellipsis_plain(&name, width), style));
    }
    let left_width = width - badge_width - 1;
    let left = truncate_with_ellipsis_plain(&name, left_width);
    let pad = width.saturating_sub(UnicodeWidthStr::width(left.as_str()) + badge_width);
    let badge_fg = if matches!(style.bg, Some(Color::LightBlue)) {
        Color::Black
    } else {
        Color::Yellow
    };
    Line::from(vec![
        Span::styled(left, style),
        Span::styled(" ".repeat(pad), style),
        Span::styled(
            badge,
            Style::default()
                .fg(badge_fg)
                .bg(style.bg.unwrap_or(Color::Reset))
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

// 根据终端宽度和语言包列表计算网格布局参数（列数、内宽、外宽）
pub fn grid_metrics(term_width: u16, languages: &[i18n::LanguagePack]) -> GridMetrics {
    use crate::app::settings::types::GridMetrics;
    if languages.is_empty() {
        return GridMetrics { cols: 1, inner_width: 6, outer_width: 8 };
    }
    let max_name_width = languages.iter().map(|pack| UnicodeWidthStr::width(pack.name.as_str())).max().unwrap_or(4);
    let inner_width = (max_name_width + 2) as u16;
    let outer_width = inner_width + 2;
    let cols_by_width = (((term_width as usize) + H_GAP as usize) / (outer_width as usize + H_GAP as usize)).max(1);
    let cols = languages.len().min(MAX_COLS).min(cols_by_width).max(1);
    GridMetrics { cols, inner_width, outer_width }
}

// 在二维网格中处理方向键移动，计算新的选中索引
pub fn move_selection(selected: usize, key: crossterm::event::KeyCode, metrics: GridMetrics, total: usize) -> usize {
    if total == 0 { return 0; }
    let cols = metrics.cols.max(1);
    let row = selected / cols;
    let col = selected % cols;
    match key {
        crossterm::event::KeyCode::Left => if col > 0 { selected - 1 } else { selected },
        crossterm::event::KeyCode::Right => if col + 1 < cols && selected + 1 < total { selected + 1 } else { selected },
        crossterm::event::KeyCode::Up => if row > 0 { selected.saturating_sub(cols) } else { selected },
        crossterm::event::KeyCode::Down => if selected + cols < total { selected + cols } else { selected },
        _ => selected,
    }
}

// 渲染一个居中的带边框设置框，标题格式为 ── title 
pub fn render_settings_box(
    frame: &mut ratatui::Frame<'_>,
    title: String,
    min_content_width: u16,
    lines: Vec<Line<'static>>,
) -> Rect {
    let area = frame.area();
    let inner_width = lines
        .iter()
        .map(|line| line.width() as u16)
        .max()
        .unwrap_or(1)
        .max(min_content_width)
        .max(1);
    let width = (inner_width + 2).min(area.width.max(1));
    let height = ((lines.len() as u16) + 2).min(area.height.max(1));
    let render_area = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(Span::styled(
                    format!("── {} ", title),
                    Style::default().fg(Color::White),
                )))
                .border_style(Style::default().fg(Color::White)),
        ),
        render_area,
    );
    render_area
}

// 在设置框下方渲染绿色粗体成功提示
pub fn render_box_success_hint(frame: &mut ratatui::Frame<'_>, rect: Rect, message: String) {
    let y = rect.y.saturating_add(rect.height);
    if y >= frame.area().y.saturating_add(frame.area().height) {
        return;
    }
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            message,
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center),
        Rect::new(rect.x, y, rect.width, 1),
    );
}

// 在设置框下方渲染深灰色返回提示
pub fn render_box_back_hint(frame: &mut ratatui::Frame<'_>, rect: Rect, message: String) {
    let y = rect.y.saturating_add(rect.height).saturating_add(2);
    if y >= frame.area().y.saturating_add(frame.area().height) {
        return;
    }
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            message,
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(Alignment::Center),
        Rect::new(rect.x, y, rect.width, 1),
    );
}

// 在指定偏移位置渲染提示行
pub fn render_box_hint_line(frame: &mut ratatui::Frame<'_>, rect: Rect, offset: u16, message: String) {
    let y = rect.y.saturating_add(rect.height).saturating_add(offset);
    if y >= frame.area().y.saturating_add(frame.area().height) {
        return;
    }
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            message,
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(Alignment::Center),
        Rect::new(rect.x, y, rect.width, 1),
    );
}

// 将纯文本按宽度自动换行，在词边界处（空格）优先断行
pub fn wrap_plain_text_lines(text: &str, width: usize, style: Style) -> Vec<Line<'static>> {
    let width = width.max(1);
    let mut out = Vec::new();
    for raw_line in text.lines() {
        let mut remaining = raw_line.trim_end().to_string();
        if remaining.is_empty() {
            out.push(Line::from(""));
            continue;
        }
        while !remaining.is_empty() {
            if UnicodeWidthStr::width(remaining.as_str()) <= width {
                out.push(Line::from(Span::styled(remaining.clone(), style)));
                break;
            }
            let mut cur = String::new();
            let mut cur_width = 0usize;
            let mut last_space_byte = None;
            for (idx, ch) in remaining.char_indices() {
                let ch_width = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]));
                if cur_width + ch_width > width { break; }
                cur.push(ch);
                cur_width += ch_width;
                if ch.is_whitespace() { last_space_byte = Some(idx); }
            }
            if cur.is_empty() {
                if let Some(ch) = remaining.chars().next() {
                    out.push(Line::from(Span::styled(ch.to_string(), style)));
                    remaining = remaining[ch.len_utf8()..].trim_start().to_string();
                }
                continue;
            }
            if let Some(space_idx) = last_space_byte {
                let head = remaining[..space_idx].trim_end().to_string();
                if !head.is_empty() { out.push(Line::from(Span::styled(head, style))); }
                remaining = remaining[space_idx + 1..].trim_start().to_string();
            } else {
                out.push(Line::from(Span::styled(cur.clone(), style)));
                remaining = remaining[cur.len()..].trim_start().to_string();
            }
        }
    }
    if out.is_empty() { out.push(Line::from("")); }
    out
}