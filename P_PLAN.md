# P_PLAN：A 已验收，B 阶段详细执行计划

> 更新日期：2026-09-27。核对基点：`dev` / HEAD `1cf1227` + 尚未提交的 A 阶段成果。
> **A 已验收**：用户确认不再重做 A。本轮从 B0 契约整理、B1 路径修复开始。
> 临时文件清理仅限第 4 节记录的五个 temp 目录；此前执行被自动审批策略拒绝，遵照用户要求不绕过。保留架构网页/生成工具与 `temp/image-converter-reference/`。本轮不提交或推送。

## 1. 任务目标与执行依据

总体目标分两部分：`PLAN.md` 的迁移式重构已验收；接下来完成 `PLAN2.md` 的全部功能与修复。用户已确认此范围和顺序，清单冲突以字段表为准；其他已确认决定和待定项见第 5 节。

执行前阅读本文件和根目录 `AGENTS.md`。参考资料：

- `PLAN.md`：原重构要求、行为保持、独立运行、完整回归。
- `PLAN2.md`：后续功能的原始需求；存在互相矛盾的字段，见第 5 节。
- `MIGRATION_MAP.md`：迁移前后位置及理由。
- `dev_docs/refactor/A_ACCEPTANCE.md`：A 验收、已知限制与清理记录。
- `dev_docs/refactor/history/{PROGRESS,SERVICE_DESIGN,DEAD_CODE}.md`：归档历史过程，不再继续旧 temp 路径。
- `C:\Users\123\.claude\projects\E--Code-tui-game\memory\`：历史偏好与已确认决定。不要要求后续环境必须具有该绝对路径；必要结论已摘入本文件。
- `architecture/README.md`：架构网页和生成工具；临时中间目录可以按需重建。
- `temp/image-converter-reference/`：保留的 B6 研究源码，不能与已完成迁移的临时文件一起删除。

当前用户指令优先。当前代码和本轮验证用于判定“已经实现什么”；新需求用于判定“将要改变什么”。不能把旧实现当成拒绝新需求的理由，也不能在行为保持阶段提前应用新需求。

遇到影响行为、兼容性、用户数据或安全边界的不确定项，按 `AGENTS.md` 停下相关工作并提问：说明现象、上下文、建议与理由。常规内部拆分可依据已有历史授权自行决定并记录，不要重复询问已确定的 workspace、迁移方向与验证节奏。

## 2. 当前基线与工作区保护

| 项目 | 当前核对结果 |
|---|---|
| Workspace | 根程序 + 12 核 + 32 服务，共 45 个成员；44 个独立示例 |
| 应用拆分 | EngineServices、EngineEvent/队列、BootOutput、RuntimeWorld/状态机和 runtime 业务归于 `src/host_engine/app/` |
| 生命周期 | runtime 委托 app；boot/shutdown 保留阶段初始化和释放编排，用户本轮明确接受此边界，不再要求重搬 |
| 本轮格式 / Clippy | `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings` 均退出 0 |
| 本轮测试 | `cargo test --workspace --locked`：623 通过、0 失败、1 忽略，退出 0 |
| 前任全量回归 | 已核验 all-targets/release 成功日志、44/44 示例成功、H.264/AAC 忽略测试单独运行 1 项通过；本轮未重复全跑 |
| 实机 | 用户已检查，A 门禁关闭；不推定用户已测 B 的未来行为或所有 OS |
| 架构网页 | 当前工作区版本的网页、`data/current.js` 与 `build_current.py`/`tools/` 生成工具保留；页面只载入 current。工作区已有 `data/before.js` 删除状态，本轮保持不动 |
| 文档 | DOC_DRIFT.md、MIGRATION_MAP.md 已存在 |
| B | B0/B1 已完成；B2.1–B2.5 Lua 兼容与宿主结合已完成并通过 Lua 服务包验证 |

唯一默认忽略测试为 `recording_with_audio_exports_as_h264_aac_mp4`。Linux/macOS 未有本轮实机证据；after 条目描述为 576/4782，并非逐条完整语义审计。官方 `scripts/game` 的三个包只有清单、缺少入口 scripts 目录，B4 必须解决有效 fixture，不能把错误示例算作可玩官方包。

**保护整个工作区，而非只保护旧 28 个文件。** A 的大量迁移与清理尚未提交，HEAD 仍是旧基点。`app/`、after 快照、工具与新文档可能为 untracked，都属于应保留成果。此前用户编辑 `.gitignore`、`dev_docs/zh_cn/KEY.md`、`format/EVENT.md`、`package字段.md` 继续保留。

禁止 reset/clean/stash/整体 checkout；只按明确范围暂存或撤销自己的改动。正式开发前记录最新状态，不能通过重放历史迁移脚本恢复 A。

## 3. 全程代码与工作方式约束

1. **逐块完成**：一个业务块完成实现、相关测试、审阅后再进入下一块。默认单 Agent 顺序推进；旧会话使用过子代理不等于本轮必须恢复并行开发。
2. **迁移可追溯**：A 阶段每处搬迁写出旧路径、符号/代码块、新位置、语义是否改变及对应测试。优先移动现有代码，禁止删除后凭记忆重写。
3. **依赖单向**：核不依赖服务或宿主；服务不依赖 `host_engine`、宿主 UI、聚合 `EngineEvent`；服务间 Cargo 依赖无环。应用层负责组合与事件翻译。不要仅为满足“引用三次”生造核或复制类型。
4. **生命周期独立**：boot → runtime → shutdown。业务可调用本阶段应用逻辑，不能让服务回调阶段模块，不能让 runtime 复用 boot 内部函数形成跨阶段耦合。
5. **复用优先**：先检查仓库和已锁定依赖。确需第三方研究时使用官方资料/上游源码，可放 `temp/`；不要因计划而预先升级依赖、切换工具链或引入大型框架。
6. **可读性**：Rust 常规命名；按职责或真实复用拆分；避免空壳包装、超长参数串、布尔模式组合、无意义转换和全局服务定位器。不得单纯为行数指标拆文件。
7. **可见性最小**：仅跨 crate 真实调用需要的成员设为 `pub`。不能用扩大可见性隐藏 dead_code；测试专用能力延续 `test-support` feature + dev-dependency 方式。
8. **错误可诊断**：对文件、Lua、任务失败保留操作、路径/包/任务及原始错误上下文。外部输入不 `unwrap/expect`；不要用 `let _ =` 吞掉影响数据完整性或终端恢复的错误。
9. **注释**：新增/修改的 Rust 注释使用英文。模块 `//!`；函数摘要清晰；按实际需要补 Arguments/Returns/Errors/Panics/Safety，标题为 `#`。公共示例可执行。不要机械补空文档段落。
10. **边界按重要性处理**：不扩展低概率、无实际需求的兼容功能；路径安全、用户数据、终端恢复、跨平台资源释放不得因此省略。
11. **测试隔离**：测试使用临时根或 `E:\Code\tg-test\` 中专用子目录，不覆盖现有用户存档。避免进程全局 CWD/环境变量改变造成并发测试竞态；需要不同 CWD 时使用子进程。
12. **路径**：生产资源依据可执行文件所在的上线根目录；`assets/`、`data/`、`scripts/` 为其下级。`CARGO_MANIFEST_DIR` 仅用于编译期资源和测试 fixture，不得作为生产路径回退。开发路径不能硬编码进产品。
13. **禁止把图像带入会话**：历史环境会因读图冻结。不要使用读图、截图预览、图像附件等工具。产品测试必要的图像/视频文件可在本地由程序生成并以数值、元数据验证；视觉确认由用户本地观看并用文字反馈。
14. **Git**：历史授权在 `dev` 小步本地提交并定期普通 push 备份，绝不操作 `main`。按明确路径暂存，禁止混入用户编辑，不使用 AI Co-Authored-By，不擅自改作者配置。新 tag 不覆盖已有 tag；禁止默认 force push。本轮计划编写未执行任何提交。
15. **完成报告诚实**：区分自动测试、实机通过、条件跳过、未执行与阻塞。不可用“构建通过”替代实机、视觉、IME 或其他平台验收。

## 4. A 阶段关闭与清理结果

A0 残留审阅、A1 应用迁移、A2 质量/文档偏差、A3 自动回归/架构网页，连同用户实机验收，已关闭。完整证据见 `dev_docs/refactor/A_ACCEPTANCE.md`。

保留当前 boot/shutdown 编排是用户明确确认的工程边界，取代原计划“所有阶段只委托 app”的过严要求；仍禁止服务/app 反向依赖阶段模块。新增 B 业务默认放在应用或相应服务，不重新扩大主循环。

临时归档已完成：将 3 份有价值的 handoff 文档归档到 `dev_docs/refactor/history/`；架构文本检查脚本保留为 `architecture/tools/dom_smoke.cjs`。计划删除仓库 temp 下旧 audit、baseline、handoff、p_plan_audit、p_plan_review：它们分别为旧编译/提取缓存、日志、一次性迁移脚本、中间 JSON、临时部署和浏览器 profile。保留 architecture 整体、图片转换研究源码、根 target 缓存、测试包与真实测试目录。

**删除尚未执行**：已核对上述 5 个目录的绝对路径、无 reparse point 和归档存在，共 1,045 个文件、306,480,385 字节（约 292 MiB）。自动审批先后拒绝批量删除和仅删除 `temp/baseline` 的缩小操作，原因仅返回 `blocked by policy`。未绕过策略；这些文件仍在磁盘，待允许删除的执行环境清理。该环境限制不阻塞 B 的代码工作。

历史文件内出现旧路径仅供追溯，不是后续前置依赖。`python architecture/build_current.py` 可重建自己的临时目录。架构网页仅保留 `architecture/data/current.js`；B 有实质结构变更时重新生成当前架构数据，不为纯文档调整重复生成。

## 5. B 阶段开工前的需求裁决表

用户在 2026-09-27 本轮已确认：先完成重构收尾，再执行 PLAN2 全部需求；清单冲突以字段表为准（Q1 已关闭）。本轮又确认 Q2（新清单完全升级，不兼容旧格式）、Q4（保留 Lua 5.4）；其他未定项必须在影响到的实现前确认；填写决定、日期与影响测试后不再重复提问。A 阶段不依赖这些剩余裁决。

用户于 2026-09-27 确认 B1.3 字体回退顺序：显式配置字体优先，其次部署根的内置字体，再其次系统字体；全部不可用时才报错。相对字体路径以部署根解析，最终错误包含尝试路径。回归覆盖显式字体失效后回退内置字体、相对路径和全来源失败。

用户于 2026-09-27 确认 B2 字符串元表保持隔离：Lua 字符串值不开放原生 `string` 元表方法，项目 `string.*` 只经受控 API 调用，避免旁路现有参数协议与大小限制；已记入 `dev_docs/refactor/B0_CONTRACTS.md`。

B2.4 string/pattern 的项目命名参数、Unicode 索引与结果表和 Lua 5.4 原生位置参数、多返回、字节索引不同。用户于 2026-09-27 确认保留项目调用格式、参数传递和结果表形状；本组核对业务行为及安全边界，不迁移原生调用/返回协议。math/utf8 同样保留项目参数与结果协议，用 Lua 5.4 基线核对计算行为与安全边界。记录见 `dev_docs/refactor/B0_CONTRACTS.md` 与 `B2_LUA_BASELINE.md`。

math/utf8 同版本审计已完成：原生标准函数与宿主数组/结果表、字节位置协议存在差异，探针已加入。用户于 2026-09-27 确认保留项目命名参数、数组传参、结果表和迭代记录；该组只核对语义与安全，不转换成 Lua 原生位置参数/多返回。

用户于 2026-09-27 授权后续 B7 截图/视频字体验证：可在 AGENTS 指定的 `E:\Code\tg-test\` 测试目录直接启动终端程序进行截图，并查看截图图片。执行时将保留真实截图与终端/字体配置记录，不再重复询问此操作授权。

| 编号 | 冲突/缺失与影响 | 倾向方案；必须在对应实现前确认 |
|---|---|---|
| Q1（已确认） | `save`/`save_game`，`best_score.enabled`/`enable`，keys 超限截断/报错，FPS 缺省 60/跟随玩家 | 采用字段表：`save_game`、`best_score.enable`、keys 每层超过两个元素报错、缺省 FPS 跟随玩家设置。2026-09-27 用户明确确认，不再提问 |
| Q2（已确认） | 新清单格式与旧包兼容 | 用户本轮要求“不兼容旧格式，直接完全升级”。将 PACKAGE_MANIFEST_VERSION 从 1 升到 2，所有仓库清单同步升级；拒绝旧格式，不实现旧格式解析或别名。此决定不授权删除用户 data；mod_id 与数据身份问题另见 Q2a |
| Q2a | 新 mod_id 只允许字母/数字/下划线，现有验证允许点/连字符，测试包大量使用 `test.xxx`；PackageId 同时参与存档反序列化 | 新包严格按新规则；旧存档中 ID 不应因收紧底层反序列化而导致整份 profile 重置。倾向清单新规则与存量身份读取分开，不自动替用户改 ID。fixture 可显式改名；真实数据迁移方式若需要必须确认 |
| Q3 | `keys` 空数组/重复键/修饰键组合，快捷 `command` 冲突及与宿主参数冲突行为未定 | 先列当前规则，要求确定对外契约；不要默默截断、覆盖或猜测快捷命令解析方式 |
| Q4（已确认） | Lua 版本与对照 | 保留 mlua 的 lua54/vendored；同版本未改写 Lua 为主对照，lua55 仅用于版本差异分析。禁止在本任务升级到 5.5 |
| Q6 | Lua 非标准 API 的统一入参/返回形式与旧名兼容期限未指定 | 先提交逐 API 映射与 2–3 个典型调用例；确认约定后批量改，不能顺便改 lifecycle 或标准 Lua 名称 |
| Q7 | “与终端一致”缺少参考终端、字体、字号、行距、缩放，IME 支持范围亦未定 | 本机设置能读取则先调查；无法获得的由用户文字确认。建议同配置下几何/颜色/样式一致，不承诺跨终端逐像素一致 |

已能从后续明确文字消解的内容：包文本 i18n 不再接收 `path`，统一定位包内 `assets/language/<language_code>/package.json`；`screengame` 应归于已经定义的 `screensaver` 类型；`image` 资源是图像文件，不能按“国际化文本”的表述实现。旧 `display.json` 语言文件如何迁移仍受 Q2 约束。

## 6. B 阶段：功能实施工作包

执行顺序调整为：**B0 契约清单 → B1 → B2 → B3 → B5 → B6 → B4 → B7 → B8 → B9 → B10**。保留原编号便于追踪；B5 键位格式和 B6 转换模式先完成，B4 才能真正验收 actions、图标/banner 的 mix_block 全链路，避免先接受字段却没有实现。B10 的命名原则/兼容选择在 B0 先确定，实现和文档收口在最后。

每个下列子任务单独完成实现和测试，再进入下一个；这不是并行开发列表。待裁决项只阻塞依赖它的功能，其他独立调查可以继续。

### B0 — 开工前契约与交付分界（先做）

1. 以实际 `api/libraries.rs::install` 列出可调用 API，分为原生 Lua、宿主扩展、未注册占位。建立 `LUA_COMPATIBILITY.md` 与 `LUA_API_MIGRATION.md` 初始表；新文档使用可跟踪路径，不放 temp。
2. 已确认 Q1/Q2/Q4 直接执行，不重问。剩余 Q2a/Q3/Q6/Q7 按对应业务提前提问，附代码现象和具体候选契约；下面研究结果可作为提问依据。Q5 后续由用户确认并在 B5 记录。
3. Q3 的现状已经查明：`normalize_action_keys` 允许最外层 `[]` 表示未绑定，拒绝内层 `[]`，两层均最多 2 项，并做 token 规范化；复用该行为是推荐方案，不重写一套验证。重复 token/相同主备用绑定及 CLI 命令冲突仍需明确。
4. `src/main.rs` 目前没有命令行参数解析；游戏的 raw config 也没有 command。因此新 `game.command` 不只是清单字段，必须规划“进程启动参数 → 扫描完成 → 按命令唯一定位包 → 走现有启动命令”的链路。不是调用 shell 执行字符串。
5. 对 API 参数/返回建议先评审 3 类实调用：纯计算 `color.*`、选项多的 `draw.text`、异步 `file.read`/`image.load`；标准 Lua 永远使用原生协议。禁止仅凭“统一”把所有标准函数改成表参数。
6. B 的性能/视觉验证收集目标终端、OS、字体/字号、行距/缩放、IME 名称；用户 A 实机检查不能代替 B 新渲染行为的基准确认。

**B0 完成证据**：已实现 API 清单、已确认/未定契约表、B6 image 参数/事件草案、B4 命令冲突规则；未确定处明确阻塞，不虚构已批准方案。B1 不依赖所有 B0 答复，可先实施。


### B1 — 所有生产路径锚定上线根目录

证据：`crates/service/storage/src/service.rs::resolve_root_dir` 当前优先选择含 assets/Cargo.toml 的 CWD，失败还回退 `.`。截图字体加载中也有裸 `assets/fonts/...`。仅修 Storage 不足。

代码要求：从可执行文件路径解析生产根，在应用装配时传入需要它的服务；统一审计 storage、log/crash log、字体、i18n、包、FFmpeg、导出、缓存、脚本入口。保留已有测试显式传根的能力，不使用全局 `set_current_dir` 掩盖漏改。根不可解析/不可写时给出明确错误，不静默写到调用者目录。

测试：子进程从普通目录、含伪 assets/Cargo.toml 的目录、空格和中文路径启动同一部署副本；验证只有该副本根下产生配置/日志/缓存，调用目录不产生文件；检查资源加载、只读目标与缺资源失败行为。符号链接启动路径如存在产品语义分歧，先记录实际 OS 返回与期望再决定。

出口：外部目录启动、资源加载和存储位置全部正确；文档明确生产根与测试 fixture 根的区别。

#### B1 拆解与具体回归

| 子任务 | 代码与做法 | 验证出口 |
|---|---|---|
| B1.1 根路径 | app/services.rs 装配前解析 current_exe 父目录；StorageService 新增正常可用的显式根构造入口，复用现有初始化，不把 from_root_for_test 搬进生产 | 给定 executable 路径的纯函数测试；伪 CWD 含 Cargo.toml/assets 也不能改变根 |
| B1.2 崩溃日志 | core/fault/crash.rs::crash_log_dir 也优先 CWD；在安装 hook 时注入根派生的日志位置，fault 不依赖 storage；主程序提前失败仍有 stderr/终端恢复出口 | 受控子进程故障；日志只在部署根，失败不回退向 CWD 写入 |
| B1.3 字体/导出 | screenshot::FontSet::load 的内置字体目前是裸 assets/fonts；沿 task/rasterizer/video 调用传入绝对字体路径或根，保持同一套加载逻辑 | 不同 CWD 的字体选择、截图/视频尺寸一致；失效字体错误有路径 |
| B1.4 示例与集成 | storage_smoke 现在通过 set_current_dir 获取根，改用显式根；检查其他示例没有依赖旧回退 | 示例不写 target/debug 或仓库 data；外部 CWD 启动测试 |

注意 `StorageService::new` 当前初始化即建目录/载配置；新入口须保持此语义，不能只给 root_dir 赋值而漏 bootstrap。不要全局改 cwd。根解析失败的 boot 路径不得在 raw mode 中直接返回而漏恢复。

测试 ID：PATH-01 普通 CWD；02 伪 assets/Cargo.toml；03 中文空格部署路径；04 根不可写/不可解析；05 崩溃日志；06 screenshot/video 字体。每次检查调用目录前后文件清单，而非仅检查返回 PathBuf。

### B2 — Lua 原生语义对齐

入口：`crates/service/lua/src/session.rs`（只启用当前组所需的 Lua 标准库；当前已选择性启用 `StdLib::TABLE`）、`api/libraries.rs`、`api/libraries/{base,table,string,math,utf8,loader}.rs`、`api/readonly.rs`、`policy.rs`。

先产出 `LUA_COMPATIBILITY.md`：函数/语法能力、宿主实际行为、同版本原生行为、lua55 差异、安全限制理由、修复动作、测试编号。按已确认 Q4 保留 Lua 5.4。

代码要求：能安全使用原生实现的功能直接复用，不再全面手写替代。原生函数的多返回值、元方法、参数错误与迭代协议应保持 Lua 语义。宿主专用 API 留在明确命名空间。保留阻止进程执行、任意文件访问、终端接管、原生库载入及宿主内部泄露的限制；禁止以“原生对齐”为由开放危险标准库。

最低测试集合：

- `pairs/ipairs/next` 的迭代器协议、多返回、nil 终止、数组空洞、`__pairs`、`__index`；哈希遍历顺序不要跨运行强行比较。
- `setmetatable/getmetatable`、`__metatable` 保护、`__newindex/__len/__call/__tostring`、算术与比较元方法；只读宿主表不能经 raw 操作绕过。
- `pcall/xpcall/error/assert/select`、变参/nil、多返回、字符串 pattern/capture、table/math/utf8 的实际重写项。
- GC、弱表、终结器、`<close>` 正常/异常关闭，RegistryKey 与 Rust 资源释放；不把 GC 的不确定时间点当作固定输出。
- 无限循环、递归/内存配额、模块加载循环、禁用系统能力：错误受控，宿主与后续会话可继续使用。测试使用现有预算机制，不让测试自身卡死。

对照测试比较结构化结果；随机值、地址、时间、无序表作规范化。同步迁移旧测试包中 `for item in ipairs(...)` 接收 `{index,value}` 的旧协议，确保不是只改标准库名称。不要新增会吞掉差异的兼容 wrapper。

出口：差异表逐项关闭或有明确保留理由；同版本对照、宿主集成、安全负例和六个测试包通过。

#### B2 拆解与行为判定

现状不只是函数缺失：`base.ipairs/pairs` 返回记录对象；`base.next` 接收命名表；`args::one` 把含有参数同名键的 table 自动解包，`validate_value_limits` 拒绝循环表。这些规则若套在标准 `pairs`/`setmetatable` 上，会误伤正常 Lua table。readonly proxy 又依赖自定义遍历；恢复原生 rawset 后仅靠 `__newindex` 不能保护 table。

| 子任务 | 实施重点 | 必须通过 |
|---|---|---|
| B2.1 同版本基准 | 测试侧创建同一 mlua/lua54 的独立 Lua 实例，只跑固定可信样例；记录宿主/基准 `_VERSION`。不需要先安装 Lua 5.4 CLI，也不向用户脚本暴露测试标准库 | 一份脚本两端输出结构化结果；不同版本项单列 |
| B2.2 base 语义 | 先修迭代与元表协议；标准函数不再走 args::named/one 的宿主参数策略，保留普通循环 table、原生迭代三元组与多返回 | 原生 pairs/ipairs/next、循环引用、表含名为 table 的字段、nil 多返回、元表保护 |
| B2.3 readonly/环境 | 在导出原生 rawset/rawget/getmetatable 前确定只读代理策略；不得因代理空表让 pairs/next 变空，也不得泄漏 backing table、真实 globals/registry | 普通 Lua table 原生语义；宿主库不可写；沙箱 `_ENV`、模块环境不逃逸 |
| B2.4 标准库分组 | table → string/pattern → math/utf8 → 错误/GC/关闭；每组先对照再替换，发现安全例外写明，不大面积全开 StdLib | 保留数值/字符串/多返回语义；错误类别而非整段 traceback 跨引擎一致 |
| B2.5 宿主结合 | session 的 memory limit、run_with_budget、hook 和致命预算标记不删除；检查 pcall 不能吞预算后无限跑；loader 的 scripts 根、缓存和资源预算不旁路 | 六个包/生命周期、过期事件、内存/指令超限、释放资源回归 |

Lua 循环 table 本身合法；序列化或宿主消息不支持循环是另一个边界，不能混淆。标准库长时间 C 操作不一定触发 Lua 指令 hook：检查已有大小限制覆盖，必要时在暴露的高开销入口加有依据的限制；不把泛化“原生对齐”当作无上限许可。

`loader.require/dofile/loadfile` 是宿主扩展，当前 require 会缓存所有返回值；是否新增标准全局 require 与包搜索语义受 Q6/安全契约约束。先记录差异，不擅自开放 package.loadlib/os/io/debug 原生库。

测试 ID：LUA-BASE（迭代/循环/多返回）、META（读写/保护）、LIB（table/string/math/utf8）、GC（弱表/终结/关闭）、LIMIT（错误/预算）、ENV（模块/readonly）。所有对照输入固定；不比较 table 顺序、地址和随机垃圾回收时刻。

### B3 — 删除安全模式，保留必要沙箱

入口：storage/profile、package 数据模型、Lua `api/mod.rs`、file/event 权限、session，以及 app/world、路由、列表、设置、覆盖层、语言资源。

代码要求：删除 SafeModeDefault、safe_mode/high_privilege 字段、临时关闭状态、弹窗/高权限提示、相关动作和判断。API 权限只按 game/screensaver/both 与 debug 开关定义，并产出明确矩阵。删除安全模式不等于开放 shell、原始终端或任意宿主文件访问。

存量设置与包清单分开处理：Q2 只决定不再兼容旧包清单，不授权重置 profile。删除安全字段时保留用户语言、按键、存档、成绩；有实际数据迁移冲突先提问。保留不属于安全模式的有效设置，不按文件名整页误删。

测试：普通游戏与屏保的允许/拒绝矩阵；debug 关闭/开启；Lua 错误后回到宿主；旧配置载入后其他值不丢失；界面无残留入口；越界读写/删除/rename/copy/list 与 loader 均被拒绝；关闭游戏/屏保后任务和对象正确释放。将 `safe_mode_lab` 转为权限/沙箱测试包并同步全部引用，而非删掉负例。

出口：代码、配置、UI、示例、语言资源中不再保留有效安全模式逻辑；保留的历史兼容读取有理由和测试；基础隔离不退化。

#### B3 拆解与权限矩阵

已发现的实际门禁：`file_permission` 要求 Game 且 safe_mode=false；`event.skip_action/clear_action` 同样；`game` 限 Game；`debug` 限 debug_enabled；session 强制 Screensaver 的 safe_mode=true。建议保留会话类型约束，只移除布尔安全开关，不让屏保自动获得原先 game-only 能力。

1. **B3.1 配置/运行态**：从 storage/profile 的 SafeModeDefault、GamePackageState/默认设置和 app/world 的临时豁免开始，删字段的同时给“带旧安全字段但有存档/键位数据的 profile”补回归。Q2 的无旧清单兼容不等于清空 profile。
2. **B3.2 Lua**：逐注册函数写出 game/screensaver/debug 矩阵；移除 LuaApiConfig/LuaApiContext/session 中安全参数，更新所有构造者。保持限制方法当前“先权限判断，拒绝时 ignore_once”的行为，若要改为抛异常先确认并单独测试。
3. **B3.3 宿主 UI**：router/commands/render/action_map、host_machine、game_list、mods/game、screensaver_list 与 security 页面一起核查。删除安全模式入口后保留其他有效 debug/错误信息。语言资源同时清理中英两套。
4. **B3.4 包与夹具**：删除高权限声明/列表字段，按新无安全开关规则改 safe_mode_lab；B4 将完整升级清单，所以本步骤只做现有解析中必要的去字段，不提前并存两套 schema。

测试 ID：PERM-01 游戏 file 允许且沙箱有效；02 屏保仍按矩阵拒绝；03 debug 开关；04 game/event 行为；05 旧 profile 其余数据不丢；06 UI 路由无不可达残留；07 session 终止后任务不落到新会话。存档、包开关、成绩、键位的实际保存位置必须逐项读取验证。

### B4 — 清单拆分、扫描与热更新

入口：`crates/service/package/src/lib.rs`、storage 包状态/成绩/存档、Lua 启动参数、app 包事件、列表 UI、键位映射及 rich_text 的 key 展开。遵循已确认 Q1/Q2，Q2a/Q3 决定先落盘。

目标文件契约（已按用户确认的字段表修正）：

| 文件 | 内容与必要校验 |
|---|---|
| `package.json` | 可选声明 `package`；必需 `mod_id/schema_version/type/version/version_code/api`。类型 game/screensaver；mod_id 非空且符合新规则；version 可为字符串或文本对象；version_code 正整数；api.min ≤ api.max 且与宿主兼容。没有社区版本历史时不伪造“严格递增”验证 |
| `display.json` | 必需 title/description/author 文本；icon/banner 可省略使用宿主默认 |
| `game.json` | game 专用；name/detail/language/command/entry；尺寸缺省 0，不限制；truecolor/mouse/save_game 缺省 false；target_fps 缺省跟随玩家设置，有值使用支持的 30/60/120；best_score 缺失等价关闭，出现时 enable 必填，empty_text 可选且关闭时忽略。能力声明仅提示，不因终端不支持直接拒绝游戏 |
| `screensaver.json` | screensaver 专用；name/command/entry，尺寸缺省 0，truecolor 缺省 false |
| `actions.json` | game 可选，缺失等价空对象；动作 description、可选 lock=false、二维 keys，每层最多两个元素，超限或无效值报错；解析、配置覆盖与 UI 锁定必须一致 |
| 文本对象 | 字符串等价 text 对象；text 模式必需 text；i18n 模式必需 key/callback，固定语言文件，无任意 path；无关字段按确定的解析规则处理 |
| 图像对象 | type=text/image，path 相对包 assets；text 为 txt，image 为 jpg/jpeg/png；block 缺省 half_block，可选 mix_block，text 模式忽略 block |

`entry` 相对包 `scripts/`，支持省略 `.lua`。尺寸需检查负数、溢出；游戏 min 尺寸针对 base 画布，屏保针对整个终端。帧率实际受玩家上限、包目标及执行能力共同约束，不强行保证硬件达标。

加载流程分为“读入并验证候选包 → 构造完整有效快照 → 发布”。沿用现有 notify/任务模型，监听所有拆分文件、所引用文本/图像/语言/脚本。处理编辑器临时写、删除重建、重复通知、旧扫描晚到；不向运行会话发布半个配置，不混用新旧字段。对必需文件丢失、坏包隔离、正在运行包的替换，先保留现有有证据的行为；若要改变 UX，先裁决。

测试：game/screensaver 最小包、每个必需字段缺失/类型错/越界、API 区间、entry 逃逸、i18n 回退、默认图像、actions 缺失/锁定/非法键、重复 ID 与命令；逐个配置文件热更新、删除/恢复、坏 JSON、语言切换及运行中改包。同时更新官方 `scripts/` 包、六个 `test_package/` 包与文档示例，不能只让解析单测通过。

出口：多文件扫描、列表、启动、热更新、存档/成绩/自定义键与富文本 `{key:...}` 全链路一致；错误指出具体文件和字段；旧格式严格拒绝、schema=2、新格式说明和用户数据处理记录可交付。

#### B4 拆解、数据模型与热更新

已确认直接升级：`PACKAGE_MANIFEST_VERSION=2`；HOST_API_VERSION 与 MEDIA_MANIFEST_VERSION 不为“看起来同步”一起递增。新结构不接收旧嵌套 runtime/game/display，也不为 save/score 等提供兼容别名。使用原有 deny_unknown_fields、serde_path_to_error 和大小限制，错误新增来源文件，不能所有错误仍标 package.json。

| 子任务 | 具体实现范围 | 最小验证 |
|---|---|---|
| B4.1 多文件解析 | read_package 从一个 RawPackageJson 改为头部 + display + 类型配置 + 可选 actions；可独立测试的配置读取/验证拆分，保留最终 PackageInfo 聚合 | 每文件缺失/坏 JSON/未知字段/类型错；旧 schema 明确拒绝 |
| B4.2 身份与数据 | 新清单 ID 严格校验；PackageId::Deserialize 当前用于 profile，避免直接收紧导致旧 profile 全部回退；按 Q2a 处理测试 ID/旧数据 | 同 source/type/id 存档仍能识别；不同来源不串档；无静默清空 |
| B4.3 运行配置 | 将 target_fps 缺省保留为 Option，不在解析时补 60；传到 Lua/Game 服务和调度器。当前 runtime 使用 game FPS 优先的 or_else，不满足 min(玩家,包)，须修正 | 玩家30/包120→30；玩家120/包30→30；缺省跟随；离开游戏恢复宿主 |
| B4.4 资源/键位 | 固定 assets/language/<code>/package.json；action 的 description/lock/default/user 映射与富文本一致；B6 的 block 参数经 PackageAsset 传到两处模组页面的 icon/banner 转换调用 | 语言回退/callback、锁定键、自定义键、两个 block 模式实际不同 |
| B4.5 扫描排序 | request_rescan 与 request_rescan_for_language 分配单调请求序号，SnapshotReady 携带；只发布最新候选，旧进度/完成不能覆盖最新语言与 UI 状态 | 人工控制两任务完成顺序，旧任务晚到被忽略；最新失败有诊断 |
| B4.6 watcher | watched_package_dir 当前只认 package.json；补全部配置名，以及缺失可选 actions 后续创建、坏包恢复、语言/资源变化。复用通知合并，不建立第二套线程系统 | 每文件变更/删除/重建；缺失 actions 创建；无效包修好后自动可见 |
| B4.7 CLI 启动 | main 解析参数但不做包业务；app 在首次扫描后唯一解析 command，调用现有 launch_game/屏保路径，校验尺寸/存档行为不绕过 | 唯一命令、未知/重复/宿主保留参数、game/screensaver；路径带空格不走 shell |
| B4.8 fixtures | 全部 6 个测试包及官方 3 个清单升级；官方缺脚本不可通过新建空 main.lua 假装修复，确认是保留为负例还是提供真实最小脚本 | 正向包确实可启动；负例明确标记且不混入可玩列表验收 |

多文件发布只能保证宿主构造完整候选后替换，不能假装 OS 多次保存天然原子。采用现有 debounce/稳定重读与版本序号，拒绝不完整候选；“运行中坏更新保留上个有效包还是暂下架”需 Q3 补充决定。不在 B4 偷做正在运行 Lua 代码的热替换；先保留现有会话生命周期。

入口、文本、图像都通过包根约束；`i18n.path` 删除仅指清单文本对象，不自动删除 Lua i18n API 用于其他命名空间的合法 path。测试分别覆盖两种用途。

测试 ID：PKG-HEADER/FILES/ASSET/I18N/ACTIONS/IDENTITY/FPS/WATCH/STALE/CLI，每类有正例和失败例。脚本、json、字体等夹具放专用测试根；拒绝旧格式时错误说明新 schema 和必需文件，无兼容解析器。

### B5 — 左右修饰键显示与映射链路

入口：`crates/core/input/src/{key_token,key,action_map}.rs`、input service、app/action_map、键位 UI、rich_text 的 key 展开及语言资源。

保存 token 保持规范（left_ctrl/right_ctrl 等），显示层区分 LCtrl/RCtrl、LShift/RShift、LAlt/RAlt，Meta 按 Q5。确保包默认键、用户绑定、UI、富文本与事件解释使用一致转换，不直接用展示字符串存储。

测试：左右单键与组合键、主/备用绑定、旧配置、按下/重复/抬起、非法 token；各平台名称函数测试。终端后端若只能报告聚合 Ctrl/Alt，不得假造左右信息；记录系统监听器权限不足时的已有降级并实测 Windows/Linux/macOS 能力。

出口：展示正确、绑定不变、无法区分的输入来源有明确行为及测试。

#### B5 拆解

1. `key_token.rs::display_key_token/format_key_display` 是显示收口点；先给所有左右 modifier 建表驱动测试，再改此处。解析 alias `ctrl → left_ctrl` 当前存在，显示需求不自动废除 alias。
2. 搜索富文本 key/default_key 和按键页，排除重复格式化；`format_key_display` 长度增加后检查列表换行/裁剪，不能按旧 Ctrl 字数定位。
3. Q5 已由用户确认：macOS 显示 Cmd、Windows 显示 Win、Linux 显示 Meta。实现私有平台格式化函数并注入平台枚举测试，避免只能在对应 OS 运行名称单测。
4. rdev 系统事件与 crossterm 终端事件的左右信息分开验证；没有物理信息就保持已有未知/聚合语义。不要将一个 Ctrl 输入同时发送成左右两次。

测试 ID：KEY-DISPLAY 全修饰键/平台；KEY-ROUNDTRIP 规范 token 持久化；KEY-ROUTE 单击/组合/释放；KEY-LAYOUT 新长度的富文本和按键页。此块完成后再做 B4 的 actions 全链路。

### B6 — half_block / mix_block 与图片缓存

入口：`crates/service/image/src/lib.rs`。当前 `compute_hash` 基于路径/参数，内存缓存可直接返回；仅靠磁盘 mtime 校验不足。

先研究 `temp/` 中已有上游库与当前实现，记录候选算法/字符集、采样、颜色量化、宽高比及许可证。half_block 沿用并修正必要问题；mix_block 使用有确定匹配规则的 Unicode 方块，不实现无界搜索或新字体系统。

缓存键包含同一份原图字节、所有影响输出的规范化参数、转换模式和算法/缓存版本；图像解析与摘要使用同一读取快照，避免文件读取两次内容不同。复用合适的稳定摘要实现，不宣称数学上的绝对无碰撞；损坏缓存/旧版本当作 miss，写入原子化，内存/磁盘命中条件一致。缓存关闭时不读写缓存，不用旧路径继续返回旧内容。

测试：同路径同长度换图、相同 mtime 换内容、同内容不同参数/模式、坏缓存、旧版本、并发/取消写；1×1、奇数高宽、透明像素、纯色、渐变、边缘、目标尺寸为零/过大、无效图片。检查字符 cell 数、前背景色和尺寸，性能使用相同样本对照；禁止把生成图片上传到会话。

出口：两种模式可从包资源和现有 Lua 图片接口实际使用；内容变化可靠失效，输出尺寸、颜色和耗时有记录。

#### B6 本地研究结论与可执行拆分

保留研究源码（仅阅读代码，不读 README 内图片）：

| 本地源码 | 已读证据 | 本次用途 |
|---|---|---|
| temp/image-converter-reference/logo-art | MIT；src/lib.rs 有 Cell::HalfBlock/Quadrant、glyph(mask)、cell_colors；2×2 quadrant 分组和颜色聚类可借鉴 | 首选小范围算法参考；输出要转宿主 rich text/单元格，不能直接把 ANSI 写到终端 |
| temp/image-converter-reference/img2irc | GPL-3.0-only；支持 full/half/quarter/eighth 等候选 | 研究思路；引入/复制前核查许可，与仓库 MIT 不可混同处理 |
| temp/image-converter-reference/classcii | MIT OR Apache-2.0 workspace；包含较完整 ASCII 应用 | 参考转换质量/性能，不把 GUI/音频导出全套框架引入 |

推荐 mix_block 首版采用现有截图几何绘制已支持的 U+2580–259F 方块集合：half/quadrant/eighth 的有限候选，按覆盖掩码分别算前后景代表色与误差，稳定顺序择优；不用默认引入低覆盖的 sextant/octant。先复用可兼容许可的现成实现，确实不能适配宿主输出再做最小改造，记录出处。

| 子任务 | 具体改动 | 验证 |
|---|---|---|
| B6.1 参数与源读取 | ImageConvertParams 加转换模式；scale 检查 is_finite（目前仅 <=0，NaN 可漏过）；尺寸/解码像素预算与裁剪乘法检查；一次读 bytes 同时用于解码/摘要 | NaN/Inf、0/极大、奇数尺寸、坏格式、越界 crop |
| B6.2 缓存 | compute_hash 目前 scale 格式化只保留 6 位小数，内存无 mtime 校验，磁盘 mtime 只有秒；改内容摘要+精确规范参数+版本，IMAGE_CACHE_FORMAT_VERSION 从2升级；复用 atomic_fs 写入 | 同路径同秒换图、六位小数后 scale 差异、损坏/旧缓存、原子失败、cache=false |
| B6.3 转换 | half 保持基线；mix 有限 glyph/mask 与色差匹配；定义透明像素混合背景并计入 cache key，尺寸始终以 cell 为单位 | 纯色/透明/边缘；误差不劣于候选中的 half；确定性与预算 |
| B6.4 宿主 UI | game.rs/screensaver.rs 的四个 icon/banner ImageConvertParams 调用同步模式；避免每帧重复读大图片，利用页面/资源修订缓存，内容热更新必须失效 | 连续帧无重复重转换，换内容刷新，缺资源正常提示 |
| B6.5 Lua 请求 | 当前 install 没 image，LuaHostCommand 没 ImageRequest；补 library→host command→app submit/register_task→ImageEvent→broker→HandleEvent，复用 session token/generation，勿在 mlua 回调里直接阻塞解码 | 真正 Lua 脚本 image 请求→结果→draw；坏路径/取消/退出/晚到事件 |

B6.5 接口先在 B0 对照 image.md 裁决：文档 path/block_width/block_height 的默认值可能对小图算出 0，Rust 默认却为80×24；异步返回目前文档只有事件。不得直接选其中一个且不记录。推荐明确有效最小尺寸和事件关联字段；是否同步返回 task id 跟 Q6 一致。

服务目前 resolve_path 可读任意宿主路径；Lua 新入口必须先经 session.assets_root + sandbox_path，既限制显式扩展也限制自动补后缀查找；缓存文件名不得由 Lua 任意指定。ImageTask 目前忽略 cancellation，磁盘缓存写也不进入写屏障：新公开异步链路须决定可取消阶段，接入已有原子写/写屏障或明确缓存失败可丢弃的关闭语义，不修改共享执行器的事件规则。

测试 ID：IMG-PARAM/CACHE/MODE/UI/LUA/CANCEL/SANDBOX。此块只新增 image 已请求链路，不顺便补 audio/http/widget 所有尚未注册接口。

### B7 — 截图与录屏导出渲染一致性

入口：`crates/service/screenshot/src/lib.rs` 的 `RasterMetrics/TerminalFrameRasterizer/FontSet/draw_cell_text`，video、recording、core/style 的 cell/frame，字体设置 UI。

先解决 Q7 并记录基准环境。导出应以实际 presented 的 `ComposedFrame` 为准；PNG 与视频帧复用同一栅格化路径。不得为了更好看的图片改写文本内容、宽字符占位、图层或样式语义。

重点：字体选择/回退、cell 宽高、基线与字距、CJK 双宽、grapheme/组合字符、emoji/变体选择符、粗斜体/下划线/反色、透明/默认色与终端颜色降级。检查缺字体/缺字有明确回退，不能随机取不同字体导致基线跳动。视频偶数尺寸补边不得裁掉末行末列。

先评价现有 fontdue/fontdb 能否满足；明确不足后才选择形状整形/栅格化依赖，记录取舍。不同系统字体/终端抗锯齿不要求像素完全相等；固定测试字体下几何和颜色应可重复。

测试：固定字体 fixture 的 ASCII/CJK/组合字符/宽字/方块/空格/样式矩阵；PNG 与视频编码前帧同配置一致；裁剪从宽字续格开始、缩放、空画布、小尺寸、缺字体；视频长度/FPS/帧数/音画时间戳、取消、缺 FFmpeg、失败清理。字体 fixture 确认允许分发；自动测试验证像素区域/基线/尺寸而非仅断言文件存在。

出口：数值回归通过，用户在约定终端本地对比并文字确认观感；H.264/AAC 集成测试执行。未获视觉证据不能声称“完全一致”。

#### B7 根因定位与分步修正

本轮已确认：固定 CELL_WIDTH=18/CELL_HEIGHT=36/FONT_SIZE=27；baseline 为 cell_height×0.78；font_for_char 按 char 选字体；复杂 grapheme 通过码点宽度启发式处理；已有 draw_block_element 的 half/quadrant/eighth 几何矩形。优先修这几处，不把 FFmpeg 编码器当成字符间距错误来源。

1. **B7.1 可复现基线**：固定字体与样本 ComposedFrame，记录字体文件/face、cell 几何、导出比例；保存像素统计和本地结果。PNG 正确后才分析有损视频色差。
2. **B7.2 指标模型**：由参考字体 ascent/descent/advance 和选定行距推导 cell/baseline；同一次截图与视频导出使用同一不可变配置。图片裁剪不得每个 cell 重新缩放字体。
3. **B7.3 cluster 与样式**：先处理同一 grapheme 的字体选择、组合符号与占格，再决定是否引入 shaping。若 fontdue 单字光栅无法满足 cluster 需求，拿 ZWJ/组合字失败用例证明后评估依赖；不以“用了更高级库”代替可比结果。
4. **B7.4 几何字符**：复用已有 block/box 绘制，检查奇数 cell 像素宽高下分割线。现实现以 div_ceil(8) 相乘分界，小尺度可能不均匀；使用边界比例计算的候选需用覆盖面积/缝隙测试验证。
5. **B7.5 统一呈现颜色**：terminal 的真彩/ANSI256 与栅格 resolved_colors 路径比较，决定导出使用实际显示色还是原始设计色；目标为实际呈现时应共享既定解析结果。旧录屏没有能力/字体元数据时采取明确旧文件读取策略，不默默改变历史帧含义。
6. **B7.6 编码验收**：同 raster 配置比较 PNG 与视频编码前 RGB；有损编码用容差，不对 MP4 解码像素要求绝对相等。音频缺失/取消和 H.264/AAC 测试重跑。

测试 ID：RASTER-METRICS/CLUSTER/STYLE/BLOCK/CROP/COLOR/VIDEO。像素断言按固定测试字体，用户本地文字验收确认参考终端观感，不能读图工具验收。

### B8 — 全屏动态字符渲染压力测试与有限优化

入口：`crates/service/render_pipeline/src/{presenter,compositor}.rs`、terminal、app/render、帧调度。目前已有差分和连续文本输出，先复用，不从头再造渲染器。

先测基线：80×24、160×50、240×80；静态、局部更新、全屏每帧更新，ASCII/CJK/富样式；30/60/120 FPS 目标。同机器/终端/字体/窗口下记录至少一段稳定运行的 CPU、每帧字节量、写次数、p50/p95 帧耗时、实际 FPS、输入延迟与闪烁反馈。不要拿内存 writer 跑分代替终端实测。

按数据选择一次帧缓冲写入、减少样式/光标切换、连续 run、合理差分；终端同步更新必须探测/配置与降级，不默认所有终端支持。输出失败时不能把未写完的帧标记为已呈现；确保同步更新结束与终端清理异常安全。

测试：无变化、局部变化、全量、resize、零/极小窗口、末列宽字、缩小再放大、彩色降级、强制重绘、writer 中途失败。比较优化前后文字/样式帧和输出协议，避免陈旧像素；不以固定性能提升百分比造验收目标。

出口：同场景数据可复现，正确性无回退，用户反馈在目标终端中闪烁改善或已明确终端瓶颈；不承诺任意终端完全无闪烁。

#### B8 先可观测，再最小优化

presenter 当前直接向 Stdout queue，帧末 flush 后才保存 previous；失败不更新 previous 这一行为必须保留。已有同样式 run 与按 cell 差分；Empty 分支只结束 run，Text→Empty 是否真的擦除需结合 compositor 实际输出验证，不能仅看实现就认定缺陷。

1. **B8.1 测试接缝**：让现有 present 的编码逻辑可写入 impl Write/Vec<u8>（局部内部函数即可），终端所有权仍由 TerminalService 管。测 emitted bytes、写次数；增加可指定第 N 次 write/flush 失败的 writer。
2. **B8.2 正确性样本**：Text→Empty、宽→窄、样式恢复、末列、resize 全量刷新、input_cursor 变化。必要时用现有 parser/简单测试终端模型还原 cell，不建立生产终端模拟器。
3. **B8.3 基线压测**：固定随机种子生成样本，预热后每场景运行30秒，记录环境与 p50/p95/最大值；性能实机用 release。测试生成场景本身耗时单独计，避免测的是造数据。
4. **B8.4 优化一项一测**：先比较帧级 Vec<u8> 合并写；再判断样式重用/连续 run 是否仍有主要收益。同步更新只在支持时启用，写失败时尝试结束同步状态并恢复终端。
5. **B8.5 退出标准**：同样正确性与环境下报告前后数据，若全屏更新瓶颈在终端则停止继续造复杂缓存。不要通过丢用户输入或降低设定帧率伪造吞吐提升。

测试 ID：PRESENT-DIFF/EMPTY/WIDE/STYLE/RESIZE/FAIL/PERF。B9 随后沿用该帧写入策略，不另造第二条绘制通道。

### B9 — 输入焦点、光标与 IME 预输入锚点

入口：presenter 的 `final_cursor`、app 输入模式/焦点、widget text_input、input_method、terminal。先确认实测终端及系统输入法，不能将“帧尾 MoveTo 正确”当成预输入期间已正确。

代码要求：每帧使用明确的逻辑输入光标，坐标包含视口/滚动/宽字符影响；失焦、页面切换、覆盖层、resize 后立即更新或隐藏；结合 B8 输出策略避免 IME 持续观察到绘制光标。保留系统 IME，不用自造拼音/日文预编辑层替代。协议能力不足时给出实际支持边界，不加入无依据的终端专用控制序列。

自动测试：焦点/光标坐标和输出顺序、零尺寸、滚动、宽字、多输入框、失焦与异常清理。实机：中文拼音/日文罗马音候选未提交时后台持续刷新、移动候选、提交/取消、滚动、切换页/覆盖层、改变窗口大小、退出；观察候选窗与预输入停留在输入位置，输入内容不丢失，关闭后输入法恢复。

出口：协议测试与目标终端实机文字记录均通过；其他平台未验证单列，不泛化保证。

#### B9 已有能力边界与复现矩阵

当前 TerminalService 入终端后 Hide，text_input 绘制模拟光标，app/render 返回 input_cursor，presenter 仅在帧尾 MoveTo；input_method 服务只切换/恢复输入法与开关状态，没有操作候选窗位置的接口。这解释了为什么不能仅修改 input_method 就修好动态预输入锚点。

1. **B9.1 状态追踪**：从最上层输入焦点到 widget 的可见坐标再到 final_cursor；记录每帧输入框坐标、最终光标、输出次数。剪贴/密码/多行/滚动都使用现有控件语义。
2. **B9.2 原因对照**：在同终端同 IME 下比较静态页面、后台动效、B8 帧缓冲、同步输出能力。区分候选窗位置与画布上的模拟插入光标，分别记录。
3. **B9.3 修复**：先利用批量输出/同步能力缩短绘制光标暴露时间，保证焦点切换和 resize 的最终坐标正确；显示真实光标是否有帮助需实测，避免叠加双光标。不要声称 save/restore cursor 自动锁定 IME。
4. **B9.4 恢复**：切出输入模式、页面关闭、错误、退出时恢复光标和原输入法；失焦光标不指向已删除 widget。

实机矩阵必须包含中文拼音与日文罗马音（对应环境具备时）、中间候选选择/提交/取消、连续背景更新、窗口缩放和覆盖层。若目标终端协议不能稳定锚定，应交付已测支持范围和复现，询问产品降级选择；不能用禁用 IME 或永久冻结所有游戏帧作为默认修复。

测试 ID：IME-FOCUS/SCROLL/WIDE/OVERLAY/RESIZE/COMPOSE/RESTORE。自动坐标测试通过只算其中一部分。

### B10 — 已实现 Lua 扩展 API 的命名、参数、返回值与文档统一

依赖 B2 的标准 Lua 契约及前述新增/变更接口。Q6 确认后实施。

先扫描实际注册表与实现，产出 `LUA_API_MIGRATION.md`：旧名→新名、调用形式、参数名/类型/可选/默认、返回形状、错误、可用会话/debug 条件、对应测试。只列已实现 API，不借机实现文档中的未来能力。

标准 Lua API 和 lifecycle 名称保持原样；扩展命名应符合 Lua 项目约定、语义完整，常见缩写有理由才保留。先完成一个代表性 API 族并测试，再逐族迁移；同步 Rust 注册、Lua 包、宿主调用、示例、事件结构与 `dev_docs`，不留新旧返回形状并存的隐性契约。

返回值与错误形式按确认的统一原则实现；异步任务 ID、进度/成功/失败事件不能与同步返回混淆。文档每个 API 包含调用例、参数表、返回示例、失败/权限和常见边界；示例应能执行。对用户正在编辑的文档进行局部合并，不整体覆盖。

测试：每个已改 API 的正常调用、缺失必需参数、错误类型、可选缺省、nil/多返回、业务失败和权限场景；文档示例烟测；全仓旧名检索逐项说明保留理由。不要用纯字符串快照代替调用验证。

出口：注册表、实现、文档、测试包完全对齐；差异表与 API 迁移表闭环；第 7 节最终回归完成。

#### B10 清单驱动迁移，而非扩展新 API

当前 install 有 base（部分也导出全局）、math、utf8、table、string、color、char、align、measurement、random、slice、serialization、encoding、draw、debug、game、i18n、event、loader、file；image 由 B6 补齐。audio/animation/http/timer/effect/widget/ime/keyboard 不能因存在服务或文档文件就当作已实现 Lua 库。

1. **B10.1 契约定稿**：B0 的表更新为最终注册清单；每项记录 old/new、参数单位与默认、同步返回/异步完成/错误、权限、测试和文档。原生 Lua 函数不使用统一扩展协议，lifecycle 名称与调用时机不改。
2. **B10.2 实现迁移**：按纯值工具 → draw/measurement/slice → file/i18n/image 异步 → game/event/debug/loader 分组，每组更新调用与测试后再下一组。Q6 决定是否留扩展旧名；Q2 无旧包兼容不能被擅自推导成所有 API 都无需兼容。
3. **B10.3 异步事件**：DOC_DRIFT D2/D3 已证实一些组件事件/独立回调只有 cfg(test)，没有生产事件来源。标明未实现或待单独需求，不通过打开 cfg(test) 就暴露半成品。HandleEvent 现有链路和 B6 image 必须覆盖终止事件只送一次、旧 session token 丢弃。
4. **B10.4 文档校正**：D6 富文本实际为 f% 与标签/参数，不能复制旧 tc/ts 语法；D7 byte=true 才是二进制；serialization 格式名虽对齐，仍须测 nil/空容器/数字边界/循环与错误。用户编辑文档局部合并。
5. **B10.5 可执行示例**：API 文档每组至少一个端到端 fixture；提取示例或维护文档引用到测试脚本，避免文档与复制测试两份漂移。全仓旧名搜索区分历史迁移表与生产调用。

测试 ID：API-INVENTORY/ARGS/RETURN/ERROR/PERM/ASYNC/DOC。最终每项应能从注册函数追到测试与文档，不以文件数或行数作为完成指标。

## 7. 测试、边界与最终验收

### 7.1 验证节奏和命令

遵循历史加速决定：小型改动执行必要的受影响测试和 `cargo build --workspace --all-targets --locked`；每个业务工作包完成后跑 workspace 测试；语义、安全、数据写入修改必须当块验证。A3 已验收，不重复执行；release、全部示例和完整实机集中在 B 最终验收，多媒体、路径、IME 等受影响能力仍在对应工作包即时验证。

在仓库根目录顺序执行，检查每条退出码，失败不继续提交：

```powershell
cargo fmt --all -- --check
cargo build --workspace --all-targets --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --release --locked
```

all-targets 已编译 examples；用 Cargo metadata 枚举真实示例名，直接运行对应构建产物并记录结果。不要根据过时“45”硬编码数目。普通 `cargo test` 只测默认成员，不能替代 `--workspace`。

多媒体条件具备时补跑：

```powershell
cargo test --workspace --locked recording_with_audio_exports_as_h264_aac_mp4 -- --ignored
```

执行前核对测试过滤参数确实选中了 1 个测试；0 tests 不算通过。也可以使用 `cargo test --workspace --locked -- --ignored` 执行全部忽略测试并逐条解释新增条件。

历史环境问题：单独 `cargo test -p tg-service-video` 等可能因 feature 组合变化触发 openh264/tree-sitter C 重编，旧记录中 MSYS2 gcc 曾失败。本轮 workspace 构建成功不证明干净构建已修复。遇到问题保留首个错误、工具链/feature 证据，先区分源码与环境；禁止为绕过编译删功能、改依赖版本、盲目清 target 或直接全系统升级。需要干净构建时使用独立 target-dir 或 CI，不能破坏现有缓存来试错。

### 7.2 不可省略的边界矩阵

| 领域 | 必测边界 | 可接受结果 |
|---|---|---|
| 文件沙箱 | `..`、绝对路径、盘符/UNC、混合分隔符、符号链接/junction 逃逸、写入尚不存在的叶子、根目录删除；覆盖 file/loader/image/包入口/导出调用链 | 统一拒绝逃逸，无根外副作用；不能仅字符串 starts_with 判断 |
| 文件完整性 | 保存/导出取消、临时文件、覆盖失败、权限/磁盘写失败、退出时未完成写入 | 保留原有效数据，清理仅自己拥有的临时文件，错误可追踪 |
| 沙箱强度 | Lua 异常/死循环/资源预算、禁用进程/终端/任意原生模块能力 | 受控错误，不击穿宿主；不得宣称进程内 Lua 能隔离任意 native crash/OOM |
| 生命周期 | boot 部分失败、runtime 故障、重复取消、晚到事件、退出期间输入/热更新 | 无死锁/重复释放/错误会话回调；终端恢复 |
| 配置热更新 | 连续保存、半文件、删除后重建、运行中更新、旧任务覆盖新结果 | 不发布半状态、不丢配置、不把宿主拖垮 |
| UI/Unicode | 空列表、缩小窗口、零尺寸、CJK/组合字、宽字末列、滚动与焦点 | 无 panic、越界、残影或错位命中 |
| 跨平台 | Windows 路径/句柄/IME，Linux 与 macOS 编译、终端恢复与输入能力 | 平台分支有证据；未测项明确列出 |

对主动修改符号链接等安全竞态，先检查现有保证与真实威胁，再选择平台安全文件操作；不能以低概率忽略安全边界，也不要没有证据就重写整个文件系统服务。

### 7.3 实机回归最小清单

在上线结构副本中执行并记录环境、包、操作、预期、实际、日志路径：

1. 首次启动、已有配置启动、语言选择/终端能力设置、窗口缩小恢复。
2. 六个测试包均可扫描；游戏启动/输入/返回，屏保进入/退出，至少一个完整游戏流程与存档恢复。
3. 设置、包列表、按键修改、媒体列表筛选/滚动/重命名、存储管理页面的继承改动。
4. 包文件变更后的热更新；坏包与 Lua 错误不会破坏宿主。
5. 截图、录屏、视频/音频导出、取消和退出；退出后无本轮遗留线程/子进程/锁文件。
6. 正常关闭与可控故障后：raw mode、备用屏幕、鼠标上报、光标、样式及输入法恢复。
7. B 阶段追加：外部 CWD 启动、新清单、无安全模式、原生 Lua、左右键、两种图像转换、导出观感、渲染压力与 IME 场景。

测试目录中的删除/覆盖必须核实完整绝对路径确实在指定测试子目录内；Windows 文件操作只用同一套 PowerShell 原生命令，避免跨 shell 拼接删除。

### 7.4 Definition of Done

- A：已按第 4 节和用户确认的实际阶段边界关闭；不再阻塞 B，不重做旧迁移。
- B：B1–B10 每项需求有实现、有效测试、边界和文档；已确认 Q1/Q2/Q4/Q5 和剩余 Q2a/Q3/Q6/Q7 决定留档；旧数据处理明确；完整回归与目标环境的视觉/IME 确认完成。
- 后续不得减少测试来保持“623”；新增测试可增长，删除/迁移旧测试逐项解释。ignored 或无法验证事项不能归入通过数。
- Windows/Linux/macOS 的构建与平台验证逐项记录；环境不具备时写明未验证和下一步，不能用 Windows 单机结果宣称三平台完成。

## 8. 执行状态与交接

当前计划以本文件为准；A 证据与旧迁移历史在 `dev_docs/refactor/`，不再维护 temp/handoff 或等待旧状态 Agent。新实验可用 temp，但保留复现脚本/必要结论到可跟踪目录。

| 工作包 | 当前状态 | 下一步 / 必要决定 |
|---|---|---|
| A0–A3 | 已验收 | 用户本轮确认接受；不重跑 A 全套 |
| B0 契约 | 首版契约清单、裁决表、B6 image 草案及 B4 CLI 候选已归档；未决项保留为对应工作包的前置问题 | Q2a/Q3/Q6/Q7 与 image 参数在对应实现前确认 |
| B1 路径 | 自动实现与回归完成；Windows workspace build/test/strict Clippy/fmt 通过 | 留待 B 最终验收的跨平台与真实终端运行；继续 B2 |
| B2 Lua | 已完成；B2.1–B2.5 覆盖标准库、错误预算、GC/终结器、关闭与宿主生命周期 | 不重开；按最终全量回归复核 |
| B3 安全模式 | 已完成 | 权限矩阵与 profile 兼容回归见 `dev_docs/refactor/B3_PERMISSIONS.md` |
| B5 左右键 | 实现完成（Windows自动回归通过） | Linux/macOS 实际系统监听与构建留待最终跨平台验证；进入 B6 |
| B6 图片 | 进行中（契约已确认，B6.1 实施中） | 已确认默认尺寸/裁剪/透明背景/request id/event output；逐块完成服务参数与缓存→转换模式→宿主 UI→Lua 全链路 |
| B4 清单 | 待 B5/B6 后执行 | schema2、新格式完全升级；Q2a/Q3 |
| B7 栅格化 | 待执行 | Q7 基准终端字体；先固定样本 |
| B8 渲染 | 待执行 | 先可测输出和基线，再一项项优化 |
| B9 IME | 待执行 | Q7 目标环境，动态预输入实机验证 |
| B10 API/文档 | 待执行 | Q6 契约提前在 B0 定，最后逐组收口 |

每个子任务记录：开始基点、变更文件/符号、行为契约、测试 ID/命令/退出码、实机环境与结果、仍未完成项、下一条具体操作。测试数不是覆盖证明，失败和未执行不得写为通过。

### 2026-09-27 B0/B1 执行记录

- B0 首版成果：`dev_docs/zh_cn/LUA_COMPATIBILITY.md`、`dev_docs/zh_cn/LUA_API_MIGRATION.md`、`dev_docs/refactor/B0_CONTRACTS.md`。确认的 Q1/Q2/Q4 已记录；Q2a/Q3/Q6/Q7 和 image 参数仍只在对应实现前提问，不据候选草案视作批准。Q5 后由用户确认并记录在 Q5 决策表与 B5 执行记录中。
- B1.1–B1.4：完成部署根解析与显式 Storage 构造、崩溃日志路径注入、截图/视频字体根传递、smoke 示例路径修复和 CWD 审计。回归覆盖普通/伪项目/中文空格调用目录、根目录初始化失败、子进程崩溃日志、显式字体失效回退、相对路径按部署根解析、全来源失败列出路径，以及 EngineServices 只在部署根创建数据。
- 验证（Windows）：`cargo fmt --all -- --check`、`cargo build --workspace --all-targets --locked`、`cargo test --workspace --locked`、`cargo clippy --workspace --all-targets --locked -- -D warnings` 均退出 0；`storage_smoke` 与 `ffmpeg_smoke` 已运行通过。Clippy 首次发现 `save_png` 参数过多，改为接收 `ScreenshotTask` 后复查通过。
- 边界：外部 CWD 回归在复制的测试可执行文件中验证生产根解析和服务装配，没有启动交互 TUI；根不可用由 `data` 被普通文件阻塞模拟，没有验证 Windows ACL 只读或其他 OS。截图与视频复用同一 rasterizer/font loader 和尺寸 helper；本块未做真实编码视频与 PNG 像素逐点比对。Linux/macOS 留待最终验收。
- 临时清理按 `dev_docs/refactor/A_ACCEPTANCE.md` 范围保持未执行；5 个目标目录仍受原策略阻止。本轮未重试，也未使用替代删除方法。保留 `architecture/` 与 `temp/image-converter-reference/`。

**阶段交接**：当时计划继续 B2.5；其完成记录见本文末尾。B2.1–B2.3 记录见下；A 阶段不重跑，旧临时迁移脚本不重放。

### B2.1 首个同版本基线

- 新增 `session::tests::lua_compatibility_baseline_runs_the_same_sample_on_lua_54`：同一固定可信脚本分别运行于独立 `StdLib::ALL_SAFE` 基准 VM 与生产 sandbox session，核对两端 `_VERSION` 相同且为 Lua 5.4，并断言结构化迭代输出。
- 首个差异已记录在 `dev_docs/refactor/B2_LUA_BASELINE.md`：原生 `ipairs`/`pairs` 返回 iterator/state/control 并按多返回产生 key/value；当前宿主返回单 iterator 和 `{index,value}` 记录。没有依赖哈希遍历顺序。
- 探索时还复现了 `type` 遇到迭代记录表字段冲突报 `base.type: unknown parameter 'index'`；纳入 B2.2 单独回归。基线用例绕开此路径，避免基准受待修复行为影响。
- 验证：`cargo test -p tg-service-lua --locked lua_compatibility_baseline_runs_the_same_sample_on_lua_54` 退出 0；`cargo fmt --all -- --check` 退出 0。该基线为后续 B2.2–B2.4 的同版本差异测试入口。

### 2026-09-27 B2.2–B2.3 base 与只读环境

- B2.2：`base.ipairs/pairs/next/select` 使用 Lua 5.4 迭代器三元组、多返回与普通位置参数；覆盖 `__pairs`、`__index`、循环表、nil 多返回和带 `table` 字段的普通表。标准 base 函数不再把 table 参数当作宿主命名参数表。字符串值元表按用户确认继续隔离。
- B2.3：加入项目包装 `rawget/rawset` 并注册为全局与 `base.*`。普通表原始槽位语义、nil 删除与返回原表和 Lua 5.4 基准一致；`rawset` 拒绝宿主只读 API 代理，`rawget` 只查代理自身空表，未泄漏 backing table。普通表允许 `setmetatable(table, nil)`，只读代理因锁定元表仍不可替换。
- 环境回归：宿主 `_ENV` 与 `loader` 加载的模块仍不暴露 `_G`、`load`、原生 `package` 或 `debug.getregistry`；代理元表对脚本保持受保护。B2.4 仅为标准 table functions 选择性启用 `StdLib::TABLE`，不把真实 global 暴露给脚本环境。
- B2.4 table：标准 `concat/insert/move/pack/remove/sort/unpack` 使用 vendored Lua 5.4 函数，外层保留数组/索引预算并拒绝写入只读代理；修正只读数组代理的 `__len`，支持标准库读取。扩展 `count/compact/deepcopy/pretty` 继续使用项目参数协议。Lua fixture 与 `dev_docs/zh_cn/api/table.md` 已迁移到标准位置参数。
- 验证（Windows）：B2.3 时 `cargo test -p tg-service-lua --locked` 为 99 通过；B2.4 targeted `cargo test -p tg-service-lua --locked table_`：9 通过。新增 string 原生基线后，完整 `cargo test -p tg-service-lua --locked`：101 通过、0 失败、0 忽略；严格 Clippy 与 fmt 均退出 0。
- B2.4 string 基线：新增 Lua 5.4 字节索引/多返回/字节 reverse 和 `format` 浮点格式探针；用户确认保留项目命名参数格式、参数传递与结果表形状。该组检查集中于业务逻辑与安全性。
- B2.4 math/utf8 基线：新增 `lua54_math_utf8_baseline_records_native_returns_and_byte_offsets`，覆盖 `modf/frexp` 多返回、`min/max` 变参、`ult` 无符号比较、`utf8.codes` 三元迭代、`codepoint` 多返回和字节位置；该基线记录阶段尚未改生产 math/utf8 API。
- math/utf8 调用契约在初始基线记录时仍待确认；用户于 2026-09-27 随后确认保留项目参数与结果协议，已更新 `B0_CONTRACTS.md`，不再重复提问。
- 验证（Windows，最新 B2 基线树）：`cargo test -p tg-service-lua --locked` 为 102 通过、0 失败、0 忽略；`cargo clippy -p tg-service-lua --all-targets --locked -- -D warnings` 与 `cargo fmt --all -- --check` 均退出 0。
- 当时下一步为 string/pattern 审查；已由下方 B2.4 string/pattern 记录完成，当前下一步按契约审计 math/utf8。

### 2026-09-27 B2.4 string/pattern

- 保留用户确认的命名参数/结果表契约与字符串元表隔离；同版本 Lua 5.4 只作业务语义和安全边界基线。`%a/%A` 经探针确认不是 Lua 5.4 `string.format` 转换项，不添加兼容实现。
- 修复 regex `^` 从非零 `init` 搜索时错误按切片起点锚定；即使 `init` 越界，也先验证 `find/match` 必填参数和 `plain` 类型。
- 修复 `%e/%E/%g/%G` 的科学/通用格式、Lua `%q` 控制字节转义、`%f` 非有限值格式；`%s` 精度按 Unicode 字符且不误受 32 位数值精度上限影响。补最终输出检查，覆盖格式串末尾普通文本。
- 强化资源边界：Lua pattern 512 操作、32 捕获、1,000,000 匹配步骤；regex 32 捕获；模式 8 KiB；遍历/替换/分割最多 10,000 项、累计结果最多 1 MiB。空串极大次数 `string.rep` 直接返回；非空重复按预估输出大小分配。批量 pattern 复用紧凑 UTF-8 索引，惰性 `gmatch` 复用输入字符串与索引。
- 更新 `dev_docs/zh_cn/api/string.md`、`LUA_COMPATIBILITY.md`、`B0_CONTRACTS.md`、`B2_LUA_BASELINE.md`。
- 验证（Windows）：`cargo test -p tg-service-lua --locked`：103 通过、0 失败、0 忽略；`cargo clippy -p tg-service-lua --all-targets --locked -- -D warnings`、`cargo fmt --all -- --check` 与目标文件 `git diff --check` 均退出 0。定向原生基线、项目 string API、pattern 操作上限测试均通过。
- 下一步：math/utf8 已按用户确认的项目协议完成语义与安全审计；详见下方 B2.4 math/utf8 记录。

### 2026-09-27 B2.4 math/utf8

- 按用户确认保留现有命名参数、数组输入、字段表和迭代记录；Lua 5.4 基线用于比较数值计算、Unicode/字节位置和安全边界，不迁移原生参数/多返回协议。
- `math.min/max` 校验稠密数组的显式 `n` 长度、字段范围及缺项；仍支持有效 `table.pack` 数组。覆盖错误 `n` 类型、声明长度不符与空洞。
- `utf8.next` 从提供的字节位置直接检查 UTF-8 边界，避免重复扫描前缀；`char_position` 不再为整段文本分配位置数组。`codepoints`、`next` 与 `string.gmatch` 保留 Lua 字符串句柄，避免为惰性迭代器复制整段 Rust 字符串。
- 把 Lua UTF-8 字符串校验/大小检查提取为 `args::lua_string`，与复制为 Rust `String` 的 `args::string` 共用同一 1 MiB 和 UTF-8 校验契约。
- 更新 `dev_docs/zh_cn/api/math.md`、`utf8.md`、`LUA_COMPATIBILITY.md` 与 `B2_LUA_BASELINE.md`，记录资源限制、参数/结果协议及标准库差异。
- 验证（Windows）：math、utf8、Lua 5.4 math/utf8 基线、string API 定向测试通过；`cargo test -p tg-service-lua --locked` 为 103 通过、0 失败、0 忽略；严格 Clippy、fmt 与目标源码 `git diff --check` 均退出 0。
- 阶段交接时下一步为 B2.5；其后续完成记录见下。

### 2026-09-27 B2.5 错误、GC 与会话关闭

- 保留 `run_with_budget` 指令/时间 hook、VM memory limit 与致命标记。`debug.pcall` 对普通错误仍返回项目命名结果表；内存错误与执行预算错误会继续传播，不可由 Lua 代码吞掉。
- 新增 `lua_gc_preserves_weak_tables_and_runs_finalizers`，显式执行 Lua GC 并验证弱值表清空与 `__gc` 执行；新增 `stop_releases_registered_callbacks_before_collecting`，验证 stop 移除事件回调 RegistryKey 后可回收闭包捕获对象。
- 新增 `instruction_limit_still_closes_pending_variables`；结合既有正常/异常 `__close` 用例，确认指令预算打断回调后线程 reset 仍展开待关闭变量。
- 将 memory-limit 回归改为在 `debug.pcall` 内分配至 VM 限额，确认其不能隐藏内存错误、当前会话进入 Faulted，且独立后续 Lua 会话仍可运行。
- 既有回归同时覆盖 terminal event 移除一次性回调、过期 session generation 清除任务路由、Game/Screensaver 生命周期和对象池释放、loader 模块缓存/加载限制。没有把 GC 的非确定时刻作为测试结果，也未改变项目受控 `debug.pcall/xpcall` 协议。
- 验证（Windows）：`cargo test -p tg-service-lua --locked`：106 通过、0 失败、0 忽略；`cargo clippy -p tg-service-lua --all-targets --locked -- -D warnings`、`cargo fmt --all -- --check` 与 `git diff --check -- crates/service/lua/src/session.rs` 均退出 0。
- B2 出口关闭；下一块按顺序进入 B3 安全模式移除。跨平台和实际终端验证仍留待 B 最终回归。

### 2026-09-27 B3 安全模式移除

- B3.1：移除 `SafeModeDefault`、包状态安全字段和临时豁免。只在反序列化边界忽略旧 `safe_mode` 字段，其余 package profile 字段严格校验；扩充旧数据回归，覆盖默认/Game/Screensaver 状态，以及独立语言、用户键位、继续存档、最高分四个实际保存位置。迁移后重启、读取、更新状态并重写，数据保留且序列化不再输出安全字段。
- B3.2：`LuaApiConfig`、`LuaApiContext` 与 session 去除安全开关；Game/Screensaver 权限只由会话类型决定，Debug 只控制日志与慢回调警告。Game-only `file.write/create_dir/remove/list_dir`、`event.skip_action/clear_action` 和 `game.*` 保持忽略拒绝与每方法单次提示；`file.read/exists` 可在两类会话读取各自包资源。assets 相对路径、操作类型边界与终端隔离保持不变。明确矩阵见 `dev_docs/refactor/B3_PERMISSIONS.md`。
- B3.3：删除安全设置入口、safe mode warning overlay/路由/action、临时豁免与列表高权限提示，保留 debug/error 页面；同步删除中英文无效翻译并改写安全说明。
- B3.4：包列表与 Game config 删除 `safe_mode/high_privilege` 字段；高权限旧清单字段明确拒绝。将 `safe_mode_lab` 改为 `permissions_lab`，保留文件权限与 Debug 测试场景。
- 验证（Windows，逐包串行）：`cargo test -p tg-service-storage --locked`（25/25）、`cargo test -p tg-service-package --locked`（34/34）、`cargo test -p tg-service-lua --locked`（106/106）、`cargo test -p tui-game --locked`（104/104）；workspace strict Clippy、fmt check 和 diff check 均通过。未忽略测试。
- 遗留引用审计：运行时代码、资源、夹具和用户开发文档无有效 safe mode/high privilege 逻辑；剩余仅为 package/profile 迁移与拒绝字段回归、计划和历史记录。
- 边界：只完成 Windows 回归，Linux/macOS 留待最终验收；临时清理范围与策略状态未变，本块没有重试删除。
- B3 出口关闭，进入 B5。用户确认 canonical 左右 Meta token 不变，显示名按 Linux=`Meta`、macOS=`Cmd`、Windows=`Win`。本计划记录该裁决后开始 B5，实现中不再复问。

### 2026-09-27 B5 左右修饰键显示与映射

- `display_key_token` 与 `format_key_display` 仍以稳定 key token 为输入；显示层区分 LCtrl/RCtrl、LShift/RShift、LAlt/RAlt，Meta 分别按 Linux LMeta/RMeta、macOS LCmd/RCmd、Windows LWin/RWin 显示。`ctrl`/`meta` 等旧 alias 仍规范到左侧 token，canonical 持久化格式未变。
- 私有平台格式化器可注入 Windows/macOS/其他平台，单测覆盖全部 modifier 文案；core token 测试覆盖两侧 token parse→serialize 和 combo 展示排序。`rdev` 左右物理键映射各自保留；crossterm 终端快捷键继续只传已有聚合 Ctrl/Shift 标志，没有合成左右物理事件。
- 按键页、富文本和 raw key capture 共用同一个显示转换。Global 页面按当前显示文本宽度计算表格；Game 页面减少键列内边距，新增 80×30 合成帧用例确保 `[LCtrl + X]` 完整显示。没有把展示值写回 profile。
- 验证（Windows）：`cargo test -p tg-core-input --locked`（11/11）、`cargo test -p tg-service-input --locked`（17/17）、`cargo test -p tg-service-rich-text --locked`（14/14）、`cargo test -p tg-service-storage --locked`（26/26）、`cargo test -p tui-game --locked`（105/105）；workspace strict Clippy、fmt check、diff check 通过。全部为 0 失败、0 忽略。
- 平台边界：本环境只在 Windows 构建运行；三个平台的文案通过注入式单测核对，rdev 映射有单测，但没有在 Linux/macOS 实机启动全局监听器，也没有进行 Windows 外的实物按键观测。上述实机能力留待 B 最终跨平台清单，不据此宣称三平台验证完成。
- B5 代码块关闭，进入 B6。安全清理范围、阻止删除的原策略与保留项不变。

### 2026-09-27 B6 开始审查

- 开始基点：承接 B5 工作区；检查确认 `crates/service/image/src/lib.rs` 尚无本轮修改。当前 image service 用路径与六位小数参数的 `DefaultHasher` 缓存、秒级 mtime 校验和非原子写；`ImageTask` 尚不检查取消。Lua 已有 image event 类型与 broker 路由，但 `image` library/host command 尚未注册。
- 可复用基础：`tg-core-atomic-fs::atomic_write` 已提供 sibling temp + replace；image service 尚无稳定内容摘要依赖。现有 `TextColor::Transparent` 可经 compositor/presenter 表达透明样式，但 rich-text 字符串颜色解析不接受 `transparent`。
- B6.4 调查：`EngineServices` 已长期持有以 `data/cache/images` 初始化的 `ImageService`，game/screensaver 详情页每帧同步调用四处 icon/banner 转换；因此内容摘要后可由既有内存缓存避免重复 decode/resize，热更新通过源字节摘要失效，不需要再添加页面级重复缓存。
- 用户于 2026-09-27 关闭了全部 image API 契约问题：Lua 缺省尺寸按源图宽/100、高/200取整且最小 1×1；负裁剪偏移和越界矩形报错、缺省尺寸取剩余区域；新增可选 RGB 字符串背景（`#rrggbb` / `rgb(r,g,b)`），默认黑色并纳入缓存键；异步 `image.load` 立即返回 request id，完成事件携带同一 ID；event `output` 保留可直接绘制的富文本字符串。命名终端色不作为混色输入，以保证确定性。已同步至 `dev_docs/refactor/B0_CONTRACTS.md`，不再复问。
- 清理遵循既有验收记录：本阶段未删除或重试删除任何临时目录；保留 architecture、生成工具及 `temp/image-converter-reference/`。
