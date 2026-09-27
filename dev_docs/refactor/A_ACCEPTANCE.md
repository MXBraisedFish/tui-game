# A 阶段验收与清理记录

日期：2026-09-27。Git：`dev`，HEAD `1cf1227`；A 阶段成果主要仍在未提交工作区，不能用 HEAD 内容代替当前成果。本轮没有提交、推送或修改 Rust/Lua 实现。

## 结论

A 阶段已完成，可进入 B。用户本轮确认已做实机检查，并明确接受当前分层边界：runtime 业务及宿主状态/服务组合归于 app；boot 继续编排初始化、语言、扫描与启动界面，shutdown 继续编排资源释放。禁止 app/服务反向依赖阶段模块，后续不再为了目录纯度重搬一次。

## 本轮简单复核

| 项目 | 结果 |
|---|---|
| `cargo fmt --all -- --check` | 退出 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 退出 0 |
| `cargo test --workspace --locked` | 退出 0；623 通过、0 失败、1 忽略 |
| 应用层 | app 下具有 EngineServices、EngineEvent、队列、BootOutput、RuntimeWorld、状态机与 runtime 业务；生命周期 runtime 入口委托 app |
| 文档 | DOC_DRIFT.md 与 MIGRATION_MAP.md 存在，记录迁移与后续偏差 |
| 架构产物 | before/after、网页、生成器与工具均保留；after 45 个唯一单元、127 条边，边端点和源码文件引用均有效 |
| 实机 | 用户报告已检查；本轮不重复启动交互终端，不替用户补写未提供的操作细节 |

## 已核验的前任自动验收证据

清理前读取日志并核对：all-targets build、release build 成功；44 个示例的日志均有 `EXIT_CODE=0`；H.264/AAC 的 ignored 测试单独选中 1 项并通过。上述是前任执行记录，本轮未重新执行全部示例或编码测试。历史过程全文见 `history/PROGRESS.md`。

唯一默认忽略项为 `recording_with_audio_exports_as_h264_aac_mp4`，需要 FFmpeg。B 涉及多媒体后须重新验证，不能永久复用 A 结果。

## 不应抹去的限制

- A 已关闭不表示 B 的已知缺陷消失：CWD 根目录、Lua 非标准协议、旧清单、图片 Lua 入口、导出与 IME 仍待修复。
- 三个官方包目前仅有 package.json，没有清单所指向的嵌套 scripts/，前任临时部署因此不能启动它们。B4 需处理官方 fixture 的有效性；未将此解释为用户实机失败。
- Linux/macOS 未有本轮运行证据；不宣称三平台全部验收。后续平台相关变更仍须验证。
- after 快照统计为 4,782 个条目，576 个含描述；`findings=0` 是生成数据值，不证明代码零缺陷或完成逐条语义审计。
- 原用户编辑的 `dev_docs/zh_cn/package字段.md` 两处尾随空白保留；不因此改动用户文档。
- 源码还存在旧中文注释；严格 Clippy 通过不等于全部注释完成英文化。B 修改涉及的代码继续遵循英文注释约定，不开展无关全仓翻译。

## 临时文件清理策略

已将 `temp/handoff/{PROGRESS,SERVICE_DESIGN,DEAD_CODE}.md` 归档到 `history/`，将架构文本 DOM 检查脚本保留为 `architecture/tools/dom_smoke.cjs`。

清理对象限于仓库 `temp/` 内：

- `audit/`：旧提取器编译缓存、中间数据、审计代理草稿；生成工具已在 architecture 内独立保存。
- `baseline/`：已过时的初始基线日志。
- `handoff/`：已归档文字外的一次性迁移脚本与中间片段。
- `p_plan_audit/`：重复构建日志、生成中间 JSON、临时部署副本、Edge 临时 profile、ConPTY 尝试文件。
- `p_plan_review/`：本轮简要复核原始日志；结论保存在本文。

保留 `architecture/` 整体（含前后快照、生成器、工具和文本 DOM 检查脚本）、`temp/image-converter-reference/` 三套 B6 研究源码、根 `target/` 构建缓存、所有业务资源与测试包。`E:\Code\tg-test\` 和外部 Agent 记忆目录未清理。

删除方案使用 PowerShell 原生命令；已核对每个解析后的绝对路径属于上述仓库 temp 目录，且没有 reparse point；归档和架构文件均存在。不使用 git clean 或跨 shell 删除。

**执行结果：未删除。** 自动审批先后拒绝批量删除与仅删除 `E:\Code\tui-game\temp\baseline` 的缩小命令，原因仅返回 `blocked by policy`，没有更具体说明。未尝试绕过策略。5 个目标目录仍存在：1,045 个文件、306,480,385 字节（约 292 MiB），待允许删除的环境处理。用户已授权清理，无须再次询问意愿；此为工具执行阻塞，不是用户未授权。

## 后续

执行计划唯一当前入口为根目录 P_PLAN.md；历史文件只用于追溯，不继续维护旧 STATUS 代理或 temp/handoff 路径。B 的状态按 P_PLAN 工作包与证据更新。
