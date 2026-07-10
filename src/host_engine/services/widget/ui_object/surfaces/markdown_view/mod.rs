mod state;
mod types;

pub(crate) use self::state::MarkdownViewObjects;
use self::state::{MarkdownLinkHit, MarkdownViewState};
pub use self::types::{
  MarkdownEvent, MarkdownRenderParams, MarkdownTheme, MarkdownViewId, MarkdownViewOptions,
};
use crate::host_engine::services::text_layout::{self, DrawTextParams, TextWrapMode};
use crate::host_engine::services::ui::UiObjectPool;
use crate::host_engine::services::unicode::display_width;
use crate::host_engine::services::{
  CanvasService, CodeHighlightService, MouseButton, MouseEvent, MouseEventKind, Rect,
  RichTextSegment, ScrollBoxId, Size, SliceId, TextAlign, TextStyle,
};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

pub struct MarkdownService;

#[derive(Clone, Copy)]
enum MarkdownTarget {
  Base,
  Slice(SliceId),
  ScrollBox(ScrollBoxId),
  Host,
}

#[derive(Clone, Debug)]
enum MdBlock {
  Text {
    segments: Vec<RichTextSegment>,
    links: Vec<LinkSpan>,
  },
  Code {
    language: Option<String>,
    code: String,
  },
  Rule,
  Table {
    rows: Vec<Vec<String>>,
  },
  Blank,
}

#[derive(Clone, Debug)]
struct LinkSpan {
  start: usize,
  width: usize,
  href: String,
  text: String,
}

#[derive(Clone)]
struct InlineState {
  style_stack: Vec<TextStyle>,
  current: TextStyle,
  segments: Vec<RichTextSegment>,
  links: Vec<LinkSpan>,
  active_link: Option<ActiveLink>,
}

#[derive(Clone)]
struct ActiveLink {
  href: String,
  start: usize,
  text: String,
}

struct ListState {
  ordered: bool,
  next: u64,
}

impl MarkdownService {
  pub fn new() -> Self {
    Self
  }

  pub fn create(
    &self,
    pool: &mut UiObjectPool,
    options: MarkdownViewOptions,
  ) -> Option<MarkdownViewId> {
    let id = MarkdownViewId(pool.markdown_views.next_id);
    pool.markdown_views.next_id += 1;
    pool.markdown_views.views.insert(
      id,
      MarkdownViewState {
        options,
        hits: Vec::new(),
        pressed: None,
      },
    );
    Some(id)
  }

  pub fn remove(&self, pool: &mut UiObjectPool, id: MarkdownViewId) -> bool {
    if pool.markdown_views.views.remove(&id).is_none() {
      return false;
    }
    pool.events.retain(|event| event.markdown_id() != Some(id));
    true
  }

  pub fn exists(&self, pool: &UiObjectPool, id: MarkdownViewId) -> bool {
    pool.markdown_views.views.contains_key(&id)
  }

  pub fn markdown<'a>(&self, pool: &'a UiObjectPool, id: MarkdownViewId) -> Option<&'a str> {
    Some(&pool.markdown_views.views.get(&id)?.options.markdown)
  }

  pub fn set_markdown(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    markdown: String,
  ) -> bool {
    let Some(state) = pool.markdown_views.views.get_mut(&id) else {
      return false;
    };
    state.options.markdown = markdown;
    true
  }

  pub fn set_theme(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    theme: MarkdownTheme,
  ) -> bool {
    let Some(state) = pool.markdown_views.views.get_mut(&id) else {
      return false;
    };
    state.options.theme = theme;
    true
  }

  pub fn set_code_theme(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    theme: crate::host_engine::services::CodeHighlightTheme,
  ) -> bool {
    let Some(state) = pool.markdown_views.views.get_mut(&id) else {
      return false;
    };
    state.options.code_theme = theme;
    true
  }

  pub fn measure(
    &self,
    pool: &UiObjectPool,
    id: MarkdownViewId,
    width: u16,
    code_highlight: &CodeHighlightService,
  ) -> Option<Size> {
    let state = pool.markdown_views.views.get(&id)?;
    let blocks = parse_markdown(&state.options, code_highlight);
    Some(Size {
      width,
      height: measure_blocks(&blocks, &state.options, width, code_highlight),
    })
  }

  pub fn render(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    params: MarkdownRenderParams,
    canvas: &mut CanvasService,
    code_highlight: &CodeHighlightService,
  ) -> bool {
    self.render_to(
      pool,
      id,
      params,
      MarkdownTarget::Base,
      canvas,
      code_highlight,
    )
  }

  pub fn render_on(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    slice: SliceId,
    params: MarkdownRenderParams,
    canvas: &mut CanvasService,
    code_highlight: &CodeHighlightService,
  ) -> bool {
    self.render_to(
      pool,
      id,
      params,
      MarkdownTarget::Slice(slice),
      canvas,
      code_highlight,
    )
  }

  pub fn render_in_scroll_box(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    scroll_box: ScrollBoxId,
    params: MarkdownRenderParams,
    canvas: &mut CanvasService,
    code_highlight: &CodeHighlightService,
  ) -> bool {
    self.render_to(
      pool,
      id,
      params,
      MarkdownTarget::ScrollBox(scroll_box),
      canvas,
      code_highlight,
    )
  }

  pub(crate) fn render_host(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    params: MarkdownRenderParams,
    canvas: &mut CanvasService,
    code_highlight: &CodeHighlightService,
  ) -> bool {
    self.render_to(
      pool,
      id,
      params,
      MarkdownTarget::Host,
      canvas,
      code_highlight,
    )
  }

  fn render_to(
    &self,
    pool: &mut UiObjectPool,
    id: MarkdownViewId,
    params: MarkdownRenderParams,
    target: MarkdownTarget,
    canvas: &mut CanvasService,
    code_highlight: &CodeHighlightService,
  ) -> bool {
    if params.width == 0 {
      return false;
    }
    let Some(options) = pool
      .markdown_views
      .views
      .get(&id)
      .map(|state| state.options.clone())
    else {
      return false;
    };
    let blocks = parse_markdown(&options, code_highlight);
    if let Some(state) = pool.markdown_views.views.get_mut(&id) {
      state.hits.clear();
      state.pressed = None;
    }
    let mut y = params.y;
    let bottom = params
      .max_height
      .map(|height| params.y.saturating_add(height));
    for block in blocks {
      if bottom.is_some_and(|bottom| y >= bottom) {
        break;
      }
      y = draw_block(
        pool,
        id,
        &block,
        &options,
        params.x,
        y,
        params.width,
        target,
        canvas,
        code_highlight,
      );
    }
    true
  }

  pub(crate) fn route_mouse_event(
    &self,
    pool: &mut UiObjectPool,
    text_input: &crate::host_engine::services::TextInputService,
    event: MouseEvent,
  ) -> bool {
    if !matches!(event.kind, MouseEventKind::Press | MouseEventKind::Release)
      || event.button != Some(MouseButton::Left)
    {
      return false;
    }
    let Some((id, link_index, order)) = markdown_hit(pool, event.x, event.y) else {
      if event.kind == MouseEventKind::Release {
        for state in pool.markdown_views.views.values_mut() {
          state.pressed = None;
        }
      }
      return false;
    };
    if pool
      .hit_areas
      .hit(event.x, event.y)
      .is_some_and(|(_, hit_order)| hit_order > order)
      || pool
        .hyperlinks
        .hit(event.x, event.y)
        .is_some_and(|(_, hyperlink_order)| hyperlink_order > order)
      || text_input
        .mouse_hit_order(pool, event.x, event.y)
        .is_some_and(|text_order| text_order > order)
    {
      return false;
    }
    match event.kind {
      MouseEventKind::Press => {
        if let Some(state) = pool.markdown_views.views.get_mut(&id) {
          state.pressed = Some(link_index);
        }
        true
      }
      MouseEventKind::Release => {
        let Some(state) = pool.markdown_views.views.get_mut(&id) else {
          return false;
        };
        let clicked = state.pressed.take() == Some(link_index);
        if clicked {
          let hit = state.hits[link_index].clone();
          pool.push_markdown_event(MarkdownEvent::LinkClicked {
            id,
            href: hit.href,
            text: hit.text,
          });
        }
        true
      }
      _ => false,
    }
  }
}

fn parse_markdown(
  options: &MarkdownViewOptions,
  _code_highlight: &CodeHighlightService,
) -> Vec<MdBlock> {
  let mut parser_options = Options::empty();
  parser_options.insert(Options::ENABLE_TABLES);
  parser_options.insert(Options::ENABLE_TASKLISTS);
  parser_options.insert(Options::ENABLE_STRIKETHROUGH);
  let parser = Parser::new_ext(&options.markdown, parser_options);

  let mut blocks = Vec::new();
  let mut inline: Option<InlineState> = None;
  let mut list_stack: Vec<ListState> = Vec::new();
  let mut quote_depth = 0usize;
  let mut code: Option<(Option<String>, String)> = None;
  let mut table_rows: Option<Vec<Vec<String>>> = None;
  let mut table_row: Option<Vec<String>> = None;
  let mut table_cell = String::new();
  let mut in_table_cell = false;
  let mut ignore_image_depth = 0usize;

  for event in parser {
    if let Some((_, code_text)) = code.as_mut() {
      match event {
        Event::End(TagEnd::CodeBlock) => {
          let (language, code_text) = code.take().unwrap();
          blocks.push(MdBlock::Code {
            language,
            code: code_text,
          });
          blocks.push(MdBlock::Blank);
        }
        Event::Text(text) | Event::Code(text) | Event::Html(text) | Event::InlineHtml(text) => {
          code_text.push_str(&text)
        }
        Event::SoftBreak | Event::HardBreak => code_text.push('\n'),
        _ => {}
      }
      continue;
    }

    if ignore_image_depth > 0 {
      match event {
        Event::Start(Tag::Image { .. }) => ignore_image_depth += 1,
        Event::End(TagEnd::Image) => ignore_image_depth = ignore_image_depth.saturating_sub(1),
        _ => {}
      }
      continue;
    }

    match event {
      Event::Start(Tag::Heading { level, .. }) => {
        inline = Some(InlineState::new(heading_style(&options.theme, level)));
      }
      Event::End(TagEnd::Heading(_)) => finish_text(&mut blocks, inline.take(), true),
      Event::Start(Tag::Paragraph) => {
        let mut state = InlineState::new(if quote_depth > 0 {
          options.theme.quote.clone()
        } else {
          options.theme.paragraph.clone()
        });
        for _ in 0..quote_depth {
          state.push_text(&options.theme.quote_marker, &options.theme.quote);
        }
        inline = Some(state);
      }
      Event::End(TagEnd::Paragraph) => finish_text(&mut blocks, inline.take(), true),
      Event::Start(Tag::BlockQuote(_)) => quote_depth += 1,
      Event::End(TagEnd::BlockQuote(_)) => {
        quote_depth = quote_depth.saturating_sub(1);
        blocks.push(MdBlock::Blank);
      }
      Event::Start(Tag::List(start)) => list_stack.push(ListState {
        ordered: start.is_some(),
        next: start.unwrap_or(1),
      }),
      Event::End(TagEnd::List(_)) => {
        list_stack.pop();
        blocks.push(MdBlock::Blank);
      }
      Event::Start(Tag::Item) => {
        let marker = list_stack
          .last_mut()
          .map(|list| {
            if list.ordered {
              let marker = format!("{}. ", list.next);
              list.next += 1;
              marker
            } else {
              "• ".to_string()
            }
          })
          .unwrap_or_else(|| "• ".to_string());
        let mut state = InlineState::new(options.theme.paragraph.clone());
        state.push_text(&marker, &options.theme.paragraph);
        inline = Some(state);
      }
      Event::End(TagEnd::Item) => finish_text(&mut blocks, inline.take(), false),
      Event::TaskListMarker(checked) => {
        if let Some(state) = inline.as_mut() {
          let marker = if checked { "[x] " } else { "[ ] " };
          let style = if checked {
            &options.theme.task_checked
          } else {
            &options.theme.task_unchecked
          };
          state.push_text(marker, style);
        }
      }
      Event::Start(Tag::CodeBlock(kind)) => {
        let language = match kind {
          CodeBlockKind::Fenced(info) => info.split_whitespace().next().map(str::to_string),
          CodeBlockKind::Indented => None,
        };
        code = Some((language, String::new()));
      }
      Event::Rule => {
        blocks.push(MdBlock::Rule);
        blocks.push(MdBlock::Blank);
      }
      Event::Start(Tag::Table(_)) => table_rows = Some(Vec::new()),
      Event::End(TagEnd::Table) => {
        blocks.push(MdBlock::Table {
          rows: table_rows.take().unwrap_or_default(),
        });
        blocks.push(MdBlock::Blank);
      }
      Event::Start(Tag::TableRow) => table_row = Some(Vec::new()),
      Event::End(TagEnd::TableRow) => {
        if let (Some(rows), Some(row)) = (table_rows.as_mut(), table_row.take()) {
          rows.push(row);
        }
      }
      Event::Start(Tag::TableCell) => {
        table_cell.clear();
        in_table_cell = true;
      }
      Event::End(TagEnd::TableCell) => {
        in_table_cell = false;
        if let Some(row) = table_row.as_mut() {
          row.push(std::mem::take(&mut table_cell));
        }
      }
      Event::Start(Tag::Strong) => push_inline_style(&mut inline, &options.theme.bold),
      Event::End(TagEnd::Strong) => pop_inline_style(&mut inline),
      Event::Start(Tag::Emphasis) => push_inline_style(&mut inline, &options.theme.italic),
      Event::End(TagEnd::Emphasis) => pop_inline_style(&mut inline),
      Event::Start(Tag::Strikethrough) => push_inline_style(&mut inline, &options.theme.strike),
      Event::End(TagEnd::Strikethrough) => pop_inline_style(&mut inline),
      Event::Start(Tag::Link { dest_url, .. }) => {
        if let Some(state) = inline.as_mut() {
          state.start_link(dest_url.to_string(), &options.theme.link);
        }
      }
      Event::End(TagEnd::Link) => {
        if let Some(state) = inline.as_mut() {
          state.end_link();
        }
      }
      Event::Start(Tag::Image { .. }) => ignore_image_depth = 1,
      Event::Text(text) | Event::Html(text) | Event::InlineHtml(text) => {
        if in_table_cell {
          table_cell.push_str(&text);
        } else if let Some(state) = inline.as_mut() {
          state.push_current(&text);
        }
      }
      Event::Code(text) => {
        if in_table_cell {
          table_cell.push_str(&text);
        } else if let Some(state) = inline.as_mut() {
          state.push_text(&text, &options.theme.inline_code);
        }
      }
      Event::SoftBreak | Event::HardBreak => {
        if in_table_cell {
          table_cell.push(' ');
        } else if let Some(state) = inline.as_mut() {
          state.push_current("\n");
        }
      }
      _ => {}
    }
  }
  blocks
}

fn draw_block(
  pool: &mut UiObjectPool,
  id: MarkdownViewId,
  block: &MdBlock,
  options: &MarkdownViewOptions,
  x: u16,
  y: u16,
  width: u16,
  target: MarkdownTarget,
  canvas: &mut CanvasService,
  code_highlight: &CodeHighlightService,
) -> u16 {
  match block {
    MdBlock::Text { segments, links } => {
      let params = text_params(x, y, width, None);
      draw_segments(canvas, target, segments, &params);
      register_links(pool, id, links, x, y, target, canvas);
      y.saturating_add(measure_segments(segments, &params).height)
    }
    MdBlock::Code { language, code } => draw_code_block(
      canvas,
      target,
      x,
      y,
      width,
      language.as_deref(),
      code,
      options,
      code_highlight,
    ),
    MdBlock::Rule => {
      draw_plain(
        canvas,
        target,
        x,
        y,
        &"─".repeat(width as usize),
        options.theme.horizontal_rule.clone(),
      );
      y.saturating_add(1)
    }
    MdBlock::Table { rows } => draw_table(canvas, target, x, y, width, rows, &options.theme),
    MdBlock::Blank => y.saturating_add(1),
  }
}

fn draw_code_block(
  canvas: &mut CanvasService,
  target: MarkdownTarget,
  x: u16,
  y: u16,
  width: u16,
  language: Option<&str>,
  code: &str,
  options: &MarkdownViewOptions,
  code_highlight: &CodeHighlightService,
) -> u16 {
  let lines = code.trim_end_matches('\n').split('\n').collect::<Vec<_>>();
  let max_line = lines
    .iter()
    .map(|line| display_width(line))
    .max()
    .unwrap_or(0) as u16;
  let inner_width = max_line.saturating_add(2).clamp(20, width.max(1));
  let right = inner_width.saturating_sub(1);
  draw_plain(
    canvas,
    target,
    x,
    y,
    &format!("┌{}┐", "─".repeat(right as usize)),
    options.theme.code_border.clone(),
  );
  if let Some(language) = language.filter(|language| !language.is_empty()) {
    draw_plain(
      canvas,
      target,
      x.saturating_add(2),
      y,
      language,
      options.theme.code_border.clone(),
    );
  }
  let code_language = language.and_then(|language| code_highlight.language_from_name(language));
  let mut row = y.saturating_add(1);
  for line in lines {
    draw_plain(
      canvas,
      target,
      x,
      row,
      "│",
      options.theme.code_border.clone(),
    );
    draw_plain(
      canvas,
      target,
      x.saturating_add(inner_width),
      row,
      "│",
      options.theme.code_border.clone(),
    );
    let segments = code_language
      .map(|language| code_highlight.highlight_segments(line, language, &options.code_theme))
      .unwrap_or_else(|| {
        vec![RichTextSegment {
          text: line.to_string(),
          style: options.theme.code_block.clone(),
        }]
      });
    draw_segments(
      canvas,
      target,
      &segments,
      &DrawTextParams {
        x: x.saturating_add(1),
        y: row,
        wrap_mode: TextWrapMode::None,
        max_width: Some(inner_width.saturating_sub(1)),
        ..Default::default()
      },
    );
    row = row.saturating_add(1);
  }
  draw_plain(
    canvas,
    target,
    x,
    row,
    &format!("└{}┘", "─".repeat(right as usize)),
    options.theme.code_border.clone(),
  );
  row.saturating_add(1)
}

fn draw_table(
  canvas: &mut CanvasService,
  target: MarkdownTarget,
  x: u16,
  y: u16,
  width: u16,
  rows: &[Vec<String>],
  theme: &MarkdownTheme,
) -> u16 {
  if rows.is_empty() || width < 3 {
    return y;
  }
  let columns = rows.iter().map(Vec::len).max().unwrap_or(0).max(1);
  let cell_width = ((width.saturating_sub(columns as u16 + 1)) / columns as u16).max(1);
  let mut row_y = y;
  draw_plain(
    canvas,
    target,
    x,
    row_y,
    &table_border(columns, cell_width, '┌', '┬', '┐'),
    theme.table_border.clone(),
  );
  row_y = row_y.saturating_add(1);
  for (index, row) in rows.iter().enumerate() {
    let mut line = String::from("│");
    for col in 0..columns {
      let text = row.get(col).map(String::as_str).unwrap_or("");
      line.push_str(&fit_plain(text, cell_width as usize));
      line.push('│');
    }
    draw_plain(canvas, target, x, row_y, &line, theme.paragraph.clone());
    row_y = row_y.saturating_add(1);
    let sep = if index + 1 == rows.len() {
      table_border(columns, cell_width, '└', '┴', '┘')
    } else {
      table_border(columns, cell_width, '├', '┼', '┤')
    };
    draw_plain(canvas, target, x, row_y, &sep, theme.table_border.clone());
    row_y = row_y.saturating_add(1);
  }
  row_y
}

fn measure_blocks(
  blocks: &[MdBlock],
  options: &MarkdownViewOptions,
  width: u16,
  code_highlight: &CodeHighlightService,
) -> u16 {
  let mut height = 0u16;
  for block in blocks {
    height = match block {
      MdBlock::Text { segments, .. } => {
        height.saturating_add(measure_segments(segments, &text_params(0, 0, width, None)).height)
      }
      MdBlock::Code { code, .. } => height
        .saturating_add(code.lines().count() as u16)
        .saturating_add(2),
      MdBlock::Rule => height.saturating_add(1),
      MdBlock::Table { rows } => height.saturating_add(rows.len() as u16 * 2 + 1),
      MdBlock::Blank => height.saturating_add(1),
    };
  }
  let _ = (options, code_highlight);
  height
}

fn register_links(
  pool: &mut UiObjectPool,
  id: MarkdownViewId,
  links: &[LinkSpan],
  x: u16,
  y: u16,
  target: MarkdownTarget,
  canvas: &CanvasService,
) {
  for link in links {
    let rect = Rect {
      x: x.saturating_add(link.start as u16),
      y,
      width: link.width as u16,
      height: 1,
    };
    let resolved = match target {
      MarkdownTarget::Base => canvas.base_hit_rect(rect),
      MarkdownTarget::Slice(slice) => canvas.slice_hit_rect(slice, rect),
      MarkdownTarget::ScrollBox(scroll_box) => canvas.scroll_box_hit_rect(scroll_box, rect),
      MarkdownTarget::Host => canvas.host_hit_rect(rect),
    };
    let Some((rect, _, surface_rank)) = resolved else {
      continue;
    };
    let order = pool.next_render_order();
    if let Some(state) = pool.markdown_views.views.get_mut(&id) {
      state.hits.push(MarkdownLinkHit {
        rect,
        order,
        surface_rank,
        href: link.href.clone(),
        text: link.text.clone(),
      });
    }
  }
}

fn markdown_hit(
  pool: &UiObjectPool,
  x: u16,
  y: u16,
) -> Option<(MarkdownViewId, usize, (usize, u64))> {
  pool
    .markdown_views
    .views
    .iter()
    .flat_map(|(id, state)| {
      state
        .hits
        .iter()
        .enumerate()
        .filter_map(move |(index, hit)| {
          hit
            .rect
            .contains(x, y)
            .then_some((*id, index, (hit.surface_rank, hit.order)))
        })
    })
    .max_by_key(|(_, _, order)| *order)
}

fn draw_segments(
  canvas: &mut CanvasService,
  target: MarkdownTarget,
  segments: &[RichTextSegment],
  params: &DrawTextParams,
) -> bool {
  match target {
    MarkdownTarget::Base => {
      canvas.rich_text_segments(segments, params);
      true
    }
    MarkdownTarget::Slice(slice) => canvas.rich_text_segments_on(slice, segments, params),
    MarkdownTarget::ScrollBox(scroll_box) => {
      canvas.rich_text_segments_in_scroll_box(scroll_box, segments, params)
    }
    MarkdownTarget::Host => {
      canvas.host_rich_text_segments(segments, params);
      true
    }
  }
}

fn draw_plain(
  canvas: &mut CanvasService,
  target: MarkdownTarget,
  x: u16,
  y: u16,
  text: &str,
  style: TextStyle,
) -> bool {
  match target {
    MarkdownTarget::Base => {
      canvas.styled_text(x, y, text, style);
      true
    }
    MarkdownTarget::Slice(slice) => canvas.styled_text_on(slice, x, y, text, style),
    MarkdownTarget::ScrollBox(scroll_box) => {
      canvas.styled_text_in_scroll_box(scroll_box, x, y, text, style)
    }
    MarkdownTarget::Host => {
      canvas.host_styled_text(x, y, text, style);
      true
    }
  }
}

fn text_params(x: u16, y: u16, width: u16, height: Option<u16>) -> DrawTextParams {
  DrawTextParams {
    x,
    y,
    wrap_mode: TextWrapMode::Auto,
    non_truncate_word_wrap: true,
    max_width: Some(width),
    max_height: height,
    line_align: TextAlign::Left,
    ..Default::default()
  }
}

fn measure_segments(segments: &[RichTextSegment], params: &DrawTextParams) -> Size {
  let (width, height) = text_layout::measure_rich_text_segments(segments, params);
  Size { width, height }
}

fn finish_text(blocks: &mut Vec<MdBlock>, state: Option<InlineState>, blank_after: bool) {
  let Some(state) = state else {
    return;
  };
  if state
    .segments
    .iter()
    .any(|segment| !segment.text.is_empty())
  {
    blocks.push(MdBlock::Text {
      segments: state.segments,
      links: state.links,
    });
  }
  if blank_after {
    blocks.push(MdBlock::Blank);
  }
}

fn push_inline_style(inline: &mut Option<InlineState>, style: &TextStyle) {
  if let Some(state) = inline {
    state.push_style(style);
  }
}

fn pop_inline_style(inline: &mut Option<InlineState>) {
  if let Some(state) = inline {
    state.pop_style();
  }
}

impl InlineState {
  fn new(style: TextStyle) -> Self {
    Self {
      style_stack: vec![style.clone()],
      current: style,
      segments: Vec::new(),
      links: Vec::new(),
      active_link: None,
    }
  }

  fn push_current(&mut self, text: &str) {
    self.push_text(text, &self.current.clone());
  }

  fn push_text(&mut self, text: &str, style: &TextStyle) {
    if text.is_empty() {
      return;
    }
    if let Some(link) = self.active_link.as_mut() {
      link.text.push_str(text);
    }
    self.segments.push(RichTextSegment {
      text: text.to_string(),
      style: style.clone(),
    });
  }

  fn push_style(&mut self, style: &TextStyle) {
    let merged = merge_style(&self.current, style);
    self.style_stack.push(merged.clone());
    self.current = merged;
  }

  fn pop_style(&mut self) {
    if self.style_stack.len() > 1 {
      self.style_stack.pop();
    }
    self.current = self.style_stack.last().cloned().unwrap_or_default();
  }

  fn start_link(&mut self, href: String, style: &TextStyle) {
    let start = self
      .segments
      .iter()
      .map(|segment| display_width(&segment.text))
      .sum();
    self.active_link = Some(ActiveLink {
      href,
      start,
      text: String::new(),
    });
    self.push_style(style);
  }

  fn end_link(&mut self) {
    self.pop_style();
    let Some(link) = self.active_link.take() else {
      return;
    };
    let width = display_width(&link.text);
    if width > 0 {
      self.links.push(LinkSpan {
        start: link.start,
        width,
        href: link.href,
        text: link.text,
      });
    }
  }
}

fn heading_style(theme: &MarkdownTheme, level: HeadingLevel) -> TextStyle {
  match level {
    HeadingLevel::H1 => theme.h1.clone(),
    HeadingLevel::H2 => theme.h2.clone(),
    HeadingLevel::H3 => theme.h3.clone(),
    HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => theme.h4_to_h6.clone(),
  }
}

fn merge_style(base: &TextStyle, overlay: &TextStyle) -> TextStyle {
  TextStyle {
    foreground: overlay
      .foreground
      .clone()
      .or_else(|| base.foreground.clone()),
    background: overlay
      .background
      .clone()
      .or_else(|| base.background.clone()),
    bold: base.bold || overlay.bold,
    italic: base.italic || overlay.italic,
    underline: base.underline || overlay.underline,
    strike: base.strike || overlay.strike,
    blink: base.blink || overlay.blink,
    reverse: base.reverse || overlay.reverse,
    hidden: base.hidden || overlay.hidden,
    dim: base.dim || overlay.dim,
  }
}

fn table_border(columns: usize, width: u16, left: char, middle: char, right: char) -> String {
  let mut line = String::new();
  line.push(left);
  for index in 0..columns {
    line.push_str(&"─".repeat(width as usize));
    line.push(if index + 1 == columns { right } else { middle });
  }
  line
}

fn fit_plain(text: &str, width: usize) -> String {
  let mut result = String::new();
  let mut used = 0usize;
  for grapheme in crate::host_engine::services::unicode::graphemes(text) {
    if used + grapheme.display_width > width {
      break;
    }
    used += grapheme.display_width;
    result.push_str(&grapheme.text);
  }
  result.push_str(&" ".repeat(width.saturating_sub(used)));
  result
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn markdown_measure_handles_basic_blocks() {
    let service = MarkdownService::new();
    let code = CodeHighlightService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(
        &mut pool,
        MarkdownViewOptions::new("# Title\n\nHello **world**\n\n```rust\nfn main() {}\n```"),
      )
      .unwrap();
    let size = service.measure(&pool, id, 40, &code).unwrap();
    assert!(size.height > 3);
  }

  #[test]
  fn markdown_link_is_recorded_as_hit_after_render() {
    let service = MarkdownService::new();
    let code = CodeHighlightService::new();
    let mut pool = UiObjectPool::new();
    let id = service
      .create(
        &mut pool,
        MarkdownViewOptions::new("[Open](https://example.com)"),
      )
      .unwrap();
    let mut canvas = CanvasService::new();
    let layout = crate::host_engine::services::LayoutService::new();
    canvas.begin_frame(&layout);
    canvas.prepare(&pool, &layout);
    assert!(service.render(
      &mut pool,
      id,
      MarkdownRenderParams {
        x: 0,
        y: 0,
        width: 30,
        max_height: None,
      },
      &mut canvas,
      &code,
    ));
    assert_eq!(pool.markdown_views.views[&id].hits.len(), 1);
  }

  #[test]
  fn markdown_images_are_ignored() {
    let code = CodeHighlightService::new();
    let options = MarkdownViewOptions::new("before ![alt text](image.png) after");
    let blocks = parse_markdown(&options, &code);
    let MdBlock::Text { segments, .. } = &blocks[0] else {
      panic!("expected text block");
    };
    let text = segments
      .iter()
      .map(|segment| segment.text.as_str())
      .collect::<String>();
    assert_eq!(text, "before  after");
  }
}
