//! Width-aware Markdown rows shared by measurement, rendering and text conversion.

use super::*;
use std::collections::HashMap;

/// Laid-out Markdown rows with exact link rectangles and measured content dimensions.
#[derive(Clone, Debug, Default)]
pub struct MarkdownLayout {
  rows: Vec<Row>,
  size: Size,
}
#[derive(Clone, Debug, Default)]
struct Row {
  segments: Vec<RichTextSegment>,
  links: Vec<(u16, u16, String, String)>,
}
impl MarkdownLayout {
  /// Return the complete content size in terminal cells.
  pub fn size(&self) -> Size {
    self.size
  }
  fn push(&mut self, row: Row) {
    if self.rows.len() >= u16::MAX as usize {
      return;
    }
    self.size.width = self.size.width.max(
      row
        .segments
        .iter()
        .map(|segment| display_width(&segment.text))
        .sum::<usize>()
        .min(u16::MAX as usize) as u16,
    );
    self.rows.push(row);
    self.size.height = self.rows.len() as u16;
  }
}
impl MarkdownService {
  /// Lay out Markdown with optional horizontal overflow and continuation indentation.
  ///
  /// # Arguments
  ///
  /// * `options` - The document text and styles.
  /// * `width` - The content viewport width in cells.
  /// * `allow_overflow` - Whether to preserve natural line widths instead of soft wrapping.
  /// * `code` - The syntax highlighting service.
  pub fn layout(
    &self,
    options: &MarkdownViewOptions,
    width: u16,
    allow_overflow: bool,
    code: &CodeHighlightService,
  ) -> MarkdownLayout {
    let mut result = MarkdownLayout::default();
    if width == 0 {
      return result;
    }
    for block in parse_markdown(options, code) {
      match block {
        MdBlock::Text {
          segments,
          links,
          continuation,
        } => text_rows(
          &mut result,
          &segments,
          &links,
          &continuation,
          width,
          allow_overflow,
        ),
        MdBlock::Blank => result.push(Row::default()),
        MdBlock::Rule => result.push(row(
          "─".repeat(width as usize),
          &options.theme.horizontal_rule,
        )),
        MdBlock::Code {
          language,
          code: source,
        } => {
          let mut content = MarkdownLayout::default();
          let inner = width.saturating_sub(2).max(1);
          for line in source.trim_end_matches('\n').split('\n') {
            let expanded = line.replace('\t', "    ");
            let line = expanded.as_str();
            let segments = language
              .as_deref()
              .and_then(|name| code.language_from_name(name))
              .map(|language| code.highlight_segments(line, language, &options.code_theme))
              .unwrap_or_else(|| {
                vec![RichTextSegment {
                  text: line.into(),
                  style: options.theme.code_block.clone(),
                }]
              });
            let indent = line
              .chars()
              .take_while(|character| character.is_whitespace())
              .map(|character| {
                if character == '\t' {
                  "    ".to_string()
                } else {
                  character.to_string()
                }
              })
              .collect::<String>();
            text_rows(
              &mut content,
              &segments,
              &[],
              &[RichTextSegment {
                text: indent,
                style: options.theme.code_block.clone(),
              }],
              inner,
              allow_overflow,
            );
          }
          let columns = content
            .size
            .width
            .max(if allow_overflow { 1 } else { inner });
          let label = language.as_deref().unwrap_or("");
          let top = if !label.is_empty() && display_width(label) + 2 <= columns as usize {
            format!(
              "┌─ {}{}┐",
              label,
              "─".repeat(columns as usize - display_width(label) - 2)
            )
          } else {
            format!("┌{}┐", "─".repeat(columns as usize))
          };
          result.push(row(top, &options.theme.code_border));
          for mut line in content.rows {
            let line_width = line
              .segments
              .iter()
              .map(|segment| display_width(&segment.text))
              .sum::<usize>();
            line
              .segments
              .insert(0, segment("│", &options.theme.code_border));
            line.segments.push(segment(
              &" ".repeat((columns as usize).saturating_sub(line_width)),
              &options.theme.code_block,
            ));
            line.segments.push(segment("│", &options.theme.code_border));
            result.push(line);
          }
          result.push(row(
            format!("└{}┘", "─".repeat(columns as usize)),
            &options.theme.code_border,
          ));
        }
        MdBlock::Table { rows, aligns } => {
          let count = rows.iter().map(Vec::len).max().unwrap_or(0);
          if count == 0 {
            continue;
          }
          let widths = if allow_overflow {
            (0..count)
              .map(|column| {
                rows
                  .iter()
                  .filter_map(|row| row.get(column))
                  .map(|text| visible_width(text))
                  .max()
                  .unwrap_or(1)
                  .max(1)
              })
              .collect()
          } else {
            markdown_table_widths(&rows, width, count)
          };
          result.push(row(
            table_border(&widths, '┌', '┬', '┐'),
            &options.theme.table_border,
          ));
          for (index, cells) in rows.iter().enumerate() {
            let style = if index == 0 {
              &options.theme.table_header
            } else {
              &options.theme.paragraph
            };
            let laid = cells
              .iter()
              .enumerate()
              .map(|(column, text)| {
                let mut lines = MarkdownLayout::default();
                text_rows(
                  &mut lines,
                  &[segment(text, style)],
                  &[],
                  &[],
                  widths[column],
                  allow_overflow,
                );
                lines.rows
              })
              .collect::<Vec<_>>();
            let height = laid.iter().map(Vec::len).max().unwrap_or(1).max(1);
            for line in 0..height {
              let mut output = row("│".into(), &options.theme.table_border);
              for (column, width) in widths.iter().enumerate() {
                let content = laid
                  .get(column)
                  .and_then(|lines| lines.get(line))
                  .cloned()
                  .unwrap_or_default();
                let used = content
                  .segments
                  .iter()
                  .map(|segment| display_width(&segment.text))
                  .sum::<usize>();
                let padding = (*width as usize).saturating_sub(used);
                let left = match aligns.get(column) {
                  Some(TextAlign::Right) => padding,
                  Some(TextAlign::Center) => padding / 2,
                  _ => 0,
                };
                output.segments.push(segment(&" ".repeat(left), style));
                output.segments.extend(content.segments);
                output
                  .segments
                  .push(segment(&" ".repeat(padding - left), style));
                output
                  .segments
                  .push(segment("│", &options.theme.table_border));
              }
              result.push(output);
            }
            result.push(row(
              if index + 1 == rows.len() {
                table_border(&widths, '└', '┴', '┘')
              } else {
                table_border(&widths, '├', '┼', '┤')
              },
              &options.theme.table_border,
            ));
          }
        }
      }
    }
    if !allow_overflow {
      result.size.width = result.size.width.min(width);
    }
    result
  }

  /// Render laid-out Markdown inside a clipped content slice and register its links.
  ///
  /// # Arguments
  ///
  /// * `pool` - The owning widget pool.
  /// * `id` - The document identity.
  /// * `slice` - The prepared content surface.
  /// * `layout` - The measured rows to replay.
  /// * `canvas` - The destination canvas.
  pub fn render_layout_on(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    slice: SliceId,
    layout: &MarkdownLayout,
    canvas: &mut CanvasService,
  ) -> bool {
    render_layout(
      pool,
      id,
      MarkdownTarget::Slice(slice),
      layout,
      MarkdownRenderParams {
        x: 0,
        y: 0,
        width: layout.size.width.max(1),
        max_height: None,
      },
      canvas,
    )
  }

  /// Convert Markdown to tagged rich text without applying a viewport layout or interactions.
  pub fn markdown_to_rich_text(&self, markdown: &str) -> String {
    let options = MarkdownViewOptions::new(markdown);
    let code = CodeHighlightService::new();
    let mut output = String::from("f%");
    for block in parse_markdown(&options, &code) {
      match block {
        MdBlock::Text { segments, .. } => {
          for part in segments {
            append_rich(&mut output, &part);
          }
          output.push('\n');
        }
        MdBlock::Blank => output.push('\n'),
        MdBlock::Rule => {
          append_rich(&mut output, &segment("───", &options.theme.horizontal_rule));
          output.push('\n');
        }
        MdBlock::Code {
          language,
          code: source,
        } => {
          for line in source.trim_end_matches('\n').split('\n') {
            let parts = language
              .as_deref()
              .and_then(|name| code.language_from_name(name))
              .map(|language| code.highlight_segments(line, language, &options.code_theme))
              .unwrap_or_else(|| vec![segment(line, &options.theme.code_block)]);
            for part in parts {
              append_rich(&mut output, &part);
            }
            output.push('\n');
          }
        }
        MdBlock::Table { rows, .. } => {
          for (index, cells) in rows.iter().enumerate() {
            append_rich(
              &mut output,
              &segment(
                &cells.join(" | "),
                if index == 0 {
                  &options.theme.table_header
                } else {
                  &options.theme.paragraph
                },
              ),
            );
            output.push('\n');
          }
        }
      }
    }
    output.push_str("<reset>");
    output
  }
}

pub(super) fn render_layout(
  pool: &mut UiObjectPool,
  id: MarkdownViewId,
  target: MarkdownTarget,
  layout: &MarkdownLayout,
  params: MarkdownRenderParams,
  canvas: &mut CanvasService,
) -> bool {
  let Some(state) = pool.markdown_views.views.get_mut(&id) else {
    return false;
  };
  state.hits.clear();
  for (index, row) in layout.rows.iter().enumerate() {
    let y = params.y.saturating_add(index as u16);
    if params
      .max_height
      .is_some_and(|height| index >= height as usize)
    {
      break;
    }
    let hit = match target {
      MarkdownTarget::Base => canvas.base_hit_rect(Rect {
        x: params.x,
        y,
        width: params.width,
        height: 1,
      }),
      MarkdownTarget::Slice(slice) => canvas.slice_hit_rect(
        slice,
        Rect {
          x: params.x,
          y,
          width: params.width,
          height: 1,
        },
      ),
      MarkdownTarget::ScrollBox(scroll) => canvas.scroll_box_hit_rect(
        scroll,
        Rect {
          x: params.x,
          y,
          width: params.width,
          height: 1,
        },
      ),
    };
    if hit.is_none() {
      continue;
    }
    draw_segments(
      canvas,
      target,
      &row.segments,
      &DrawTextParams {
        x: params.x,
        y,
        wrap_mode: TextWrapMode::None,
        max_width: Some(params.width),
        ..Default::default()
      },
    );
    for (start, width, href, text) in &row.links {
      register_links(
        pool,
        id,
        &[LinkSpan {
          start: *start as usize,
          width: *width as usize,
          href: href.clone(),
          text: text.clone(),
        }],
        params.x,
        y,
        target,
        canvas,
      );
    }
  }
  true
}
fn row(text: String, style: &TextStyle) -> Row {
  Row {
    segments: vec![segment(&text, style)],
    links: Vec::new(),
  }
}
fn segment(text: &str, style: &TextStyle) -> RichTextSegment {
  RichTextSegment {
    text: text.into(),
    style: style.clone(),
  }
}
fn text_rows(
  output: &mut MarkdownLayout,
  segments: &[RichTextSegment],
  links: &[LinkSpan],
  continuation: &[RichTextSegment],
  width: u16,
  overflow: bool,
) {
  let mut source_links = HashMap::new();
  let mut byte = 0;
  let mut column = 0;
  for part in segments {
    for grapheme in tg_core_unicode::graphemes(&part.text) {
      if let Some(index) = links
        .iter()
        .position(|link| column >= link.start && column < link.start + link.width)
      {
        source_links.insert(byte, index);
      }
      byte += grapheme.text.len();
      column += grapheme.display_width;
    }
  }
  let prefix_width = continuation
    .iter()
    .map(|part| display_width(&part.text))
    .sum::<usize>()
    .min(width.saturating_sub(1) as usize);
  let mut first_prefix = Vec::new();
  let mut body = segments.to_vec();
  let mut remaining = prefix_width;
  for part in &mut body {
    if remaining == 0 {
      break;
    }
    let mut bytes = 0;
    for grapheme in tg_core_unicode::graphemes(&part.text) {
      if grapheme.display_width > remaining {
        break;
      }
      bytes += grapheme.text.len();
      remaining -= grapheme.display_width;
      if remaining == 0 {
        break;
      }
    }
    first_prefix.push(segment(&part.text[..bytes], &part.style));
    part.text = part.text[bytes..].into();
  }
  let prefix_bytes = first_prefix
    .iter()
    .map(|part| part.text.len())
    .sum::<usize>();
  let params = DrawTextParams {
    max_width: if overflow {
      None
    } else {
      Some(width.saturating_sub(prefix_width as u16).max(1))
    },
    wrap_mode: TextWrapMode::Auto,
    non_truncate_word_wrap: true,
    ..Default::default()
  };
  for (index, line) in text_layout::layout_rich_text_segments(&body, &params)
    .into_iter()
    .enumerate()
  {
    let prefix = if index == 0 {
      &first_prefix
    } else {
      continuation
    };
    let mut row = Row {
      segments: prefix.to_vec(),
      links: Vec::new(),
    };
    let mut x = prefix
      .iter()
      .map(|part| display_width(&part.text))
      .sum::<usize>();
    for item in line.items {
      if let Some(link) = item
        .source_offset
        .and_then(|offset| source_links.get(&(offset + prefix_bytes)))
        .map(|index| &links[*index])
      {
        if let Some(previous) = row.links.last_mut().filter(|previous| {
          previous.0 as usize + previous.1 as usize == x
            && previous.2 == link.href
            && previous.3 == link.text
        }) {
          previous.1 = previous.1.saturating_add(item.width as u16);
        } else {
          row.links.push((
            x.min(u16::MAX as usize) as u16,
            item.width as u16,
            link.href.clone(),
            link.text.clone(),
          ));
        }
      }
      x += item.width;
      row.segments.push(RichTextSegment {
        text: item.text,
        style: item.style,
      });
    }
    output.push(row);
  }
}
fn append_rich(output: &mut String, part: &RichTextSegment) {
  output.push_str("<reset>");
  for (name, color) in [
    ("fg", &part.style.foreground),
    ("bg", &part.style.background),
  ] {
    if let Some(TextColor::Rgb { r, g, b } | TextColor::ForceRgb { r, g, b }) = color {
      output.push_str(&format!("<{name}:#{r:02X}{g:02X}{b:02X}>"));
    }
  }
  for (name, enabled) in [
    ("bold", part.style.bold),
    ("italic", part.style.italic),
    ("underline", part.style.underline),
    ("strike", part.style.strike),
    ("blink", part.style.blink),
    ("reverse", part.style.reverse),
    ("hidden", part.style.hidden),
    ("dim", part.style.dim),
  ] {
    if enabled {
      output.push_str(&format!("<{name}>"));
    }
  }
  for character in part.text.chars() {
    if matches!(character, '<' | '>' | '{' | '}' | '\\') {
      output.push('\\');
    }
    output.push(character);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  fn text(row: &Row) -> String {
    row.segments.iter().map(|part| part.text.as_str()).collect()
  }
  #[test]
  fn continuation_prefixes_and_link_rows_match_layout() {
    let service = MarkdownService::new();
    let code = CodeHighlightService::new();
    let options=MarkdownViewOptions::new("- [a very long link label](https://example.com)\n\n> quoted words that wrap\n\n```lua\n    print(123456789)\n```".replace("\\n","\n"));
    let layout = service.layout(&options, 12, false, &code);
    assert!(
      layout
        .rows
        .iter()
        .any(|row| !row.links.is_empty() && text(row).starts_with("  "))
    );
    assert!(
      layout
        .rows
        .iter()
        .filter(|row| text(row).starts_with("▌ "))
        .count()
        > 1
    );
    assert!(
      layout
        .rows
        .iter()
        .filter(|row| text(row).starts_with("│    "))
        .count()
        > 1
    );
    assert!(layout.rows.iter().all(|row| {
      row
        .links
        .iter()
        .all(|(start, width, _, _)| *start + *width <= 12)
    }));
    assert!(service.layout(&options, 12, true, &code).size().width > 12);
  }
  #[test]
  fn rich_conversion_escapes_markup_and_leaves_text_unwrapped() {
    let rich = MarkdownService::new().markdown_to_rich_text("**bold** `<fg:red>{value}` \\ path");
    let mut canvas = CanvasService::new();
    canvas.text(&DrawTextParams::new(0, 0, rich));
    assert_eq!(canvas.cell_at(0, 0).unwrap().text, "b");
    assert!(canvas.cell_at(0, 0).unwrap().style.bold);
    let text = (0..40)
      .filter_map(|x| canvas.cell_at(x, 0))
      .map(|cell| cell.text.as_str())
      .collect::<String>();
    assert!(text.contains("<fg:red>{value}"), "rendered: {text:?}");
  }
}
