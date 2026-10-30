//! Verify hitbox contracts through independent Lua sessions and their public APIs.

use std::{
  fs,
  path::PathBuf,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_service_layout::Size;
use tg_service_lua::{LuaService, LuaSessionKind, LuaSessionSpec};

struct Deployment(PathBuf);
impl Drop for Deployment {
  fn drop(&mut self) {
    fs::remove_dir_all(&self.0).expect("remove test deployment");
  }
}

#[test]
fn slice_targets_passthrough_coordinates_and_hidden_frames_follow_rendered_geometry() {
  use tg_core_input::{MouseButton, MouseEvent, MouseEventKind};
  use tg_service_canvas::CanvasService;
  use tg_service_layout::{LayoutService, Rect};
  use tg_service_lua::{
    LuaDrawCommand, LuaEventBroker, LuaEventData, LuaEventRoute, LuaSessionToken,
  };
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root = Deployment(
    std::env::temp_dir().join(format!("tg-hitbox-layer-{}-{nonce}", std::process::id())),
  );
  fs::create_dir_all(root.0.join("scripts")).unwrap();
  let entry = root.0.join("scripts/main.lua");
  fs::write(
    &entry,
    r#"
    function Init(ctx)
      owner = slice.create(6, 4)
      overlay = slice.create(1, 1)
      lower = widget.hitbox.create(5, 3, {drag = true})
      upper = widget.hitbox.create(1, 1)
      mode, clicks = 0, 0
    end
    function Render()
      if mode ~= 3 then slice.draw(owner, -1, 1) end
      slice.draw(overlay, 1, 2)
      widget.hitbox.draw(lower, mode == 5 and -2147483648 or -2, -1, {slice_layer = owner})
      if mode == 2 then widget.hitbox.draw(upper, 0, 0, {slice_layer = overlay}) end
    end
    function HandleEvent(event)
      debug.assert(event.type == "hitbox")
      local data = event.data
      if data.kind == "clicked" then
        clicks = clicks + 1
        if mode == 2 then
          debug.assert(data.id == upper and data.x == 0 and data.y == 0)
          debug.assert(data.relative_x == 0 and data.relative_y == 0)
        else
          debug.assert(data.id == lower and data.x == 2 and data.y == 1)
          debug.assert(data.relative_x == 4 and data.relative_y == 2)
        end
        debug.assert(data.dx == nil and data.dy == nil)
      end
      if mode == 5 then debug.assert(data.relative_x == 2147483650) end
    end
    function UpdateFrame(dt, alpha)
      debug.assert(clicks == ({0, 1, 2, 2, 3, 3})[mode + 1])
    end
    function Update(dt)
      mode = mode + 1
      if mode == 1 then slice.set(overlay, {event_passthrough = true}) end
      if mode == 4 then
        slice.set(overlay, {event_passthrough = false})
        slice.set(owner, {layer = 2})
      end
    end
  "#,
  )
  .unwrap();
  let mut session = LuaService::new()
    .create_session(LuaSessionSpec {
      package_id: "hitbox_layers".into(),
      session_kind: LuaSessionKind::Game,
      entry_path: entry,
      fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
      base_size: Size {
        width: 10,
        height: 6,
      },
      continue_data: None,
      best_data: None,
      save_game_enabled: false,
      save_best_enabled: false,
    })
    .unwrap();
  let mut layout = LayoutService::new();
  layout.resize_physical(10, 8);
  layout.set_developer_viewport(Rect {
    x: 0,
    y: 2,
    width: 10,
    height: 6,
  });
  let mut canvas = CanvasService::new();
  let token = LuaSessionToken {
    kind: LuaSessionKind::Game,
    generation: 1,
  };
  let mut broker = LuaEventBroker::new();
  broker.synchronize_sessions(Some(token), None);
  for mode in 0..=5 {
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
          if let LuaDrawCommand::Hitbox { id, target, x, y } = command {
            objects.draw_hitbox(id, target, (x, y), &canvas);
          }
        }
      })
      .unwrap();
    let mut clicked = 0;
    for kind in [MouseEventKind::Press, MouseEventKind::Release] {
      let events = session
        .with_objects_mut(|objects| {
          objects.route_hitbox_mouse(
            &canvas,
            MouseEvent {
              kind,
              button: Some(MouseButton::Left),
              scroll: None,
              x: 1,
              y: 4,
            },
          )
        })
        .unwrap();
      for event in events {
        if matches!(&event, LuaEventData::HitArea(data) if data.kind == "clicked") {
          clicked += 1;
        }
        broker
          .push_owned(token, mode + 1, event, LuaEventRoute::HandleEvent)
          .unwrap();
      }
    }
    assert_eq!(
      clicked,
      usize::from([1, 2, 4].contains(&mode)),
      "mode={mode}"
    );
    for delivery in broker.drain_frame(LuaSessionKind::Game) {
      session.dispatch_event(&delivery).unwrap();
    }
    session.update_frame(Duration::ZERO, 0.0).unwrap();
    session.update().unwrap();
  }
  session.stop();
  assert!(!session.has_objects());
}

#[test]
fn hitbox_options_lifecycle_limits_and_sessions_are_independent() {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root =
    Deployment(std::env::temp_dir().join(format!("tg-hitbox-test-{}-{nonce}", std::process::id())));
  fs::create_dir_all(root.0.join("scripts")).unwrap();
  let entry = root.0.join("scripts/main.lua");
  fs::write(
    &entry,
    r##"
    function Init(ctx)
      local h = widget.hitbox
      local function fails(call) return not select(1, debug.pcall(call)) end
      id = h.create(20, 10)
      debug.assert(id == "hitbox_001" and h.count() == 1 and h.exists(id))
      local info = h.get_info(id)
      debug.assert(info.id == id and info.width == 20 and info.height == 10)
      debug.assert(not info.hover_move and not info.drag and info.bg == nil)
      debug.assert(h.set(id, {width = 25, height = 5, hover_move = true, drag = true}))
      debug.assert(h.set(id, {drag = false}) and h.set(id, {}) and h.set(id, nil))
      info = h.get_info(id)
      debug.assert(info.width == 25 and info.height == 5 and info.hover_move and not info.drag)
      debug.assert(fails(function() h.set(id, {width = 1, height = 0}) end))
      debug.assert(h.get_info(id).width == 25)
      local second = h.create(1, 65535)
      local list = h.list()
      debug.assert(list.n == 2 and list[1].id == id and list[2].id == second)
      debug.assert(h.delete(id) and not h.delete(id) and not h.exists(id) and h.get_info(id) == nil)
      debug.assert(not h.set(id, {width = 2}) and not h.exists("hitbox_999"))
      for _, call in ipairs({
        function() h.create(0, 1) end,
        function() h.create(1, 65536) end,
        function() h.create(1.5, 2) end,
        function() h.create(1, 2, {drag = 1}) end,
        function() h.create(1, 2, {bg = color.BLUE}) end,
        function() h.create(1, 2, {[1] = true}) end,
        function() h.create(1, 2, setmetatable({}, {})) end,
        function() h.draw(second, 0, 0, {unknown = true}) end,
        function() h.draw(second, 0, 0, {slice_layer = "slice_999"}) end,
        function() h.draw(second, 0, 0, {slice_layer = false}) end,
        function() h.draw(second, 2147483648, 0) end,
        function() h.draw(second, 0, -2147483649) end,
        function() h.draw(id, 0, 0) end,
        function() h.exists("slice_001") end,
        function() h.count(1) end,
        function() h.list({}) end,
        function() h.clear({}) end,
        function() h.create = false end,
        function() widget.hitbox = false end
      }) do debug.assert(fails(call)) end
      debug.assert(select("#", h.draw(second, -2147483648, 2147483647)) == 0)
      local surface = slice.create(4, 3, {event_passthrough = true})
      debug.assert(slice.get_info(surface).event_passthrough == true)
      debug.assert(slice.set(surface, {event_passthrough = false}))
      debug.assert(slice.get_info(surface).event_passthrough == false)
      debug.assert(fails(function() slice.set(surface, {width = 1, event_passthrough = 1}) end))
      debug.assert(slice.get_info(surface).width == 4)
      debug.assert(fails(function() slice.create(1, 1, {event_passthrough = "yes"}) end))
      h.draw(second, -1, 0, {slice_layer = surface})
      debug.assert(h.clear() and h.clear() and h.count() == 0 and h.list().n == 0)
      for index = 1, 1024 do h.create(1, 1) end
      debug.assert(fails(function() h.create(1, 1) end) and h.count() == 1024)
      debug.assert(h.clear())
      survivor = h.create(2, 2)
      debug.assert(survivor ~= second and h.count() == 1)
    end
    function HandleEvent(event) end
    function Update(dt) end
    function UpdateFrame(dt, alpha) end
    function Render() end
  "##,
  )
  .unwrap();
  let mut sessions = Vec::new();
  for kind in [LuaSessionKind::Game, LuaSessionKind::Screensaver] {
    sessions.push(
      LuaService::new()
        .create_session(LuaSessionSpec {
          package_id: "hitbox_test".into(),
          session_kind: kind,
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
        })
        .unwrap(),
    );
  }
  let first_pool = sessions[0]
    .with_objects(|objects| objects.ui().id())
    .unwrap();
  let second_pool = sessions[1]
    .with_objects(|objects| objects.ui().id())
    .unwrap();
  assert_ne!(first_pool, second_pool);
  sessions[0].stop();
  assert!(!sessions[0].has_objects());
  assert!(sessions[1].has_objects());
  sessions[1].render().unwrap();
  sessions[1].stop();
}
