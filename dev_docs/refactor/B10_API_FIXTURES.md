# B10.5 Lua API 示例与测试锚点盘点

更新时间：2026-09-28。此表把当前 `libraries.rs::install` 的注册面映射到 Lua session 端到端测试；它是迁移前 inventory，不代表所有文档片段都已作为 fixture 执行。实现位置以 `crates/service/lua/src/session.rs` 中所列测试为准。

| API 族 | 当前生产注册 | 已有可执行 Lua 测试锚点 | 迁移/示例缺口 |
|---|---|---|---|
| Lua base / table | Lua 5.4 基础函数及宿主限制包装 | `standard_base_functions_follow_lua54_table_and_vararg_semantics`; `base_library_matches_native_lua54_on_plain_table_operations`; `table_standard_library_matches_native_lua54_behavior` | 属于 B2 标准语义；不得套用 B10 扩展参数协议。 |
| `string` / `math` / `utf8` | 项目扩展与标准包装 | `string_api_matches_documented_tables_unicode_and_limits`; `math_api_matches_documented_names_types_and_boundaries`; `utf8_api_uses_scalar_indices_lazy_iterators_and_table_results` | B2 参数、迭代记录和结果表已单独确认；示例须保留已确认协议。 |
| `color` / `char` / `align` | 已安装 | `align_resolve_rect_returns_a_named_coordinate_table`; `api_tables_are_read_only_and_iterators_do_not_expose_backing_tables`; `image_load_returns_request_id_routes_rich_text_and_rejects_unsafe_arguments`（含 `color.rgb`） | 还需拆出 API 专属的色彩校验/常量样例；当前测试覆盖散落在集成用例中。 |
| `measurement` / `draw` | 已安装 | `lua_text_modes_share_the_expected_measurement_semantics`; `measurement_api_returns_documented_types_and_uses_draw_text_layout`; `measurement_api_rejects_position_style_target_and_invalid_layout_parameters`; `drawing_is_allowed_in_all_callbacks_but_render_requests_are_not_reentrant` | 已有正常、错误和生命周期例；迁移后再按最终参数表同步文档示例。 |
| `random` / `slice` | 已安装 | `random_slice_serialization_and_encoding_libraries_work_together`; `random_and_slice_follow_the_documented_object_protocol`; `random_and_slice_objects_are_isolated_and_released_with_the_session` | 多数契约在合并测试中；可在迁移分组时拆成单族 fixture。 |
| `serialization` / `encoding` | 已安装 | `random_slice_serialization_and_encoding_libraries_work_together`; `serialization_rejects_cycles_sparse_tables_entities_and_malformed_encoding`; `serialization_json_top_level_null_empty_and_numeric_edges_are_stable`; `serialization_formats_follow_the_public_parameter_and_result_protocol` | encoding `{s=...}` 和 serialization 单表参数已迁移；JSON/YAML `serialization.NULL` 保真，CSV/INI/TOML/XML 拒绝哨兵。 |
| `debug` | 已安装，部分方法受 debug 开关约束 | `debug_print_uses_header_free_defaults`; `debug_assert_accepts_nil_and_reports_an_assertion_failure`; `debug_print_constants_and_convenience_methods_use_the_standard_options`; `protected_calls_return_named_result_tables`; `slow_callback_warnings_require_debug_mode` | `print/info/warn/error/assert/pcall/xpcall` 均用单个命名表；便捷日志统一 `{message=...}`，protected-call 具名结果保持。 |
| `game` / `event` | 已安装；按会话类型/调用阶段受限 | `game_commands_enforce_callback_reentrancy_boundaries`; `event_action_controls_require_a_game_session`; `restricted_calls_are_ignored_before_parameter_validation` | 无参命令改为 `{}`；会话类型、回调阶段和拒绝时先门控再校验的行为保持并由现有回归覆盖。 |
| `file` / `i18n` / `image` | 已安装，使用异步 `HandleEvent` | `file_apis_share_current_directory_and_parent_traversal_rules`; `game_file_access_and_debug_logging_are_independently_gated`; `i18n_api_loads_asynchronously_and_applies_data_before_handle_event`; `image_load_returns_request_id_routes_rich_text_and_rejects_unsafe_arguments` | file 与 i18n 现已返回会话 request id，并在终态事件回传；image 同样按 B6 契约返回 ID 与富文本结果。 |
| `loader` | 已安装 | `loader_matches_module_semantics_and_rejects_unsafe_sources` | 路径参数已改 `{path=...}`；模块原始多返回值按 OQ-B10-04 确认保留；已有缓存、循环加载、越界路径和字节码负例。 |
| `animation`, `audio`, `effect`, `http`, `timer`, `widget.*`, `ime`, `keyboard` | 未注册 | 无生产 API fixture | 对应文档占位页现已说明不可调用；不能为满足 API 示例数量而伪造接口。 |

## 当前结论

- 多数已注册 API 已有 Lua VM 内执行的端到端测试，但测试粒度和文档样例协议并不统一。测试锚点不是每个 API 方法的完整覆盖清单。
- B10.5 的下一步应在 B10.2 每组迁移后，将该组文档示例直接引用到相应 Lua fixture；不复制一份难以同步的 Lua 文件。
- 批量签名迁移与 JSON null 语义未定前，不重写相应示例。当前 `cargo test --workspace --locked` 的全量通过记录见 `P_PLAN.md` B9.3 执行日志；本文件只做静态映射，没有重新运行测试。
