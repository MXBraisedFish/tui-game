//! Exercise Lua scrollboxes, nested layers, paint order and events without the host.
//!
//! Run `cargo run -p tg-service-lua --example lua_scrollbox_smoke --locked`.
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
  LuaEventBroker, LuaEventData, LuaEventRoute, LuaService, LuaSession, LuaSessionKind,
  LuaSessionSpec, LuaSessionToken,
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
fn hit(events: &[LuaEventData], id: u64) -> bool {
  events
    .iter()
    .any(|event| matches!(event,LuaEventData::HitArea(data) if data.id==id && data.kind=="pressed"))
}

pub(crate) fn main() {
  lifecycle_and_order();
  nested_and_occlusion();
  content_shrink_and_limits();
}
pub(crate) fn lifecycle_and_order() {
  let mut harness = Harness::new(
    r#"
    function Init(ctx)
      box=widget.scrollbox.create(6,4)
      debug.assert(box=="scrollbox_001" and widget.scrollbox.count()==1 and widget.scrollbox.list().n==1)
      local info=widget.scrollbox.get_info(box)
      debug.assert(info.mouse and info.include_scrollbar and info.wheel_step_vertical==2 and info.wheel_step_horizontal==3)
      for _,options in ipairs({{width=0},{height=65536},{mouse=1},{track_shape_vertical="wide"},{thumb_shape_horizontal="界"},{wheel_step_vertical=0},{unknown=true},{scroll_offset_vertical=widget.scrollbox.TO_LEFT}}) do
        debug.assert(not debug.pcall(widget.scrollbox.set,box,options))
      end
      debug.assert(widget.scrollbox.get_info(box).width==6)
      debug.assert(not debug.pcall(function()widget.scrollbox.TO_TOP=nil end))
      before=widget.hitbox.create(8,5,{drag=true});inside=widget.hitbox.create(8,8);after=widget.hitbox.create(1,1)
      phase=0;changed=0
    end
    function Render()
      widget.hitbox.draw(before,0,0)
      draw.fill_rect(0,0,20,10,{char="C",slice_layer=box})
      if phase>=2 then draw.erase_rect(0,0,20,10,{slice_layer=box}) end
      widget.hitbox.draw(inside,-1,-1,{slice_layer=box})
      widget.scrollbox.draw(box,1,0)
      draw.text(1,0,"B")
      widget.hitbox.draw(after,1,0)
    end
    function HandleEvent(event)
      if event.type=="scrollbox" then
        debug.assert(event.data.id==box and event.data.kind=="scrolled")
        debug.assert(event.data.scroll_offset_horizontal~=nil and event.data.scroll_offset_vertical~=nil)
        debug.assert(event.data.x==nil and event.data.y==nil)
        changed=changed+1
      end
    end
    function Update(dt)
      phase=phase+1
      if phase==1 then
        local info=widget.scrollbox.get_info(box)
        debug.assert(info.content_width==5 and info.content_height==3 and info.scrollbox_width==6 and info.scrollbox_height==4)
        debug.assert(widget.scrollbox.scroll(box,widget.scrollbox.TO_BOTTOM,widget.scrollbox.TO_RIGHT))
      elseif phase==2 then
        local info=widget.scrollbox.get_info(box)
        debug.assert(changed==1 and info.scroll_offset_vertical==7 and info.scroll_offset_horizontal==15)
        debug.assert(widget.scrollbox.set(box,{width=30,height=20}))
      elseif phase==3 then
        local info=widget.scrollbox.get_info(box)
        debug.assert(changed==2 and info.scroll_offset_vertical==0 and info.scroll_offset_horizontal==0)
        debug.assert(info.content_width==30 and info.content_height==20)
        debug.assert(widget.scrollbox.delete(box) and not widget.scrollbox.exists(box))
        debug.assert(not widget.scrollbox.scroll(box,0,0) and not widget.scrollbox.set(box,{mouse=false}))
        debug.assert(widget.scrollbox.get_info(box)==nil and widget.scrollbox.clear() and widget.scrollbox.list().n==0)
      end
    end
    function UpdateFrame(dt,alpha) end
  "#,
  );
  harness.render();
  assert_eq!(harness.cell(1, 2), "B");
  assert_eq!(harness.cell(2, 2), "C");
  assert!(hit(&harness.mouse(1, 2, MouseEventKind::Press, None), 3));
  harness.mouse(1, 2, MouseEventKind::Release, None);
  let events = harness.mouse(2, 2, MouseEventKind::Press, None);
  assert!(hit(&events, 2));
  assert!(!hit(&events, 1));
  assert!(events.iter().any(|event|matches!(event,LuaEventData::HitArea(data) if data.kind=="pressed" && data.x==1 && data.y==0 && data.relative_x==2 && data.relative_y==1)));
  harness.mouse(2, 2, MouseEventKind::Release, None);
  assert!(hit(&harness.mouse(0, 2, MouseEventKind::Press, None), 1));
  let events = harness.mouse(2, 2, MouseEventKind::Drag, None);
  assert!(!events.iter().any(
    |event| matches!(event,LuaEventData::HitArea(data) if data.id==1 && data.kind=="dragged")
  ));
  harness.mouse(2, 2, MouseEventKind::Release, None);
  harness.mouse(0, 2, MouseEventKind::Press, None);
  harness.mouse(6, 2, MouseEventKind::Release, None);
  let events = harness.mouse(0, 2, MouseEventKind::Drag, None);
  assert!(!events.iter().any(
    |event| matches!(event,LuaEventData::HitArea(data) if data.id==1 && data.kind=="dragged")
  ));
  // The scrollbar consumes the lower base hitbox even without its own button event.
  assert!(!hit(&harness.mouse(6, 2, MouseEventKind::Press, None), 1));
  harness.mouse(6, 2, MouseEventKind::Release, None);
  harness.update();
  harness.render();
  let events = harness.events();
  harness.deliver(events);
  harness.update();
  harness.render();
  let events = harness.events();
  harness.deliver(events);
  harness.update();
}
pub(crate) fn nested_and_occlusion() {
  let mut harness = Harness::new(
    r#"
    function Init(ctx)
      outer=widget.scrollbox.create(8,5)
      inner=widget.scrollbox.create(3,2,{include_scrollbar=false,mouse=false})
      owner=slice.create(4,2,{event_passthrough=true})
      cover=slice.create(1,1,{layer=10})
      child=widget.hitbox.create(10,10)
      phase=0
    end
    function Render()
      draw.fill_rect(0,0,20,12,{char="o",slice_layer=outer})
      draw.fill_rect(0,0,8,8,{char="i",slice_layer=inner})
      widget.hitbox.draw(child,-1,-1,{slice_layer=inner})
      widget.scrollbox.draw(inner,1,0,{slice_layer=owner})
      slice.draw(owner,1,1,{slice_layer=outer})
      widget.scrollbox.draw(outer,0,0)
      slice.draw(cover,2,1)
      debug.assert(not debug.pcall(widget.scrollbox.draw,outer,0,0,{slice_layer=inner}))
      debug.assert(not debug.pcall(slice.draw,owner,0,0,{slice_layer=owner}))
    end
    function Update(dt)
      phase=phase+1
      if phase==1 then widget.scrollbox.set(inner,{mouse=true});slice.set(cover,{event_passthrough=true}) end
    end
    function HandleEvent(event) end
    function UpdateFrame(dt,alpha) end
  "#,
  );
  harness.render();
  assert_eq!(harness.cell(3, 3), "i");
  assert!(!hit(&harness.mouse(2, 3, MouseEventKind::Press, None), 1));
  harness.mouse(2, 3, MouseEventKind::Release, None);
  assert!(hit(&harness.mouse(3, 3, MouseEventKind::Press, None), 1));
  harness.mouse(3, 3, MouseEventKind::Release, None);
  // mouse=false leaves the child interactive and lets an eligible ancestor handle the wheel.
  let events = harness.mouse(3, 3, MouseEventKind::Scroll, Some(ScrollDirection::Down));
  assert!(
    events
      .iter()
      .any(|event| matches!(event,LuaEventData::ScrollBox(data) if data.id==1 && data.y==2))
  );
  harness.update();
  harness.render();
  // Enabled nested containers consume their own wheel events before an ancestor.
  // The outer offset is two rows, so the child's clipped visible row is now physical row 2.
  let events = harness.mouse(3, 2, MouseEventKind::Scroll, Some(ScrollDirection::Down));
  assert!(
    events
      .iter()
      .any(|event| matches!(event,LuaEventData::ScrollBox(data) if data.id==2 && data.y==2))
  );
  assert!(
    !events
      .iter()
      .any(|event| matches!(event,LuaEventData::ScrollBox(data) if data.id==1))
  );
}

pub(crate) fn content_shrink_and_limits() {
  let mut harness = Harness::new(
    r#"
    function Init(ctx)
      box=widget.scrollbox.create(5,3)
      child=widget.input.create(2,2,{text="ok"})
      link=widget.link.create("abc","https://example.com")
      phase=0
      widget.scrollbox.scroll(box,widget.scrollbox.TO_BOTTOM,0)
    end
    function Render()
      if phase==0 then
        draw.text(0,9,"text",{slice_layer=box})
        widget.link.draw(link,7,0,{slice_layer=box})
        widget.input.draw(child,0,12,{slice_layer=box})
      else draw.text(0,0,"x",{slice_layer=box}) end
      widget.scrollbox.draw(box,-1,-1)
    end
    function Update(dt)
      phase=phase+1
      local info=widget.scrollbox.get_info(box)
      if phase==1 then debug.assert(info.scroll_offset_vertical==12 and info.content_width==4 and info.content_height==2)
      elseif phase==2 then
        debug.assert(info.scroll_offset_vertical==0 and info.content_width==5 and info.content_height==3)
        widget.scrollbox.clear()
        for index=1,1024 do widget.scrollbox.create(1,1) end
        debug.assert(not debug.pcall(widget.scrollbox.create,1,1))
        debug.assert(widget.scrollbox.clear() and widget.scrollbox.count()==0)
      end
    end
    function UpdateFrame(dt,alpha) end
    function HandleEvent(event) end
  "#,
  );
  harness.render();
  harness.update();
  harness.render();
  harness.update();
}
