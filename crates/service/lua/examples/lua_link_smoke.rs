//! Exercise Lua links and input mouse defaults through independent public APIs.
//!
//! Run `cargo run -p tg-service-lua --example lua_link_smoke --locked`.
//! Require a writable temporary deployment directory. Mouse events are injected; activation
//! commands are inspected without opening external applications or changing the clipboard.

use std::{
  fs,
  path::PathBuf,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_core_input::{MouseButton, MouseEvent, MouseEventKind};
use tg_service_canvas::CanvasService;
use tg_service_layout::{LayoutService, Rect, Size};
use tg_service_lua::{
  LuaDrawCommand, LuaEventBroker, LuaEventData, LuaEventRoute, LuaHostCommand, LuaService,
  LuaSession, LuaSessionKind, LuaSessionSpec, LuaSessionToken,
};

struct Deployment(PathBuf);
impl Drop for Deployment {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
pub(crate) fn main() {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root =
    Deployment(std::env::temp_dir().join(format!("tg-lua-link-{}-{nonce}", std::process::id())));
  fs::create_dir_all(root.0.join("scripts")).unwrap();
  fs::create_dir_all(root.0.join("assets/folder")).unwrap();
  fs::write(root.0.join("assets/manual.txt"), "link smoke").unwrap();
  let entry = root.0.join("scripts/main.lua");
  fs::write(&entry,r#"
    function Init(ctx)
      first=widget.link.create("abcd","./manual.txt",{copy=true,hover_fg=color.RED})
      debug.assert(first=="link_001")
      local info=widget.link.get_info(first)
      debug.assert(info.fg==color.BRIGHT_BLUE and info.bg==color.NONE and info.underline and info.mouse)
      debug.assert(info.hover_fg==color.RED and info.hover_bg==color.NONE and info.copy)
      debug.assert(widget.link.count()==1 and widget.link.list().n==1)
      for _,options in ipairs({{link="../secret"},{link="C:/outside.txt"},{link="file:///outside"},
        {link="mailto:user@example.com"},{link="https://"},{text=""},{mouse=1},{copy=1},{fg=color.TRANSPARENT},{unknown=true}}) do
        debug.assert(not debug.pcall(widget.link.set,first,options))
      end
      debug.assert(widget.link.get_info(first).link=="./manual.txt")
      debug.assert(not debug.pcall(widget.link.create,"","x"))
      debug.assert(not debug.pcall(function() widget.link.draw=nil end))
      debug.assert(not debug.pcall(widget.link.draw,first,2147483648,0))
      debug.assert(widget.link.set(first,{mouse=false}))
      debug.assert(widget.link.set(first,{copy=true}))
      debug.assert(widget.link.get_info(first).mouse==false)
      debug.assert(widget.link.set(first,{mouse=true}))
      local deleted=widget.link.create("deleted","./missing",{mouse=false})
      debug.assert(widget.link.get_info(deleted).mouse==false)
      debug.assert(widget.link.click(deleted) and widget.link.delete(deleted))
      debug.assert(not widget.link.click(deleted) and not widget.link.exists(deleted))
      debug.assert(widget.link.get_info(deleted)==nil)
      input=widget.input.create(1,1)
      debug.assert(widget.input.get_info(input).mouse==true)
      local disabled=widget.input.create(1,1,{mouse=false})
      debug.assert(widget.input.get_info(disabled).mouse==false)
      widget.input.delete(disabled)
      owner=slice.create(3,2)
      overlay=slice.create(1,1)
      phase,clicks,input_focused=0,0,false
    end
    function Render()
      slice.draw(owner,0,0)
      slice.draw(overlay,0,0)
      widget.link.draw(first,-1,0,{slice_layer=owner})
      widget.input.draw(input,0,1,{slice_layer=owner})
    end
    function HandleEvent(event)
      if event.type=="link" then
        debug.assert(event.data.id==first and event.data.kind=="click")
        debug.assert(event.data.link=="./manual.txt" or event.data.link=="./folder")
        clicks=clicks+1
      elseif event.type=="input" then
        debug.assert(event.data.id==input)
        if event.data.kind=="focus" then debug.assert(widget.input.focus(input)) end
        if event.data.kind=="focused" then input_focused=true end
      end
    end
    function Update(dt)
      phase=phase+1
      if phase==1 then
        debug.assert(clicks==0)
        slice.set(overlay,{event_passthrough=true})
      elseif phase==2 then
        debug.assert(clicks==1 and input_focused)
        debug.assert(widget.link.set(first,{text="new",link="./folder",copy=false,mouse=false,fg=color.GREEN,bold=true}))
        local info=widget.link.get_info(first)
        debug.assert(info.hover_fg==color.RED and info.fg==color.GREEN and info.underline and info.bold and info.mouse==false)
        debug.assert(widget.link.click(first))
      elseif phase==3 then
        debug.assert(clicks==2)
        debug.assert(widget.link.click(first))
        debug.assert(widget.link.delete(first))
        debug.assert(widget.link.count()==0 and widget.link.clear())
        for index=1,1024 do widget.link.create("x","./folder") end
        debug.assert(not debug.pcall(widget.link.create,"x","./folder"))
        debug.assert(widget.link.clear() and widget.link.list().n==0)
        debug.assert(widget.input.clear())
      end
    end
    function UpdateFrame(dt,alpha) end
  "#).unwrap();
  let mut session = LuaService::new()
    .create_session(spec(entry.clone(), LuaSessionKind::Game))
    .unwrap();
  let token = LuaSessionToken {
    kind: LuaSessionKind::Game,
    generation: 1,
  };
  let mut broker = LuaEventBroker::new();
  broker.synchronize_sessions(Some(token), None);
  let mut layout = LayoutService::new();
  layout.resize_physical(12, 8);
  layout.set_developer_viewport(Rect {
    x: 0,
    y: 2,
    width: 12,
    height: 6,
  });
  let mut canvas = CanvasService::new();
  for phase in 0..2 {
    session
      .with_objects_mut(|objects| objects.begin_frame())
      .unwrap();
    session.render().unwrap();
    let commands = session.take_draw_commands();
    canvas.begin_frame(&layout);
    session
      .with_objects_mut(|objects| {
        objects.ui_mut().begin_render();
        objects.ui().prepare_canvas(&mut canvas, &layout);
        for command in commands {
          match command {
            LuaDrawCommand::Link { id, target, x, y } => {
              objects.draw_link(id, target, (x, y), &mut canvas)
            }
            LuaDrawCommand::Input { id, target, x, y } => {
              objects.draw_input(id, target, (x, y), &mut canvas)
            }
            _ => {}
          }
        }
      })
      .unwrap();
    let mut all = Vec::new();
    for kind in [MouseEventKind::Press, MouseEventKind::Release] {
      all.extend(
        session
          .with_objects_mut(|objects| objects.route_widget_mouse(&canvas, mouse(kind, 0, 2)))
          .unwrap(),
      );
    }
    assert_eq!(
      all
        .iter()
        .filter(|event| matches!(event, LuaEventData::Hyperlink(_)))
        .count(),
      phase
    );
    deliver(&mut session, &mut broker, token, all);
    if phase == 0 {
      session.update().unwrap();
    }
  }
  let requests = take_requests(&mut session);
  assert_eq!(requests.len(), 1);
  assert_eq!(requests[0].0, root.0.join("assets"));
  assert_eq!(requests[0].1, "./manual.txt");
  assert!(requests[0].2);
  assert!(tg_service_widget::resolve_link_target(&requests[0].0, &requests[0].1).is_some());
  let events = session
    .with_objects_mut(|objects| {
      objects.route_widget_mouse(&canvas, mouse(MouseEventKind::Press, 0, 3))
    })
    .unwrap();
  deliver(&mut session, &mut broker, token, events);
  flush(&mut session, &mut broker, token);
  session.update().unwrap();
  flush(&mut session, &mut broker, token);
  let requests = take_requests(&mut session);
  assert_eq!(requests.len(), 1);
  assert_eq!(requests[0].1, "./folder");
  assert!(!requests[0].2);
  session.update().unwrap();
  flush(&mut session, &mut broker, token);
  assert!(take_requests(&mut session).is_empty());
  let stale = LuaEventData::Hyperlink(tg_service_lua::LuaHyperlinkEvent {
    id: 1,
    link: "https://example.com".into(),
  });
  deliver(&mut session, &mut broker, token, vec![stale]);
  assert!(take_requests(&mut session).is_empty());
  session.stop();
  assert!(session.with_objects(|_| ()).is_none());
  fs::write(
    &entry,
    r#"
    function Init(ctx)
      local id = widget.link.create("saver", "./folder")
      debug.assert(id == "link_001")
      debug.assert(widget.link.click(id) == false)
    end
    function Render() end
    function Update(dt) end
    function UpdateFrame(dt, alpha) end
    function HandleEvent(event) end
  "#,
  )
  .unwrap();
  let mut saver = LuaService::new()
    .create_session(spec(entry, LuaSessionKind::Screensaver))
    .unwrap();
  assert!(
    saver
      .with_objects_mut(|objects| objects.take_widget_events())
      .unwrap()
      .is_empty()
  );
  assert!(take_requests(&mut saver).is_empty());
  saver.stop();
  println!(
    "Lua link ok: strict API, slice clipping and passthrough, input mouse defaults, click events, activation requests and cleanup"
  );
}
fn spec(entry_path: PathBuf, session_kind: LuaSessionKind) -> LuaSessionSpec {
  LuaSessionSpec {
    package_id: "lua_link_smoke".into(),
    session_kind,
    entry_path,
    fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
    base_size: Size {
      width: 12,
      height: 6,
    },
    continue_data: None,
    best_data: None,
    save_game_enabled: false,
    save_best_enabled: false,
  }
}
fn mouse(kind: MouseEventKind, x: u16, y: u16) -> MouseEvent {
  MouseEvent {
    kind,
    button: Some(MouseButton::Left),
    scroll: None,
    x,
    y,
  }
}
fn flush(session: &mut LuaSession, broker: &mut LuaEventBroker, token: LuaSessionToken) {
  let events = session
    .with_objects_mut(|objects| objects.take_widget_events())
    .unwrap();
  deliver(session, broker, token, events);
}
fn deliver(
  session: &mut LuaSession,
  broker: &mut LuaEventBroker,
  token: LuaSessionToken,
  events: Vec<LuaEventData>,
) {
  for data in events {
    broker
      .push_owned(token, 1, data, LuaEventRoute::HandleEvent)
      .unwrap();
  }
  for delivery in broker.drain_frame(LuaSessionKind::Game) {
    session.dispatch_event(&delivery).unwrap();
  }
}
fn take_requests(session: &mut LuaSession) -> Vec<(PathBuf, String, bool)> {
  session
    .take_host_commands()
    .into_iter()
    .filter_map(|command| {
      if let LuaHostCommand::OpenLink {
        assets_root,
        target,
        copy,
      } = command
      {
        Some((assets_root, target, copy))
      } else {
        None
      }
    })
    .collect()
}
