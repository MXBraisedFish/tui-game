//! Exercise shader lifecycle, staged composition, cell moves, clipping, and callback isolation.
//!
//! Run `cargo run -p tg-service-lua --example lua_effect_smoke --locked`.
//! Require a writable temporary deployment directory; no terminal or external tools are needed.

use std::{
  fs,
  path::PathBuf,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_core_style::{ComposedCell, ComposedFrame};
use tg_service_canvas::CanvasService;
use tg_service_layout::{LayoutService, Rect, Size};
use tg_service_lua::{
  LuaService, LuaSession, LuaSessionError, LuaSessionKind, LuaSessionSpec, LuaSessionState,
};
use tg_service_render::RenderService;
use tg_service_render_pipeline::FrameCompositor;

struct Deployment(PathBuf);
impl Drop for Deployment {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

struct Harness {
  session: LuaSession,
  canvas: CanvasService,
  layout: LayoutService,
  _deployment: Deployment,
}

impl Harness {
  fn new(source: &str, kind: LuaSessionKind) -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let deployment =
      Deployment(std::env::temp_dir().join(format!("tg-effect-{}-{nonce}", std::process::id())));
    fs::create_dir_all(deployment.0.join("scripts")).unwrap();
    fs::create_dir_all(deployment.0.join("assets")).unwrap();
    let entry = deployment.0.join("scripts/main.lua");
    fs::write(&entry, format!("function Init(ctx) end\nfunction Render() end\nfunction Update(dt) end\nfunction UpdateFrame(dt, alpha) end\nfunction HandleEvent(event) end\n{source}")).unwrap();
    let session = LuaService::new()
      .create_session(LuaSessionSpec {
        package_id: "effect_smoke".into(),
        session_kind: kind,
        entry_path: entry,
        fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
        base_size: Size {
          width: 12,
          height: 6,
        },
        continue_data: None,
        best_data: None,
        save_game_enabled: false,
        save_best_enabled: false,
      })
      .unwrap();
    let mut layout = LayoutService::new();
    layout.resize_physical(14, 9);
    layout.set_developer_viewport(Rect {
      x: 1,
      y: 2,
      width: 12,
      height: 6,
    });
    Self {
      session,
      canvas: CanvasService::new(),
      layout,
      _deployment: deployment,
    }
  }

  fn render(&mut self) -> Result<ComposedFrame, LuaSessionError> {
    self
      .session
      .with_objects_mut(|objects| objects.begin_frame())
      .unwrap();
    self.session.render()?;
    self.canvas.begin_frame(&self.layout);
    self
      .session
      .with_objects_mut(|objects| {
        objects.ui_mut().begin_render();
        objects.ui().prepare_canvas(&mut self.canvas, &self.layout);
      })
      .unwrap();
    self
      .canvas
      .host_cell(0, 0, tg_core_style::CanvasCell::new("H"));
    self
      .session
      .replay_draw_commands(&mut RenderService::new(), &mut self.canvas)?;
    Ok(FrameCompositor::new().compose(&self.canvas))
  }
}

fn text(frame: &ComposedFrame, x: u16, y: u16) -> &str {
  match frame.get(x + 1, y + 2).unwrap() {
    ComposedCell::Text(cell) if cell.text.is_empty() => "",
    ComposedCell::Text(cell) => &cell.text,
    ComposedCell::Empty => " ",
  }
}

pub(crate) fn main() {
  lifecycle_and_identity();
  staged_order();
  coordinates_and_snapshots();
  wide_graphemes_and_styles();
  nested_containers();
  completed_frame_queries();
  nested_source_queries();
  wide_character_queries();
  callback_failures();
}

fn completed_frame_queries() {
  for kind in [LuaSessionKind::Game, LuaSessionKind::Screensaver] {
    let mut harness = Harness::new(
      r#"
      function Init(ctx)
        layer_a, layer_b = slice.create(3,1,{bg=color.BLUE}), slice.create(3,1)
        layer_blank = slice.create(2,1,{bg=color.RED})
        rendered = 0
        debug.assert(effect.cells(0,0,1,1).n == 0)
        debug.assert(effect.cells(0,0,1,1,{scope=effect.SLICE,slice_layer=layer_a}).n == 0)
        for _, options in ipairs({{scope="bad"},{include_continuation_cells=1},{include_continuation=true},{scope=effect.SLICE,slice_layer="slice_999"},{unknown=true}}) do
          debug.assert(not debug.pcall(effect.cells,0,0,1,1,options))
        end
        for _, invalid in ipairs({0,-1,65536,1.5,false}) do
          debug.assert(not debug.pcall(effect.cells,0,0,invalid,1))
        end
        debug.assert(not debug.pcall(effect.cells,2147483648,0,1,1))
      end
      function Render()
        rendered = rendered + 1
        local previous = effect.cells(0,0,1,1)
        debug.assert(rendered == 1 and previous.n == 0 or rendered == 2 and previous[1].char == "8")
        slice.draw(layer_blank,8,0)
        if rendered == 2 then draw.text(0,0,"9"); return end
        draw.text(0,0,"888",{fg=color.WHITE})
        draw.text(0,0,"aaa",{fg=color.WHITE,slice_layer=layer_a})
        draw.text(0,0,"bbb",{fg=color.WHITE,slice_layer=layer_b})
        slice.draw(layer_a,1,0)
        slice.draw(layer_b,3,0)
        effect.effect(0,0,6,1,{process=function(cell)
          debug.assert(effect.cells(0,0,1,1).n == 0)
          return {fg=color.GREEN,bg=color.NONE}
        end})
      end
      function Update(dt)
        local chars = effect.cells(0,0,6,1,{slice_layer="ignored"})
        debug.assert(chars.n == 6)
        local blank_canvas = effect.cells(8,0,2,1)
        local blank_slice = effect.cells(0,0,2,1,{scope=effect.SLICE,slice_layer=layer_blank})
        for index=1,2 do
          debug.assert(blank_canvas[index].char == "" and blank_canvas[index].bg == color.RED)
          debug.assert(blank_slice[index].char == "" and blank_slice[index].bg == color.RED)
        end
        if rendered == 2 then
          debug.assert(chars[1].char == "9" and chars[2].char == "")
          debug.assert(effect.cells(0,0,3,1,{scope=effect.SLICE,slice_layer=layer_a}).n == 0)
          return
        end
        local text = {"8","a","a","b","b","b"}
        for index, cell in ipairs(chars) do
          debug.assert(cell.char == text[index] and cell.x == index-1 and cell.y == 0)
          debug.assert(cell.fg == color.GREEN and cell.bg == color.NONE)
          debug.assert(cell.bold == false and cell.italic == false and cell.underline == false and cell.strike == false)
          debug.assert(cell.blink == false and cell.reverse == false and cell.hidden == false and cell.dim == false)
          debug.assert(cell.relative_x == nil and cell.relative_y == nil)
        end
        local base = effect.cells(0,0,6,1,{scope=effect.SLICE})
        debug.assert(base.n == 6 and base[1].char == "8" and base[2].char == "8" and base[3].char == "8" and base[4].char == "")
        debug.assert(base[1].fg == color.GREEN and base[2].fg == color.WHITE and base[3].fg == color.WHITE)
        local a = effect.cells(0,0,3,1,{scope=effect.SLICE,slice_layer=layer_a})
        local b = effect.cells(0,0,3,1,{scope=effect.SLICE,slice_layer=layer_b})
        for index=1,3 do
          debug.assert(a[index].char == "a" and a[index].x == index-1)
          debug.assert(a[index].fg == (index <= 2 and color.GREEN or color.WHITE))
          debug.assert(b[index].char == "b" and b[index].fg == color.GREEN)
        end
        debug.assert(a[1].bg == color.NONE and a[2].bg == color.NONE and a[3].bg == color.BLUE)
        chars[1].char = "changed by caller"
        debug.assert(effect.cells(0,0,1,1)[1].char == "8")
        local clipped = effect.cells(-1,0,3,1)
        debug.assert(clipped.n == 2 and clipped[1].x == 0 and clipped[2].x == 1)
        debug.assert(effect.cells(2147483647,-2147483648,65535,65535).n == 0)
      end
    "#,
      kind,
    );
    for _ in 0..2 {
      harness.render().unwrap();
      harness.session.update().unwrap();
    }
    harness.session.stop();
    assert!(!harness.session.has_objects());
  }
}

fn nested_source_queries() {
  let mut harness = Harness::new(
    r#"
    function Init(ctx) parent=slice.create(4,2); child=slice.create(3,1) end
    function Render()
      draw.text(0,0,"PPPP",{fg=color.WHITE,slice_layer=parent})
      draw.text(0,0,"ABC",{fg=color.WHITE,slice_layer=child})
      slice.draw(child,-1,0,{slice_layer=parent})
      slice.draw(parent,2,1)
      effect.effect(0,0,2,1,{scope=effect.SLICE,slice_layer=parent,process=function(cell) return {fg=color.RED} end})
      effect.effect(2,1,1,1,{process=function(cell) return {x=3,char="Z",fg=color.GREEN} end})
      effect.effect(3,1,1,1,{process=function(cell) return {fg=color.BLUE,bold=true} end})
    end
    function Update(dt)
      local parent_chars=effect.cells(0,0,4,1,{scope=effect.SLICE,slice_layer=parent})
      for index=1,4 do debug.assert(parent_chars[index].char == "P" and parent_chars[index].fg == color.WHITE) end
      local child_chars=effect.cells(0,0,3,1,{scope=effect.SLICE,slice_layer=child})
      debug.assert(child_chars.n == 2 and child_chars[1].x == 1 and child_chars[1].char == "")
      debug.assert(child_chars[2].x == 2 and child_chars[2].char == "Z" and child_chars[2].fg == color.BLUE and child_chars[2].bold)
      local final=effect.cells(3,1,1,1)
      debug.assert(final[1].char == "Z" and final[1].x == 3 and final[1].fg == color.BLUE)
    end
  "#,
    LuaSessionKind::Game,
  );
  harness.render().unwrap();
  harness.session.update().unwrap();
}

fn wide_character_queries() {
  let mut harness = Harness::new(
    r#"
    function Render()
      draw.text(2,0,"我́a")
      effect.effect(2,0,3,1,{process=function(cell) return {fg=color.BLUE} end})
    end
    function Update(dt)
      for _, scope in ipairs({effect.CANVAS,effect.SLICE}) do
        debug.assert(effect.cells(3,0,1,1,{scope=scope}).n == 0)
        local wide=effect.cells(3,0,1,1,{scope=scope,include_continuation_cells=true})
        debug.assert(wide.n == 1 and wide[1].x == 2 and wide[1].char == "我́" and wide[1].fg == color.BLUE)
        local all=effect.cells(2,0,3,1,{scope=scope,include_continuation_cells=true})
        debug.assert(all.n == 2 and all[2].x == 4 and all[2].char == "a")
      end
    end
  "#,
    LuaSessionKind::Game,
  );
  harness.render().unwrap();
  harness.session.update().unwrap();
}

fn lifecycle_and_identity() {
  for kind in [LuaSessionKind::Game, LuaSessionKind::Screensaver] {
    let mut harness = Harness::new(
      r#"
      local function assert_missing(id)
        debug.assert(not effect.exists(id))
        for _, method in ipairs({"apply", "set", "delete", "get_info"}) do
          local ok, message
          if method == "apply" then
            ok, message = debug.pcall(effect[method], id, 0, 0)
          elseif method == "set" then
            ok, message = debug.pcall(effect[method], id, {width=2})
          else
            ok, message = debug.pcall(effect[method], id)
          end
          debug.assert(not ok)
          debug.assert(string.find(message, "effect." .. method, {plain=true}) ~= nil)
          debug.assert(string.find(message, id, {plain=true}) ~= nil)
          debug.assert(string.find(message, "does not exist", {plain=true}) ~= nil)
        end
      end
      function Init(ctx)
        debug.assert(effect.CANVAS == "canvas" and effect.SLICE == "slice")
        debug.assert(effect.chars == nil and type(effect.cells) == "function")
        debug.assert(not debug.pcall(function() effect.CANVAS = "slice" end))
        ef = effect.create(3, 1)
        debug.assert(ef == "effect_001" and effect.count() == 1)
        local info = effect.get_info(ef)
        debug.assert(info.width == 3 and info.height == 1 and info.scope == "canvas")
        debug.assert(info.include_continuation_cells == false and type(info.process) == "function")
        debug.assert(info.include_continuation == nil)
        local original = {char="a"}
        debug.assert(info.process(original) == original)
        for _, options in ipairs({{width=0}, {height=65536}, {scope="unknown"}, {process=false}, {include_continuation_cells=1}, {include_continuation=true}, {unknown=true}}) do
          debug.assert(not debug.pcall(effect.set, ef, options))
          debug.assert(effect.get_info(ef).width == 3)
        end
        for _, invalid in ipairs({0, -1, 65536, 1.5, "3", false}) do
          debug.assert(not debug.pcall(effect.create, invalid, 1))
        end
        debug.assert(not debug.pcall(effect.create, 1, 1, {include_continuation=true}))
        debug.assert(not debug.pcall(effect.effect, 0, 0, 1, 1, {include_continuation=true}))
        local process = function(cell) return {char="changed"} end
        debug.assert(effect.set(ef, {process=process, include_continuation_cells=true}))
        debug.assert(effect.set(ef, {process=nil}))
        debug.assert(effect.get_info(ef).process == process)
        debug.assert(effect.list()[1].include_continuation_cells == true)
        debug.assert(effect.list()[1].include_continuation == nil)
        debug.assert(effect.set(ef, {process=info.process, include_continuation_cells=false}))
        debug.assert(effect.list().n == 1 and effect.list()[1].id == ef)
        debug.assert(effect.exists(ef) and not effect.exists("effect_999"))
        assert_missing("effect_999")
        debug.assert(effect.count() == 1 and effect.get_info(ef).width == 3)
        debug.assert(effect.set(ef, {process=nil, scope=effect.CANVAS}))
      end
      function Render()
        effect.apply(ef, 2, 1, {slice_layer="ignored"})
        draw.text(2, 1, "abc")
        effect.effect(-2147483648, 2147483647, 65535, 65535)
      end
      function Update(dt)
        debug.assert(effect.delete(ef) and effect.count() == 0)
        assert_missing(ef)
        debug.assert(effect.create(1,1) == "effect_002")
        debug.assert(effect.clear() and effect.list().n == 0)
        assert_missing("effect_002")
      end
    "#,
      kind,
    );
    let frame = harness.render().unwrap();
    assert_eq!(
      (2..5).map(|x| text(&frame, x, 1)).collect::<String>(),
      "abc"
    );
    assert!(matches!(frame.get(0,0), Some(ComposedCell::Text(cell)) if cell.text == "H"));
    harness.session.update().unwrap();
    harness.session.stop();
    assert!(!harness.session.has_objects());
  }
}

fn staged_order() {
  let mut harness = Harness::new(
    r#"
    function Init(ctx)
      layer = slice.create(4, 2)
      order = ""
      ef = effect.create(1, 1, {scope=effect.SLICE, process=function(cell)
        debug.assert(cell.char == "@" and cell.x == 0 and cell.y == 0)
        debug.assert(effect.count() == 1)
        debug.assert(random.randint({min=0, max=1}) >= 0)
        order = order .. "B"
        return {char="B"}
      end})
    end
    function Render()
      order = ""
      effect.effect(2,1,1,1,{process=function(cell)
        debug.assert(cell.char == "C")
        order = order .. "A"
        return {char="D"}
      end})
      effect.apply(ef,0,0,{slice_layer=layer})
      effect.effect(0,0,1,1,{scope=effect.SLICE,slice_layer=layer,process=function(cell)
        debug.assert(cell.char == "B")
        order = order .. "C"
        return {char="C"}
      end})
      draw.text(0,0,"@",{slice_layer=layer})
      slice.draw(layer,2,1)
      effect.effect(2,1,1,1,{process=function(cell)
        debug.assert(cell.char == "D" and cell.relative_x == 0)
        order = order .. "D"
        return {char="E"}
      end})
    end
    function Update(dt) debug.assert(order == "BCAD") end
  "#,
    LuaSessionKind::Game,
  );
  for _ in 0..2 {
    let frame = harness.render().unwrap();
    assert_eq!(text(&frame, 2, 1), "E");
    harness.session.update().unwrap();
  }
}

fn coordinates_and_snapshots() {
  let mut harness = Harness::new(
    r#"
    function Render()
      draw.text(1,0,"ab")
      effect.effect(1,0,2,1,{process=function(cell)
        if cell.char == "a" then return {x=5,relative_x=7} end
        return {relative_x=2}
      end})
      draw.text(0,2,"&")
      draw.text(0,3,"@")
      effect.effect(0,2,1,2,{process=function(cell) return {relative_y=cell.relative_y+1} end})
      draw.text(1,5,"abc")
      effect.effect(1,5,3,1,{process=function(cell) return {x=8} end})
      draw.text(10,0,"X")
      effect.effect(10,0,1,1,{process=function(cell) return {char=""} end})
    end
  "#,
    LuaSessionKind::Game,
  );
  let frame = harness.render().unwrap();
  assert_eq!(text(&frame, 1, 0), " ");
  assert_eq!(text(&frame, 2, 0), " ");
  assert_eq!(text(&frame, 3, 0), "b");
  assert_eq!(text(&frame, 5, 0), "a");
  assert_eq!(text(&frame, 0, 2), " ");
  assert_eq!(text(&frame, 0, 3), "&");
  assert_eq!(text(&frame, 0, 4), "@");
  assert_eq!(text(&frame, 8, 5), "c");
  assert_eq!(text(&frame, 10, 0), " ");
}

fn wide_graphemes_and_styles() {
  let mut harness = Harness::new(
    r#"
    function Init(ctx) calls=0 end
    function Render()
      draw.text(2,0,"我́a",{fg=color.RED,bg=color.BLUE})
      effect.effect(3,0,1,1,{scope=effect.SLICE,process=function(cell) debug.assert(false) end})
      effect.effect(3,0,1,1,{scope=effect.SLICE,include_continuation_cells=true,process=function(cell)
        calls=calls+1
        debug.assert(cell.char == "我́" and cell.x == 2 and cell.relative_x == -1)
        debug.assert(cell.fg == color.RED and cell.bg == color.BLUE)
        return {char="你́",fg=color.GREEN,bold=true,italic=true,underline=true,strike=true,blink=true,reverse=true,hidden=true,dim=true}
      end})
      draw.text(5,0," ")
      effect.effect(5,0,2,1,{scope=effect.SLICE,process=function(cell)
        if cell.x == 5 then debug.assert(cell.char == " "); return {} end
        debug.assert(cell.char == ""); return {char="Z"}
      end})
    end
    function Update(dt) debug.assert(calls == 1) end
  "#,
    LuaSessionKind::Game,
  );
  let frame = harness.render().unwrap();
  assert_eq!(text(&frame, 2, 0), "你́");
  assert_eq!(text(&frame, 4, 0), "a");
  assert_eq!(text(&frame, 6, 0), "Z");
  let Some(ComposedCell::Text(cell)) = frame.get(3, 2) else {
    panic!("missing styled cell")
  };
  let style = &cell.style;
  assert!(
    style.bold
      && style.italic
      && style.underline
      && style.strike
      && style.blink
      && style.reverse
      && style.hidden
      && style.dim
  );
  assert!(style.foreground.is_some() && style.background.is_some());
  harness.session.update().unwrap();
}

fn nested_containers() {
  let mut harness = Harness::new(
    r#"
    function Init(ctx)
      layer = slice.create(2,2)
      child = slice.create(2,1)
      box = widget.scrollbox.create(4,2)
    end
    function Render()
      draw.text(0,0,"abc",{slice_layer=child})
      effect.effect(0,0,3,1,{scope=effect.SLICE,slice_layer=child,process=function(cell)
        debug.assert(cell.x == 1 and cell.relative_x == 1)
        return {char="X"}
      end})
      slice.draw(child,-1,0,{slice_layer=layer})
      slice.draw(layer,3,2)
      draw.text(0,0,"B",{slice_layer=box})
      effect.effect(0,0,1,1,{scope=effect.SLICE,slice_layer=box,process=function(cell) return {char="C"} end})
      widget.scrollbox.draw(box,7,2)
    end
  "#,
    LuaSessionKind::Game,
  );
  let frame = harness.render().unwrap();
  assert_eq!(text(&frame, 3, 2), "X");
  assert_eq!(text(&frame, 7, 2), "C");
}

fn callback_failures() {
  for expression in [
    "false",
    "{x='wrong'}",
    "{relative_x=1.5}",
    "{bold=1}",
    "{fg='invalid'}",
    "{bg=false}",
    "{char='ab'}",
    "{char='\\n'}",
    "{unknown=true}",
  ] {
    let source = format!(
      "function Render() draw.text(0,0,'a'); effect.effect(0,0,1,1,{{process=function(cell) return {expression} end}}) end"
    );
    let mut harness = Harness::new(&source, LuaSessionKind::Game);
    assert!(
      harness.render().is_err(),
      "accepted invalid shader output: {expression}"
    );
    assert_eq!(harness.session.state(), LuaSessionState::Faulted);
    assert!(!harness.session.has_objects());
  }
  let mut harness = Harness::new(
    "function Render() effect.effect(0,0,1,1,{process=function(cell) while true do end end}) end",
    LuaSessionKind::Game,
  );
  let error = harness.render().unwrap_err();
  assert!(error.to_string().contains("execution limit"), "{error}");
  assert_eq!(harness.session.state(), LuaSessionState::Faulted);
  assert!(!harness.session.has_objects());
}
