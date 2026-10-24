//! Exercise Lua hitboxes, clipping, events, and cleanup without starting the host.
//!
//! Run `cargo run -p tg-service-lua --example lua_hitbox_smoke --locked`.
//! Require a writable temporary directory. No terminal or mouse device is required;
//! pointer input is supplied through the public service API rather than a physical device.

use std::{
  fs,
  path::PathBuf,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_core_input::{MouseButton, MouseEvent, MouseEventKind};
use tg_service_canvas::CanvasService;
use tg_service_layout::{LayoutService, Rect, Size};
use tg_service_lua::{
  LuaDrawCommand, LuaEventBroker, LuaEventRoute, LuaService, LuaSessionKind, LuaSessionSpec,
  LuaSessionToken,
};

struct Deployment(PathBuf);
impl Drop for Deployment {
  fn drop(&mut self) {
    fs::remove_dir_all(&self.0).expect("remove smoke deployment");
  }
}

fn main() {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root = Deployment(
    std::env::temp_dir().join(format!("tg-hitbox-smoke-{}-{nonce}", std::process::id())),
  );
  fs::create_dir_all(root.0.join("scripts")).unwrap();
  let entry = root.0.join("scripts/main.lua");
  fs::write(&entry, r#"
    function Init(ctx)
      first = widget.hitbox.create(5, 4, {hover_move = true, drag = true})
      top = widget.hitbox.create(1, 1)
      surface = slice.create(3, 2, {event_passthrough = true})
      sliced = widget.hitbox.create(3, 2)
      slice_clicks = 0
      events = {}
    end
    function Render()
      slice.draw(surface, 4, 1)
      widget.hitbox.draw(sliced, -1, 0, {slice_layer = surface})
      widget.hitbox.draw(first, -2, -1)
      widget.hitbox.draw(top, 0, 0)
    end
    function HandleEvent(event)
      debug.assert(event.type == "hitbox" and event.sequence > 0)
      local data = event.data
      if data.id == sliced then
        if data.kind == "click" then
          debug.assert(data.x == 0 and data.y == 0 and data.relative_x == 1 and data.relative_y == 0)
          slice_clicks = slice_clicks + 1
        end
        return
      end
      debug.assert(data.id == first or data.id == top)
      debug.assert(data.relative_x == data.x + (data.id == first and 2 or 0))
      debug.assert(data.relative_y == data.y + (data.id == first and 1 or 0))
      table.insert(events, data.kind)
      if data.kind == "press" then
        debug.assert(data.button == "left")
        debug.assert(data.id == (data.x == 0 and top or first))
        debug.assert(data.y == (data.x == 0 and 0 or 1))
      end
      if data.kind == "drag" then debug.assert(data.id == first and data.dx == 1 and data.dy == 0) end
    end
    function Update(dt)
      local sequence = table.concat(events, {separator = ","})
      debug.assert(sequence == "hover_enter,press,release,click,hover_leave,hover_enter,press,drag,hover_move,release,click", {message = sequence})
      debug.assert(widget.hitbox.delete(first) and widget.hitbox.clear())
      debug.assert(widget.hitbox.count() == 0)
    end
    function UpdateFrame(dt, alpha) debug.assert(#events == 11 and slice_clicks == 1) end
  "#).unwrap();
  let mut session = LuaService::new()
    .create_session(LuaSessionSpec {
      package_id: "hitbox_smoke".into(),
      session_kind: LuaSessionKind::Game,
      entry_path: entry,
      fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
      base_size: Size {
        width: 8,
        height: 4,
      },
      continue_data: None,
      best_data: None,
      save_game_enabled: false,
      save_best_enabled: false,
    })
    .unwrap();
  let mut canvas = CanvasService::new();
  let mut layout = LayoutService::new();
  layout.resize_physical(8, 6);
  layout.set_developer_viewport(Rect {
    x: 0,
    y: 2,
    width: 8,
    height: 4,
  });
  canvas.begin_frame(&layout);
  session.render().unwrap();
  let commands = session.take_draw_commands();
  session
    .with_objects_mut(|objects| {
      objects.ui_mut().begin_render();
      objects.ui().prepare_canvas(&mut canvas, &layout);
      for command in commands {
        if let LuaDrawCommand::Hitbox { id, target, x, y } = command {
          objects.draw_hitbox(id, target, (x, y), &canvas);
        }
      }
    })
    .unwrap();
  let token = LuaSessionToken {
    kind: LuaSessionKind::Game,
    generation: 1,
  };
  let mut broker = LuaEventBroker::new();
  broker.synchronize_sessions(Some(token), None);
  for (kind, x, y) in [
    (MouseEventKind::Press, 4, 3),
    (MouseEventKind::Release, 4, 3),
    (MouseEventKind::Press, 0, 2),
    (MouseEventKind::Release, 0, 2),
    (MouseEventKind::Press, 1, 3),
    (MouseEventKind::Drag, 2, 3),
    (MouseEventKind::Release, 2, 3),
  ] {
    let events = session
      .with_objects_mut(|objects| {
        objects.route_hitbox_mouse(
          &canvas,
          MouseEvent {
            kind,
            button: Some(MouseButton::Left),
            scroll: None,
            x,
            y,
          },
        )
      })
      .unwrap();
    for event in events {
      broker
        .push_owned(token, 1, event, LuaEventRoute::HandleEvent)
        .unwrap();
    }
  }
  let deliveries = broker.drain_frame(LuaSessionKind::Game);
  for delivery in &deliveries {
    session.dispatch_event(delivery).unwrap();
  }
  session.update().unwrap();
  for delivery in &deliveries {
    session.dispatch_event(delivery).unwrap();
  }
  session.update_frame(Duration::ZERO, 0.0).unwrap();
  let events = session
    .with_objects_mut(|objects| {
      objects.route_hitbox_mouse(
        &canvas,
        MouseEvent {
          kind: MouseEventKind::Press,
          button: Some(MouseButton::Left),
          scroll: None,
          x: 0,
          y: 2,
        },
      )
    })
    .unwrap();
  assert!(events.is_empty());
  session.stop();
  assert!(!session.has_objects());
  println!(
    "hitbox ok: Lua API, topmost hits, viewport clipping, hover/drag/click, broker delivery and cleanup"
  );
}
