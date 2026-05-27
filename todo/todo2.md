# 步骤 2：加入 EngineServices 和 RuntimeWorld（空的）

## 目标

引入两个将持有所有引擎状态的中心结构体，但保持完全为空。
证明它们能贯穿整个生命周期流转。

## 背景

- `EngineServices` — 将持有全部服务（包管理、输入、UI 等）
- `RuntimeWorld` — 将持有全部可变运行时状态（UI 页面、会话等）

## 期望输出

与步骤 1 相同，但结构体在 boot 中创建、贯穿 runtime、在 shutdown 中被消费。

## 操作

### 2.1 创建 `src/app/services.rs`

```rust
/// 持有全部引擎服务。当前为空。
pub struct EngineServices {}

impl EngineServices {
    pub fn new() -> Self {
        Self {}
    }
}
```

### 2.2 创建 `src/app/world.rs`

```rust
/// 持有全部可变运行时世界状态。当前为空。
pub struct RuntimeWorld {}

impl RuntimeWorld {
    pub fn new() -> Self {
        Self {}
    }
}
```

### 2.3 更新 `src/app/mod.rs`

添加新模块：

```rust
pub mod app;
pub mod boot;
pub mod runtime;
pub mod services;
pub mod shutdown;
pub mod world;

pub use app::run;
```

### 2.4 更新 `src/app/boot.rs`

返回两个结构体：

```rust
use super::services::EngineServices;
use super::world::RuntimeWorld;

pub struct BootOutput {
    pub services: EngineServices,
    pub world: RuntimeWorld,
}

pub fn prepare() -> BootOutput {
    println!("[Boot] Preparing engine...");
    let services = EngineServices::new();
    let world = RuntimeWorld::new();
    BootOutput { services, world }
}
```

### 2.5 更新 `src/app/runtime.rs`

接收结构体：

```rust
use std::{thread, time::Duration};
use super::services::EngineServices;
use super::world::RuntimeWorld;

pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) {
    for i in 0..5 {
        println!("[Runtime] Running... ({})", i + 1);
        thread::sleep(Duration::from_secs(1));
    }
}
```

### 2.6 更新 `src/app/shutdown.rs`

消费结构体：

```rust
use super::services::EngineServices;
use super::world::RuntimeWorld;

pub fn close(services: EngineServices, _world: RuntimeWorld) {
    let _ = services; // 后续会用到
    println!("[Shutdown] Engine closed.");
}
```

### 2.7 更新 `src/app/app.rs`

```rust
use super::{boot, runtime, shutdown};

pub fn run() {
    let BootOutput { mut services, mut world } = boot::prepare();
    runtime::run(&mut services, &mut world);
    shutdown::close(services, world);
}
```

## 验证

```bash
cargo build
cargo run
```

输出与步骤 1 相同。结构体正确贯穿整个生命周期。
