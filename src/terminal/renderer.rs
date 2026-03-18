use std::io::{Write, stdout};

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// 清理整个终端并把光标放到0，0的位置
pub fn clear() -> Result<()> {
    // 获取标准输出的笔(应该叫做句柄,但是我看不懂就写成笔了)
    let mut out = stdout();

    // 将命令加入队列
    // 清空并移动光标至0,0
    queue!(out, Clear(ClearType::All), MoveTo(0, 0))?;

    // 刷新输出,真正执行命令
    out.flush()?;
    Ok(())
}

// 在终端指定位置绘制内容
// 绝对坐标
pub fn draw_text(x: u16, y: u16, text: &str) -> Result<()> {
    let mut out = stdout();

    // 移动光标到x,y并打印文本
    queue!(out, MoveTo(x, y), Print(text))?;

    out.flush()?;
    Ok(())
}

// 根据文本宽度自动换行
// 会保留单词完整性避免跨单词换行
// 用到了unicode_width库
pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    // 如果最大宽度位0,返回空字符串
    if max_width == 0 {
        return vec![String::new()];
    }

    let mut lines = Vec::new();

    // 处理每一行(保留原始换行)
    for raw_line in text.lines() {
        // 如果整行宽度小于最大宽度,直接保留
        if UnicodeWidthStr::width(raw_line) <= max_width {
            lines.push(raw_line.to_string());
            continue;
        }

        // 当前正在构建的行
        let mut current = String::new();

        // 当前行的显示宽度
        let mut width = 0;

        // 遍历每个字符
        for ch in raw_line.chars() {
            // 获取字符的显示宽度(这个库汉字=2,字母=1)
            let w = UnicodeWidthChar::width(ch).unwrap_or(0);

            // 如果加上这个字符回超出宽度,且当行不为空
            if width + w > max_width && !current.is_empty() {
                lines.push(current.clone()); // 保存当前行
                current.clear(); // 开始新行
                width = 0;
            }

            // 添加字符到当前行
            current.push(ch);
            width += w;
        }

        // 添加最后一行
        if !current.is_empty() {
            lines.push(current);
        }
    }

    // 确保至少有一行
    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}
