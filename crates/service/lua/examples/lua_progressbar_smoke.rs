//! Exercise progress bar lifecycle, signed drawing, styles and container clipping without the host.
//!
//! Run `cargo run -p tg-service-lua --example lua_progressbar_smoke --locked`.
//! Require a writable temporary deployment directory; no terminal or devices are needed.

use std::{
  fs,
  path::PathBuf,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_core_input::{MouseButton, MouseEvent, MouseEventKind, ScrollDirection};
use tg_service_canvas::CanvasService;
use tg_service_layout::{LayoutService, Rect, Size};
use tg_service_lua::{LuaEventData, LuaService, LuaSession, LuaSessionKind, LuaSessionSpec};
use tg_service_render::RenderService;

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
  fn new(source: &str) -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let deployment = Deployment(
      std::env::temp_dir().join(format!("tg-progressbar-{}-{nonce}", std::process::id())),
    );
    fs::create_dir_all(deployment.0.join("scripts")).unwrap();
    fs::create_dir_all(deployment.0.join("assets")).unwrap();
    let entry = deployment.0.join("scripts/main.lua");
    fs::write(&entry, source).unwrap();
    let session = LuaService::new()
      .create_session(LuaSessionSpec {
        package_id: "progressbar_smoke".into(),
        session_kind: LuaSessionKind::Game,
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
    layout.resize_physical(12, 8);
    layout.set_developer_viewport(Rect {
      x: 0,
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
  fn render(&mut self) {
    self
      .session
      .with_objects_mut(|objects| objects.begin_frame())
      .unwrap();
    self.session.render().unwrap();
    let commands = self.session.take_draw_commands();
    self.canvas.begin_frame(&self.layout);
    self
      .session
      .with_objects_mut(|objects| {
        objects.ui_mut().begin_render();
        objects.ui().prepare_canvas(&mut self.canvas, &self.layout);
        objects
          .replay_draw_commands(&mut RenderService::new(), &mut self.canvas, commands)
          .unwrap();
      })
      .unwrap();
  }
  fn mouse(
    &mut self,
    x: u16,
    y: u16,
    kind: MouseEventKind,
    scroll: Option<ScrollDirection>,
  ) -> Vec<LuaEventData> {
    self
      .session
      .with_objects_mut(|objects| {
        objects.route_widget_mouse(
          &self.canvas,
          MouseEvent {
            x,
            y,
            kind,
            button: Some(MouseButton::Left),
            scroll,
          },
        )
      })
      .unwrap()
  }
  fn cell(&self, x: u16, y: u16) -> String {
    self.canvas.cell_at(x, y - 2).unwrap().text.clone()
  }
  fn update(&mut self) {
    self.session.update().unwrap();
  }
}

pub(crate) fn main() {
  let mut harness = Harness::new(
    r#"
    function Init(ctx)
      id=widget.progressbar.create(10,{completed_ratio=0.3,preview_ratio=0.6,completed_shape="C",preview_shape="P",remaining_shape="R"})
      debug.assert(id=="progressbar_001" and widget.progressbar.count()==1 and widget.progressbar.list().n==1)
      local info=widget.progressbar.get_info(id)
      debug.assert(info.width==10 and info.completed_ratio==0.3 and info.preview_ratio==0.6 and info.remaining_fg==color.WHITE)
      local defaults=widget.progressbar.create(20)
      local default_info=widget.progressbar.get_info(defaults)
      debug.assert(default_info.completed_shape=="█" and default_info.preview_shape=="█" and default_info.remaining_shape=="─")
      debug.assert(default_info.completed_fg==color.GREEN and default_info.preview_fg==color.BRIGHT_BLUE and default_info.remaining_fg==color.WHITE)
      debug.assert(widget.progressbar.delete(defaults))
      for _,options in ipairs({{width=0},{width=65536},{completed_ratio=-0.1},{preview_ratio=1.1},{completed_ratio=0/0},{preview_ratio=1/0},{completed_ratio="0.3"},{preview_ratio=false},{completed_shape=""},{preview_shape="ab"},{remaining_shape="界"},{completed_shape="\n"},{remaining_fg=color.TRANSPARENT},{completed_fg="bad"},{origin="right"},{mouse=true},{bg=color.BLUE}}) do
        debug.assert(not debug.pcall(widget.progressbar.set,id,options))
      end
      debug.assert(not debug.pcall(widget.progressbar.set,id,{width=20,completed_ratio=1,remaining_shape="界"}))
      debug.assert(widget.progressbar.get_info(id).width==10 and widget.progressbar.get_info(id).completed_ratio==0.3)
      debug.assert(not debug.pcall(widget.progressbar.create,0))
      debug.assert(not debug.pcall(widget.progressbar.create,10,{width=2}))
      debug.assert(not debug.pcall(widget.progressbar.draw,id,2147483648,0))
      debug.assert(not debug.pcall(function()widget.progressbar.draw=nil end))
      slice_id=slice.create(4,1)
      scroll=widget.scrollbox.create(4,2)
      phase=0
    end
    function Render()
      widget.progressbar.draw(id,-2,0)
      slice.draw(slice_id,0,1)
      widget.progressbar.draw(id,-2,0,{slice_layer=slice_id})
      widget.progressbar.draw(id,0,0,{slice_layer=scroll})
      widget.scrollbox.draw(scroll,0,2)
      widget.progressbar.draw(id,2147483647,2147483647)
      widget.progressbar.draw(id,-2147483648,-2147483648)
      if phase==0 then draw.text(0,0,"X") end
    end
    function Update(dt)
      phase=phase+1
      if phase==1 then
        debug.assert(widget.scrollbox.scroll(scroll,0,widget.scrollbox.TO_RIGHT))
        debug.assert(widget.progressbar.set(id,{completed_ratio=0.8,completed_fg=color.RED,remaining_fg=color.NONE}))
        local info=widget.progressbar.get_info(id)
        debug.assert(info.completed_ratio==0.8 and info.preview_ratio==0.6 and info.remaining_fg==color.NONE and info.completed_fg==color.RED)
        debug.assert(widget.progressbar.set(id,{completed_ratio=nil,preview_shape="q"}))
        debug.assert(widget.progressbar.get_info(id).completed_ratio==0.8)
      elseif phase==2 then
        debug.assert(widget.progressbar.delete(id) and not widget.progressbar.delete(id) and not widget.progressbar.exists(id))
        debug.assert(widget.progressbar.get_info(id)==nil and not widget.progressbar.set(id,{width=5}))
        debug.assert(widget.progressbar.clear() and widget.progressbar.count()==0 and widget.progressbar.list().n==0)
      end
    end
    function UpdateFrame(dt,alpha) end
    function HandleEvent(event) debug.assert(false) end
  "#,
  );
  harness.render();
  assert_eq!(
    (0..8).map(|x| harness.cell(x, 2)).collect::<String>(),
    "XPPPRRRR"
  );
  let slice_text = harness
    .canvas
    .prepared_surfaces()
    .find_map(|surface| match surface {
      tg_service_canvas::PreparedSurface::Slice(slice) => Some(slice.buffer.row_text(0)),
      _ => None,
    })
    .unwrap();
  assert_eq!(slice_text, "CPPP");
  // A simple drawing component registers no pointer region or events.
  assert!(harness.mouse(0, 2, MouseEventKind::Press, None).is_empty());
  harness.update();
  harness.render();
  assert_eq!(
    (0..8).map(|x| harness.cell(x, 2)).collect::<String>(),
    "CCCCCCRR"
  );
  assert_eq!(
    (0..3).map(|x| harness.cell(x, 4)).collect::<String>(),
    "CCR"
  );
  harness.update();
  fill_directions();
  screensaver_and_limits();
  markdown_container();
}
fn fill_directions() {
  let mut harness = Harness::new(
    r#"
    function Init(ctx)
      local progressbar = widget.progressbar
      directions = {align.LEFT, align.RIGHT, align.HORIZONTAL_CENTER, align.AUTO}
      bars = {}
      for index, direction in ipairs(directions) do
        bars[index] = progressbar.create(10, {
          completed_ratio=0.3, preview_ratio=0.6,
          completed_shape="C", preview_shape="P", remaining_shape="R",
          fill_direction=direction
        })
        debug.assert(progressbar.get_info(bars[index]).fill_direction == direction)
      end
      local defaults = progressbar.create(1)
      debug.assert(progressbar.get_info(defaults).fill_direction == align.LEFT)
      debug.assert(progressbar.delete(defaults))
      local centered = progressbar.create(10, {fill_direction=align.CENTER})
      debug.assert(progressbar.get_info(centered).fill_direction == align.CENTER)
      debug.assert(progressbar.delete(centered))
      local list = progressbar.list()
      debug.assert(list.n == 4)
      for index, direction in ipairs(directions) do
        debug.assert(list[index].fill_direction == direction)
      end
      for _, invalid in ipairs({align.TOP, align.VERTICAL_CENTER, align.BOTTOM, "LEFT", "unknown", false, 1}) do
        debug.assert(not debug.pcall(progressbar.create, 10, {fill_direction=invalid}))
        debug.assert(not debug.pcall(progressbar.set, bars[2], {width=3, fill_direction=invalid}))
        local info = progressbar.get_info(bars[2])
        debug.assert(info.width == 10 and info.fill_direction == align.RIGHT)
      end
      debug.assert(progressbar.count() == 4)
    end
    function Render()
      for index, id in ipairs(bars) do widget.progressbar.draw(id, 0, index-1) end
    end
    function Update(dt)
      debug.assert(widget.progressbar.set(bars[1], {fill_direction=align.RIGHT}))
      debug.assert(widget.progressbar.set(bars[2], {fill_direction=align.HORIZONTAL_CENTER}))
      debug.assert(widget.progressbar.set(bars[3], {fill_direction=align.CENTER}))
      debug.assert(widget.progressbar.set(bars[4], {fill_direction=nil, width=10}))
      debug.assert(widget.progressbar.get_info(bars[4]).fill_direction == align.AUTO)
      debug.assert(widget.progressbar.set(bars[2], {preview_ratio=0.6}))
      debug.assert(widget.progressbar.get_info(bars[2]).fill_direction == align.HORIZONTAL_CENTER)
      local list = widget.progressbar.list()
      debug.assert(list[1].fill_direction == align.RIGHT)
      debug.assert(list[2].fill_direction == align.HORIZONTAL_CENTER)
      debug.assert(list[3].fill_direction == align.CENTER)
      debug.assert(list[4].fill_direction == align.AUTO)
    end
    function UpdateFrame(dt, alpha) end
    function HandleEvent(event) debug.assert(false) end
  "#,
  );
  harness.render();
  for (row, expected) in ["CCCPPPRRRR", "RRRRPPPCCC", "RRPCCCPPRR", "CCCPPPRRRR"]
    .iter()
    .enumerate()
  {
    assert_eq!(
      (0..10)
        .map(|column| harness.cell(column, row as u16 + 2))
        .collect::<String>(),
      *expected
    );
  }
  harness.update();
  harness.render();
  for (row, expected) in ["RRRRPPPCCC", "RRPCCCPPRR", "RRPCCCPPRR", "CCCPPPRRRR"]
    .iter()
    .enumerate()
  {
    assert_eq!(
      (0..10)
        .map(|column| harness.cell(column, row as u16 + 2))
        .collect::<String>(),
      *expected
    );
  }
}
fn screensaver_and_limits() {
  let mut harness = Harness::new(
    r#"
    function Init(ctx)
      id=widget.progressbar.create(1,{completed_ratio=1})
      debug.assert(id=="progressbar_001")
      for index=2,1024 do widget.progressbar.create(1) end
      debug.assert(not debug.pcall(widget.progressbar.create,1))
    end
    function Render() widget.progressbar.draw(id,0,0) end
    function Update(dt) debug.assert(widget.progressbar.clear() and widget.progressbar.count()==0) end
    function UpdateFrame(dt,alpha) end
    function HandleEvent(event) end
  "#,
  );
  harness.render();
  assert_eq!(harness.cell(0, 2), "█");
  harness.update();
  let entry = harness._deployment.0.join("scripts/main.lua");
  let mut saver = LuaService::new()
    .create_session(LuaSessionSpec {
      package_id: "progressbar_saver".into(),
      session_kind: LuaSessionKind::Screensaver,
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
  saver.render().unwrap();
  let commands = saver.take_draw_commands();
  harness.canvas.begin_frame(&harness.layout);
  saver
    .with_objects_mut(|objects| {
      objects.ui_mut().begin_render();
      objects
        .ui()
        .prepare_canvas(&mut harness.canvas, &harness.layout);
      objects
        .replay_draw_commands(&mut RenderService::new(), &mut harness.canvas, commands)
        .unwrap();
    })
    .unwrap();
  assert_eq!(harness.cell(0, 2), "█");
  saver.stop();
}

fn markdown_container() {
  let mut harness = Harness::new(
    r#"
    function Init(ctx)
      id=widget.progressbar.create(8,{completed_ratio=1,completed_shape="Z"})
      owner=widget.markdown.create(5,3,"",{allow_overflow=true})
      widget.markdown.scroll(owner,widget.scrollbox.TO_BOTTOM,widget.scrollbox.TO_RIGHT)
    end
    function Render()
      widget.progressbar.draw(id,0,2,{slice_layer=owner})
      widget.markdown.draw(owner,0,0)
    end
    function Update(dt) end
    function UpdateFrame(dt,alpha) end
    function HandleEvent(event) end
  "#,
  );
  harness.render();
  assert_eq!(
    (0..4).map(|x| harness.cell(x, 3)).collect::<String>(),
    "ZZZZ"
  );
}
