# 步骤 13+：后续迁移路线图（骨架完成之后）

## 步骤 12 完成后的目录结构

```
app/
├── mod.rs
├── app.rs          — 编排：boot → runtime → shutdown
├── boot.rs         — 准备 services + world
├── runtime.rs      — 帧循环：poll → update → render
├── shutdown.rs     — 清理
├── frame.rs        — FrameScheduler
├── clock.rs        — EngineClock
├── world.rs        — RuntimeWorld
└── services/
    ├── mod.rs      — EngineServices（9 个服务）
    ├── terminal.rs — TerminalService（交替屏幕 + raw 模式）
    ├── input.rs    — InputService（crossterm 按键轮询）
    ├── render.rs   — RenderService（双缓冲 + present）
    ├── storage.rs  — StorageService（目录创建）
    ├── package.rs  — PackageService（扫描 + 列表）
    ├── ui.rs       — UiService（页面注册 + 导航）
    ├── lua.rs      — LuaService（VM，无脚本）
    ├── game.rs     — GameService（会话桩）
    └── overlay.rs  — OverlayService（会话桩）
```

全部 9 个服务已存在。运行时循环是干净的帧调度器。
旧 `host_engine/` 模块未动，但 `main.rs` 不引用。

## 真实系统的迁移顺序

以下每步将一个桩服务从旧 `host_engine/` 代码库迁入真实逻辑。
不破坏任何一步的编译。

### 步骤 13 — StorageService：Profile + Cache

将 `host_engine::storage::profile_store` 和 `cache_store` 迁入
`app/services/storage.rs`。暴露查询方法：
- `StorageService::profile()` → `&ProfileStore`
- `StorageService::cache()` → `&CacheStore`
- `StorageService::save_package_state(uid, state)`

### 步骤 14 — PackageService：启用/禁用 + 热重载

给 `PackageService` 添加：
- `set_enabled(uid, bool)` — 持久化到 profile
- `reconcile_states()` — 与 profile store 同步
- `rebuild_display_orders()` — 从 profile 读取
- `hot_reload()` — 重新扫描目录检测变更

### 步骤 15 — LuaService：API 注册 + 脚本加载

将 `host_engine::boot::preload::lua_runtime` 迁入 `LuaService`：
- API 作用域安装
- 按路径加载脚本
- 回调验证
- Lua 宿主桥接（用请求而非全局状态）

### 步骤 16 — InputService → ActionService

将 `InputService` 拆分为：
- `InputService` — 原始按键轮询（保持现状）
- `ActionService` — 语义动作解析（EngineServices 新槽位）

将 `host_engine::keybind::keybind_manager` 迁入 `ActionService`。

### 步骤 17 — UiService：真实页面

将旧 `ui/pages/*.rs` 各页面迁入新 `UiService`：
- HomePage → 真实渲染
- GameListPage → 从 PackageService 读取包列表
- ModScreensaverListPage → overlay 列表
- Setting 页面 → 接入 StorageService

### 步骤 18 — GameService：Lua 游戏会话

将 `host_engine::runtime::game_engine` 迁入 `GameService`：
- 用 package_id 启动游戏会话
- 加载 Lua 入口脚本
- 调用 init / handle_event / update / render 生命周期
- 处理游戏消息和退出

### 步骤 19 — OverlayService：屏保/Boss 会话

将 `host_engine::runtime::overlay` 迁入 `OverlayService`：
- 用 F2/F3 切换屏保/boss
- 会话生命周期
- 空闲自动进入

### 步骤 20 — RenderService：完整渲染器

将 `host_engine::runtime::renderer` 迁入 `RenderService`：
- Canvas 操作
- 颜色支持
- 富文本
- 图片缓存渲染

### 步骤 21 — 清理

删除 `host_engine/` 中已完全迁移的死代码。
`official_ui/` 保留作为参考，不作为运行时依赖。

## 每一步必须遵守的规则

1. **先编译，再运行。** 每一步必须产出可运行的二进制文件。
2. **每次只动一个服务。** 绝不在一步中同时修改两个服务。
3. **旧代码迁移前不动。** 旧 `host_engine/` 模块保留作为参考。
   只有完全替换后才删除代码。
4. **桩 → 骨架 → 真实。** 服务渐进：空结构体 → 带签名的方法 → 真实实现。
5. **禁止跨服务直接修改。** 服务 A 查询服务 B，绝不直接修改 B 的内部状态。
