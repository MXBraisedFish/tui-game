//! Exercise Markdown conversion, scrolling, link activation and clipping without the host.
//!
//! Run `cargo run -p tg-service-lua --example lua_markdown_smoke --locked`.
//! Require a writable temporary deployment directory; no terminal or devices are needed.

use std::{
  fs,
  path::PathBuf,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_core_input::{MouseButton, MouseEvent, MouseEventKind, ScrollDirection};
use tg_service_canvas::CanvasService;
use tg_service_layout::{LayoutService, Rect, Size};
use tg_service_lua::{
  LuaEventBroker, LuaEventData, LuaEventRoute, LuaHostCommand, LuaService, LuaSession,
  LuaSessionKind, LuaSessionSpec, LuaSessionToken,
};
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
    let deployment =
      Deployment(std::env::temp_dir().join(format!("tg-scrollbox-{}-{nonce}", std::process::id())));
    fs::create_dir_all(deployment.0.join("scripts")).unwrap();
    fs::create_dir_all(deployment.0.join("assets")).unwrap();
    let entry = deployment.0.join("scripts/main.lua");
    fs::write(&entry, source).unwrap();
    let session = LuaService::new()
      .create_session(LuaSessionSpec {
        package_id: "scrollbox_smoke".into(),
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
        objects.replay_draw_commands(&mut RenderService::new(), &mut self.canvas, commands);
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
  fn deliver(&mut self, events: Vec<LuaEventData>) {
    let token = LuaSessionToken {
      kind: LuaSessionKind::Game,
      generation: 1,
    };
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(Some(token), None);
    for event in events {
      broker
        .push_owned(token, 1, event, LuaEventRoute::HandleEvent)
        .unwrap();
    }
    for delivery in broker.drain_frame(LuaSessionKind::Game) {
      self.session.dispatch_event(&delivery).unwrap();
    }
  }
  fn events(&mut self) -> Vec<LuaEventData> {
    self
      .session
      .with_objects_mut(|objects| objects.take_widget_events())
      .unwrap()
  }
  fn update(&mut self) {
    self.session.update().unwrap();
  }
}

pub(crate) fn main() {
  let mut harness = Harness::new(
    r#"
    function Init(ctx)
      id=widget.markdown.create(10,5,"[Open](./manual.txt)\n\n- long list with words that wrap onto several lines\n\n```lua\n    print(123456789)\n```",{wheel_step_horizontal=4})
      debug.assert(id=="markdown_001" and widget.markdown.count()==1 and widget.markdown.list().n==1)
      debug.assert(widget.scrollbox.count()==0 and widget.scrollbox.list().n==0)
      debug.assert(not widget.scrollbox.exists("scrollbox_001"))
      local external=widget.scrollbox.create(2,2)
      debug.assert(external=="scrollbox_001" and widget.scrollbox.clear() and widget.markdown.exists(id))
      for _,options in ipairs({{width=0},{height=65536},{allow_overflow=1},{unknown=true},{wheel_step_horizontal=0},{mouse=false},{bg="none"},{include_scrollbar=false}}) do debug.assert(not debug.pcall(widget.markdown.set,id,options)) end
      debug.assert(not debug.pcall(function()widget.markdown.draw=nil end))
      rich=widget.markdown.markdown_to_rich_text("**bold** <fg:red>{value}")
      debug.assert(type(rich)=="string")
      phase=0;clicks=0;scrolled=0
    end
    function Render() widget.markdown.draw(id,0,0);draw.text(0,5,rich) end
    function HandleEvent(event)
      debug.assert(event.type=="markdown" and event.data.id==id)
      if event.data.kind=="click" then debug.assert(event.data.link=="./manual.txt" and event.data.text=="Open");clicks=clicks+1
      elseif event.data.kind=="scroll" then debug.assert(event.data.scroll_offset_horizontal~=nil and event.data.scroll_offset_vertical~=nil);scrolled=scrolled+1 end
    end
    function Update(dt)
      phase=phase+1
      local info=widget.markdown.get_info(id)
      if phase==1 then
        debug.assert(clicks==1 and info.width==10 and info.height==5 and not info.allow_overflow and info.wheel_step_horizontal==4)
        debug.assert(info.width==10 and info.scroll_offset_horizontal==0 and info.bg==nil and info.mouse==nil and info.content_width==nil)
        debug.assert(widget.markdown.scroll(id,0,widget.scrollbox.TO_RIGHT))
        debug.assert(widget.markdown.set(id,{allow_overflow=true,markdown="abcdefghijklmnopqrstuv\n\none\n\ntwo\n\nthree\n\nfour\n\nfive"}))
        debug.assert(widget.markdown.scroll(id,widget.scrollbox.TO_BOTTOM,widget.scrollbox.TO_RIGHT))
      elseif phase==2 then
        debug.assert(info.scroll_offset_horizontal>0 and info.scroll_offset_vertical>0 and scrolled>0)
        debug.assert(widget.markdown.set(id,{allow_overflow=false,markdown="small"}))
      elseif phase==3 then
        debug.assert(info.scroll_offset_horizontal==0 and info.scroll_offset_vertical==0 and info.width==10 and info.height==5)
        debug.assert(widget.markdown.delete(id) and not widget.markdown.exists(id) and widget.markdown.get_info(id)==nil)
        debug.assert(not widget.markdown.scroll(id,1) and not widget.markdown.set(id,{width=1}))
        debug.assert(widget.markdown.clear() and widget.markdown.count()==0 and widget.scrollbox.count()==0)
      end
    end
    function UpdateFrame(dt,alpha) end
  "#,
  );
  harness.render();
  assert_eq!(harness.cell(0, 2), "O");
  assert_eq!(harness.cell(0, 7), "b");
  harness.mouse(1, 2, MouseEventKind::Press, None);
  harness.render();
  let events = harness.mouse(1, 2, MouseEventKind::Release, None);
  assert!(
    events
      .iter()
      .any(|event| matches!(event,LuaEventData::Markdown(data) if data.scroll_offset.is_none()))
  );
  harness.deliver(events);
  assert!(harness.session.take_host_commands().iter().any(|command|matches!(command,LuaHostCommand::OpenLink{target,copy:false,..} if target=="./manual.txt")));
  harness.update();
  harness.render();
  let events = harness.events();
  harness.deliver(events);
  harness.update();
  harness.render();
  let events = harness.events();
  harness.deliver(events);
  harness.update();
  nested_clipping();
}
fn nested_clipping() {
  let mut harness = Harness::new(
    r#"
    function Init(ctx)
      phase=0
      owner=slice.create(2,2)
      outer=widget.scrollbox.create(8,5)
      id=widget.markdown.create(5,5,"[Open](https://example.com)\n\n# Heading")
    end
    function Render()
      if phase==0 then widget.markdown.draw(id,0,0,{slice_layer=owner});slice.draw(owner,0,0,{slice_layer=outer});widget.scrollbox.draw(outer,1,0)
      else widget.markdown.draw(id,0,0) end
    end
    function Update(dt) phase=phase+1;slice.delete(owner) end
    function UpdateFrame(dt,alpha) end
    function HandleEvent(event) end
  "#,
  );
  harness.render();
  assert_eq!(harness.cell(1, 2), "O");
  assert_eq!(harness.cell(2, 2), "p");
  assert_eq!(harness.cell(3, 2), " ");
  harness.mouse(3, 2, MouseEventKind::Press, None);
  assert!(
    !harness
      .mouse(3, 2, MouseEventKind::Release, None)
      .iter()
      .any(|event| matches!(event, LuaEventData::Markdown(_)))
  );
  harness.mouse(1, 2, MouseEventKind::Press, None);
  assert!(
    harness
      .mouse(1, 2, MouseEventKind::Release, None)
      .iter()
      .any(|event| matches!(event, LuaEventData::Markdown(_)))
  );
  harness.mouse(1, 2, MouseEventKind::Press, None);
  harness.update();
  harness.render();
  assert!(
    !harness
      .mouse(0, 2, MouseEventKind::Release, None)
      .iter()
      .any(|event| matches!(event, LuaEventData::Markdown(_)))
  );
}
