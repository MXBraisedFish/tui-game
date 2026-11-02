//! Exercise arbitrary animation tracks, scheduling, easing callbacks, and cleanup independently.
//!
//! Run `cargo run -p tg-service-lua --example lua_animation_smoke --locked`.
//! A temporary deployment root is created; no terminal, assets, or wall-clock waits are required.

use std::{
  fs,
  path::PathBuf,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tg_service_layout::Size;
use tg_service_lua::{LuaService, LuaSession, LuaSessionKind, LuaSessionSpec};
use tg_service_time::TimeService;

struct Deployment(PathBuf);
impl Drop for Deployment {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn spec(entry: PathBuf, kind: LuaSessionKind) -> LuaSessionSpec {
  LuaSessionSpec {
    package_id: "animation_smoke".into(),
    session_kind: kind,
    entry_path: entry,
    fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
    base_size: Size {
      width: 80,
      height: 24,
    },
    continue_data: None,
    best_data: None,
    save_game_enabled: false,
    save_best_enabled: false,
  }
}

pub(crate) fn main() {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root = Deployment(
    std::env::temp_dir().join(format!("tg_animation_smoke_{}_{nonce}", std::process::id())),
  );
  fs::create_dir_all(root.0.join("scripts")).unwrap();
  let entry = root.0.join("scripts/main.lua");
  fs::write(&entry, r#"
    function Init(ctx)
      frame = 0
      local function fails(f) return not debug.pcall(f) end
      local empty = animation.create(1, {})
      debug.assert(empty == "animation_001" and animation.count() == 1)
      debug.assert(animation.sample(empty) == nil and animation.sample(empty, {key="missing"}) == nil)
      debug.assert(animation.exists(empty) and animation.get_info(empty).state == "idle")
      debug.assert(animation.list().n == 1 and animation.list()[1].id == empty)
      debug.assert(animation.delete(empty) and not animation.delete(empty))
      debug.assert(animation.get_info(empty) == nil and animation.sample(empty) == nil)
      debug.assert(not animation.play(empty) and not animation.pause(empty))
      debug.assert(animation.clear() and animation.clear())
      debug.assert(fails(function() animation.LINEAR = "other" end))
      for _, invalid in ipairs({false, -1, 1/0, 0/0, 18446744073709551616.0}) do
        debug.assert(fails(function() animation.create(invalid, {}) end))
      end
      for _, options in ipairs({{["repeat"]=false}, {["repeat"]=0}, {["repeat"]=2.5},
        {delay=-1}, {interval=-1}, {play_direction="sideways"}, {repeat_fill_mode="none"},
        {easing="unknown"}, {default_easing={x={animation.LINEAR}}}, {unknown=1}}) do
        debug.assert(fails(function() animation.create(1, {}, options) end))
      end
      for _, track in ipairs({{{timestamp=-1,key={x=0}}}, {{timestamp=2,key={x=0}}},
        {{timestamp="0",key={x=0}}}, {{timestamp=0,key={[1]=0}}}}) do
        debug.assert(fails(function() animation.create(1, track) end))
      end
      local sparse = animation.create(3, {
        {timestamp=2,key={a=2,b=1}}, {timestamp=0,key={b=3}},
        {timestamp=1,key={a=1,c=4}}, {timestamp=3,key={}}, {timestamp=1,key={a=99,discarded=1}},
      }, {easing={b={[1]=animation.LINEAR,[99]=animation.STEP}}, default_easing={a=animation.LINEAR}})
      local info = animation.get_info(sparse)
      debug.assert(#info.track == 4 and info.track[1].key.a == 1 and info.track[4].key.c == 4)
      debug.assert(info.track[2].key.b == nil and info.track[2].key.discarded == nil)
      debug.assert(animation.sample(sparse).b == 3 and animation.sample(sparse).a == 1)
      debug.assert(not debug.pcall(animation.set, sparse, {duration=1}))
      debug.assert(animation.get_info(sparse).duration == 3)
      debug.assert(animation.set(sparse, {track={{timestamp=0,key={x=0}},{timestamp=3,key={x=6}}}, easing=animation.LINEAR}))
      debug.assert(animation.play(sparse) and not animation.play(sparse))
      debug.assert(animation.pause(sparse) and animation.get_info(sparse).state == "paused")
      debug.assert(animation.play(sparse) and animation.reset(sparse))
      debug.assert(animation.get_info(sparse).state == "idle" and animation.replay(sparse))
      debug.assert(animation.delete(sparse))

      animations = {}
      for _, direction in ipairs({animation.NORMAL,animation.REVERSE,animation.ALTERNATE,animation.ALTERNATE_REVERSE}) do
        for _, fill in ipairs({animation.FORWARDS,animation.BACKWARDS}) do
          local id = animation.create(2, {{timestamp=0,key={x=0}},{timestamp=2,key={x=10}}}, {
            delay=0.5, interval=0.5, loop=true, ["repeat"]=2, easing=animation.LINEAR,
            play_direction=direction, repeat_fill_mode=fill,
          })
          debug.assert(animation.play(id) and animation.get_info(id).state == "delay")
          debug.assert(animation.sample(id, {key="x"}) == ((direction == "reverse" or direction == "alternate_reverse") and 10 or 0))
          animations[#animations+1] = {id=id, direction=direction, fill=fill}
        end
      end
      custom = animation.create(2, {{timestamp=0,key={x=0,label="a"}},{timestamp=2,key={x=10,label="b"}}}, {
        delay=0.5, easing={x=function(t, first, last, a, b)
          debug.assert(first == 0 and last == 2 and a == 0 and b == 10)
          return {current=t}
        end}, default_easing={label=animation.STEP},
      })
      debug.assert(animation.play(custom))
      debug.assert(animation.sample(custom, {key="label"}) == "a")
      local bad = animation.create(1, {{timestamp=0,key={x="a"}},{timestamp=1,key={x="b"}}}, {easing=animation.LINEAR})
      debug.assert(fails(function() animation.sample(bad) end))
      animation.delete(bad)
      local vector_meta = {}
      vector_meta.__sub = function(a,b) return setmetatable({x=a.x-b.x},vector_meta) end
      vector_meta.__mul = function(a,b) return setmetatable({x=a.x*b},vector_meta) end
      vector_meta.__add = function(a,b) return setmetatable({x=a.x+b.x},vector_meta) end
      local vector = animation.create(1, {{timestamp=0,key={v=setmetatable({x=2},vector_meta)}},
        {timestamp=1,key={v=setmetatable({x=4},vector_meta)}}}, {easing=animation.LINEAR})
      debug.assert(animation.sample(vector,{key="v"}).x == 2)
      animation.delete(vector)
      local timed = timer.create(1)
      debug.assert(timer.count() == 1 and animation.count() == 9)
      timer.clear()
      debug.assert(animation.count() == 9)
      unused = animation.create(0, {}, {loop=true,["repeat"]=math.POSITIVE_INFINITE})
      debug.assert(animation.set(unused, {["repeat"]=math.INFINITE}))
      debug.assert(animation.get_info(unused)["repeat"] == nil)
      animation.delete(unused)
    end
    function HandleEvent(event) debug.assert(false, "animations must not emit public timer events") end
    function Update(dt) end
    function UpdateFrame(dt, alpha)
      frame = frame + 1
      local states = {"delay","running","running","interval","interval","running","running","finished"}
      local current = ({0,0,0.5,2,2,0,1,2})[frame]
      for _, item in ipairs(animations) do
        local info = animation.get_info(item.id)
        debug.assert(info.state == states[frame])
        local cycle = frame >= 6 and 1 or 0
        local t = current
        if (frame == 4 or frame == 5) and item.fill == "backwards" then cycle,t = 1,0 end
        local reverse = item.direction == "reverse" or (item.direction == "alternate" and cycle == 1)
          or (item.direction == "alternate_reverse" and cycle == 0)
        local expected = (reverse and (2-t) or t) * 5
        debug.assert(animation.sample(item.id, {key="x"}) == expected)
        debug.assert(animation.sample(item.id).x == expected)
        if frame == 4 then
          debug.assert(animation.pause(item.id) and animation.get_info(item.id).state == "paused")
          debug.assert(animation.sample(item.id).x == expected)
          debug.assert(animation.play(item.id) and animation.get_info(item.id).state == "interval")
        end
      end
      local custom_time = ({0,0,0.5,2,2,2,2,2})[frame]
      debug.assert(animation.sample(custom, {key="x"}).current == custom_time)
      if frame == 8 then
        local timed = timer.create(1)
        debug.assert(animation.clear() and animation.count() == 0 and animation.list().n == 0)
        debug.assert(timer.exists(timed) and timer.delete(timed))
      end
    end
    function Render() end
  "#).unwrap();
  for kind in [LuaSessionKind::Game, LuaSessionKind::Screensaver] {
    let mut session = LuaService::new()
      .create_session(spec(entry.clone(), kind))
      .unwrap();
    for delta in [0.25, 0.25, 0.5, 1.5, 0.25, 0.25, 1.0, 1.0] {
      let delta = Duration::from_secs_f64(delta);
      let events = session
        .with_objects_mut(|objects| {
          TimeService::new().update(&mut objects.runtime_mut().time, delta);
          objects.take_timer_events()
        })
        .unwrap();
      assert!(
        events.is_empty(),
        "animation schedules are not public timer objects"
      );
      session.update().unwrap();
      session.update_frame(delta, 0.0).unwrap();
      session.render().unwrap();
    }
    session.stop();
    assert!(!session.has_objects());
  }

  fs::write(&entry, r#"
    function Init(ctx)
      local values = {{timestamp=0,key={x=0}},{timestamp=6,key={x=10}}}
      local choices = {animation.STEP,animation.LINEAR,animation.EASE_IN_QUAD,animation.EASE_OUT_QUAD,
        animation.EASE_IN_OUT_CUBIC,animation.EASE_IN_OUT_REVERSE_CUBIC}
      eased = {}
      for _, easing in ipairs(choices) do
        local options = {x={[1]=easing}}
        local id = animation.create(6, values, {easing=options})
        options.x[1] = animation.STEP
        animation.get_info(id).easing.x[1] = animation.STEP
        animation.get_info(id).track[2].key.x = 999
        debug.assert(animation.play(id))
        eased[#eased+1] = id
      end
      sparse = animation.create(3, {
        {timestamp=2,key={a=2,b=1}}, {timestamp=0,key={b=3}},
        {timestamp=1,key={a=1,c=4}}, {timestamp=3,key={}},
      }, {easing={b={[1]=animation.LINEAR}}, default_easing={a=animation.LINEAR}})
      animation.play(sparse)
      removed = animation.create(1, {{timestamp=0,key={x=0}},{timestamp=1,key={x=1}}}, {
        easing=function(t, first, last, a, b)
          debug.assert(animation.delete(removed))
          return "removed"
        end,
      })
      debug.assert(animation.sample(removed,{key="x"}) == "removed")
      debug.assert(not animation.exists(removed))
    end
    function HandleEvent(event) end
    function Update(dt) end
    function UpdateFrame(dt,alpha)
      local expected = {0,2.5,0.625,4.375,0.625,4.375}
      for index,id in ipairs(eased) do debug.assert(animation.sample(id,{key="x"}) == expected[index]) end
      local sampled = animation.sample(sparse)
      debug.assert(sampled.a == 1.5 and sampled.b == 1.5 and sampled.c == 4)
    end
    function Render() end
  "#).unwrap();
  let mut session = LuaService::new()
    .create_session(spec(entry.clone(), LuaSessionKind::Game))
    .unwrap();
  session.with_objects_mut(|objects| {
    TimeService::new().update(&mut objects.runtime_mut().time, Duration::from_millis(1500))
  });
  session
    .update_frame(Duration::from_millis(1500), 0.0)
    .unwrap();
  session.stop();

  for failure in ["error('easing failed')", "while true do end"] {
    fs::write(
      &entry,
      format!(
        r#"
      function Init(ctx)
        id = animation.create(1, {{{{timestamp=0,key={{x=0}}}},{{timestamp=1,key={{x=1}}}}}},
          {{easing=function() {failure} end}})
        animation.play(id)
      end
      function HandleEvent(event) end
      function Update(dt) animation.sample(id) end
      function UpdateFrame(dt,alpha) end
      function Render() end
    "#
      ),
    )
    .unwrap();
    let mut session: LuaSession = LuaService::new()
      .create_session(spec(entry.clone(), LuaSessionKind::Game))
      .unwrap();
    assert!(session.update().is_err());
    assert!(!session.has_objects());
  }
  println!(
    "animation Lua ok: sparse tracks, easing, directions, interval fills, timing, ownership and callback budgets"
  );
}
