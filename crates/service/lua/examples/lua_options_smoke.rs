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
      for name, methods in pairs({source={table.concat,table.unpack},
        target={table.insert,table.remove,table.sort}}) do
        for _, method in ipairs(methods) do
          local ok, message = debug.pcall(method)
          debug.assert(not ok and string.find(message,name,{plain=true}) ~= nil)
        end
      end
      for _, method in ipairs({function() return table.insert(char.ASCII_LETTER,"!") end,
        function() return table.remove(char.ASCII_LETTER) end,
        function() return table.sort(char.ASCII_LETTER) end}) do
        local ok, message = debug.pcall(method)
        debug.assert(not ok and string.find(message,"target",{plain=true}) ~= nil)
      end
      for _, method in ipairs({table.deepcopy,table.pretty,table.count,table.count_array,table.count_hash}) do
        local ok, message = debug.pcall(method)
        debug.assert(not ok and string.find(message,"source",{plain=true}) ~= nil)
      end
      local ok, message = debug.pcall(table.compact)
      debug.assert(not ok and string.find(message,"target",{plain=true}) ~= nil)
      for index, name in ipairs({"source","start_index","end_index","target_start_index"}) do
        local arguments = table.pack({},1,2)
        arguments.n = index-1
        local ok, message = debug.pcall(table.move,table.unpack(arguments,{end_index=arguments.n}))
        debug.assert(not ok and string.find(message,name,{plain=true}) ~= nil)
      end
      local ordered = {3,1,2}
      for _, options in ipairs({{},{comparator=nil}}) do
        table.sort(ordered,options)
        debug.assert(ordered[1]==1 and ordered[2]==2 and ordered[3]==3)
        table.sort(ordered,{comparator=function(left,right) return left>right end})
        debug.assert(ordered[1]==3 and ordered[2]==2 and ordered[3]==1)
      end
      debug.assert(not debug.pcall(table.sort,ordered,{comp=function(left,right) return left<right end}))
      debug.assert(not debug.pcall(table.sort,ordered,{comparator=false}))
      local entries = {"a","c"}
      table.insert(entries,"b",{position=2})
      debug.assert(table.remove(entries,{position=2})=="b")
      debug.assert(not debug.pcall(table.insert,entries,"b",{pos=2}))
      debug.assert(not debug.pcall(table.remove,entries,{pos=1}))
      for _, call in ipairs({function() return string.format() end,
        function() return string.format(123) end}) do
        local ok, message = debug.pcall(call)
        debug.assert(not ok and string.find(message,"template",{plain=true}) ~= nil)
      end
      debug.assert(string.format("%s:%04d", "value", 7) == "value:0007")
      debug.assert(string.format("literal") == "literal")
      local rich_text = "f%<fg:red>Hello</fg> {value:missing} {key:missing}"
      local default_plain = string.rich_text_to_plain_text(rich_text)
      debug.assert(default_plain == string.rich_text_to_plain_text(rich_text,{rich_params={}}))
      debug.assert(default_plain == string.rich_text_to_plain_text(rich_text,{rich_params=nil}))
      debug.assert(default_plain == "Hello {value:missing} {key:missing}")
      debug.assert(string.rich_text_to_plain_text("f%{value:name}",{rich_params={name="TUI"}}) == "TUI")
      debug.assert(string.rich_text_to_plain_text("f%<fg:red>TUI</fg>",
        {rich_params={},strip_header=false}) == "f%TUI")
      debug.assert(not debug.pcall(string.rich_text_to_plain_text,"text",{rich_params=false}))
      local function check_missing_slice(id)
        debug.assert(not slice.exists(id))
        local count = slice.count()
        for name, method in pairs({delete=slice.delete, get_info=slice.get_info,
          set=function(id) return slice.set(id, {width=2}) end,
          draw=function(id) return slice.draw(id, 0, 0) end}) do
          local ok, message = debug.pcall(method, id)
          debug.assert(not ok and string.find(message,"slice."..name,{plain=true}) ~= nil
            and string.find(message,id,{plain=true}) ~= nil)
          debug.assert(slice.count() == count)
        end
      end
      check_missing_slice("slice_999")
      local first_slice = slice.create(2,2)
      local second_slice = slice.create(3,3)
      debug.assert(slice.exists("base") and slice.get_info("base").layer == 0)
      for name, method in pairs({delete=slice.delete, set=slice.set,
        set_options=function(id) return slice.set(id,{width=1}) end}) do
        local ok, message = debug.pcall(method,"base")
        local expected = name=="delete" and "slice.delete" or "slice.set"
        debug.assert(not ok and string.find(message,expected,{plain=true}) ~= nil
          and string.find(message,"base",{plain=true}) ~= nil)
        debug.assert(slice.count()==2 and slice.get_info("base").width==ctx.base.width)
      end
      debug.assert(not debug.pcall(slice.draw,"base",0,0))
      debug.assert(not debug.pcall(slice.draw,first_slice,0,0,{slice_id="base"}))
      debug.assert(not debug.pcall(slice.draw,first_slice,0,0,{slice_layer="base"}))
      debug.assert(not debug.pcall(draw.text,0,0,"x",{slice_layer=first_slice}))
      debug.assert(not debug.pcall(align.resolve_rect,1,1,align.LEFT,align.TOP,{slice_layer=first_slice}))
      debug.assert(not debug.pcall(effect.cells,0,0,1,1,{slice_layer=first_slice}))
      debug.assert(slice.delete(first_slice))
      check_missing_slice(first_slice)
      debug.assert(slice.exists(second_slice) and slice.get_info(second_slice).layer == 1)
      debug.assert(slice.clear())
      check_missing_slice(second_slice)
      for _, dimensions in ipairs({{0,1},{1,0},{65536,1},{1,65536},{-1,1}}) do
        debug.assert(not debug.pcall(slice.create,dimensions[1],dimensions[2]))
      end
      local packed = serialization.binary_pack("<I2z", 513, "ok")
      local unpacked, next_position = serialization.binary_unpack("<I2z", packed..packed)
      debug.assert(unpacked[1] == 513 and unpacked[2] == "ok" and next_position == #packed+1)
      local second, end_position = serialization.binary_unpack("<I2z", packed..packed, {position=next_position})
      debug.assert(second[1] == 513 and second[2] == "ok" and end_position == #packed*2+1)
      debug.assert(serialization.binary_packsize("<I2c2x") == 5)
      debug.assert(not debug.pcall(serialization.binary_packsize,"<I2z"))
      debug.assert(not debug.pcall(serialization.binary_unpack,"<I2z",packed,{pos=1}))
      for _, position in ipairs({0, #packed+2}) do
        local ok, message = debug.pcall(serialization.binary_unpack,"<I2z",packed,{position=position})
        debug.assert(not ok and string.find(message,"position",{plain=true}) ~= nil)
      end
      for _, method in ipairs({serialization.binary_pack, serialization.binary_unpack, serialization.binary_packsize}) do
        local ok, message = debug.pcall(method)
        debug.assert(not ok and string.find(message,"format",{plain=true}) ~= nil)
      end
      local function check_missing_random(id)
        debug.assert(not random.exists(id))
        local count = random.count()
        for name, method in pairs({delete=random.delete, generate=random.generate, get_info=random.get_info,
          set=function(id) return random.set(id, {offset=0}) end}) do
          local ok, message = debug.pcall(method, id)
          debug.assert(not ok and string.find(message,"random."..name,{plain=true}) ~= nil
            and string.find(message,id,{plain=true}) ~= nil)
          debug.assert(random.count() == count)
        end
      end
      check_missing_random("rng_999")
      local random_options = {seed=42, offset=7, min=-10, max=10}
      local first_random = random.create(random_options)
      local second_random = random.create(random_options)
      debug.assert(random.generate(first_random) == random.generate(second_random))
      debug.assert(random.get_info(first_random).offset == 8 and random.get_info(first_random).step == nil)
      local random_list = random.list()
      debug.assert(random_list.n == 2 and random_list[1].offset == 8 and random_list[1].step == nil)
      debug.assert(random.set(first_random, {offset=7}))
      debug.assert(random.set(second_random, {offset=7}))
      debug.assert(random.generate(first_random) == random.generate(second_random))
      for _, options in ipairs({{step=0}, {offset=-1}, {offset=math.INFINITE}}) do
        debug.assert(not debug.pcall(random.create, options))
        debug.assert(not debug.pcall(random.set, first_random, options))
        debug.assert(random.get_info(first_random).offset == 8)
      end
      for _, kind in ipairs({random.INT, random.FLOAT}) do
        debug.assert(random.set(first_random, {type=kind, offset=math.MAX_INTEGER-1}))
        debug.assert(type(random.generate(first_random)) == "number")
        debug.assert(random.get_info(first_random).offset == math.MAX_INTEGER)
        debug.assert(type(random.generate(first_random)) == "number")
        debug.assert(random.get_info(first_random).offset == 0)
        debug.assert(random.set(second_random, {type=kind, offset=0}))
        debug.assert(random.generate(first_random) == random.generate(second_random))
        debug.assert(random.get_info(first_random).offset == 1)
        for _, info in ipairs(random.list()) do debug.assert(info.offset == 1) end
      end
      debug.assert(random.delete(first_random))
      check_missing_random(first_random)
      debug.assert(random.exists(second_random) and random.get_info(second_random).offset == 1)
      debug.assert(random.clear())
      check_missing_random(second_random)
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
      table.insert(list, "b", {position = 2})
      debug.assert(table.concat(list, {separator = "-"}) == "a-b-c")
      debug.assert(table.concat(list, {separator = "-", start_index = 2, end_index = 3}) == "b-c")
      debug.assert(table.concat(list, {end_index = 2}) == "ab")
      for _, options in ipairs({{sep = "-"}, {i = 1}, {j = 3}}) do
        debug.assert(not select(1, debug.pcall(function() table.concat(list, options) end)))
      end
      debug.assert(table.remove(list, {position = 2}) == "b")
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
      table.sort(numbers, {comparator = function(left, right) return left > right end})
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
