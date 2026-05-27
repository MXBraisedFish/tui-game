# 步骤 1：最小可执行生命周期骨架

## 目标

创建最简单的引擎生命周期，能编译、能运行，证明三阶段边界：
**准备 → 运行 → 关闭**。

## 期望输出

```
[Boot] Preparing engine...
[Runtime] Running...
[Runtime] Running...
[Runtime] Running...
[Runtime] Running...
[Runtime] Running...
[Shutdown] Engine closed.
```

## 操作

### 1.1 创建目录结构

```
src/
├── main.rs          (修改：只保留 app::run() 调用)
├── host_engine/     (保留不动，不引用)
└── app/
    ├── mod.rs       (新建)
    ├── app.rs       (新建)
    ├── boot.rs      (新建)
    ├── runtime.rs   (新建)
    └── shutdown.rs  (新建)
```

### 1.2 `src/app/mod.rs`

```rust
pub mod app;
pub mod boot;
pub mod runtime;
pub mod shutdown;

pub use app::run;
```

### 1.3 `src/app/app.rs`

只做编排，不含任何逻辑：

```rust
use super::{boot, runtime, shutdown};

pub fn run() {
    boot::prepare();
    runtime::run();
    shutdown::close();
}
```

### 1.4 `src/app/boot.rs`

```rust
pub fn prepare() {
    println!("[Boot] Preparing engine...");
}
```

### 1.5 `src/app/runtime.rs`

```rust
use std::{thread, time::Duration};

pub fn run() {
    for _ in 0..5 {
        println!("[Runtime] Running...");
        thread::sleep(Duration::from_secs(1));
    }
}
```

### 1.6 `src/app/shutdown.rs`

```rust
pub fn close() {
    println!("[Shutdown] Engine closed.");
}
```

### 1.7 `src/main.rs`

替换当前内容：

```rust
mod app;

fn main() {
    app::run();
}
```

## 验证

```bash
cargo build
cargo run
```

必须打印期望输出并正常退出。
旧 `host_engine/` 模块保留但 `main.rs` 不引用。
只调用 `app::run()`。
