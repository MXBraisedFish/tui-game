# B3 Lua 权限矩阵与验收记录

基点：2026-09-27 B3 改动后的 `crates/service/lua/src/api/libraries.rs`、session 配置和宿主包设置。矩阵描述当前生产 `install` 注册面；它不表示尚未注册的服务或文档占位 API。

## 会话权限矩阵

| API | Game | Screensaver | Debug 开关 | 边界 |
|---|---|---|---|---|
| `base`、`math`、`utf8`、`table`、`string`、`color`、`char`、`align`、`measurement`、`random`、`slice`、`serialization`、`encoding` | 可用 | 可用 | 无关 | Lua VM、API 代理与会话对象仍各自隔离 |
| `draw` | 可用 | 可用 | 无关 | 仍遵守 callback 阶段与非重入限制 |
| `i18n`、`loader` | 可用 | 可用 | 无关 | loader 仅可加载当前包允许的 Lua 源；没有原生 `package` 库 |
| `file` 常量、`file.read`、`file.exists` | 可用 | 可用 | 无关 | 路径限于当前包 `assets/`；相对路径与符号链接检查继续生效 |
| `file.write`、`file.create_dir`、`file.remove`、`file.list_dir` | 可用 | 拒绝 | 无关 | Screensaver 拒绝时先忽略调用并每方法报告一次，不解析调用参数；Game 也受 assets 沙箱和操作类型路径规则约束 |
| `event.skip_action`、`event.clear_action` | 可用 | 拒绝 | 无关 | Screensaver 拒绝时先忽略调用并每方法报告一次，不解析参数 |
| `game.exit_game`、`game.save_game`、`game.save_best` | 可用 | 拒绝 | 无关 | Screensaver 拒绝时先忽略调用并每方法报告一次，不解析参数；Game 中仍检查 callback 阶段与存档能力 |
| `debug.print`、`debug.info`、`debug.warn`、`debug.error` | 可用 | 可用 | 开启时输出；关闭时忽略 | 关闭时每方法只报告一次忽略；输出仍限速并截断 |
| `debug.assert`、`debug.pcall`、`debug.xpcall` 与级别常量 | 可用 | 可用 | 无关 | 属于脚本逻辑/受控错误处理，不受 Debug 输出开关关闭 |

Game 和 Screensaver 都不接收原生 `io`、`os`、`package`、完整 Lua `debug` 库或终端/shell 控制权。会话类型与 Debug 是独立维度；删除旧安全开关没有扩大这些能力。

## profile 兼容

包设置 profile 中旧 `defaults.safe_mode` 与 game state `safe_mode` 字段仅在反序列化边界忽略。有效 `enabled`、`debug`、screensaver `playlist_enabled`/`order` 继续加载；下次写 package state 时不再序列化安全字段。语言、键位、继续存档和最高分仍保存在各自 profile 文件，旧字段迁移不清空它们。

## 覆盖与验证

| 验收项 | 证据 |
|---|---|
| Game 文件写入可用，Screensaver 文件变更拒绝；路径沙箱仍有效 | `game_file_access_and_debug_logging_are_independently_gated`、`restricted_calls_are_ignored_before_parameter_validation`、`file_apis_share_current_directory_and_parent_traversal_rules`、`task_registration_rejects_unsafe_virtual_paths_and_screensaver_writes` |
| Game-only `event`/`game` 方法与 callback 限制 | `event_action_controls_require_a_game_session`、`game_commands_enforce_callback_reentrancy_boundaries` |
| Debug 输出开关不影响断言和受控保护调用 | `debug_print_constants_and_convenience_methods_use_the_standard_options`、`debug_assert_accepts_nil_and_reports_an_assertion_failure`、`protected_calls_return_named_result_tables` |
| 旧 package state 迁移并保留各存储位置用户数据 | `legacy_safe_mode_fields_are_ignored_without_losing_package_settings`、`legacy_safe_mode_profile_update_preserves_other_user_data` |
| 旧包字段明确拒绝，迁移后的权限夹具可加载并执行 | `removed_high_privilege_game_field_is_rejected`、`checked_in_lua_test_packages_have_valid_complete_manifests`、`test_package_entries_execute_the_basic_lifecycle`；夹具 `test_package/game/permissions_lab/` |
| 宿主设置与路由删除安全模式入口；会话关闭与对象/任务生命周期仍有效 | `cargo test -p tui-game --locked` 的 security settings、overlay、boot/runtime/session 与 task routing 测试 |

Windows 验证：`cargo test -p tg-service-storage --locked`（25 通过）、`cargo test -p tg-service-package --locked`（34 通过）、`cargo test -p tg-service-lua --locked`（106 通过）、`cargo test -p tui-game --locked`（104 通过），均 0 失败、0 忽略。`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo fmt --all -- --check`、`git diff --check` 通过。Linux/macOS 尚未验证，留待最终跨平台回归。
