# 步骤 4：添加桩服务（全部为空）

## 目标

在 `EngineServices` 中创建全部服务槽位。每个服务是一个空结构体，
拥有其未来领域的所有权。什么也不做 — 只证明服务注册模式能跑通。

## 需要桩的服务

| 服务 | 未来职责 |
|------|---------|
| `PackageService` | 包发现、验证、注册表、启用/禁用 |
| `InputService` | 物理输入轮询、终端事件、按键状态 |
| `UiService` | 宿主 UI 页面、导航、焦点、渲染 |
| `GameService` | 游戏会话生命周期 |
| `OverlayService` | 屏保/boss 会话生命周期 |
| `StorageService` | 配置/缓存/存档持久化 |
| `LuaService` | Lua 虚拟机、沙箱、API 作用域、脚本会话 |
| `RenderService` | 画布、表面、呈现 |

## 操作

### 4.1 创建 `src/app/services/` 目录

将 `services.rs` 改为 `services/mod.rs`，添加桩文件：

```
src/app/services/
├── mod.rs           (EngineServices, 重导出)
├── package.rs       (PackageService 桩)
├── input.rs         (InputService 桩)
├── ui.rs            (UiService 桩)
├── game.rs          (GameService 桩)
├── overlay.rs       (OverlayService 桩)
├── storage.rs       (StorageService 桩)
├── lua.rs           (LuaService 桩)
└── render.rs        (RenderService 桩)
```

### 4.2 各桩文件内容（全部相同模式）

```rust
// package.rs / input.rs / ui.rs / game.rs / overlay.rs / storage.rs / lua.rs / render.rs
pub struct XxxService {}

impl XxxService {
    pub fn new() -> Self { Self {} }
}
```

### 4.3 `src/app/services/mod.rs`

```rust
mod game;
mod input;
mod lua;
mod overlay;
mod package;
mod render;
mod storage;
mod ui;

pub use game::GameService;
pub use input::InputService;
pub use lua::LuaService;
pub use overlay::OverlayService;
pub use package::PackageService;
pub use render::RenderService;
pub use storage::StorageService;
pub use ui::UiService;

/// 持有全部引擎服务实例。
pub struct EngineServices {
    pub package: PackageService,
    pub input: InputService,
    pub ui: UiService,
    pub game: GameService,
    pub overlay: OverlayService,
    pub storage: StorageService,
    pub lua: LuaService,
    pub render: RenderService,
}

impl EngineServices {
    pub fn new() -> Self {
        Self {
            package: PackageService::new(),
            input: InputService::new(),
            ui: UiService::new(),
            game: GameService::new(),
            overlay: OverlayService::new(),
            storage: StorageService::new(),
            lua: LuaService::new(),
            render: RenderService::new(),
        }
    }
}
```

### 4.4 删除旧的 `src/app/services.rs`

（原本是单个文件，现在变成目录模块 `services/mod.rs`）

### 4.5 更新 `src/app/runtime.rs`

引用所有服务以证明它们可访问：

```rust
fn update(services: &mut EngineServices, _world: &mut RuntimeWorld, _frame: FrameInfo) {
    let _ = &services.package;
    let _ = &services.input;
    let _ = &services.ui;
    let _ = &services.game;
    let _ = &services.overlay;
    let _ = &services.storage;
    let _ = &services.lua;
    let _ = &services.render;
}
```

## 验证

```bash
cargo build
cargo run
```

输出与步骤 3 相同。全部 8 个服务作为空结构体编译通过。
