//! Exercise strict optional parameters through independent game and screensaver sessions.
//!
//! Run `cargo run -p tg-service-lua --example lua_options_smoke --locked` with a writable
//! temporary directory. No terminal, audio device, or deployed assets are required.

use std::{
  fs,
  path::PathBuf,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use tg_service_layout::Size;
use tg_service_lua::{
  LuaApiConfig, LuaDrawCommand, LuaHostCommand, LuaService, LuaSessionKind, LuaSessionSpec,
};
use tg_service_render::BorderStyle;
use tg_service_rich_text::TextMode;

/// The temporary deployment tree owned by this smoke run.
struct SmokeDeployment(PathBuf);

impl Drop for SmokeDeployment {
  fn drop(&mut self) {
    if let Err(error) = fs::remove_dir_all(&self.0) {
      eprintln!(
        "failed to remove smoke deployment {}: {error}",
        self.0.display()
      );
    }
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
  let root = std::env::temp_dir().join(format!(
    "tg_lua_options_smoke_{}_{nonce}",
    std::process::id()
  ));
  fs::create_dir(&root)?;
  let deployment = SmokeDeployment(root);
  fs::create_dir(deployment.0.join("scripts"))?;
  let entry = deployment.0.join("scripts/main.lua");
  fs::write(
    &entry,
    r##"
    function Init(ctx)
      for _, measure in ipairs({measurement.get_text_size, measurement.get_text_width, measurement.get_text_height}) do
        local text = "f%<fg:green>Hello</fg>\nTUI"
        local defaults = table.pack(measure(text))
        local explicit = table.pack(measure(text, {rich_params={}, max_width=math.POSITIVE_INFINITE,
          max_height=math.INFINITE, parse_mode=string.AUTO}))
        debug.assert(defaults.n == explicit.n)
        for index=1,defaults.n do debug.assert(defaults[index] == explicit[index]) end
        for _, value in ipairs({0,-1,1.5,65536,math.NEGATIVE_INFINITE,0/0,false,"infinite"}) do
          debug.assert(not debug.pcall(measure,text,{max_width=value}))
          debug.assert(not debug.pcall(measure,text,{max_height=value}))
        end
        debug.assert(not debug.pcall(measure,text,{rich_params=false}))
        debug.assert(not debug.pcall(measure,text,{text_mode=string.AUTO}))
        debug.assert(not debug.pcall(measure,text,{parse_mode="unknown"}))
      end
      debug.assert(measurement.get_text_width("f%{value:word}",
        {rich_params={word="value"}, parse_mode=string.AUTO}) == 5)
      debug.assert(measurement.get_text_width("f%<fg:green>Test", {parse_mode=string.PLAIN_TEXT}) == 16)
      debug.assert(math.percent == nil and math.ratio(1, 4) == 0.25)
      debug.assert(math.round_to(1.25, 1) == 1.3)
      debug.assert(not debug.pcall(math.round_to, 1.25, 1.0))
      debug.assert(math.normalize_angle(-90.25) == 269.75)
      debug.assert(math.normalize_angle(math.MAX_INTEGER) == 7)
      debug.assert(not debug.pcall(math.normalize_angle, math.INFINITE))
      debug.assert(math.type(math.INFINITE) == "float" and math.tointeger(math.INFINITE) == nil)
      for _, name in ipairs({"enable_action_event", "disable_action_event", "enable_key_event", "disable_key_event"}) do
        debug.assert(type(keyboard[name]) == "function")
      end
      debug.assert(keyboard.receive_action_event == nil and keyboard.reject_action_event == nil)
      debug.assert(keyboard.receive_key_event == nil and keyboard.reject_key_event == nil)
      for _, format in ipairs({"base64", "url", "hex"}) do
        local encode, decode = encoding[format .. "_encode"], encoding[format .. "_decode"]
        local original = "Tui Game\0\255"
        debug.assert(decode(encode(original)) == original)
        for _, case in ipairs({{encode,"text"}, {decode,"data"}}) do
          for _, invalid in ipairs({{}, {false}}) do
            local ok, message = debug.pcall(case[1], table.unpack(invalid))
            debug.assert(not ok and string.find(message, case[2], {plain=true}) ~= nil)
          end
        end
      end
      for _, case in ipairs({
        {char.LINE,"┌","┐","└","┘"}, {char.BOLD_LINE,"┏","┓","┗","┛"},
        {char.DOUBLE_LINE,"╔","╗","╚","╝"}, {char.ROUNDED_LINE,"╭","╮","╰","╯"},
      }) do
        local border = case[1]
        debug.assert(border.top_left == case[2] and border.top_right == case[3])
        debug.assert(border.bottom_left == case[4] and border.bottom_right == case[5])
        debug.assert(border.left_top == nil and border.right_top == nil and border.left_bottom == nil and border.right_bottom == nil)
      end
      debug.print("plain log")
      debug.print("unset log headers", {title=nil, level=nil})
      debug.print("configured log", {title="Options", level=debug.INFO, show_time=true, show_session_type=true})
      for _, options in ipairs({{time=true}, {type_head=true}}) do
        debug.assert(not debug.pcall(debug.print, "legacy log options", options))
      end
      debug.assert(tonumber("ff", {base = 16}) == 255)
      local payload = {pos = "data"}
      local list = {"a", "c"}
      table.insert(list, "b", {pos = 2})
      debug.assert(table.concat(list, {separator = "-"}) == "a-b-c")
      debug.assert(table.concat(list, {separator = "-", start_index = 2, end_index = 3}) == "b-c")
      debug.assert(table.concat(list, {end_index = 2}) == "ab")
      for _, options in ipairs({{sep = "-"}, {i = 1}, {j = 3}}) do
        debug.assert(not select(1, debug.pcall(function() table.concat(list, options) end)))
      end
      debug.assert(table.remove(list, {pos = 2}) == "b")
      table.insert(list, payload)
      debug.assert(list[3] == payload)
      local target = {}
      debug.assert(table.move(list, 1, 3, 1, {target = target}) == target)
      debug.assert(target[3] == payload)
      local packed = table.pack("a", nil, "c")
      local a, b, c = table.unpack(packed, {end_index = packed.n})
      debug.assert(a == "a" and b == nil and c == "c")
      debug.assert(select("#", table.unpack(packed, {end_index = packed.n})) == 3)
      local middle, last = table.unpack(packed, {start_index = 2, end_index = 3})
      debug.assert(middle == nil and last == "c")
      for _, options in ipairs({{i = 1}, {j = 3}}) do
        debug.assert(not select(1, debug.pcall(function() table.unpack(packed, options) end)))
      end
      local numbers = {1, 3, 2}
      table.sort(numbers, {comp = function(left, right) return left > right end})
      debug.assert(numbers[1] == 3 and numbers[3] == 1)
      local seen = 0
      local key, value = next(numbers)
      while key ~= nil do
        seen = seen + value
        key, value = next(numbers, {key = key})
      end
      debug.assert(seen == 6)
      seen = 0
      for _, value in pairs(numbers) do seen = seen + value end
      debug.assert(seen == 6)
      debug.assert(not select(1, debug.pcall(function() table.remove(list, 1) end)))
      debug.assert(not select(1, debug.pcall(function() tonumber("ff", {unknown = true}) end)))
    end
    function HandleEvent(event) end
    function Update(dt) end
    function UpdateFrame(dt, alpha) end
    function Render()
      draw.text(0,0,"unlimited",{max_width=math.POSITIVE_INFINITE,max_height=math.INFINITE})
      draw.text(0,1,"default",{parse_mode=string.PLAIN_TEXT})
      draw.text(0,2,"f%{value:word}",{parse_mode=string.RICH_TEXT,rich_params={word="value"},max_width=3,max_height=1})
      draw.fill_rect(0,3,2,1)
      draw.stroke_rect(0,4,4,3,{border_chars=char.ROUNDED_LINE})
      draw.stroke_rect(5,4,4,3,{border_chars={top="T",bottom="B",left="L",right="R",
        top_left="A",top_right="C",bottom_left="D",bottom_right="E"}})
      for _, value in ipairs({0,-1,1.5,65536,math.NEGATIVE_INFINITE,0/0,false,"infinite"}) do
        debug.assert(not debug.pcall(draw.text,0,0,"invalid",{max_width=value}))
        debug.assert(not debug.pcall(draw.text,0,0,"invalid",{max_height=value}))
      end
      debug.assert(not debug.pcall(draw.text,0,0,"invalid",{rich_params=false}))
      debug.assert(not debug.pcall(draw.text,0,0,"invalid",{text_mode=string.PLAIN_TEXT}))
      debug.assert(not debug.pcall(draw.text,0,0,"invalid",{parse_mode="unknown"}))
      debug.assert(not debug.pcall(draw.stroke_rect,0,0,2,2,{border_char=char.LINE}))
      for _, value in ipairs({"","ab","我"}) do
        debug.assert(not debug.pcall(draw.fill_rect,0,0,1,1,{char=value}))
      end
    end
    "##,
  )?;
  for session_kind in [LuaSessionKind::Game, LuaSessionKind::Screensaver] {
    let mut session = LuaService::new().create_session_with_api(
      LuaSessionSpec {
        package_id: "options_smoke".to_string(),
        session_kind,
        entry_path: entry.clone(),
        fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
        base_size: Size {
          width: 80,
          height: 24,
        },
        continue_data: None,
        best_data: None,
        save_game_enabled: false,
        save_best_enabled: false,
      },
      LuaApiConfig {
        debug_enabled: true,
        ..LuaApiConfig::default()
      },
    )?;
    let commands = session.take_host_commands();
    for expected in ["plain log", "unset log headers"] {
      assert!(commands.iter().any(|command| matches!(command,
        LuaHostCommand::Print {message, title:None, level:None, time:false, type_head:false}
          if message == expected)));
    }
    assert!(commands.iter().any(|command| matches!(command,
      LuaHostCommand::Print {message, title:Some(title), level:Some(level), time:true, type_head:true}
        if message == "configured log" && title == "Options" && level == "info")));
    session.update()?;
    session.render()?;
    let drawings = session.take_draw_commands();
    assert_eq!(drawings.len(), 6);
    for (command, expected_mode) in
      drawings[..3]
        .iter()
        .zip([TextMode::Auto, TextMode::Plain, TextMode::Rich])
    {
      assert!(
        matches!(command, LuaDrawCommand::Text {params,..} if params.text_mode == expected_mode)
      );
    }
    for command in &drawings[..2] {
      let LuaDrawCommand::Text { params, .. } = command else {
        panic!("expected text drawing");
      };
      assert!(params.max_width.is_none() && params.max_height.is_none());
      let rich = params
        .params
        .as_ref()
        .expect("default rich parameter table");
      assert!(
        rich.values.is_empty()
          && rich.key_actions.is_empty()
          && rich.key_default_actions.is_empty()
      );
    }
    assert!(matches!(&drawings[2], LuaDrawCommand::Text {params,..}
      if params.max_width == Some(3) && params.max_height == Some(1)
        && params.params.as_ref().unwrap().values.get("word").map(String::as_str) == Some("value")));
    assert!(
      matches!(&drawings[3], LuaDrawCommand::FillRect {fill_char:Some(value),..} if value == " ")
    );
    for (command, expected) in drawings[4..]
      .iter()
      .zip([['╭', '╮', '╰', '╯'], ['A', 'C', 'D', 'E']])
    {
      let LuaDrawCommand::StrokeRect {
        border: BorderStyle::Custom(border),
        ..
      } = command
      else {
        panic!("expected custom border drawing");
      };
      assert_eq!(
        [
          border.left_top.char,
          border.right_top.char,
          border.left_bottom.char,
          border.right_bottom.char
        ],
        expected.map(Some)
      );
    }
  }
  println!("Lua options ok: game and screensaver calls, payloads, iteration and strict errors");
  Ok(())
}
