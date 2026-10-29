//! Lua Markdown documents with embedded scrolling and standalone rich-text conversion.
use super::widget_scrollbox as scrolling;
use super::*;
use crate::object_pool::{LuaMarkdownBinding, LuaScrollboxBinding};
use tg_service_widget::{
  MarkdownLayout, MarkdownService, MarkdownViewOptions, ScrollBoxWidget, ScrollBoxWidgetOptions,
};

/// Build the read-only Markdown library.
///
/// # Errors
///
/// Propagate Lua allocation or function registration errors.
pub(super) fn markdown(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  source.raw_set(
    "markdown_to_rich_text",
    lua.create_function(|lua, values: MultiValue| {
      let method = "widget.markdown.markdown_to_rich_text";
      let table = positional_table(lua, method, values, &["markdown"], &[])?;
      let text = args::string(
        args::required(&table, method, "markdown")?,
        method,
        "markdown",
      )?;
      let rich = MarkdownService::new().markdown_to_rich_text(&text);
      if rich.len() > args::MAX_API_STRING_BYTES {
        return Err(args::message(
          method,
          "converted rich text exceeds the string size limit",
        ));
      }
      Ok(rich)
    })?,
  )?;
  let create_state = state.clone();
  source.raw_set(
    "create",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.markdown.create";
      let fields = [
        "wheel_step_vertical",
        "wheel_step_horizontal",
        "allow_overflow",
      ];
      let table = positional_table(
        lua,
        method,
        values,
        &["width", "height", "markdown"],
        &fields,
      )?;
      let text = args::string(
        args::required(&table, method, "markdown")?,
        method,
        "markdown",
      )?;
      let allow = args::optional_bool(&table, method, "allow_overflow", false)?;
      let (mut options, background) = scrolling::options(
        &table,
        method,
        ScrollBoxWidgetOptions::default(),
        "none".into(),
      )?;
      options.horizontal = allow;
      with_pool_mut(&create_state, method, |pool| {
        if pool.markdowns.len() >= 1024 {
          return Err(args::message(method, "markdown limit of 1024 was reached"));
        }
        let widget = ScrollBoxWidget::new(&mut pool.ui, options)
          .ok_or_else(|| args::message(method, "invalid scrollbox options"))?;
        let options = MarkdownViewOptions::new(text);
        let native = MarkdownService::new()
          .create(&mut pool.ui, options.clone())
          .ok_or_else(|| args::message(method, "cannot create Markdown document"))?;
        pool.next_markdown_id += 1;
        let id = pool.next_markdown_id;
        let scrollbox = u64::MAX - id;
        let surface = pool.ui.allocate_embedded_slice();
        pool.scrollboxes.insert(
          scrollbox,
          LuaScrollboxBinding {
            widget,
            surface,
            background,
            markdown: Some(id),
          },
        );
        pool.markdowns.insert(
          id,
          LuaMarkdownBinding {
            scrollbox,
            native,
            options,
            allow_overflow: allow,
            layout: std::sync::Arc::new(MarkdownLayout::default()),
            layouts: std::collections::HashMap::new(),
          },
        );
        Ok(format_id(id))
      })
    })?,
  )?;
  let draw_state = state.clone();
  source.raw_set(
    "draw",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.markdown.draw";
      let table = positional_table(lua, method, values, &["id", "x", "y"], &["slice_layer"])?;
      let id = table_id(&table, method)?;
      let target = parse_draw_target(&table, method, &draw_state)?;
      let x = scrolling::coordinate(&table, method, "x")?;
      let y = scrolling::coordinate(&table, method, "y")?;
      let (id, sequence) = with_pool_mut(&draw_state, method, |pool| {
        let binding = pool
          .markdowns
          .get(&id)
          .ok_or_else(|| args::message(method, "unknown markdown id"))?;
        let surface = pool.scrollboxes[&binding.scrollbox].surface;
        let sequence = pool
          .place_surface(surface, target, (x, y))
          .map_err(|error| args::message(method, error))?;
        Ok((surface, sequence))
      })?;
      draw::enqueue_draw(
        &draw_state,
        method,
        LuaDrawCommand::Surface {
          id,
          target,
          x,
          y,
          sequence,
        },
      )
    })?,
  )?;
  let set_state = state.clone();
  source.raw_set(
    "set",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "widget.markdown.set";
      let fields = [
        "width",
        "height",
        "markdown",
        "wheel_step_vertical",
        "wheel_step_horizontal",
        "allow_overflow",
        "scroll_offset_vertical",
        "scroll_offset_horizontal",
      ];
      let table = positional_table(lua, method, values, &["id"], &fields)?;
      let id = table_id(&table, method)?;
      with_pool_mut(&set_state, method, |pool| {
        let Some(binding) = pool.markdowns.get_mut(&id) else {
          return Ok(false);
        };
        let text =
          args::optional_string(&table, method, "markdown", Some(&binding.options.markdown))?
            .unwrap();
        let allow = args::optional_bool(&table, method, "allow_overflow", binding.allow_overflow)?;
        let scrolling = pool.scrollboxes.get_mut(&binding.scrollbox).unwrap();
        let (mut options, background) = scrolling::options(
          &table,
          method,
          scrolling.widget.options().clone(),
          scrolling.background.clone(),
        )?;
        options.horizontal = allow;
        let horizontal = table.raw_get::<Value>("scroll_offset_horizontal")?;
        let vertical = table.raw_get::<Value>("scroll_offset_vertical")?;
        let x = if allow && !matches!(horizontal, Value::Nil) {
          Some(scrolling::offset(
            horizontal,
            method,
            "scroll_offset_horizontal",
            false,
            0,
          )?)
        } else {
          None
        };
        let y = if !matches!(vertical, Value::Nil) {
          Some(scrolling::offset(
            vertical,
            method,
            "scroll_offset_vertical",
            true,
            0,
          )?)
        } else {
          None
        };
        if binding.options.markdown != text {
          MarkdownService::new().set_markdown(&mut pool.ui, binding.native, text.clone());
        }
        if binding.options.markdown != text
          || binding.allow_overflow != allow
          || scrolling.widget.options().width != options.width
          || scrolling.widget.options().include_scrollbar != options.include_scrollbar
          || scrolling.widget.options().always_show_scrollbar != options.always_show_scrollbar
        {
          binding.layouts.clear();
        }
        binding.options.markdown = text;
        binding.allow_overflow = allow;
        scrolling.widget.set_options(options);
        scrolling.background = background;
        if x.is_some() || y.is_some() {
          scrolling.widget.scroll_to_axes(x, y);
        }
        if !allow {
          scrolling.widget.scroll_to_axes(Some(0), None);
        }
        Ok(true)
      })
    })?,
  )?;
  let scroll_state = state.clone();
  source.raw_set(
    "scroll",
    lua.create_function(move |lua, mut values: MultiValue| {
      let method = "widget.markdown.scroll";
      if values.len() == 2 || matches!(values.get(2), Some(Value::Table(_) | Value::Nil)) {
        values.insert(2, Value::Integer(0));
      }
      let table = positional_table(
        lua,
        method,
        values,
        &["id", "scroll_times_vertical", "scroll_times_horizontal"],
        &[],
      )?;
      let id = table_id(&table, method)?;
      let vertical = table.raw_get::<Value>("scroll_times_vertical")?;
      let horizontal = table.raw_get::<Value>("scroll_times_horizontal")?;
      with_pool_mut(&scroll_state, method, |pool| {
        let Some(binding) = pool.markdowns.get(&id) else {
          return Ok(false);
        };
        let widget = &mut pool.scrollboxes.get_mut(&binding.scrollbox).unwrap().widget;
        scrolling::scroll_widget(
          widget,
          vertical,
          if binding.allow_overflow {
            horizontal
          } else {
            Value::Integer(0)
          },
          method,
        )?;
        Ok(true)
      })
    })?,
  )?;
  for name in ["delete", "get_info", "exists", "clear", "count", "list"] {
    let state = state.clone();
    source.raw_set(
      name,
      lua.create_function(move |lua, values: MultiValue| {
        let method = format!("widget.markdown.{name}");
        let id = if matches!(name, "delete" | "get_info" | "exists") {
          Some(table_id(
            &positional_table(lua, &method, values, &["id"], &[])?,
            &method,
          )?)
        } else {
          args::no_args(&method, values)?;
          None
        };
        with_pool_mut(&state, &method, |pool| match name {
          "delete" => Ok(Value::Boolean(pool.delete_markdown(id.unwrap()))),
          "exists" => Ok(Value::Boolean(pool.markdowns.contains_key(&id.unwrap()))),
          "get_info" => pool
            .markdowns
            .get(&id.unwrap())
            .map(|binding| info(lua, pool, id.unwrap(), binding).map(Value::Table))
            .unwrap_or(Ok(Value::Nil)),
          "clear" => {
            for id in pool.markdowns.keys().copied().collect::<Vec<_>>() {
              pool.delete_markdown(id);
            }
            Ok(Value::Boolean(true))
          }
          "count" => Ok(Value::Integer(pool.markdowns.len() as i64)),
          _ => {
            let result = lua.create_table()?;
            for (index, (id, binding)) in pool.markdowns.iter().enumerate() {
              result.raw_set(index + 1, info(lua, pool, *id, binding)?)?;
            }
            result.raw_set("n", pool.markdowns.len())?;
            Ok(Value::Table(result))
          }
        })
      })?,
    )?;
  }
  readonly::proxy(lua, source)
}
fn table_id(table: &Table, method: &str) -> mlua::Result<u64> {
  args::string(args::required(table, method, "id")?, method, "id")?
    .strip_prefix("markdown_")
    .and_then(|id| id.parse().ok())
    .filter(|id| *id > 0)
    .ok_or_else(|| args::message(method, "invalid markdown id"))
}
fn format_id(id: u64) -> String {
  format!("markdown_{id:03}")
}
fn info(
  lua: &Lua,
  pool: &crate::LuaObjectPool,
  id: u64,
  binding: &LuaMarkdownBinding,
) -> mlua::Result<Table> {
  let table = lua.create_table()?;
  let widget = &pool.scrollboxes[&binding.scrollbox].widget;
  let options = widget.options();
  let (x, y) = widget.offset();
  table.raw_set("id", format_id(id))?;
  table.raw_set("markdown", binding.options.markdown.as_str())?;
  table.raw_set("allow_overflow", binding.allow_overflow)?;
  for (name, value) in [
    ("width", options.width),
    ("height", options.height),
    ("wheel_step_vertical", options.wheel_step_vertical),
    ("wheel_step_horizontal", options.wheel_step_horizontal),
    ("scroll_offset_vertical", y),
    ("scroll_offset_horizontal", x),
  ] {
    table.raw_set(name, value)?;
  }
  Ok(table)
}
