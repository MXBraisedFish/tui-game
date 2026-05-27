# 步骤 12：LuaService + GameService + OverlayService 生命周期桩

## 目标

给 `LuaService` 加入真正的 mlua 虚拟机（不加载脚本）。
给 `GameService` 和 `OverlayService` 加上会话生命周期桩。
证明全部 9 个服务槽位都存活且可被运行时循环调用。

（注意：现在加了 TerminalService，共 9 个服务）

## 操作

### 12.1 LuaService — 真正 VM，不跑脚本

```rust
use mlua::Lua;

pub struct LuaService {
    lua: Lua,
}

impl LuaService {
    pub fn new() -> Self {
        Self { lua: Lua::new() }
    }

    pub fn lua(&self) -> &Lua { &self.lua }

    /// 执行一小段 Lua 代码用于测试。
    pub fn eval(&self, code: &str) -> Result<String, String> {
        self.lua.load(code).eval::<String>().map_err(|e| e.to_string())
    }
}
```

在 boot 中测试：
```rust
match services.lua.eval("return 'Lua VM active'") {
    Ok(result) => println!("[Boot] Lua: {}", result),
    Err(e) => eprintln!("[Boot] Lua error: {}", e),
}
```

### 12.2 GameService — 会话生命周期桩

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameSessionState { Inactive, Running, Paused }

pub struct GameService {
    state: GameSessionState,
    active_package_uid: Option<String>,
}

impl GameService {
    pub fn new() -> Self {
        Self { state: GameSessionState::Inactive, active_package_uid: None }
    }
    pub fn start(&mut self, package_uid: &str) { /* ... */ }
    pub fn stop(&mut self) { /* ... */ }
    pub fn state(&self) -> GameSessionState { self.state }
    pub fn update(&mut self) {}
}
```

### 12.3 OverlayService — 会话生命周期桩

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayKind { Screensaver, Boss }

pub struct OverlayService {
    screensaver_state: OverlaySessionState,
    boss_state: OverlaySessionState,
    active_screensaver_uid: Option<String>,
    active_boss_uid: Option<String>,
}

impl OverlayService {
    pub fn new() -> Self { /* 全部 Inactive */ }
    pub fn start(&mut self, kind: OverlayKind, package_uid: &str) { /* ... */ }
    pub fn stop(&mut self, kind: OverlayKind) { /* ... */ }
    pub fn is_active(&self, kind: OverlayKind) -> bool { /* ... */ }
    pub fn any_active(&self) -> bool { /* ... */ }
    pub fn update(&mut self) {}
}
```

### 12.4 在状态栏显示各服务状态

```rust
let overlay_status = if services.overlay.any_active() { "OVERLAY" } else { "idle" };
let game_status = match services.game.state() {
    GameSessionState::Inactive => "idle",
    GameSessionState::Running => "RUNNING",
    GameSessionState::Paused => "PAUSED",
};
let lua_status = match services.lua.eval("return 'ok'") {
    Ok(_) => "Lua:ok",
    Err(_) => "Lua:err",
};
let status = format!("Game:{} | Overlay:{} | {} | ← → Nav | ESC Exit",
    game_status, overlay_status, lua_status);
```

## 验证

```bash
cargo build
cargo run
```

- 启动输出 "Lua VM active"
- 状态栏显示 Game:idle、Overlay:idle、Lua:ok
- 全部 9 个服务存活、可访问
- 骨架完成 — 准备接入真实逻辑
